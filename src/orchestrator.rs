use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::Arc;

use common_game::components::forge::Forge;
use common_game::components::planet::DummyPlanetState;
use common_game::components::rocket::Rocket;
use crossbeam_channel::{Receiver, Sender};

use crate::explorer::Bag;
use crate::player_log;
use crate::telemetry::TelemetryHub;
use common_game::protocols::orchestrator_explorer::{
    ExplorerToOrchestrator, OrchestratorToExplorer,
};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};

/// Commands the GUI server sends to the Orchestrator instead of writing
/// directly to planet/explorer channels.
pub enum GuiCommand {
    SendSunray {
        planet_id: u32,
    },
    SendAsteroid {
        planet_id: u32,
    },
    StartPlanet {
        planet_id: u32,
    },
    StopPlanet {
        planet_id: u32,
    },
    MoveExplorer {
        explorer_id: u32,
        dst_planet_id: u32,
    },
}

/// Validates planet message sequence protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlanetState {
    Idle,
    Starting,
    Running,
    Stopping,
}

/// Validates explorer message sequence protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ExplorerState {
    Idle,
    Starting,
    Running,
    Stopping,
}

pub struct Orchestrator {
    planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_explorer_senders: HashMap<u32, Sender<ExplorerToPlanet>>,
    planet_receiver: Receiver<PlanetToOrchestrator>,
    explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
    explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    forge: Option<Forge>,
    running: Arc<AtomicBool>,
    step_rx: Option<Receiver<()>>,
    gui_rx: Option<Receiver<GuiCommand>>,
    telemetry: Option<Arc<TelemetryHub>>,
    tick: Arc<AtomicU64>,
    explorer_planet: HashMap<u32, u32>,
    explorer_alive: HashMap<u32, bool>,
    planet_alive: HashMap<u32, bool>,
    planet_states: HashMap<u32, PlanetState>,
    explorer_states: HashMap<u32, ExplorerState>,
    explorer_reply_senders: HashMap<u32, Sender<PlanetToExplorer>>,
    topology: HashMap<u32, Vec<u32>>,
}

pub struct StopHandle {
    running: Arc<AtomicBool>,
    tick: Arc<AtomicU64>,
    pub gui_tx: Sender<GuiCommand>,
    explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
    telemetry: Option<Arc<TelemetryHub>>,
}

// ── Core ──────────────────────────────────────────────────────────────────────

impl Orchestrator {
    pub fn new(
        planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
        planet_explorer_senders: HashMap<u32, Sender<ExplorerToPlanet>>,
        planet_receiver: Receiver<PlanetToOrchestrator>,
        explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
        explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    ) -> Result<Self, String> {
        let running = Arc::new(AtomicBool::new(false));

        Ok(Self {
            planet_senders,
            planet_explorer_senders,
            planet_receiver,
            explorer_senders,
            explorer_receiver,
            forge: None,
            running,
            step_rx: None,
            gui_rx: None,
            telemetry: None,
            tick: Arc::new(AtomicU64::new(0)),
            explorer_planet: HashMap::new(),
            explorer_alive: HashMap::new(),
            planet_alive: HashMap::new(),
            planet_states: HashMap::new(),
            explorer_states: HashMap::new(),
            explorer_reply_senders: HashMap::new(),
            topology: HashMap::new(),
        })
    }

    fn get_forge(&mut self) -> &mut Forge {
        if self.forge.is_none() {
            self.forge = Some(Forge::new().expect("Failed to create forge"));
        }
        self.forge.as_mut().expect("Forge missing")
    }

    pub fn with_step_channel(mut self, step_rx: Receiver<()>) -> Self {
        self.step_rx = Some(step_rx);
        self
    }

    pub fn with_gui_channel(mut self, gui_rx: Receiver<GuiCommand>) -> Self {
        self.gui_rx = Some(gui_rx);
        self
    }

