#[allow(unused)]

use common_game::components::resource::{BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest, Generator, GenericResource };
use common_game::components::planet::{Planet, PlanetAI, PlanetState, PlanetType, DummyPlanetState};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use crossbeam_channel::{Sender, Receiver};
use common_game::utils::ID;

struct RustEzeAI {}

impl RustEzeAI {
    pub fn new() -> Self {
        RustEzeAI {}
    }
}

impl PlanetAI for RustEzeAI {
    fn handle_sunray(&mut self, state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator, sunray: Sunray) {
        state.charge_cell(sunray);
    }

    fn handle_asteroid(&mut self, _state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator) -> Option<Rocket> {
        //this planet cannot have rockets
        None
    }

    fn handle_internal_state_req(&mut self, state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator) -> DummyPlanetState {
        state.to_dummy()
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, generator: &Generator, combinator: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        match msg{
            ExplorerToPlanet::SupportedResourceRequest { .. } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes()
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { .. } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list:  combinator.all_available_recipes()
                })
            }
            ExplorerToPlanet::GenerateResourceRequest { explorer_id: _, resource }=> {
                if let Some((charged_cell, _)) = state.full_cell() {

                    let res = generator.try_make(resource, charged_cell);

                    match res {
                        Ok(resource) => { Some(PlanetToExplorer::GenerateResourceResponse { resource: Some(resource) }) }
                        Err(error) => { log::error!("{}", error); None }
                    }
                }
                else{
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                }
            }
            ExplorerToPlanet::CombineResourceRequest { .. } => {
                //this planet cannot combine complex resources
                None
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.to_dummy().charged_cells_count as u32
                })
            }
        }
    }

    fn on_explorer_arrival(&mut self, _state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator, explorer_id: ID) {
        log::info!("Explorer {} has landed on Rust-Eze", explorer_id);
    }

    fn on_explorer_departure(&mut self, _state: &mut PlanetState, _generator: &Generator, _combinator: &Combinator, explorer_id: ID) {
        log::info!("Explorer {} has left Rust-Eze", explorer_id);
    }

    fn on_start(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        log::info!("Rust Eze starting: planet id: {}", state.id());
    }

    fn on_stop(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        log::info!("Rust Eze stopping: planet id: {}", state.id());
    }
}

pub fn create_planet(
    id: ID,
    r_orchestrator: Receiver<OrchestratorToPlanet>,
    s_orchestrator: Sender<PlanetToOrchestrator>,
    r_explorer: Receiver<ExplorerToPlanet>,
) -> Result<Planet, String> {
    Planet::new(
        id,
        PlanetType::D,
        Box::new(RustEzeAI::new()),
        vec![
            BasicResourceType::Carbon,
            BasicResourceType::Hydrogen,
            BasicResourceType::Oxygen,
            BasicResourceType::Silicon
        ],
        vec![],
        (r_orchestrator, s_orchestrator),
        r_explorer
    )
}