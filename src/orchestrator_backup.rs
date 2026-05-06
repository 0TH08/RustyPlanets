use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use common_game::components::forge::Forge;
use common_game::components::planet::DummyPlanetState;
use common_game::components::rocket::Rocket;
use crossbeam_channel::{Receiver, Sender};

use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use crate::explorer::Bag;
use crate::telemetry::TelemetryHub;

/// Validates planet message sequence protocol.
/// Planets must follow: Idle -> Starting -> Running -> Stopping -> Idle
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlanetState {
    Idle,
    Starting,
    Running,
    Stopping,
}

/// Validates explorer message sequence protocol.
/// Explorers must follow: Idle -> Starting -> Running -> Stopping -> Idle
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
    forge: Forge,
    running: Arc<AtomicBool>,
    step_rx: Option<Receiver<()>>,
    telemetry: Option<Arc<TelemetryHub>>,
    tick: u64,
    explorer_planet: HashMap<u32, u32>,
    explorer_alive: HashMap<u32, bool>,
    planet_alive: HashMap<u32, bool>,
    // Message sequence validation state
    planet_states: HashMap<u32, PlanetState>,
    explorer_states: HashMap<u32, ExplorerState>,
}

pub struct StopHandle{
    running: Arc<AtomicBool>,
    tick: u64,
}

// â”€â”€ Core â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {

    pub fn new(
        planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
        planet_explorer_senders: HashMap<u32, Sender<ExplorerToPlanet>>,
        planet_receiver: Receiver<PlanetToOrchestrator>,
        explorer_senders: HashMap<u32, Sender<OrchestratorToExplorer>>,
        explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    ) -> Result<Self, String> {
        let forge = Forge::new()?;
        let running = Arc::new(AtomicBool::new(false));

        let planet_alive: HashMap<u32, bool> = planet_senders.keys().map(|&id| (id, true)).collect();
        let planet_states: HashMap<u32, PlanetState> = planet_senders.keys().map(|&id| (id, PlanetState::Idle)).collect();
        let explorer_states: HashMap<u32, ExplorerState> = HashMap::new();

        Ok(Self {
            planet_senders,
            planet_explorer_senders,
            planet_receiver,
            explorer_senders,
            explorer_receiver,
            forge,
            running,
            step_rx: None,
            telemetry: None,
            tick: 0,
            explorer_planet: HashMap::new(),
            explorer_alive: HashMap::new(),
            planet_alive,
            planet_states,
            explorer_states,
        })
    }

    /// Register a new explorer with the orchestrator.
    pub fn register_explorer(&mut self, explorer_id: u32, sender: Sender<OrchestratorToExplorer>, planet_id: u32) {
        log::info!("Registering explorer {explorer_id} on planet {planet_id}");
        self.explorer_senders.insert(explorer_id, sender);
        self.explorer_planet.insert(explorer_id, planet_id);
        self.explorer_alive.insert(explorer_id, true);
    }

    /// Register a new planet sender after construction.
    pub fn register_planet(&mut self, planet_id: u32, sender: Sender<OrchestratorToPlanet>, explorer_sender: Sender<ExplorerToPlanet>) {
        log::info!("Registering planet {planet_id}");
        self.planet_senders.insert(planet_id, sender);
        self.planet_explorer_senders.insert(planet_id, explorer_sender);
        self.planet_alive.insert(planet_id, true);
    }

    /// Builder: attach a step channel for single-step execution when paused.
    pub fn with_step_channel(mut self, step_rx: Receiver<()>) -> Self {
        self.step_rx = Some(step_rx);
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
        self.tick
    }

    /// Spawns the main simulation loop on a new thread.
    ///
    /// The loop runs until `StopHandle::stop()` is called or the running
    /// flag is set to false. Returns a handle that can control the loop.
    pub fn run(mut self) -> StopHandle {
        let running = Arc::clone(&self.running);
        let running_thread = Arc::clone(&running);
        let step_rx = self.step_rx.take();
        let telemetry = self.telemetry.take();
        let initial_tick = self.tick;

        std::thread::spawn(move || {
            // â”€â”€ Initialization phase â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
            // start all planet AIs
            let planet_ids: Vec<u32> = self.planet_senders.keys().copied().collect();
            for pid in &planet_ids {
                if let Some(sender) = self.planet_senders.get(pid) {
                    if let Err(e) = sender.send(OrchestratorToPlanet::StartPlanetAI) {
                        log::error!("Failed to start planet {pid}: {e}");
                    }
                }
            }
            log::info!("Sent StartPlanetAI to {} planets", planet_ids.len());

            // start all explorer AIs
            let explorer_ids: Vec<u32> = self.explorer_senders.keys().copied().collect();
            for eid in &explorer_ids {
                if let Some(sender) = self.explorer_senders.get(eid) {
                    if let Err(e) = sender.send(OrchestratorToExplorer::StartExplorerAI) {
                        log::error!("Failed to start explorer {eid}: {e}");
                    }
                }
            }
            log::info!("Sent StartExplorerAI to {} explorers", explorer_ids.len());

            // wait briefly for initialization acks
            std::thread::sleep(std::time::Duration::from_millis(200));

            // drain any initialization responses
            self.handle_planet_responses();
            self.handle_explorer_responses();

            // â”€â”€ Main simulation loop â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
            while running_thread.load(std::sync::atomic::Ordering::SeqCst) {
                // if a step channel exists and we're paused, block until a step arrives
                if let Some(ref rx) = step_rx {
                    while !running_thread.load(std::sync::atomic::Ordering::SeqCst) {
                        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                            Ok(()) => {
                                // execute exactly one tick, then return to paused
                                break;
                            }
                            Err(_) => continue,
                        }
                    }
                    // re-check: if still paused after step (e.g. stop was called), exit
                    if !running_thread.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                }

                // send sunray to each planet
                for &planet_id in self.planet_senders.keys() {
                    if let Err(e) = self.send_sunray(planet_id) {
                        log::warn!("Failed to send sunray to planet {planet_id}: {e}");
                    }
                }

                // maybe send asteroid (~5% chance per tick)
                if rand::random::<f32>() < 0.05 {
                    let keys: Vec<u32> = self.planet_senders.keys().copied().collect();
                    if !keys.is_empty() {
                        let idx = (rand::random::<u32>() as usize) % keys.len();
                        let planet_id = keys[idx];
                        if let Err(e) = self.send_asteroid(planet_id) {
                            log::warn!("Failed to send asteroid to planet {planet_id}: {e}");
                        }
                    }
                }

                self.handle_planet_responses();
                self.handle_explorer_responses();

                self.tick += 1;

                // update telemetry
                if let Some(ref tel) = telemetry {
                    tel.sim_status.set_tick(self.tick);
                }

                // sleep a bit between ticks
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });

        StopHandle { running, tick: initial_tick }
    }
}

