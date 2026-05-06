use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::Method;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use skycartel::common_game::protocols::orchestrator_explorer::OrchestratorToExplorer;
use skycartel::common_game::protocols::orchestrator_planet::OrchestratorToPlanet;
use skycartel::common_game::protocols::planet_explorer::ExplorerToPlanet;
use skycartel::broadcast_log::LogEntry;
use skycartel::telemetry::{TelemetryHub, RunState};

#[derive(Clone)]
pub struct AppState {
    pub telemetry: Arc<TelemetryHub>,
    pub planet_senders: Arc<HashMap<u32, Sender<OrchestratorToPlanet>>>,
    pub planet_explorer_senders: Arc<HashMap<u32, Sender<ExplorerToPlanet>>>,
    pub explorer_senders: Arc<HashMap<u32, Sender<OrchestratorToExplorer>>>,
    pub log_tx: broadcast::Sender<LogEntry>,
}

pub fn create_app_state(
    telemetry: Arc<TelemetryHub>,
    planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_explorer_senders: HashMap<u32, Sender<ExplorerToPlanet>>,
    explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
    log_tx: broadcast::Sender<LogEntry>,
) -> AppState {
    AppState {
        telemetry,
        planet_senders: Arc::new(planet_senders),
        planet_explorer_senders: Arc::new(planet_explorer_senders),
        explorer_senders: Arc::new(explorer_senders),
        log_tx,
    }
}

#[derive(Serialize)]
struct ApiSimStatus {
    run_state: String,
    tick: u64,
    speed: f32,
}

#[derive(Deserialize)]
struct SetRunStateBody {
    run_state: String,
}

#[derive(Deserialize)]
struct SetSpeedBody {
    speed: f32,
}

pub async fn run_server(addr: SocketAddr, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/simulation/status", get(get_simulation_status))
        .route("/api/simulation/run-state", post(set_run_state))
        .route("/api/simulation/speed", post(set_speed))
        .route("/api/planets", get(list_planets))
        .route("/api/planets/:id", get(get_planet))
        .fallback_service(
            ServeDir::new("rustyplanet-gui/dist")
                .fallback(ServeFile::new("rustyplanet-gui/dist/index.html"))
        )
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("GUI server bound, starting serve...");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_simulation_status(State(state): State<AppState>) -> Json<ApiSimStatus> {
    let sim = &state.telemetry.sim_status;
    Json(ApiSimStatus {
        run_state: match sim.to_api_state() {
            RunState::Running => "running".to_string(),
            RunState::Paused => "paused".to_string(),
            RunState::Stopped => "stopped".to_string(),
        },
        tick: sim.read_tick(),
        speed: sim.read_speed(),
    })
}

async fn set_run_state(State(state): State<AppState>, Json(body): Json<SetRunStateBody>) {
    let new_state = match body.run_state.as_str() {
        "running" => RunState::Running,
        "paused" => RunState::Paused,
        "stopped" => RunState::Stopped,
        _ => return;
    };
    state.telemetry.sim_status.set_state(new_state);
}

async fn set_speed(State(state): State<AppState>, Json(body): Json<SetSpeedBody>) {
    state.telemetry.sim_status.set_speed(body.speed);
}

async fn list_planets(State(state): State<AppState>) -> Json<Vec<serde_json::Value>> {
    let planets = state.telemetry.planets.lock().unwrap();
    let list: Vec<_> = planets.values().map(|p| serde_json::json!({
        "id": p.id,
        "name": p.name,
        "kind": format!("{:?}", p.kind),
    })).collect();
    Json(list)
}

async fn get_planet(axum::extract::Path(id): axum::extract::Path<u32>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let planets = state.telemetry.planets.lock().unwrap();
    if let Some(p) = planets.get(&id) {
        Json(serde_json::json!({
            "id": p.id,
            "name": p.name,
            "kind": format!("{:?}", p.kind),
        }))
    } else {
        Json(serde_json::json!({"error": "not found"}))
    }
}