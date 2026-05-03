use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use common_game::components::forge::Forge;
use common_game::components::planet::DummyPlanetState;
use common_game::components::rocket::Rocket;
use crossbeam_channel::{Receiver, Sender};

use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use crate::explorer::Bag;
use crate::telemetry::TelemetryHub;

pub struct Orchestrator {
    planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_receiver: Receiver<PlanetToOrchestrator>,
    #[allow(dead_code)]
    explorer_sender: Sender<OrchestratorToExplorer>,
    #[allow(dead_code)]
    explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    forge: Forge,
    running: Arc<AtomicBool>,
    step_rx: Option<Receiver<()>>,
    telemetry: Option<TelemetryHub>,
    tick: u64,
}

pub struct StopHandle{
    running: Arc<AtomicBool>,
}

// ── Core ──────────────────────────────────────────────────────────────────────

impl Orchestrator {

    pub fn new(
        planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
        planet_receiver: Receiver<PlanetToOrchestrator>,
        explorer_sender: Sender<OrchestratorToExplorer>,
        explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    ) -> Result<Self, String> {
        let forge = Forge::new()?;
        let running = Arc::new(AtomicBool::new(false));

        Ok(Self {
            planet_senders,
            planet_receiver,
            explorer_sender,
            explorer_receiver,
            forge,
            running,
            step_rx: None,
            telemetry: None,
            tick: 0,
        })
    }

    /// Builder: attach a step channel for single-step execution when paused.
    pub fn with_step_channel(mut self, step_rx: Receiver<()>) -> Self {
        self.step_rx = Some(step_rx);
        self
    }

    /// Builder: attach a telemetry hub for real-time state observation.
    pub fn with_telemetry(mut self, telemetry: TelemetryHub) -> Self {
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

        std::thread::spawn(move || {
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
                    let _ = self.send_sunray(planet_id);
                }

                // maybe send asteroid (~4% chance per tick)
                if rand::random::<u8>() < 10 {
                    let keys: Vec<u32> = self.planet_senders.keys().copied().collect();
                    if !keys.is_empty() {
                        let idx = (rand::random::<u32>() as usize) % keys.len();
                        let planet_id = keys[idx];
                        let _ = self.send_asteroid(planet_id);
                    }
                }

                self.handle_planet_responses();

                self.tick += 1;

                // update telemetry
                if let Some(ref tel) = telemetry {
                    tel.sim_status.set_tick(self.tick);
                }

                // sleep a bit between ticks
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });

        StopHandle { running }
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
}

// ── Planet messaging ──────────────────────────────────────────────────────────

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
            .map_err(|e| format!("Failed to send sunray: {e}"))?;

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
                 // All patterns handled above - _ is unreachable - all cases handled above
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
                // asteroid hit — planet took damage
                log::warn!("Planet {planet_id} was hit by an asteroid!");
            }
        }
    }

    // planet confirmed an explorer arrived — log success or failure
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
        // planet was destroyed by an asteroid
        // log it
        log::warn!("Planet {planet_id} has been destroyed by an asteroid");

        // remove planet_id from planet_senders
        self.planet_senders.remove(&planet_id);
        
    }

    // planet confirmed an explorer departed — log success or failure
    fn handle_outgoing_explorer_response(&self, planet_id : u32, explorer_id : u32, res: Result<(), String>) {
        match res{
            Ok(_)=> {log::info!("Explorer {explorer_id} left planet {planet_id}")},
            Err(e)=> {log::warn!("Explorer {explorer_id} failed to leave planet {planet_id}: {e}")}
        }
    }

    // planet confirmed its AI has started
    fn handle_start_planet_ai_result(&self, planet_id : u32) {
        log::info!("Planet AI started for the planet {planet_id}");
    }

    // planet confirmed its AI has stopped
    fn handle_stop_planet_ai_result(&self, planet_id : u32) {
        log::info!("Planet AI stopped for the planet {planet_id}");
    }

    // planet confirmed it has stopped
    fn handle_stopped(&self, planet_id : u32) {
        log::info!("Planet {planet_id} stopped");
    }
}