impl StopHandle {
    /// Signal the simulation loop to stop after the current tick.
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Signal the simulation loop to resume (or start) running.
    pub fn start(&self) {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if the simulation loop is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get the tick count at the time the stop handle was created.
    pub fn tick(&self) -> u64 {
        self.tick
    }
}

// â”€â”€ Planet messaging â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {

    pub fn send_sunray(&self, planet_id : u32) -> Result<(), String> {
        //1- Get the sender for this planet
        let sender = self.planet_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        //2- Create sunray
        let sunray = self.forge.generate_sunray();

        //3- Send the sunray
        sender.send(OrchestratorToPlanet::Sunray(sunray))
            .map_err(|e| {
                // channel disconnected â€” planet has crashed
                log::error!("Planet {planet_id} channel disconnected (crashed?)");
                format!("Failed to send sunray: {e}")
            })?;

        //4- Return Ok
        Ok(())
    }

    pub fn send_asteroid(&self, planet_id : u32) -> Result<(), String> {
        //1- Get the sender for this planet
        let sender = self.planet_senders
            .get(&planet_id)
            .ok_or_else(|| format!("No sender found for planet {planet_id}"))?;

        //2- Create asteroid
        let asteroid = self.forge.generate_asteroid();

        //3- Send it
        sender.send(OrchestratorToPlanet::Asteroid(asteroid))
            .map_err(|e| format!("Failed to send asteroid: {e}"))?;

        //Return Ok
        Ok(())
    }
}

