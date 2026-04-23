use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use tokio::time::interval;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RunState {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimulationStatus {
    run_state: RunState,
    tick: u64,
    speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlanetKind {
    Skycartel,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkycartelStats {
    total_resources_generated: u64,
    explorer_arrivals: u64,
    explorer_departures: u64,
    rockets_built: u64,
    asteroids_deflected: u64,
    errors_encountered: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanetSummary {
    id: u32,
    name: String,
    kind: PlanetKind,
    type_label: String,
    run_state: RunState,
    energy_cells_total: u32,
    energy_cells_charged: u32,
    resource_cells_total: Option<u32>,
    resource_cells_charged: Option<u32>,
    defense_cells_total: Option<u32>,
    defense_cells_charged: Option<u32>,
    errors: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanetDetails {
    #[serde(flatten)]
    summary: PlanetSummary,
    resources: serde_json::Value,
    skycartel_stats: Option<SkycartelStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    id: String,
    timestamp: String,
    level: LogLevel,
    source: String,
    planet_id: Option<u32>,
    message: String,
}

#[derive(Clone)]
struct AppState {
    simulation: Arc<Mutex<SimulationStatus>>,
    planets: Arc<Mutex<Vec<PlanetDetails>>>,
}

#[derive(Debug, Deserialize)]
struct SetRunStateBody {
    run_state: RunState,
}

#[derive(Debug, Deserialize)]
struct SetSpeedBody {
    speed: f32,
}

#[tokio::main]
async fn main() {
    let simulation = SimulationStatus {
        run_state: RunState::Idle,
        tick: 0,
        speed: 1.0,
    };

    let planets = vec![
        PlanetDetails {
            summary: PlanetSummary {
                id: 1,
                name: "Skycartel Alpha".to_string(),
                kind: PlanetKind::Skycartel,
                type_label: "Type A (Skycartel)".to_string(),
                run_state: RunState::Running,
                energy_cells_total: 5,
                energy_cells_charged: 4,
                resource_cells_total: Some(3),
                resource_cells_charged: Some(3),
                defense_cells_total: Some(2),
                defense_cells_charged: Some(1),
                errors: 0,
            },
            resources: serde_json::json!({
                "Carbon": 120,
                "Iron": 40,
            }),
            skycartel_stats: Some(SkycartelStats {
                total_resources_generated: 500,
                explorer_arrivals: 12,
                explorer_departures: 8,
                rockets_built: 2,
                asteroids_deflected: 3,
                errors_encountered: 1,
            }),
        },
        PlanetDetails {
            summary: PlanetSummary {
                id: 2,
                name: "Experimental Beta".to_string(),
                kind: PlanetKind::Other,
                type_label: "Type B (external)".to_string(),
                run_state: RunState::Paused,
                energy_cells_total: 6,
                energy_cells_charged: 3,
                resource_cells_total: Some(3),
                resource_cells_charged: Some(2),
                defense_cells_total: Some(3),
                defense_cells_charged: Some(1),
                errors: 2,
            },
            resources: serde_json::json!({
                "Hydrogen": 80,
            }),
            skycartel_stats: None,
        },
    ];

    let state = AppState {
        simulation: Arc::new(Mutex::new(simulation)),
        planets: Arc::new(Mutex::new(planets)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/simulation/status", get(get_simulation_status))
        .route("/api/simulation/run-state", post(set_run_state))
        .route("/api/simulation/step", post(step_simulation))
        .route("/api/simulation/speed", post(set_speed))
        .route("/api/planets", get(list_planets))
        .route("/api/planets/:id", get(get_planet))
        .route("/ws/logs", get(logs_ws_upgrade))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Skycartel GUI server listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.expect("bind failed"), app)
        .await
        .expect("server error");
}

async fn get_simulation_status(State(state): State<AppState>) -> Json<SimulationStatus> {
    let sim = state
        .simulation
        .lock()
        .expect("simulation mutex poisoned")
        .clone();
    Json(sim)
}

async fn set_run_state(
    State(state): State<AppState>,
    Json(body): Json<SetRunStateBody>,
) -> Json<SimulationStatus> {
    let mut sim = state
        .simulation
        .lock()
        .expect("simulation mutex poisoned");
    sim.run_state = body.run_state;
    Json(sim.clone())
}

async fn step_simulation(State(state): State<AppState>) -> Json<SimulationStatus> {
    let mut sim = state
        .simulation
        .lock()
        .expect("simulation mutex poisoned");
    sim.tick = sim.tick.saturating_add(1);
    Json(sim.clone())
}

async fn set_speed(
    State(state): State<AppState>,
    Json(body): Json<SetSpeedBody>,
) -> Json<SimulationStatus> {
    let mut sim = state
        .simulation
        .lock()
        .expect("simulation mutex poisoned");
    sim.speed = body.speed;
    Json(sim.clone())
}

async fn list_planets(State(state): State<AppState>) -> Json<Vec<PlanetSummary>> {
    let planets = state.planets.lock().expect("planets mutex poisoned");
    let summaries = planets.iter().map(|p| p.summary.clone()).collect();
    Json(summaries)
}

async fn get_planet(State(state): State<AppState>, axum::extract::Path(id): axum::extract::Path<u32>) -> impl IntoResponse {
    let planets = state.planets.lock().expect("planets mutex poisoned");
    if let Some(p) = planets.iter().find(|p| p.summary.id == id) {
        let details = p.clone();
        (axum::http::StatusCode::OK, Json(details)).into_response()
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Planet not found" })),
        )
            .into_response()
    }
}

async fn logs_ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_logs_ws(socket, state))
}

async fn handle_logs_ws(mut socket: WebSocket, _state: AppState) {
    let mut ticker = interval(Duration::from_secs(1));
    let mut counter: u64 = 0;

    loop {
        ticker.tick().await;
        counter = counter.wrapping_add(1);
        let now = chrono::Utc::now();
        let level = match counter % 4 {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            _ => LogLevel::Error,
        };

        let entry = LogEntry {
            id: format!("{}-{}", now.timestamp_millis(), counter),
            timestamp: now.to_rfc3339(),
            level,
            source: "mock-engine".to_string(),
            planet_id: if counter % 2 == 0 { Some(1) } else { None },
            message: format!("Sample log event #{counter}"),
        };

        let text = match serde_json::to_string(&entry) {
            Ok(t) => t,
            Err(_) => continue,
        };

        if socket.send(Message::Text(text)).await.is_err() {
            break;
        }
    }
}

