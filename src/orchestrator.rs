use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use common_game::components::forge::Forge;
use common_game::components::planet::DummyPlanetState;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::components::rocket::Rocket;
use crossbeam_channel::{Receiver, Sender};

use common_game::protocols::orchestrator_explorer::{ExplorerToOrchestrator, OrchestratorToExplorer};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use crate::explorer::Bag;

// ── Types ─────────────────────────────────────────────────────────────────────

pub struct Orchestrator {
    planet_senders: HashMap<u32, Sender<OrchestratorToPlanet>>,
    planet_receiver: Receiver<PlanetToOrchestrator>,
    explorer_sender: Sender<OrchestratorToExplorer>,
    explorer_receiver: Receiver<ExplorerToOrchestrator<Bag>>,
    forge: Forge,
    running: Arc<AtomicBool>,
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
        // only one can exist — returns Err if already created
        let forge = Forge::new()?;

        // false — loop hasn't started yet
        let running = Arc::new(AtomicBool::new(false));

        Ok(Self {
            planet_senders,
            planet_receiver,
            explorer_sender,
            explorer_receiver,
            forge,
            running,
        })
    }

    pub fn run(mut self) -> StopHandle {
        // set flag to true — loop is starting
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);

        // clone the Arc so the thread can share the flag
        let running = Arc::clone(&self.running);

        // second clone — this one moves into the thread, running is returned to caller
        let running_thread = Arc::clone(&running);

        std::thread::spawn(move || {
            while running_thread.load(std::sync::atomic::Ordering::SeqCst) {

                //send sunray to each planet
                for &planet_id in self.planet_senders.keys() {
                    let _ = self.send_sunray(planet_id);
                }

                // maybe send asteroid
                // ~4% chance per tick to send asteroid to a random planet
                if rand::random::<u8>() < 10 {
                    // pick a random planet id
                    let keys: Vec<u32> = self.planet_senders.keys().copied().collect();
                    if !keys.is_empty() {
                        let idx = (rand::random::<u32>() as usize) % keys.len();
                        let planet_id = keys[idx];
                        let _ = self.send_asteroid(planet_id);
                    }
                }

                self.handle_planet_responses();

                // sleep a bit between ticks
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });

        StopHandle { running }
    }
}

impl StopHandle{

    pub fn stop(&self){
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
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
                    //self.handle_stopped(planet_id);
                }
                _ => {}
            }
        }
    }

    // TODO: configure a logger to see these messages

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

    fn handle_internal_state_response(&self, planet_id : u32, planet_state : DummyPlanetState) {
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