// â”€â”€ Planet lifecycle â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {
    // â”€â”€ Message Sequence Validation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    fn validate_planet_message(&mut self, planet_id: u32, expected_state: PlanetState, msg_name: &str) -> bool {
        let state = self.planet_states.entry(planet_id).or_insert(PlanetState::Idle);
        if *state != expected_state {
            log::warn!(
                "Protocol violation: Planet {planet_id} in state {:?}, expected {:?} for {msg_name}",
                state, expected_state, msg_name
            );
            return false;
        }
        true
    }

    fn transition_planet_state(&mut self, planet_id: u32, new_state: PlanetState) {
        self.planet_states.insert(planet_id, new_state);
        log::debug!("Planet {planet_id} transitioned to {:?}", new_state);
    }

    fn validate_explorer_message(&mut self, explorer_id: u32, expected_state: ExplorerState, msg_name: &str) -> bool {
        let state = self.explorer_states.entry(explorer_id).or_insert(ExplorerState::Idle);
        if *state != expected_state {
            log::warn!(
                "Protocol violation: Explorer {explorer_id} in state {:?}, expected {:?} for {msg_name}",
                state, expected_state, msg_name
            );
            return false;
        }
        true
    }

    fn transition_explorer_state(&mut self, explorer_id: u32, new_state: ExplorerState) {
        self.explorer_states.insert(explorer_id, new_state);
        log::debug!("Explorer {explorer_id} transitioned to {:?}", new_state);
    }

    pub fn start_planet(&self, planet_id: u32) -> Result<(), String> {
        let sender = self.planet_senders.get(&planet_id)
            .ok_or_else(|| format!("Planet {planet_id} not found"))?;
        sender.send(OrchestratorToPlanet::StartPlanetAI)
            .map_err(|e| format!("Failed to start planet {planet_id}: {e}"))?;
        // Transition state to Starting
        self.planet_states.insert(planet_id, PlanetState::Starting);
        log::info!("Sent StartPlanetAI to planet {planet_id}");
        Ok(())
    }

    pub fn stop_planet(&self, planet_id: u32) -> Result<(), String> {
        let sender = self.planet_senders.get(&planet_id)
            .ok_or_else(|| format!("Planet {planet_id} not found"))?;
        sender.send(OrchestratorToPlanet::StopPlanetAI)
            .map_err(|e| format!("Failed to stop planet {planet_id}: {e}"))?;
        // Transition state to Stopping
        self.planet_states.insert(planet_id, PlanetState::Stopping);
        log::info!("Sent StopPlanetAI to planet {planet_id}");
        Ok(())
    }

    pub fn kill_planet(&self, planet_id: u32) -> Result<(), String> {
        let sender = self.planet_senders.get(&planet_id)
            .ok_or_else(|| format!("Planet {planet_id} not found"))?;
        sender.send(OrchestratorToPlanet::KillPlanet)
            .map_err(|e| format!("Failed to kill planet {planet_id}: {e}"))?;
        log::warn!("Sent KillPlanet to planet {planet_id}");
        Ok(())
    }

    /// Start all registered planets.
    pub fn start_all_planets(&self) {
        let planet_ids: Vec<u32> = self.planet_senders.keys().copied().collect();
        for pid in planet_ids {
            if let Err(e) = self.start_planet(pid) {
                log::error!("{e}");
            }
        }
    }

    /// Stop all registered planets.
    pub fn stop_all_planets(&self) {
        let planet_ids: Vec<u32> = self.planet_senders.keys().copied().collect();
        for pid in planet_ids {
            if let Err(e) = self.stop_planet(pid) {
                log::error!("{e}");
            }
        }
    }
}