    pub fn with_explorer_reply_senders(
        mut self,
        senders: HashMap<u32, Sender<PlanetToExplorer>>,
    ) -> Self {
        self.explorer_reply_senders = senders;
        self
    }

    pub fn with_topology(mut self, topology: HashMap<u32, Vec<u32>>) -> Self {
        self.topology = topology;
        self
    }

    /// Builder: attach a telemetry hub for real-time state observation.
    pub fn with_telemetry(mut self, telemetry: Arc<TelemetryHub>) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    /// Returns a reference to the atomic running flag, allowing external
    /// code (e.g. the GUI server) to start/pause the simulation loop.
    pub fn running_flag(&self) -> &Arc<AtomicBool> {
        &self.running
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.tick.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn handle_gui_command(&mut self, cmd: GuiCommand) {
        match cmd {
            GuiCommand::SendSunray { planet_id } => {
                if let Err(error) = self.send_sunray(planet_id) {
                    log::warn!("GUI sunray to planet {planet_id} failed: {error}");
                }
            }
            GuiCommand::SendAsteroid { planet_id } => {
                if let Err(error) = self.send_asteroid(planet_id) {
                    log::warn!("GUI asteroid to planet {planet_id} failed: {error}");
                }
            }
            GuiCommand::StartPlanet { planet_id } => {
                if let Err(error) = self.start_planet(planet_id) {
                    log::warn!("GUI start planet {planet_id} failed: {error}");
                } else if let Some(ref telemetry) = self.telemetry {
                    if let Some(entry) = telemetry.planets.read().unwrap().get(&planet_id) {
                        entry
                            .ai_running
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            GuiCommand::StopPlanet { planet_id } => {
                if let Err(error) = self.stop_planet(planet_id) {
                    log::warn!("GUI stop planet {planet_id} failed: {error}");
                } else if let Some(ref telemetry) = self.telemetry {
                    if let Some(entry) = telemetry.planets.read().unwrap().get(&planet_id) {
                        entry
                            .ai_running
                            .store(false, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }
            GuiCommand::MoveExplorer {
                explorer_id,
                dst_planet_id,
            } => {
                let current = self.explorer_planet.get(&explorer_id).copied().unwrap_or(0);
                self.handle_travel_request(explorer_id, current, dst_planet_id);
            }
        }
    }

    /// Spawns the main simulation loop on a new thread.
    ///
    /// The loop runs until `StopHandle::stop()` is called or the running
    /// flag is set to false. Returns a handle that can control the loop.
    pub fn run(mut self) -> StopHandle {
        // Initialize all planets and explorers
        if let Err(e) = self.initialize() {
            log::error!("Failed to initialize orchestrator: {e}");
        }

        let running = Arc::clone(&self.running);
        let running_thread = Arc::clone(&running);
        let gui_rx = self.gui_rx.take();
        let telemetry = self.telemetry.clone();
        let stop_handle_telemetry = self.telemetry.clone();
        let tick_arc = Arc::clone(&self.tick);
        let explorer_control_senders = self.explorer_senders.clone();

        let (gui_tx, gui_rx_owned) = crossbeam_channel::unbounded::<GuiCommand>();
        // If a gui_rx was provided via with_gui_channel, use it; otherwise use the new one.
        let gui_rx_final = gui_rx.unwrap_or(gui_rx_owned);

        std::thread::spawn(move || {
            eprintln!("[orchestrator thread] starting loop");

            loop {
                while !running_thread.load(std::sync::atomic::Ordering::SeqCst) {
                    while let Ok(cmd) = gui_rx_final.try_recv() {
                        self.handle_gui_command(cmd);
                    }

                    self.handle_planet_responses();
                    self.handle_explorer_responses();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                eprintln!("[orchestrator thread] tick");

                // Drain GUI commands before processing planet/explorer responses.
                while let Ok(cmd) = gui_rx_final.try_recv() {
                    self.handle_gui_command(cmd);
                }

                // Send a sunray to every alive planet each tick so they can charge
                // cells and respond to explorer resource requests.
                let alive_planet_ids: Vec<u32> = self
                    .planet_alive
                    .iter()
                    .filter_map(|(&id, &alive)| if alive { Some(id) } else { None })
                    .collect();
                for planet_id in alive_planet_ids {
                    if let Err(e) = self.send_sunray(planet_id) {
                        log::warn!("Failed to send sunray to planet {planet_id}: {e}");
                    }
                }

                // maybe send asteroid (~5% chance per tick)
                // (deferred - asteroids would need coordination)
                // if rand::random::<f32>() < 0.05 {
                //     let alive_planets: Vec<u32> = self.planet_alive
                //         .iter()
                //         .filter_map(|(&id, &alive)| if alive { Some(id) } else { None })
                //         .collect();

                //     if !alive_planets.is_empty() {
                //         let idx = (rand::random::<u32>() as usize) % alive_planets.len();
                //         let planet_id = alive_planets[idx];
                //         let _ = self.send_asteroid(planet_id);
                //         log::info!("Asteroid targeting planet {planet_id}");
                //     }
                // }

                self.handle_planet_responses();
                self.handle_explorer_responses();

                // increment tick counter
                self.tick.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let current_tick = self.tick.load(std::sync::atomic::Ordering::SeqCst);

                // update telemetry
                if let Some(ref tel) = telemetry {
                    tel.sim_status.set_tick(current_tick);
                }

                // sleep between ticks — respect the speed setting
                let speed = telemetry
                    .as_ref()
                    .map_or(1.0, |t| t.sim_status.read_speed().max(0.1));
                let sleep_ms = (500.0 / speed).max(20.0) as u64;
                std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
            }
        });

        StopHandle {
            running,
            tick: tick_arc,
            gui_tx,
            explorer_senders: explorer_control_senders,
            telemetry: stop_handle_telemetry,
        }
    }
}

impl StopHandle {
    fn wait_for_explorer_ai_state(&self, expected: bool) {
        let Some(telemetry) = &self.telemetry else {
            return;
        };

        for _ in 0..100 {
            let all_match = {
                let explorers = telemetry.explorers.read().unwrap();
                explorers.values().all(|entry| {
                    entry
                        .telemetry
                        .ai_running
                        .load(std::sync::atomic::Ordering::SeqCst)
                        == expected
                })
            };

            if all_match {
                return;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        log::warn!("Timed out waiting for Explorer AI state to become {expected}");
    }

    /// Stop the simulation loop and every Explorer AI.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        for (explorer_id, sender) in &self.explorer_senders {
            if let Err(error) = sender.send(OrchestratorToExplorer::StopExplorerAI) {
                log::warn!("Failed to stop Explorer {explorer_id} AI: {error}");
            }
        }

        self.wait_for_explorer_ai_state(false);
    }

    /// Start every Explorer AI and resume the simulation loop.
    pub fn start(&self) {
        for (explorer_id, sender) in &self.explorer_senders {
            if let Err(error) = sender.send(OrchestratorToExplorer::StartExplorerAI) {
                log::warn!("Failed to start Explorer {explorer_id} AI: {error}");
            }
        }

        self.wait_for_explorer_ai_state(true);

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if the simulation loop is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns the current tick count at the time of handle creation.
    pub fn tick(&self) -> u64 {
        self.tick.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn reset_tick(&self) {
        self.tick.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

// ── Planet messaging ──────────────────────────────────────────────────────────

impl Orchestrator {
    pub fn send_sunray(&mut self, planet_id: u32) -> Result<(), String> {
        let sender = self
            .planet_senders
            .get(&planet_id)
            .cloned()
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        let sunray = self.get_forge().generate_sunray();

        sender
            .send(OrchestratorToPlanet::Sunray(sunray))
            .map_err(|e| format!("Failed to send sunray: {e}"))?;

        Ok(())
    }

    pub fn send_asteroid(&mut self, planet_id: u32) -> Result<(), String> {
        let sender = self
            .planet_senders
            .get(&planet_id)
            .cloned()
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        let asteroid = self.get_forge().generate_asteroid();

        //3- Send it
        sender
            .send(OrchestratorToPlanet::Asteroid(asteroid))
            .map_err(|e| format!("Failed to send asteroid: {e}"))?;

        //Return Ok
        Ok(())
    }
}

// ── Planet response handling ──────────────────────────────────────────────────

impl Orchestrator {
    fn handle_planet_responses(&mut self) {
        // try_recv() — non-blocking, returns immediately if nothing is there
        while let Ok(msg) = self.planet_receiver.try_recv() {
            match msg {
                PlanetToOrchestrator::SunrayAck { planet_id } => {
                    self.handle_sunray_ack(planet_id);
                }
                PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => {
                    // planet responded to asteroid — did it have a rocket to deflect?
                    self.handle_asteroid_ack(planet_id, rocket);
                }
                PlanetToOrchestrator::IncomingExplorerResponse {
                    planet_id,
                    explorer_id,
                    res,
                } => {
                    self.handle_incoming_explorer_response(planet_id, explorer_id, res);
                }
                PlanetToOrchestrator::InternalStateResponse {
                    planet_id,
                    planet_state,
                } => {
                    self.handle_internal_state_response(planet_id, planet_state);
                }
                PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                    self.handle_kill_planet_result(planet_id);
                }
                PlanetToOrchestrator::OutgoingExplorerResponse {
                    planet_id,
                    explorer_id,
                    res,
                } => {
                    self.handle_outgoing_explorer_response(planet_id, explorer_id, res);
                }
                PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
                    self.handle_start_planet_ai_result(planet_id);
                }
                PlanetToOrchestrator::StopPlanetAIResult { planet_id } => {
                    self.handle_stop_planet_ai_result(planet_id);
                }
                PlanetToOrchestrator::Stopped { planet_id } => {
                    self.handle_stopped(planet_id);
                } // All patterns handled above - _ is unreachable - all cases handled above
            }
        }
    }

    fn handle_sunray_ack(&self, planet_id: u32) {
        // planet confirmed it got the sunray
        log::debug!("Planet {planet_id} acknowledged sunray");
    }

    fn handle_asteroid_ack(&mut self, planet_id: u32, rocket: Option<Rocket>) {
        match rocket {
            Some(_rocket) => {
                player_log!("Planet {planet_id} deflected the asteroid");
            }
            None => {
                player_log!("Planet {planet_id} was hit by an asteroid and is being destroyed");
                if let Err(e) = self.kill_planet(planet_id) {
                    log::error!("Failed to kill planet {planet_id}: {e}");
                }
            }
        }
    }

    // planet confirmed an explorer arrived — log success or failure
    fn handle_incoming_explorer_response(
        &self,
        planet_id: u32,
        explorer_id: u32,
        res: Result<(), String>,
    ) {
        match res {
            Ok(_) => {
                player_log!("Explorer {explorer_id} arrived at planet {planet_id}")
            }
            Err(e) => {
                log::warn!("Explorer {explorer_id} failed to arrive at planet {planet_id}: {e}")
            }
        }
    }

    fn handle_internal_state_response(&self, planet_id: u32, _planet_state: DummyPlanetState) {
        // log that we rcvd internal state from a planet
        log::debug!("Received internal state from planet {planet_id}");
    }

    fn handle_kill_planet_result(&mut self, planet_id: u32) {
        // planet was destroyed by an asteroid
        log::warn!("Planet {planet_id} has been destroyed by an asteroid");

        // mark planet as not alive and update state
        self.planet_alive.insert(planet_id, false);
        self.planet_states.insert(planet_id, PlanetState::Idle);

        // remove planet_id from planet_senders
        self.planet_senders.remove(&planet_id);
        self.planet_explorer_senders.remove(&planet_id);

        // find and terminate all explorers mapped to this planet
        let explorers_to_kill: Vec<u32> = self
            .explorer_planet
            .iter()
            .filter_map(|(explorer_id, &pid)| {
                if pid == planet_id {
                    Some(*explorer_id)
                } else {
                    None
                }
            })
            .collect();

        for explorer_id in explorers_to_kill {
            log::warn!("Terminating explorer {explorer_id} due to planet {planet_id} destruction");
            let _ = self.kill_explorer(explorer_id);
            self.explorer_planet.remove(&explorer_id);
        }
    }

    // planet confirmed an explorer departed — log success or failure
    fn handle_outgoing_explorer_response(
        &self,
        planet_id: u32,
        explorer_id: u32,
        res: Result<(), String>,
    ) {
        match res {
            Ok(_) => {
                player_log!("Explorer {explorer_id} left planet {planet_id}")
            }
            Err(e) => {
                log::warn!("Explorer {explorer_id} failed to leave planet {planet_id}: {e}")
            }
        }
    }

    // planet confirmed its AI has started
    fn handle_start_planet_ai_result(&mut self, planet_id: u32) {
        if self.validate_planet_state_transition(planet_id, PlanetState::Running) {
            player_log!("Planet AI started for planet {planet_id}");
        }
    }

    // planet confirmed its AI has stopped
    fn handle_stop_planet_ai_result(&mut self, planet_id: u32) {
        if self.validate_planet_state_transition(planet_id, PlanetState::Stopping) {
            player_log!("Planet AI stopped for planet {planet_id}");
        }
    }

    // planet confirmed it has stopped
    fn handle_stopped(&mut self, planet_id: u32) {
        if self.validate_planet_state_transition(planet_id, PlanetState::Idle) {
            player_log!("Planet {planet_id} stopped");
        }
    }
}

// ── Explorer messaging ────────────────────────────────────────────────────────

impl Orchestrator {
    /// Send a message to a specific explorer
    pub fn send_to_explorer(
        &self,
        explorer_id: u32,
        msg: OrchestratorToExplorer,
    ) -> Result<(), String> {
        let sender = self
            .explorer_senders
            .get(&explorer_id)
            .ok_or_else(|| format!("No sender found for explorer {explorer_id}"))?;

        sender
            .send(msg)
            .map_err(|e| format!("Failed to send to explorer {explorer_id}: {e}"))
    }

    /// Send a message to an explorer via their planet's channel
    pub fn send_to_explorer_via_planet(
        &self,
        explorer_id: u32,
        msg: ExplorerToPlanet,
    ) -> Result<(), String> {
        let planet_id = self
            .explorer_planet
            .get(&explorer_id)
            .copied()
            .ok_or_else(|| format!("Explorer {explorer_id} not mapped to any planet"))?;

        let sender = self
            .planet_explorer_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No explorer sender found for planet {planet_id}"))?;

        sender.send(msg).map_err(|e| {
            format!("Failed to send to explorer {explorer_id} via planet {planet_id}: {e}")
        })
    }
}

// ── Explorer response handling ────────────────────────────────────────────────

impl Orchestrator {
    fn handle_explorer_responses(&mut self) {
        while let Ok(msg) = self.explorer_receiver.try_recv() {
            let _explorer_id = msg.explorer_id();
            match msg {
                ExplorerToOrchestrator::StartExplorerAIResult { explorer_id } => {
                    self.handle_explorer_started(explorer_id);
                }
                ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
                    self.handle_explorer_killed(explorer_id);
                }
                ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id } => {
                    log::debug!("Explorer {explorer_id} AI reset");
                }
                ExplorerToOrchestrator::StopExplorerAIResult { explorer_id } => {
                    self.explorer_states
                        .insert(explorer_id, ExplorerState::Idle);
                    player_log!("Explorer {explorer_id} AI stopped");
                }
                ExplorerToOrchestrator::MovedToPlanetResult {
                    explorer_id,
                    planet_id,
                } => {
                    self.handle_explorer_moved(explorer_id, planet_id);
                }
                ExplorerToOrchestrator::CurrentPlanetResult {
                    explorer_id,
                    planet_id,
                } => {
                    log::debug!("Explorer {explorer_id} is on planet {planet_id}");
                }
                ExplorerToOrchestrator::SupportedResourceResult {
                    explorer_id,
                    supported_resources,
                } => {
                    log::debug!(
                        "Explorer {explorer_id} supported resources: {supported_resources:?}"
                    );
                }
                ExplorerToOrchestrator::SupportedCombinationResult {
                    explorer_id,
                    combination_list,
                } => {
                    log::debug!(
                        "Explorer {explorer_id} supported combinations: {combination_list:?}"
                    );
                }
                ExplorerToOrchestrator::GenerateResourceResponse {
                    explorer_id,
                    generated,
                } => match generated {
                    Ok(()) => log::debug!("Explorer {explorer_id} generated resource"),
                    Err(e) => log::warn!("Explorer {explorer_id} failed to generate resource: {e}"),
                },
                ExplorerToOrchestrator::CombineResourceResponse {
                    explorer_id,
                    generated,
                } => match generated {
                    Ok(()) => log::debug!("Explorer {explorer_id} combined resource"),
                    Err(e) => log::warn!("Explorer {explorer_id} failed to combine resource: {e}"),
                },
                ExplorerToOrchestrator::BagContentResponse {
                    explorer_id,
                    bag_content,
                } => {
                    log::debug!("Explorer {explorer_id} bag content: {bag_content:?}");
                }
                ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id,
                    current_planet_id,
                } => {
                    self.handle_neighbors_request(explorer_id, current_planet_id);
                }
                ExplorerToOrchestrator::TravelToPlanetRequest {
                    explorer_id,
                    current_planet_id,
                    dst_planet_id,
                } => {
                    self.handle_travel_request(explorer_id, current_planet_id, dst_planet_id);
                }
            }
        }
    }

    fn handle_explorer_started(&mut self, explorer_id: u32) {
        if self.validate_explorer_state_transition(explorer_id, ExplorerState::Running) {
            player_log!("Explorer {explorer_id} AI started");
        }
    }

    fn handle_explorer_killed(&mut self, explorer_id: u32) {
        player_log!("Explorer {explorer_id} has been killed");
        self.explorer_alive.insert(explorer_id, false);
        self.explorer_states
            .insert(explorer_id, ExplorerState::Idle);
        self.explorer_planet.remove(&explorer_id);
    }

    fn handle_explorer_moved(&mut self, explorer_id: u32, planet_id: u32) {
        let old_planet = self.explorer_planet.insert(explorer_id, planet_id);
        player_log!(
            "Explorer {explorer_id} moved to planet {planet_id} (was on {:?})",
            old_planet
        );
        if let Some(ref tel) = self.telemetry {
            tel.update_explorer_planet(explorer_id, planet_id);
        }
    }

    fn handle_neighbors_request(&self, explorer_id: u32, current_planet_id: u32) {
        let neighbors: Vec<u32> = if let Some(adj) = self.topology.get(&current_planet_id) {
            // Only return topology-defined neighbors that are actually alive
            adj.iter()
                .copied()
                .filter(|id| self.planet_senders.contains_key(id))
                .collect()
        } else {
            // Fallback: treat all other known planets as neighbors
            self.planet_senders
                .keys()
                .filter(|&&id| id != current_planet_id)
                .copied()
                .collect()
        };

        let msg = OrchestratorToExplorer::NeighborsResponse { neighbors };
        let _ = self.send_to_explorer(explorer_id, msg);
    }

    fn handle_travel_request(
        &mut self,
        explorer_id: u32,
        current_planet_id: u32,
        dst_planet_id: u32,
    ) {
        if !self
            .explorer_alive
            .get(&explorer_id)
            .copied()
            .unwrap_or(false)
        {
            log::warn!("Explorer {explorer_id} is not alive, ignoring travel request");
            return;
        }

        let Some(sender_to_new_planet) = self.planet_explorer_senders.get(&dst_planet_id).cloned()
        else {
            let msg = OrchestratorToExplorer::MoveToPlanet {
                sender_to_new_planet: None,
                planet_id: dst_planet_id,
            };
            let _ = self.send_to_explorer(explorer_id, msg);
            log::warn!(
                "Explorer {explorer_id} denied travel to planet {dst_planet_id}: planet not found"
            );
            return;
        };

        // Bug D fix: tell old planet the explorer is leaving
        if let Some(old_tx) = self.planet_senders.get(&current_planet_id).cloned() {
            let _ = old_tx.send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id });
        }

        // Bug D fix: give new planet a sender to reply to this explorer
        if let Some(reply_tx) = self.explorer_reply_senders.get(&explorer_id).cloned() {
            if let Some(new_tx) = self.planet_senders.get(&dst_planet_id).cloned() {
                let _ = new_tx.send(OrchestratorToPlanet::IncomingExplorerRequest {
                    explorer_id,
                    new_sender: reply_tx,
                });
            }
        }

        let msg = OrchestratorToExplorer::MoveToPlanet {
            sender_to_new_planet: Some(sender_to_new_planet),
            planet_id: dst_planet_id,
        };
        let _ = self.send_to_explorer(explorer_id, msg);
        log::info!(
            "Explorer {explorer_id} approved to travel from {current_planet_id} to {dst_planet_id}"
        );
    }
}

// ── Lifecycle methods ─────────────────────────────────────────────────────────

impl Orchestrator {
    /// Start a planet's AI
    pub fn start_planet(&self, planet_id: u32) -> Result<(), String> {
        let sender = self
            .planet_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        sender
            .send(OrchestratorToPlanet::StartPlanetAI)
            .map_err(|e| format!("Failed to start planet {planet_id}: {e}"))?;

        log::info!("Sent start signal to planet {planet_id}");
        Ok(())
    }

    /// Stop a planet's AI
    pub fn stop_planet(&self, planet_id: u32) -> Result<(), String> {
        let sender = self
            .planet_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        sender
            .send(OrchestratorToPlanet::StopPlanetAI)
            .map_err(|e| format!("Failed to stop planet {planet_id}: {e}"))?;

        log::info!("Sent stop signal to planet {planet_id}");
        Ok(())
    }

    /// Kill a planet (destroy it)
    pub fn kill_planet(&mut self, planet_id: u32) -> Result<(), String> {
        let sender = self
            .planet_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        sender
            .send(OrchestratorToPlanet::KillPlanet)
            .map_err(|e| format!("Failed to kill planet {planet_id}: {e}"))?;

        // mark planet as not alive
        self.planet_alive.insert(planet_id, false);
        self.planet_states.insert(planet_id, PlanetState::Stopping);

        log::warn!("Sent kill signal to planet {planet_id}");
        Ok(())
    }

    /// Start an explorer
    pub fn start_explorer(&self, explorer_id: u32) -> Result<(), String> {
        self.send_to_explorer(explorer_id, OrchestratorToExplorer::StartExplorerAI)
    }

    /// Stop an explorer
    pub fn stop_explorer(&self, explorer_id: u32) -> Result<(), String> {
        self.send_to_explorer(explorer_id, OrchestratorToExplorer::StopExplorerAI)
    }

    /// Kill an explorer
    pub fn kill_explorer(&mut self, explorer_id: u32) -> Result<(), String> {
        self.send_to_explorer(explorer_id, OrchestratorToExplorer::KillExplorer)?;
        self.explorer_alive.insert(explorer_id, false);
        self.explorer_states
            .insert(explorer_id, ExplorerState::Stopping);
        Ok(())
    }
}

// ── Message sequence validation ───────────────────────────────────────────────

impl Orchestrator {
    /// Validate and update planet state according to protocol
    fn validate_planet_state_transition(&mut self, planet_id: u32, new_state: PlanetState) -> bool {
        let current = self
            .planet_states
            .get(&planet_id)
            .copied()
            .unwrap_or(PlanetState::Idle);

        let valid = matches!(
            (current, new_state),
            (PlanetState::Idle, PlanetState::Starting)
                | (PlanetState::Idle, PlanetState::Running)
                | (PlanetState::Starting, PlanetState::Running)
                | (PlanetState::Running, PlanetState::Stopping)
                | (PlanetState::Running, PlanetState::Starting)
                | (PlanetState::Stopping, PlanetState::Idle)
                | (PlanetState::Stopping, PlanetState::Starting)
                | (PlanetState::Stopping, PlanetState::Running)
        );

        if valid {
            self.planet_states.insert(planet_id, new_state);
        } else {
            log::warn!(
                "Invalid planet state transition for {planet_id}: {current:?} -> {new_state:?}"
            );
        }

        valid
    }

    /// Validate and update explorer state according to protocol
    fn validate_explorer_state_transition(
        &mut self,
        explorer_id: u32,
        new_state: ExplorerState,
    ) -> bool {
        let current = self
            .explorer_states
            .get(&explorer_id)
            .copied()
            .unwrap_or(ExplorerState::Idle);

        let valid = matches!(
            (current, new_state),
            (ExplorerState::Idle, ExplorerState::Starting)
                | (ExplorerState::Idle, ExplorerState::Running)
                | (ExplorerState::Starting, ExplorerState::Running)
                | (ExplorerState::Running, ExplorerState::Stopping)
                | (ExplorerState::Stopping, ExplorerState::Idle)
        );

        if valid {
            self.explorer_states.insert(explorer_id, new_state);
        } else {
            log::warn!(
                "Invalid explorer state transition for {explorer_id}: {current:?} -> {new_state:?}"
            );
        }

        valid
    }
}

// ── Initialization ────────────────────────────────────────────────────────────

impl Orchestrator {
    /// Initialize all planets and explorers. Explorer AIs remain stopped until
    /// the orchestrator logic is started through the API.
    pub fn initialize(&mut self) -> Result<(), String> {
        log::info!("Initializing orchestrator...");

        // start all planets
        let planet_ids: Vec<u32> = self.planet_senders.keys().copied().collect();
        for planet_id in &planet_ids {
            self.planet_alive.insert(*planet_id, true);
            self.planet_states.insert(*planet_id, PlanetState::Starting);
            self.start_planet(*planet_id)?;
        }
        // mark AI as running in telemetry hub so GUI shows correct initial state
        if let Some(ref tel) = self.telemetry {
            for planet_id in &planet_ids {
                if let Some(entry) = tel.planets.read().unwrap().get(planet_id) {
                    entry
                        .ai_running
                        .store(true, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }

        // Pre-charge all planet cells before starting explorers so the first
        // explorer visit has energy available without waiting for the first tick.
        let planet_ids_for_charge: Vec<u32> = self.planet_senders.keys().copied().collect();
        for planet_id in &planet_ids_for_charge {
            for _ in 0..5 {
                let _ = self.send_sunray(*planet_id);
            }
        }

        // mark all explorers as alive and register their reply channels with planet 1
        let explorer_ids: Vec<u32> = self.explorer_senders.keys().copied().collect();
        for explorer_id in explorer_ids {
            self.explorer_alive.insert(explorer_id, true);
            self.explorer_states
                .insert(explorer_id, ExplorerState::Idle);
            self.explorer_planet.insert(explorer_id, 1);

            // Bug E fix: give planet 1 a sender so it can respond to this explorer
            if let Some(reply_tx) = self.explorer_reply_senders.get(&explorer_id).cloned() {
                if let Some(planet_tx) = self.planet_senders.get(&1).cloned() {
                    let _ = planet_tx.send(OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id,
                        new_sender: reply_tx,
                    });
                }
            }
        }

        log::info!("Orchestrator initialization complete");
        Ok(())
    }
}
