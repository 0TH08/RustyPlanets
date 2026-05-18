use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use skycartel::broadcast_log::{BroadcastLogger, LogEntry};
use skycartel::common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use skycartel::common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use skycartel::common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use skycartel::orchestrator::{GuiCommand, Orchestrator};
use skycartel::create_planet;
use skycartel::create_planet_with_telemetry;
use skycartel::explorer::{Bag, Explorer};
use skycartel::planet_telemetry::{CellState, GenerationEntry};
use skycartel::telemetry::{ExplorerTelemetry, PlanetEntry, PlanetKind, RunState, TelemetryHub};
use skycartel::PlanetType;

// ── App state ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    telemetry: Arc<TelemetryHub>,
    gui_tx: Sender<GuiCommand>,
    log_tx: broadcast::Sender<LogEntry>,
    topology: Arc<HashMap<u32, Vec<u32>>>,
}

// ── API response types ────────────────────────────────────────────────

#[allow(dead_code)]
#[allow(non_snake_case)]
struct SimStatus {
    runState: String,
    tick: u64,
    speed: f32,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SetRunStateBody {
    runState: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct SetSpeedBody {
    speed: f32,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct StatusResponse {
    runState: String,
    tick: u64,
    speed: f32,
}

impl StatusResponse {
    fn from_hub(hub: &TelemetryHub) -> Self {
        let sim = &hub.sim_status;
        let rs = match sim.to_api_state() {
            RunState::Idle => "idle",
            RunState::Running => "running",
            RunState::Paused => "paused",
        };
        Self {
            runState: rs.to_string(),
            tick: sim.read_tick(),
            speed: sim.read_speed(),
        }
    }
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct PlanetSummary {
    id: u32,
    name: String,
    kind: String,
    aiRunning: bool,
    explorerCount: usize,
    totalResourcesGenerated: usize,
    rocketsBuilt: usize,
    asteroidsDeflected: usize,
    errorsEncountered: usize,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
struct PlanetDetails {
    summary: PlanetSummary,
    explorerArrivals: usize,
    explorerDepartures: usize,
    cells: Vec<CellState>,
    generationHistory: Vec<GenerationEntry>,
}

#[derive(Serialize)]
struct SimpleStatus {
    status: String,
}

// ── Handlers ──────────────────────────────────────────────────────────

async fn get_simulation_status(State(state): State<AppState>) -> Json<StatusResponse> {
    Json(StatusResponse::from_hub(&state.telemetry))
}

async fn set_simulation_run_state(
    State(state): State<AppState>,
    Json(body): Json<SetRunStateBody>,
) -> Json<StatusResponse> {
    let sim = &state.telemetry.sim_status;
    match body.runState.as_str() {
        "running" => sim.run_state.store(true, Ordering::SeqCst),
        "paused" => sim.run_state.store(false, Ordering::SeqCst),
        "idle" => {
            sim.run_state.store(false, Ordering::SeqCst);
            sim.set_tick(0);
        }
        _ => {}
    }
    Json(StatusResponse::from_hub(&state.telemetry))
}

async fn step_simulation(State(state): State<AppState>) -> Json<StatusResponse> {
    let sim = &state.telemetry.sim_status;
    let current = sim.read_tick();
    sim.set_tick(current + 1);
    Json(StatusResponse::from_hub(&state.telemetry))
}

async fn set_simulation_speed(
    State(state): State<AppState>,
    Json(body): Json<SetSpeedBody>,
) -> Json<StatusResponse> {
    state.telemetry.sim_status.set_speed(body.speed);
    Json(StatusResponse::from_hub(&state.telemetry))
}

async fn reset_simulation(State(state): State<AppState>) -> Json<StatusResponse> {
    let sim = &state.telemetry.sim_status;
    sim.set_tick(0);
    sim.run_state.store(false, Ordering::SeqCst);
    sim.set_speed(1.0);
    Json(StatusResponse::from_hub(&state.telemetry))
}

async fn send_sunray(State(state): State<AppState>, Path(id): Path<u32>) -> Json<SimpleStatus> {
    let _ = state.gui_tx.send(GuiCommand::SendSunray { planet_id: id });
    Json(SimpleStatus { status: "ok".to_string() })
}

async fn send_asteroid(State(state): State<AppState>, Path(id): Path<u32>) -> Json<SimpleStatus> {
    let _ = state.gui_tx.send(GuiCommand::SendAsteroid { planet_id: id });
    Json(SimpleStatus { status: "ok".to_string() })
}

async fn list_planets(State(state): State<AppState>) -> Json<Vec<PlanetSummary>> {
    let planets = state.telemetry.planets.read().unwrap();
    let list: Vec<PlanetSummary> = planets
        .iter()
        .map(|(&id, entry)| {
            let snap = entry.snapshot();
            PlanetSummary {
                id,
                name: snap.name,
                kind: format!("{:?}", snap.kind).to_lowercase(),
                aiRunning: snap.ai_running,
                explorerCount: snap.explorer_count,
                totalResourcesGenerated: snap.stats.total_resources_generated,
                rocketsBuilt: snap.stats.rockets_built,
                asteroidsDeflected: snap.stats.asteroids_deflected,
                errorsEncountered: snap.stats.errors_encountered,
            }
        })
        .collect();
    Json(list)
}

async fn get_planet_details(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Json<PlanetDetails> {
    let planets = state.telemetry.planets.read().unwrap();
    if let Some(entry) = planets.get(&id) {
        let snap = entry.snapshot();
        let cells = entry.telemetry.cell_states();
        let history = entry.telemetry.generation_history();
        Json(PlanetDetails {
            summary: PlanetSummary {
                id,
                name: snap.name.clone(),
                kind: format!("{:?}", snap.kind).to_lowercase(),
                aiRunning: snap.ai_running,
                explorerCount: snap.explorer_count,
                totalResourcesGenerated: snap.stats.total_resources_generated,
                rocketsBuilt: snap.stats.rockets_built,
                asteroidsDeflected: snap.stats.asteroids_deflected,
                errorsEncountered: snap.stats.errors_encountered,
            },
            explorerArrivals: snap.stats.explorer_arrivals,
            explorerDepartures: snap.stats.explorer_departures,
            cells,
            generationHistory: history,
        })
    } else {
        // Return empty details for planets not tracked
        Json(PlanetDetails {
            summary: PlanetSummary {
                id,
                name: format!("Planet{}", id),
                kind: "unknown".to_string(),
                aiRunning: false,
                explorerCount: 0,
                totalResourcesGenerated: 0,
                rocketsBuilt: 0,
                asteroidsDeflected: 0,
                errorsEncountered: 0,
            },
            explorerArrivals: 0,
            explorerDepartures: 0,
            cells: vec![],
            generationHistory: vec![],
        })
    }
}

async fn start_planet(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Json<SimpleStatus> {
    let _ = state.gui_tx.send(GuiCommand::StartPlanet { planet_id: id });
    Json(SimpleStatus { status: "ok".to_string() })
}

async fn stop_planet(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Json<SimpleStatus> {
    let _ = state.gui_tx.send(GuiCommand::StopPlanet { planet_id: id });
    Json(SimpleStatus { status: "ok".to_string() })
}

async fn move_explorer(
    State(state): State<AppState>,
    Path((explorer_id, planet_id)): Path<(u32, u32)>,
) -> Json<SimpleStatus> {
    let _ = state.gui_tx.send(GuiCommand::MoveExplorer { explorer_id, dst_planet_id: planet_id });
    Json(SimpleStatus { status: "ok".to_string() })
}

// ── WebSocket log stream ──────────────────────────────────────────────

async fn ws_logs(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_logs_ws(socket, state.log_tx.subscribe()))
}

async fn handle_logs_ws(mut socket: WebSocket, mut rx: broadcast::Receiver<LogEntry>) {
    loop {
        match rx.recv().await {
            Ok(entry) => {
                let Ok(json) = serde_json::to_string(&entry) else { continue };
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break,
        }
    }
}

// ── Explorer list ─────────────────────────────────────────────────────

#[derive(Serialize)]
#[allow(non_snake_case)]
struct ExplorerApiEntry {
    id: u32,
    currentPlanetId: Option<u32>,
    basicResources: HashMap<String, u32>,
    complexResources: HashMap<String, u32>,
    aiRunning: bool,
}

async fn list_explorers(State(state): State<AppState>) -> Json<Vec<ExplorerApiEntry>> {
    let explorers = state.telemetry.explorers.read().unwrap();
    let positions = state.telemetry.explorer_planet.read().unwrap();
    let mut result: Vec<ExplorerApiEntry> = explorers
        .iter()
        .map(|(id, entry)| {
            let snap = entry.snapshot();
            ExplorerApiEntry {
                id: *id,
                currentPlanetId: positions.get(id).copied(),
                basicResources: snap.basic_resources,
                complexResources: snap.complex_resources,
                aiRunning: snap.ai_running,
            }
        })
        .collect();
    result.sort_by_key(|e| e.id);
    Json(result)
}

// ── Topology ──────────────────────────────────────────────────────────

async fn get_topology(State(state): State<AppState>) -> Json<HashMap<u32, Vec<u32>>> {
    Json((*state.topology).clone())
}

// ── Server ────────────────────────────────────────────────────────────

async fn run_server(addr: SocketAddr, state: AppState) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/simulation/status", get(get_simulation_status))
        .route("/api/simulation/run-state", post(set_simulation_run_state))
        .route("/api/simulation/step", post(step_simulation))
        .route("/api/simulation/speed", post(set_simulation_speed))
        .route("/api/simulation/reset", post(reset_simulation))
        .route("/api/sunray/:id", post(send_sunray))
        .route("/api/asteroid/:id", post(send_asteroid))
        .route("/api/planets", get(list_planets))
        .route("/api/planets/:id", get(get_planet_details))
        .route("/api/planets/:id/start", post(start_planet))
        .route("/api/planets/:id/stop", post(stop_planet))
        .route("/api/explorers/:id/move/:planet_id", post(move_explorer))
        .route("/api/explorers", get(list_explorers))
        .route("/api/topology", get(get_topology))
        .route("/ws/logs", get(ws_logs))
        .layer(cors)
        .fallback_service(ServeDir::new("rustyplanet-gui/dist"))
        .with_state(state);

    println!("Serving on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

// ── Main ──────────────────────────────────────────────────────────────

struct PlanetConfig {
    name: &'static str,
    id: u32,
    kind: PlanetType,
    telemetry_kind: PlanetKind,
}

const PLANET_CONFIGS: &[PlanetConfig] = &[
    PlanetConfig { name: "Skycartel", id: 1, kind: PlanetType::Skycartel, telemetry_kind: PlanetKind::Skycartel },
    PlanetConfig { name: "Luna4", id: 2, kind: PlanetType::Luna4, telemetry_kind: PlanetKind::Luna4 },
    PlanetConfig { name: "BlackAdidasShoe", id: 3, kind: PlanetType::BlackAdidasShoe, telemetry_kind: PlanetKind::BlackAdidasShoe },
    PlanetConfig { name: "ImmutableCosmicBorrow", id: 4, kind: PlanetType::ImmutableCosmicBorrow, telemetry_kind: PlanetKind::ImmutableCosmicBorrow },
    PlanetConfig { name: "RustEze", id: 5, kind: PlanetType::Rusteze, telemetry_kind: PlanetKind::Rusteze },
    PlanetConfig { name: "Crabtorio", id: 6, kind: PlanetType::Crabtorio, telemetry_kind: PlanetKind::Crabtorio },
    PlanetConfig { name: "Orbitron", id: 7, kind: PlanetType::Orbitron, telemetry_kind: PlanetKind::Orbitron },
    PlanetConfig { name: "AstroParrot", id: 8, kind: PlanetType::AstroParrot, telemetry_kind: PlanetKind::AstroParrot },
];

fn load_topology(path: &str) -> HashMap<u32, Vec<u32>> {
    let mut map: HashMap<u32, Vec<u32>> = HashMap::new();
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("Warning: could not read {path}: {e}. Using fully-connected fallback.");
            return map;
        }
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace().filter_map(|t| t.parse::<u32>().ok());
        if let Some(src) = parts.next() {
            let neighbors: Vec<u32> = parts.collect();
            map.insert(src, neighbors);
        }
    }
    println!("Loaded galaxy topology: {} planet entries", map.len());
    map
}

fn main() {
    println!("Starting RustyPlanets...");

    let log_tx = BroadcastLogger::init().expect("Failed to init logger").0;
    println!("Logger ready");

    let telemetry = Arc::new(TelemetryHub::new());

    let mut planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>> = HashMap::new();
    let mut planet_explorer_senders: HashMap<u32, Sender<ExplorerToPlanet>> = HashMap::new();
    let mut explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>> = HashMap::new();

    // Bug A fix: one shared channel so all planets send to one orchestrator receiver
    let (planet_to_orch_tx, planet_to_orch_rx) = crossbeam_channel::unbounded::<PlanetToOrchestrator>();

    // Create planets
    for config in PLANET_CONFIGS {
        let (otx, orx) = crossbeam_channel::unbounded();
        let ptx = planet_to_orch_tx.clone();
        let (etx, erx) = crossbeam_channel::unbounded();

        let ai_running = Arc::new(AtomicBool::new(false));

        // Clone channels so fallback can use them if telemetry creation fails
        let orx_clone = orx.clone();
        let ptx_clone = planet_to_orch_tx.clone();
        let erx_clone = erx.clone();

        let result = create_planet_with_telemetry(
            config.id,
            config.name.to_string(),
            config.kind,
            orx,
            ptx,
            erx,
            ai_running.clone(),
            &telemetry,
        );

        match result {
            Ok(mut planet) => {
                println!("Created planet {} (id={})", config.name, config.id);
                thread::spawn(move || {
                    planet.run().ok();
                });
                planet_senders.insert(config.id, otx);
                planet_explorer_senders.insert(config.id, etx);
            }
            Err(_) => {
                // Fallback: create planet without telemetry
                let result = create_planet(config.id, config.kind, orx_clone, ptx_clone, erx_clone);
                if let Ok(mut planet) = result {
                    println!("Created planet {} (id={}, no telemetry)", config.name, config.id);
                    thread::spawn(move || {
                        planet.run().ok();
                    });
                    planet_senders.insert(config.id, otx);
                    planet_explorer_senders.insert(config.id, etx);

                    // Register a stub entry so the GUI can still see it
                    telemetry.register_planet(
                        config.id,
                        PlanetEntry {
                            name: config.name.to_string(),
                            kind: config.telemetry_kind,
                            telemetry: Box::new(StubTelemetry),
                            ai_running: ai_running.clone(),
                        },
                    );
                } else {
                    println!("Failed to create planet {}", config.name);
                }
            }
        }
    }

    // Shared channel: all explorers report back through one receiver (Bug B fix: receiver passed to Orchestrator below)
    let (exp_to_orch_tx, exp_to_orch_rx) = crossbeam_channel::unbounded::<ExplorerToOrchestrator<Bag>>();
    let mut explorer_reply_senders: HashMap<u32, Sender<PlanetToExplorer>> = HashMap::new();

    // Create explorers
    for explorer_id in 1..=2 {
        let (orch_to_exp_tx, orch_to_exp_rx) = crossbeam_channel::unbounded();
        let (planet_to_exp_tx, planet_to_exp_rx) = crossbeam_channel::unbounded();
        explorer_reply_senders.insert(explorer_id, planet_to_exp_tx);

        let explorer: Explorer<Bag> = Explorer::new(
            explorer_id,
            1,
            exp_to_orch_tx.clone(),
            orch_to_exp_rx,
            planet_explorer_senders[&1].clone(),
            planet_to_exp_rx,
        );

        let (bag, ai_running) = explorer.telemetry_handles();
        telemetry.register_explorer(explorer_id, ExplorerTelemetry { bag, ai_running });
        telemetry.update_explorer_planet(explorer_id, 1);

        println!("Created explorer {}", explorer_id);
        thread::spawn(move || {
            explorer.run();
        });
        explorer_senders.insert(explorer_id, orch_to_exp_tx);
    }

    println!(
        "Ready: {} planets, 2 explorers",
        PLANET_CONFIGS.len()
    );

    // Load galaxy topology from file; keep an Arc for the API server
    let topology = Arc::new(load_topology("galaxy.txt"));

    // Bug B/C fix: create and run the Orchestrator with all wired channels.
    // All GUI-triggered actions (sunray, asteroid, start/stop planet, move explorer)
    // go through gui_tx so the Orchestrator remains the single coordinator.
    let stop_handle = Orchestrator::new(
        planet_senders.clone(),
        planet_explorer_senders.clone(),
        planet_to_orch_rx,
        explorer_senders.clone(),
        exp_to_orch_rx,
    )
    .expect("Failed to create orchestrator")
    .with_explorer_reply_senders(explorer_reply_senders)
    .with_topology((*topology).clone())
    .with_telemetry(telemetry.clone())
    .run();
    let gui_tx = stop_handle.gui_tx.clone();

    // Start GUI server
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        println!("Server on http://{}", addr);

        let state = AppState {
            telemetry,
            gui_tx,
            log_tx,
            topology,
        };

        run_server(addr, state).await;
    });
}

// ── Stub telemetry for planets that don't support full telemetry ──────

struct StubTelemetry;

impl skycartel::planet_telemetry::PlanetTelemetry for StubTelemetry {
    fn snapshot(&self) -> skycartel::planet_telemetry::CommonPlanetSnapshot {
        Default::default()
    }
    fn explorer_count(&self) -> usize {
        0
    }
    fn energy_cells(&self) -> usize {
        0
    }
    fn charged_cells(&self) -> usize {
        0
    }
    fn cell_states(&self) -> Vec<CellState> {
        vec![]
    }
    fn generation_history(&self) -> Vec<GenerationEntry> {
        vec![]
    }
}