// â”€â”€ Explorer lifecycle â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {
    pub fn start_explorer(&self, explorer_id: u32) -> Result<(), String> {
        let sender = self.explorer_senders.get(&explorer_id)
            .ok_or_else(|| format!("Explorer {explorer_id} not found"))?;
        sender.send(OrchestratorToExplorer::StartExplorerAI)
            .map_err(|e| format!("Failed to start explorer {explorer_id}: {e}"))?;
        log::info!("Sent StartExplorerAI to explorer {explorer_id}");
        Ok(())
    }

    pub fn stop_explorer(&self, explorer_id: u32) -> Result<(), String> {
        let sender = self.explorer_senders.get(&explorer_id)
            .ok_or_else(|| format!("Explorer {explorer_id} not found"))?;
        sender.send(OrchestratorToExplorer::StopExplorerAI)
            .map_err(|e| format!("Failed to stop explorer {explorer_id}: {e}"))?;
        log::info!("Sent StopExplorerAI to explorer {explorer_id}");
        Ok(())
    }

    pub fn kill_explorer(&self, explorer_id: u32) -> Result<(), String> {
        let sender = self.explorer_senders.get(&explorer_id)
            .ok_or_else(|| format!("Explorer {explorer_id} not found"))?;
        sender.send(OrchestratorToExplorer::KillExplorer)
            .map_err(|e| format!("Failed to kill explorer {explorer_id}: {e}"))?;
        log::warn!("Sent KillExplorer to explorer {explorer_id}");
        Ok(())
    }

    /// Start all registered explorers.
    pub fn start_all_explorers(&self) {
        let explorer_ids: Vec<u32> = self.explorer_senders.keys().copied().collect();
        for eid in explorer_ids {
            if let Err(e) = self.start_explorer(eid) {
                log::error!("{e}");
            }
        }
    }

    /// Kill all explorers currently on a specific planet.
    pub fn kill_explorers_on_planet(&self, planet_id: u32) {
        let to_kill: Vec<u32> = self.explorer_planet.iter()
            .filter(|(_, &pid)| pid == planet_id)
            .map(|(&eid, _)| eid)
            .collect();
        for eid in to_kill {
            if let Err(e) = self.kill_explorer(eid) {
                log::error!("{e}");
            }
        }
    }
}

impl Orchestrator {

    fn handle_planet_responses(&mut self) {
        // try_recv() â€” non-blocking, returns immediately if nothing is there
        loop {
            match self.planet_receiver.try_recv() {
                Ok(msg) => {
                    match msg {
                        PlanetToOrchestrator::SunrayAck { planet_id } => {
                            self.handle_sunray_ack(planet_id);
                        }
                        PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => {
                            // planet responded to asteroid â€” did it have a rocket to deflect?
                            self.handle_asteroid_ack(planet_id, rocket);
                        }
                        PlanetToOrchestrator::IncomingExplorerResponse { planet_id, explorer_id, res } => {
                            self.handle_incoming_explorer_response(planet_id, explorer_id, res);
                        }
                        PlanetToOrchestrator::InternalStateResponse { planet_id, planet_state } => {
                            self.handle_internal_state_response(planet_id, planet_state);
                        }
                        PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                            self.handle_kill_planet_result(planet_id);
                        }
                        PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, explorer_id, res } => {
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
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    break; // no more messages
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // planet channel disconnected â€” all planets have crashed or been killed
                    log::error!("Planet channel disconnected â€” all planets have disconnected");
                    // mark all planets as dead
                    for &pid in self.planet_senders.keys() {
                        self.planet_alive.insert(pid, false);
                    }
                    break;
                }
            }
        }
    }

    fn handle_sunray_ack(&self, planet_id : u32) {
        // planet confirmed it got the sunray
        log::debug!("Planet {planet_id} acknowledged sunray");
    }

    fn handle_asteroid_ack(&self, planet_id : u32, rocket: Option<Rocket>) {
        match rocket {
            Some(_rocket) => {
                // planet deflected the asteroid
                log::info!("Planet {planet_id} deflected the asteroid");
            }
            None => {
                // asteroid hit â€” planet took damage
                log::warn!("Planet {planet_id} was hit by an asteroid!");
            }
        }
    }

    // planet confirmed an explorer arrived â€” log success or failure
    fn handle_incoming_explorer_response(&self, planet_id : u32, explorer_id : u32, res: Result<(), String>) {
        match res{
            Ok(_) => {log::info!("Explorer {explorer_id} arrived at planet {planet_id}")},
            Err(e)=>{log::warn!("Explorer {explorer_id} failed to arrive at planet {planet_id}: {e}")},
        }
    }

    fn handle_internal_state_response(&self, planet_id : u32, _planet_state : DummyPlanetState) {
        // log that we rcvd internal state from a planet
        log::debug!("Received internal state from planet {planet_id}");
    }

