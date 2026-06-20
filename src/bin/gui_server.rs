use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use skycartel::broadcast_log::LogEntry;
use skycartel::telemetry::{PlanetKind, RunState, TelemetryHub};

// ── Shared app state ─────────────────────────────────────────────────────────
// Note: Some fields will be populated by the integration binary.
// Dead-code annotations silence warnings in standalone server mode.

#[derive(Clone)]
struct AppState {
    telemetry: Arc<TelemetryHub>,
    step_tx: Option<Arc<Sender<()>>>,
    log_tx: broadcast::Sender<LogEntry>,
}

// ── API request/response types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiSimStatus {
    run_state: ApiRunState,
    tick: u64,
    speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ApiRunState {
    Idle,
    Running,
    Paused,
}

// Helper function to convert RunState to ApiRunState
fn to_api_run_state(s: RunState) -> ApiRunState {
    match s {
        RunState::Idle => ApiRunState::Idle,
        RunState::Running => ApiRunState::Running,
        RunState::Paused => ApiRunState::Paused,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanetSummary {
    id: u32,
    name: String,
    kind: PlanetKind,
    ai_running: bool,
    explorer_count: usize,
    total_resources_generated: usize,
    rockets_built: usize,
    asteroids_deflected: usize,
    errors_encountered: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanetDetails {
    #[serde(flatten)]
    summary: PlanetSummary,
    explorer_arrivals: usize,
    explorer_departures: usize,
}

#[derive(Debug, Deserialize)]
struct SetRunStateBody {
    run_state: ApiRunState,
}

#[derive(Debug, Deserialize)]
struct SetSpeedBody {
    speed: f32,
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let telemetry = Arc::new(TelemetryHub::new());
    let step_tx: Option<Arc<Sender<()>>> = None;

    // Initialize broadcast logger (falls back to silent if already set)
    let log_tx = match skycartel::broadcast_log::BroadcastLogger::init() {
        Ok((tx, _rx)) => tx,
        Err(_) => {
            // Logger already set — create a standalone channel
            // In a full integration this path is reached only in tests
            let (tx, _) = broadcast::channel::<LogEntry>(1024);
            tx
        }
    };

    let state = AppState {
        telemetry,
        step_tx,
        log_tx,
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
        .fallback_service(
            ServeDir::new("rustyplanet-gui/dist")
                .fallback(ServeFile::new("rustyplanet-gui/dist/index.html")),
        )
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Skycartel GUI server listening on http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr)
            .await
            .expect("bind failed"),
        app,
    )
    .await
    .expect("server error");
}

// ── REST handlers ────────────────────────────────────────────────────────────

async fn get_simulation_status(State(state): State<AppState>) -> Json<ApiSimStatus> {
    let sim = &state.telemetry.sim_status;
    Json(ApiSimStatus {
        run_state: to_api_run_state(sim.to_api_state()),
        tick: sim.read_tick(),
        speed: sim.read_speed(),
    })
}

async fn set_run_state(
    State(state): State<AppState>,
    Json(body): Json<SetRunStateBody>,
) -> Json<ApiSimStatus> {
    let sim = &state.telemetry.sim_status;

    match body.run_state {
        ApiRunState::Running => {
            sim.run_state
                .store(true, std::sync::atomic::Ordering::SeqCst);
            if let Some(ref tx) = state.step_tx {
                let _ = tx.try_send(());
            }
        }
        ApiRunState::Paused => {
            sim.run_state
                .store(false, std::sync::atomic::Ordering::SeqCst);
        }
        ApiRunState::Idle => {
            sim.run_state
                .store(false, std::sync::atomic::Ordering::SeqCst);
            sim.set_tick(0);
        }
    }

    Json(ApiSimStatus {
        run_state: to_api_run_state(sim.to_api_state()),
        tick: sim.read_tick(),
        speed: sim.read_speed(),
    })
}

async fn step_simulation(State(state): State<AppState>) -> Json<ApiSimStatus> {
    if let Some(ref tx) = state.step_tx {
        let _ = tx.try_send(());
    }
    let sim = &state.telemetry.sim_status;
    sim.run_state
        .store(true, std::sync::atomic::Ordering::SeqCst);

    Json(ApiSimStatus {
        run_state: to_api_run_state(sim.to_api_state()),
        tick: sim.read_tick(),
        speed: sim.read_speed(),
    })
}

async fn set_speed(
    State(state): State<AppState>,
    Json(body): Json<SetSpeedBody>,
) -> Json<ApiSimStatus> {
    state.telemetry.sim_status.set_speed(body.speed);
    let sim = &state.telemetry.sim_status;
    Json(ApiSimStatus {
        run_state: to_api_run_state(sim.to_api_state()),
        tick: sim.read_tick(),
        speed: sim.read_speed(),
    })
}

async fn list_planets(State(state): State<AppState>) -> Json<Vec<PlanetSummary>> {
    let planets = state.telemetry.planets.read().unwrap();
    let summaries: Vec<PlanetSummary> = planets
        .iter()
        .map(|(id, entry)| {
            let snap = entry.snapshot();
            PlanetSummary {
                id: *id,
                name: snap.name,
                kind: snap.kind,
                ai_running: snap.ai_running,
                explorer_count: snap.explorer_count,
                total_resources_generated: snap.stats.total_resources_generated,
                rockets_built: snap.stats.rockets_built,
                asteroids_deflected: snap.stats.asteroids_deflected,
                errors_encountered: snap.stats.errors_encountered,
            }
        })
        .collect();
    Json(summaries)
}

async fn get_planet(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> impl IntoResponse {
    let planets = state.telemetry.planets.read().unwrap();
    if let Some(entry) = planets.get(&id) {
        let snap = entry.snapshot();
        let details = PlanetDetails {
            summary: PlanetSummary {
                id,
                name: snap.name,
                kind: snap.kind,
                ai_running: snap.ai_running,
                explorer_count: snap.explorer_count,
                total_resources_generated: snap.stats.total_resources_generated,
                rockets_built: snap.stats.rockets_built,
                asteroids_deflected: snap.stats.asteroids_deflected,
                errors_encountered: snap.stats.errors_encountered,
            },
            explorer_arrivals: snap.stats.explorer_arrivals,
            explorer_departures: snap.stats.explorer_departures,
        };
        (axum::http::StatusCode::OK, Json(details)).into_response()
    } else {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Planet not found" })),
        )
            .into_response()
    }
}

// ── WebSocket — real log streaming via broadcast channel ─────────────────────

async fn logs_ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let log_rx = state.log_tx.subscribe();
    ws.on_upgrade(move |socket| handle_logs_ws(socket, log_rx))
}

async fn handle_logs_ws(mut socket: WebSocket, mut log_rx: broadcast::Receiver<LogEntry>) {
    loop {
        match log_rx.recv().await {
            Ok(entry) => {
                let text = match serde_json::to_string(&entry) {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // consumer too slow — send a notice and keep going
                let notice = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "level": "Warn",
                    "target": "ws-logs",
                    "message": format!("Lagged {} log entries", n),
                });
                if let Ok(text) = serde_json::to_string(&notice) {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}
