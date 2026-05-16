//! Luna4 core implementation module

pub(crate) mod ai;
pub(crate) mod stats;

pub use ai::Luna4AI;
pub use ai::Luna4Stats;

use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};

use crate::planet_ai::energy::EnergyManager;
use crate::planet_ai::errors::SkycartelError;
use crate::planet_ai::resources::ResourceManager;
use crate::planet_telemetry::{SharedPlanetStats, StatsTrackingAI};

use common_game::components::planet::{Planet, PlanetAI, PlanetState, DummyPlanetState, PlanetType};
use common_game::components::resource::{BasicResourceType, Combinator, Generator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;

/// Unique identifier for Luna4 planets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Luna4Id(ID);

impl Luna4Id {
    pub fn new(id: ID) -> Self { Self(id) }
    pub fn as_u32(&self) -> u32 { self.0 }
}

impl From<ID> for Luna4Id {
    fn from(id: ID) -> Self { Self::new(id) }
}

impl std::fmt::Display for Luna4Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Luna4#{}", self.0)
    }
}

struct Luna4AIWrapper(Arc<RwLock<Luna4AI>>);

impl PlanetAI for Luna4AIWrapper {
    fn handle_sunray(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, sunray: Sunray) {
        self.0.write().unwrap().handle_sunray(state, generator, combinator, sunray)
    }
    fn handle_asteroid(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> Option<Rocket> {
        self.0.write().unwrap().handle_asteroid(state, generator, combinator)
    }
    fn handle_internal_state_req(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator) -> DummyPlanetState {
        self.0.write().unwrap().handle_internal_state_req(state, generator, combinator)
    }
    fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        self.0.write().unwrap().handle_explorer_msg(state, generator, combinator, msg)
    }
    fn on_explorer_arrival(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, explorer_id: u32) {
        self.0.write().unwrap().on_explorer_arrival(state, generator, combinator, explorer_id)
    }
    fn on_explorer_departure(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, explorer_id: u32) {
        self.0.write().unwrap().on_explorer_departure(state, generator, combinator, explorer_id)
    }
    fn on_start(&mut self, state: &PlanetState, generator: &Generator, combinator: &Combinator) {
        self.0.write().unwrap().on_start(state, generator, combinator)
    }
    fn on_stop(&mut self, state: &PlanetState, generator: &Generator, combinator: &Combinator) {
        self.0.write().unwrap().on_stop(state, generator, combinator)
    }
}

/// Main Luna4 planet implementation
#[allow(dead_code)]
pub struct Luna4 {
    id: Luna4Id,
    energy: EnergyManager,
    resources: ResourceManager,
    ai_for_visualizer: Option<Arc<RwLock<Luna4AI>>>,
}

impl Luna4 {
    pub fn new(id: ID) -> Result<Self, SkycartelError> {
        let id = Luna4Id::new(id);
        let energy = EnergyManager::new(5)?;
        let resources = ResourceManager::new();
        Ok(Self { id, energy, resources, ai_for_visualizer: None })
    }

    pub fn create_planet(
        mut self,
        rx_orchestrator: Receiver<OrchestratorToPlanet>,
        tx_orchestrator: Sender<PlanetToOrchestrator>,
        rx_explorer: Receiver<ExplorerToPlanet>,
    ) -> Result<Planet, SkycartelError> {
        let ai = Luna4AI::new(self.id.as_u32());
        let ai_arc = Arc::new(RwLock::new(ai));
        self.ai_for_visualizer = Some(ai_arc.clone());
        let ai: Box<dyn PlanetAI> = Box::new(Luna4AIWrapper(ai_arc));
        Planet::new(
            self.id.as_u32(),
            PlanetType::D,
            ai,
            vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen, BasicResourceType::Carbon, BasicResourceType::Silicon],
            vec![],
            (rx_orchestrator, tx_orchestrator),
            rx_explorer,
        )
        .map_err(SkycartelError::PlanetConstruction)
    }

    pub fn create_planet_with_stats(
        self,
        rx_orchestrator: Receiver<OrchestratorToPlanet>,
        tx_orchestrator: Sender<PlanetToOrchestrator>,
        rx_explorer: Receiver<ExplorerToPlanet>,
    ) -> Result<(Planet, Arc<Mutex<SharedPlanetStats>>), SkycartelError> {
        let ai = Luna4AI::new(self.id.as_u32());
        let ai_arc = Arc::new(RwLock::new(ai));
        let wrapper = Luna4AIWrapper(ai_arc);
        let stats = Arc::new(Mutex::new(SharedPlanetStats::default()));
        let tracking_ai = StatsTrackingAI { inner: wrapper, stats: Arc::clone(&stats) };
        let planet = Planet::new(
            self.id.as_u32(),
            PlanetType::D,
            Box::new(tracking_ai),
            vec![BasicResourceType::Hydrogen, BasicResourceType::Oxygen, BasicResourceType::Carbon, BasicResourceType::Silicon],
            vec![],
            (rx_orchestrator, tx_orchestrator),
            rx_explorer,
        )
        .map_err(SkycartelError::PlanetConstruction)?;
        Ok((planet, stats))
    }

    pub fn get_current_resources(&self) -> Option<ai::AvailableResources> {
        self.ai_for_visualizer
            .as_ref()
            .and_then(|ai| ai.read().ok())
            .map(|ai| ai.get_current_resources())
    }
}