    fn handle_kill_planet_result(&mut self, planet_id : u32) {
        log::warn!("Planet {planet_id} has been destroyed by an asteroid");

        // kill all explorers currently on this planet
        let explorers_on_planet: Vec<u32> = self.explorer_planet.iter()
            .filter(|(_, &pid)| pid == planet_id)
            .map(|(&eid, _)| eid)
            .collect();
        for eid in explorers_on_planet {
            log::warn!("Killing explorer {eid} on destroyed planet {planet_id}");
            self.explorer_alive.insert(eid, false);
            self.explorer_planet.remove(&eid);
            // send kill message if sender still exists
            if let Some(sender) = self.explorer_senders.get(&eid) {
                let _ = sender.send(OrchestratorToExplorer::KillExplorer);
            }
        }

        // remove planet from all maps
        self.planet_senders.remove(&planet_id);
        self.planet_explorer_senders.remove(&planet_id);
        self.planet_alive.insert(planet_id, false);
    }

    // planet confirmed an explorer departed â€” log success or failure
    fn handle_outgoing_explorer_response(&self, planet_id : u32, explorer_id : u32, res: Result<(), String>) {
        match res{
            Ok(_)=> {log::info!("Explorer {explorer_id} left planet {planet_id}")},
            Err(e)=> {log::warn!("Explorer {explorer_id} failed to leave planet {planet_id}: {e}")}
        }
    }

    // planet confirmed its AI has started
    fn handle_start_planet_ai_result(&mut self, planet_id : u32) {
        // Validate: should be in Starting state
        if self.validate_planet_message(planet_id, PlanetState::Starting, "StartPlanetAIResult") {
            self.transition_planet_state(planet_id, PlanetState::Running);
        }
        log::info!("Planet AI started for planet {planet_id}");
        self.planet_alive.entry(planet_id).or_insert(true);
    }

    // planet confirmed its AI has stopped
    fn handle_stop_planet_ai_result(&mut self, planet_id : u32) {
        // Validate: should be in Stopping state
        if self.validate_planet_message(planet_id, PlanetState::Stopping, "StopPlanetAIResult") {
            self.transition_planet_state(planet_id, PlanetState::Idle);
        }
        log::info!("Planet AI stopped for planet {planet_id}");
        self.planet_alive.insert(planet_id, false);
    }

    // planet confirmed it has stopped
    fn handle_stopped(&self, planet_id : u32) {
        log::info!("Planet {planet_id} stopped");
    }
}

