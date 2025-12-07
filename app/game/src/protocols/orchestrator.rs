use crate::components::asteroid::Asteroid;
use crate::components::forge::Forge;
use crate::components::planet::Planet;
use crate::components::sunray::Sunray;
use crate::logging::LogEvent;
use crate::protocols::messages::{
    OrchestratorToPlanet, PlanetToOrchestrator,
};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::time::Duration;

pub struct Orchestrator {
    forge: Forge,
    planets: HashMap<u32, (Sender<OrchestratorToPlanet>, Receiver<PlanetToOrchestrator>)>,
}

impl Orchestrator {
    pub fn new() -> Result<Self, String> {
        let forge = Forge::new()?;
        Ok(Self {
            forge,
            planets: HashMap::new(),
        })
    }

    pub fn register_planet_channels(
        &mut self,
        planet_id: u32,
        tx_to_planet: Sender<OrchestratorToPlanet>,
        rx_from_planet: Receiver<PlanetToOrchestrator>,
    ) {
        self.planets.insert(planet_id, (tx_to_planet, rx_from_planet));
    }

    pub fn create_sunray(&self) -> Sunray {
        self.forge.generate_sunray()
    }

    pub fn create_asteroid(&self) -> Asteroid {
        self.forge.generate_asteroid()
    }

    pub fn send_sunray(&self, s: Sunray, planet_id: u32) -> Result<(), String> {
        if let Some((tx, _rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::Sunray(s))
                .map_err(|e| format!("send_sunray send error: {}", e))
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn send_asteroid(&self, a: Asteroid, planet_id: u32) -> Result<(), String> {
        if let Some((tx, _rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::Asteroid(a))
                .map_err(|e| format!("send_asteroid send error: {}", e))
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn start_planet_ai(&self, planet_id: u32) -> Result<(), String> {
        if let Some((tx, rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::StartPlanetAI)
                .map_err(|e| format!("start send error: {}", e))?;
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(PlanetToOrchestrator::StartPlanetAIResult { .. }) => Ok(()),
                Ok(other) => Err(format!("unexpected response: {:?}", other)),
                Err(e) => Err(format!("start_planet_ai recv error: {}", e)),
            }
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn stop_planet_ai(&self, planet_id: u32) -> Result<(), String> {
        if let Some((tx, rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::StopPlanetAI)
                .map_err(|e| format!("stop send error: {}", e))?;
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(PlanetToOrchestrator::StopPlanetAIResult { .. }) => Ok(()),
                Ok(other) => Err(format!("unexpected response: {:?}", other)),
                Err(e) => Err(format!("stop_planet_ai recv error: {}", e)),
            }
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn kill_planet(&self, planet_id: u32) -> Result<(), String> {
        if let Some((tx, rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::KillPlanet)
                .map_err(|e| format!("kill send error: {}", e))?;
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(PlanetToOrchestrator::KillPlanetResult { .. }) => Ok(()),
                Ok(other) => Err(format!("unexpected response: {:?}", other)),
                Err(e) => Err(format!("kill_planet recv error: {}", e)),
            }
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn request_internal_state(&self, planet_id: u32) -> Result<(), String> {
        if let Some((tx, rx)) = self.planets.get(&planet_id) {
            tx.send(OrchestratorToPlanet::InternalStateRequest)
                .map_err(|e| format!("internal state send error: {}", e))?;
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(PlanetToOrchestrator::InternalStateResponse { .. }) => Ok(()),
                Ok(other) => Err(format!("unexpected response: {:?}", other)),
                Err(e) => Err(format!("request_internal_state recv error: {}", e)),
            }
        } else {
            Err(format!("unknown planet {}", planet_id))
        }
    }

    pub fn log_event(&self, event: LogEvent) {
        log::info!("{:?}", event);
    }
}
