//! Skycartel AI (RustyPlanet Type A / Carbon-only).
//!
//! Rules:
//! - Type A has 5 energy cells.
//! - Prefer resource generation using cells 0..3.
//! - Reserve last 2 cells (3..5) for asteroid defense (rockets).
//! - Only Carbon generation is supported.
//! - No resource combination supported.

use std::ops::Range;
use std::sync::{Arc, Mutex};

use common_game::components::energy_cell::EnergyCell;
use common_game::components::planet::{DummyPlanetState, PlanetAI, PlanetState};
use common_game::components::resource::{BasicResource, ComplexResource, GenericResource};
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResourceRequest, Generator,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};

use crate::planet_ai::state::SkycartelState;
use crate::planet_ai::RustyPlanetId;

const RESOURCE_RANGE_END: usize = 3; // cells 0,1,2
const DEFENSE_RANGE_START: usize = 3; // cells 3,4

fn to_generic_pair(msg: ComplexResourceRequest) -> (GenericResource, GenericResource) {
    match msg {
        ComplexResourceRequest::Water(h, o) => (
            GenericResource::BasicResources(BasicResource::Hydrogen(h)),
            GenericResource::BasicResources(BasicResource::Oxygen(o)),
        ),
        ComplexResourceRequest::Diamond(c1, c2) => (
            GenericResource::BasicResources(BasicResource::Carbon(c1)),
            GenericResource::BasicResources(BasicResource::Carbon(c2)),
        ),
        ComplexResourceRequest::Life(w, c) => (
            GenericResource::ComplexResources(ComplexResource::Water(w)),
            GenericResource::BasicResources(BasicResource::Carbon(c)),
        ),
        ComplexResourceRequest::Robot(s, l) => (
            GenericResource::BasicResources(BasicResource::Silicon(s)),
            GenericResource::ComplexResources(ComplexResource::Life(l)),
        ),
        ComplexResourceRequest::Dolphin(w, l) => (
            GenericResource::ComplexResources(ComplexResource::Water(w)),
            GenericResource::ComplexResources(ComplexResource::Life(l)),
        ),
        ComplexResourceRequest::AIPartner(r, d) => (
            GenericResource::ComplexResources(ComplexResource::Robot(r)),
            GenericResource::ComplexResources(ComplexResource::Diamond(d)),
        ),
    }
}

fn first_in_range(range: Range<usize>, predicate: impl Fn(usize) -> bool) -> Option<usize> {
    for i in range {
        if predicate(i) {
            return Some(i);
        }
    }
    None
}

fn charge_prefer_resource_cells(state: &mut PlanetState, sunray: Sunray) {
    let total = state.cells_count();
    let resource_end = RESOURCE_RANGE_END.min(total);

    if let Some(i) = first_in_range(0..resource_end, |idx| !state.cell(idx).is_charged()) {
        state.cell_mut(i).charge(sunray);
        return;
    }

    if let Some(i) = first_in_range(resource_end..total, |idx| !state.cell(idx).is_charged()) {
        state.cell_mut(i).charge(sunray);
        return;
    }

    drop(sunray);
}

pub struct SkycartelPlanetAI {
    _id: RustyPlanetId,
    inner: Arc<Mutex<SkycartelState>>,
}

impl SkycartelPlanetAI {
    pub fn new(id: RustyPlanetId) -> Self {
        Self {
            _id: id,
            inner: Arc::new(Mutex::new(SkycartelState::new(id.as_u32()))),
        }
    }

    pub fn state(&self) -> Arc<Mutex<SkycartelState>> {
        Arc::clone(&self.inner)
    }

    fn record_generation(&mut self, r: BasicResourceType) {
        if let Ok(mut s) = self.inner.lock() {
            s.record_generation(r);
        }
    }

    fn record_rocket(&mut self) {
        if let Ok(mut s) = self.inner.lock() {
            s.record_rocket_built();
        }
    }

    fn record_deflection(&mut self) {
        if let Ok(mut s) = self.inner.lock() {
            s.record_asteroid_deflected();
        }
    }

    fn record_error(&mut self) {
        if let Ok(mut s) = self.inner.lock() {
            s.record_error();
        }
    }
}

impl PlanetAI for SkycartelPlanetAI {
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        charge_prefer_resource_cells(state, sunray);
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { .. } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }

            ExplorerToPlanet::SupportedCombinationRequest { .. } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }

            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }

            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _,
                resource,
            } => {
                if resource != BasicResourceType::Carbon || !generator.contains(resource) {
                    return Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                }

                let total = state.cells_count();
                let resource_end = RESOURCE_RANGE_END.min(total);

                let charged_idx =
                    first_in_range(0..resource_end, |idx| state.cell(idx).is_charged());
                let Some(i) = charged_idx else {
                    return Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                };

                let cell: &mut EnergyCell = state.cell_mut(i);

                match generator.try_make(resource, cell) {
                    Ok(basic) => {
                        self.record_generation(BasicResourceType::Carbon);
                        Some(PlanetToExplorer::GenerateResourceResponse {
                            resource: Some(basic),
                        })
                    }
                    Err(_) => {
                        self.record_error();
                        Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                    }
                }
            }

            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _,
                msg,
            } => {
                let err_msg = "Skycartel does not support resource combination.".to_string();
                let (lhs, rhs) = to_generic_pair(msg);

                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((err_msg, lhs, rhs)),
                })
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        self.record_deflection();

        let total = state.cells_count();
        let defense_start = DEFENSE_RANGE_START.min(total);

        let charged_idx = first_in_range(defense_start..total, |idx| state.cell(idx).is_charged());
        let Some(i) = charged_idx else {
            return None;
        };

        if state.build_rocket(i).is_ok() {
            self.record_rocket();
            state.take_rocket()
        } else {
            self.record_error();
            None
        }
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        state.to_dummy()
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        explorer_id: u32,
    ) {
        if let Ok(mut s) = self.inner.lock() {
            s.register_explorer_arrival(explorer_id);
        }
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        explorer_id: u32,
    ) {
        if let Ok(mut s) = self.inner.lock() {
            s.register_explorer_departure(explorer_id);
        }
    }
}
