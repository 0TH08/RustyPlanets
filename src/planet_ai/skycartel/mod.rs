// src/planet_ai/skycartel/mod.rs
//! Skycartel planet module (RustyPlanet Type A).

pub mod ai;
pub mod stats;

use crossbeam_channel::{Receiver, Sender};

use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RustyPlanetId(pub(crate) u32);

impl RustyPlanetId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for RustyPlanetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct RustyPlanet {
    id: RustyPlanetId,
}

impl RustyPlanet {
    pub fn new(id: u32) -> Result<Self, String> {
        Ok(Self {
            id: RustyPlanetId::new(id),
        })
    }

    pub fn create_planet(
        &self,
        rx_orchestrator: Receiver<OrchestratorToPlanet>,
        tx_orchestrator: Sender<PlanetToOrchestrator>,
        rx_explorer: Receiver<ExplorerToPlanet>,
    ) -> Result<Planet, String> {
        let ai = ai::SkycartelPlanetAI::new(self.id);

        let gen_rules = vec![BasicResourceType::Carbon];
        let comb_rules: Vec<ComplexResourceType> = vec![];

        Planet::new(
            self.id.as_u32(),
            PlanetType::A,
            Box::new(ai),
            gen_rules,
            comb_rules,
            (rx_orchestrator, tx_orchestrator),
            rx_explorer,
        )
    }
}
