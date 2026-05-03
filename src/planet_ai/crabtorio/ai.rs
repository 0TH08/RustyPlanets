use common_game::components::planet::{PlanetAI, PlanetState};
use common_game::components::resource::BasicResourceType::{self, *};
use common_game::components::resource::ComplexResourceRequest::{
    AIPartner, Diamond, Dolphin, Life, Robot, Water as WaterRequest,
};
use common_game::components::resource::{
    BasicResource, Combinator, ComplexResource, ComplexResourceType, Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;
use std::collections::HashSet;

pub struct AI;

impl PlanetAI for AI {
    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> common_game::components::planet::DummyPlanetState {
        state.to_dummy()
    }

    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: common_game::components::sunray::Sunray,
    ) {
        state.charge_cell(sunray);
    }

    fn handle_asteroid(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        None
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: HashSet::from([ComplexResourceType::Water]),
                })
            }
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes()
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _,
                resource,
            } => {
                if let Some((cell, _)) = state.full_cell() {
                    let res = match resource {
                        Hydrogen => generator.make_hydrogen(cell).ok().map(BasicResource::Hydrogen),
                        Oxygen => generator.make_oxygen(cell).ok().map(BasicResource::Oxygen),
                        Carbon => generator.make_carbon(cell).ok().map(BasicResource::Carbon),
                        Silicon => generator.make_silicon(cell).ok().map(BasicResource::Silicon),
                    };
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: res })
                } else {
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                }
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id: _, msg } => match msg {
                WaterRequest(h, o) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: {
                        match combinator.make_water(h, o, state.cell_mut(0)) {
                            Ok(water) => Ok(ComplexResource::Water(water)),
                            Err((s, h, o)) => Err((
                                s,
                                GenericResource::BasicResources(BasicResource::Hydrogen(h)),
                                GenericResource::BasicResources(BasicResource::Oxygen(o)),
                            )),
                        }
                    },
                }),
                Diamond(c1, c2) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "Diamond recipe not available on this planet".to_string(),
                        GenericResource::BasicResources(BasicResource::Carbon(c1)),
                        GenericResource::BasicResources(BasicResource::Carbon(c2)),
                    )),
                }),
                Life(w, c) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "Life recipe not available on this planet".to_string(),
                        GenericResource::ComplexResources(ComplexResource::Water(w)),
                        GenericResource::BasicResources(BasicResource::Carbon(c)),
                    )),
                }),
                Robot(s, l) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "Robot recipe not available on this planet".to_string(),
                        GenericResource::BasicResources(BasicResource::Silicon(s)),
                        GenericResource::ComplexResources(ComplexResource::Life(l)),
                    )),
                }),
                Dolphin(w, l) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "Dolphin recipe not available on this planet".to_string(),
                        GenericResource::ComplexResources(ComplexResource::Water(w)),
                        GenericResource::ComplexResources(ComplexResource::Life(l)),
                    )),
                }),
                AIPartner(r, d) => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "AIPartner recipe not available on this planet".to_string(),
                        GenericResource::ComplexResources(ComplexResource::Robot(r)),
                        GenericResource::ComplexResources(ComplexResource::Diamond(d)),
                    )),
                }),
            },
        }
    }
}

use common_game::components::planet::Planet;
use common_game::components::planet::PlanetType;
use crossbeam_channel::{Receiver, Sender};

pub fn create_planet(
    id: ID,
    orchestrator_channels: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>),
    explorers_receiver: Receiver<ExplorerToPlanet>,
) -> Result<Planet, String> {
    Planet::new(
        id,
        PlanetType::B,
        Box::new(AI),
        vec![BasicResourceType::Hydrogen],
        vec![
            ComplexResourceType::AIPartner,
            ComplexResourceType::Diamond,
            ComplexResourceType::Dolphin,
            ComplexResourceType::Life,
            ComplexResourceType::Robot,
            ComplexResourceType::Water,
        ],
        orchestrator_channels,
        explorers_receiver,
    )
}