// â”€â”€ Explorer response handling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {

    fn handle_explorer_responses(&mut self) {
        loop {
            match self.explorer_receiver.try_recv() {
                Ok(msg) => {
                    match msg {
                        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id } => {
                            self.handle_start_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
                            self.handle_kill_explorer_result(explorer_id);
                        }
                        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id } => {
                            self.handle_reset_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::StopExplorerAIResult { explorer_id } => {
                            self.handle_stop_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::MovedToPlanetResult { explorer_id, planet_id } => {
                            self.handle_moved_to_planet_result(explorer_id, planet_id);
                        }
                        ExplorerToOrchestrator::CurrentPlanetResult { explorer_id, planet_id } => {
                            self.handle_current_planet_result(explorer_id, planet_id);
                        }
                        ExplorerToOrchestrator::SupportedResourceResult { explorer_id, supported_resources } => {
                            self.handle_supported_resource_result(explorer_id, supported_resources);
                        }
                        ExplorerToOrchestrator::SupportedCombinationResult { explorer_id, combination_list } => {
                            self.handle_supported_combination_result(explorer_id, combination_list);
                        }
                        ExplorerToOrchestrator::GenerateResourceResponse { explorer_id, generated } => {
                            self.handle_generate_resource_response(explorer_id, generated);
                        }
                        ExplorerToOrchestrator::CombineResourceResponse { explorer_id, generated } => {
                            self.handle_combine_resource_response(explorer_id, generated);
                        }
                        ExplorerToOrchestrator::BagContentResponse { explorer_id, bag_content } => {
                            self.handle_bag_content_response(explorer_id, bag_content);
                        }
                        ExplorerToOrchestrator::NeighborsRequest { explorer_id, current_planet_id } => {
                            self.handle_neighbors_request(explorer_id, current_planet_id);
                        }
                        ExplorerToOrchestrator::TravelToPlanetRequest { explorer_id, current_planet_id, dst_planet_id } => {
                            self.handle_travel_to_planet_request(explorer_id, current_planet_id, dst_planet_id);
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    break; // no more messages
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // explorer channel disconnected â€” all explorers have crashed or been killed
                    log::error!("Explorer channel disconnected â€” all explorers have disconnected");
                    // mark all explorers as dead
                    for &eid in self.explorer_senders.keys() {
                        self.explorer_alive.insert(eid, false);
                    }
                    break;
                }
            }
        }
    }

    fn handle_start_explorer_ai_result(&mut self, explorer_id: u32) {
        // Validate: should be in Starting state
        if self.validate_explorer_message(explorer_id, ExplorerState::Starting, "StartExplorerAIResult") {
            self.transition_explorer_state(explorer_id, ExplorerState::Running);
        }
        log::info!("Explorer {explorer_id} AI started");
        self.explorer_alive.entry(explorer_id).or_insert(true);
    }

    // planet confirmed its AI has stopped
    fn handle_stop_planet_ai_result(&mut self, planet_id : u32) {
        log::info!("Planet AI stopped for planet {planet_id}");
        self.planet_alive.insert(planet_id, false);
    }

    // planet confirmed it has stopped
    fn handle_stopped(&self, planet_id : u32) {
        log::info!("Planet {planet_id} stopped");
    }
}

// â”€â”€ Explorer response handling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

impl Orchestrator {

    fn handle_explorer_responses(&mut self) {
        loop {
            match self.explorer_receiver.try_recv() {
                Ok(msg) => {
                    match msg {
                        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id } => {
                            self.handle_start_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::KillExplorerResult { explorer_id } => {
                            self.handle_kill_explorer_result(explorer_id);
                        }
                        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id } => {
                            self.handle_reset_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::StopExplorerAIResult { explorer_id } => {
                            self.handle_stop_explorer_ai_result(explorer_id);
                        }
                        ExplorerToOrchestrator::MovedToPlanetResult { explorer_id, planet_id } => {
                            self.handle_moved_to_planet_result(explorer_id, planet_id);
                        }
                        ExplorerToOrchestrator::CurrentPlanetResult { explorer_id, planet_id } => {
                            self.handle_current_planet_result(explorer_id, planet_id);
                        }
                        ExplorerToOrchestrator::SupportedResourceResult { explorer_id, supported_resources } => {
                            self.handle_supported_resource_result(explorer_id, supported_resources);
                        }
                        ExplorerToOrchestrator::SupportedCombinationResult { explorer_id, combination_list } => {
                            self.handle_supported_combination_result(explorer_id, combination_list);
                        }
                        ExplorerToOrchestrator::GenerateResourceResponse { explorer_id, generated } => {
                            self.handle_generate_resource_response(explorer_id, generated);
                        }
                        ExplorerToOrchestrator::CombineResourceResponse { explorer_id, generated } => {
                            self.handle_combine_resource_response(explorer_id, generated);
                        }
                        ExplorerToOrchestrator::BagContentResponse { explorer_id, bag_content } => {
                            self.handle_bag_content_response(explorer_id, bag_content);
                        }
                        ExplorerToOrchestrator::NeighborsRequest { explorer_id, current_planet_id } => {
                            self.handle_neighbors_request(explorer_id, current_planet_id);
                        }
                        ExplorerToOrchestrator::TravelToPlanetRequest { explorer_id, current_planet_id, dst_planet_id } => {
                            self.handle_travel_to_planet_request(explorer_id, current_planet_id, dst_planet_id);
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    break; // no more messages
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    // explorer channel disconnected â€” all explorers have crashed or been killed
                    log::error!("Explorer channel disconnected â€” all explorers have disconnected");
                    // mark all explorers as dead
                    for &eid in self.explorer_senders.keys() {
                        self.explorer_alive.insert(eid, false);
                    }
                    break;
                }
            }
        }
    }

    fn handle_start_explorer_ai_result(&mut self, explorer_id: u32) {
        log::info!("Explorer {explorer_id} AI started");
        self.explorer_alive.entry(explorer_id).or_insert(true);
    }

    fn handle_kill_explorer_result(&mut self, explorer_id: u32) {
        // Validate: should be in Running or Stopping state
        if self.validate_explorer_message(explorer_id, ExplorerState::Running, "KillExplorerResult")
            || self.validate_explorer_message(explorer_id, ExplorerState::Stopping, "KillExplorerResult") {
            self.transition_explorer_state(explorer_id, ExplorerState::Idle);
        }
        log::warn!("Explorer {explorer_id} has been killed");
        self.explorer_alive.insert(explorer_id, false);
        self.explorer_planet.remove(&explorer_id);
    }

    fn handle_reset_explorer_ai_result(&mut self, explorer_id: u32) {
        // Validate: should be in Idle state
        if self.validate_explorer_message(explorer_id, ExplorerState::Idle, "ResetExplorerAIResult") {
            self.transition_explorer_state(explorer_id, ExplorerState::Idle);
        }
        log::info!("Explorer {explorer_id} AI reset");
    }

    fn handle_stop_explorer_ai_result(&mut self, explorer_id: u32) {
        // Validate: should be in Stopping state
        if self.validate_explorer_message(explorer_id, ExplorerState::Stopping, "StopExplorerAIResult") {
            self.transition_explorer_state(explorer_id, ExplorerState::Idle);
        }
        log::info!("Explorer {explorer_id} AI stopped (manual mode)");
    }

    fn handle_moved_to_planet_result(&mut self, explorer_id: u32, planet_id: u32) {
        // Validate: should be in Running state
        if self.validate_explorer_message(explorer_id, ExplorerState::Running, "MovedToPlanetResult") {
            // state remains Running
        }
        log::info!("Explorer {explorer_id} moved to planet {planet_id}");
        self.explorer_planet.insert(explorer_id, planet_id);
    }

    fn handle_current_planet_result(&self, explorer_id: u32, planet_id: u32) {
        log::debug!("Explorer {explorer_id} reports current planet: {planet_id}");
    }

    fn handle_supported_resource_result(&self, explorer_id: u32, resources: std::collections::HashSet<common_game::components::resource::BasicResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported resources: {resources:?}");
    }

    fn handle_supported_combination_result(&self, explorer_id: u32, combos: std::collections::HashSet<common_game::components::resource::ComplexResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported combinations: {combos:?}");
    }

    fn handle_generate_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} generated resource successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to generate resource: {e}"),
        }
    }

    fn handle_combine_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} combined resource successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to combine resource: {e}"),
        }
    }

    fn handle_bag_content_response(&self, explorer_id: u32, bag_content: BagSummary) {
        log::debug!("Explorer {explorer_id} bag content: {bag_content:?}");
    }

    fn handle_neighbors_request(&mut self, explorer_id: u32, current_planet_id: u32) {
        // Validate: should be in Running state
        if self.validate_explorer_message(explorer_id, ExplorerState::Running, "NeighborsRequest") {
            // state remains Running
        }
        // Return list of all alive planets except current
        let neighbors: Vec<u32> = self.planet_alive.iter()
            .filter(|(&pid, &alive)| alive && pid != current_planet_id)
            .map(|(&pid, _)| pid)
            .collect();
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            let _ = sender.send(OrchestratorToExplorer::NeighborsResponse { neighbors });
        }
    }

    fn handle_travel_to_planet_request(&mut self, explorer_id: u32, current_planet_id: u32, dst_planet_id: u32) {
        // Validate: should be in Running state
        if self.validate_explorer_message(explorer_id, ExplorerState::Running, "TravelToPlanetRequest") {
            // state remains Running
        }
        // Check if destination planet is alive
        let planet_alive = self.planet_alive.get(&dst_planet_id).copied().unwrap_or(false);
        if !planet_alive {
            log::warn!("Explorer {explorer_id} tried to travel to dead planet {dst_planet_id}");
            if let Some(sender) = self.explorer_senders.get(&explorer_id) {
                let _ = sender.send(OrchestratorToExplorer::TravelToPlanetResponse {
                    allowed: false,
                    reason: Some(format!("Planet {dst_planet_id} is not alive")),
                });
            }
            return;
        }
        // Get the destination planet's explorer sender
        let dst_planet_sender = match self.planet_explorer_senders.get(&dst_planet_id) {
            Some(tx) => tx.clone(),
            None => {
                log::warn!("No explorer sender found for planet {dst_planet_id}");
                if let Some(sender) = self.explorer_senders.get(&explorer_id) {
                    let _ = sender.send(OrchestratorToExplorer::TravelToPlanetResponse {
                        allowed: false,
                        reason: Some(format!("Planet {dst_planet_id} not found")),
                    });
                }
                return;
            }
        };
        // Send explorer to destination planet
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            let _ = sender.send(OrchestratorToExplorer::TravelToPlanetResponse {
                allowed: true,
                reason: None,
            });
        }
        // Notify destination planet
        if let Some(planet_sender) = self.planet_senders.get(&dst_planet_id) {
            let _ = planet_sender.send(OrchestratorToPlanet::IncomingExplorer { explorer_id });
        }
        // Update explorer's current planet
        self.explorer_planet.insert(explorer_id, dst_planet_id);
    }

    fn handle_reset_explorer_ai_result(&mut self, explorer_id: u32) {
        log::info!("Explorer {explorer_id} AI reset");
    }

    fn handle_stop_explorer_ai_result(&mut self, explorer_id: u32) {
        log::info!("Explorer {explorer_id} AI stopped (manual mode)");
    }

    fn handle_moved_to_planet_result(&mut self, explorer_id: u32, planet_id: u32) {
        log::info!("Explorer {explorer_id} moved to planet {planet_id}");
        self.explorer_planet.insert(explorer_id, planet_id);
    }

    fn handle_current_planet_result(&self, explorer_id: u32, planet_id: u32) {
        log::debug!("Explorer {explorer_id} reports current planet: {planet_id}");
    }

    fn handle_supported_resource_result(&self, explorer_id: u32, resources: std::collections::HashSet<common_game::components::resource::BasicResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported resources: {resources:?}");
    }

    fn handle_supported_combination_result(&self, explorer_id: u32, combos: std::collections::HashSet<common_game::components::resource::ComplexResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported combinations: {combos:?}");
    }

    fn handle_generate_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} generated resource successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to generate resource: {e}"),
        }
    }

    fn handle_combine_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} combined resource successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to combine resource: {e}"),
        }
    }

    fn handle_bag_content_response(&self, explorer_id: u32, _bag_content: Bag) {
        log::debug!("Explorer {explorer_id} bag content requested");
    }

    fn send_to_explorer(&self, explorer_id: u32, msg: OrchestratorToExplorer) {
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            if let Err(e) = sender.send(msg) {
                log::warn!("Failed to send message to explorer {explorer_id}: {e}");
            }
        } else {
            log::warn!("No sender found for explorer {explorer_id}");
        }
    }
}

    fn handle_reset_explorer_ai_result(&mut self, explorer_id: u32) {
        log::info!("Explorer {explorer_id} AI reset");
    }

    fn handle_stop_explorer_ai_result(&mut self, explorer_id: u32) {
        log::info!("Explorer {explorer_id} AI stopped (manual mode)");
    }

    fn handle_moved_to_planet_result(&mut self, explorer_id: u32, planet_id: u32) {
        log::info!("Explorer {explorer_id} moved to planet {planet_id}");
        self.explorer_planet.insert(explorer_id, planet_id);
    }

    fn handle_current_planet_result(&self, explorer_id: u32, planet_id: u32) {
        log::debug!("Explorer {explorer_id} reports current planet: {planet_id}");
    }

    fn handle_supported_resource_result(&self, explorer_id: u32, resources: std::collections::HashSet<common_game::components::resource::BasicResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported resources: {resources:?}");
    }

    fn handle_supported_combination_result(&self, explorer_id: u32, combos: std::collections::HashSet<common_game::components::resource::ComplexResourceType>) {
        log::debug!("Explorer {explorer_id} reports supported combinations: {combos:?}");
    }

    fn handle_generate_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} generated resource successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to generate resource: {e}"),
        }
    }

    fn handle_combine_resource_response(&self, explorer_id: u32, generated: Result<(), String>) {
        match generated {
            Ok(()) => log::debug!("Explorer {explorer_id} combined resources successfully"),
            Err(e) => log::warn!("Explorer {explorer_id} failed to combine resources: {e}"),
        }
    }

    fn handle_bag_content_response(&self, explorer_id: u32, _bag_content: Bag) {
        log::debug!("Explorer {explorer_id} bag content requested");
    }

    fn send_to_explorer(&self, explorer_id: u32, msg: OrchestratorToExplorer) {
        if let Some(sender) = self.explorer_senders.get(&explorer_id) {
            if let Err(e) = sender.send(msg) {
                log::warn!("Failed to send message to explorer {explorer_id}: {e}");
            }
        } else {
            log::warn!("No sender found for explorer {explorer_id}");
        }
    }
}
