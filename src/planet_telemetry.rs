//! Common trait for planet telemetry across different planet types.

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use serde::{Serialize, Deserialize};
use common_game::components::planet::{PlanetAI, PlanetState, DummyPlanetState};
use common_game::components::resource::{Generator, Combinator};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use common_game::utils::ID;

/// Common telemetry snapshot for all planet types.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonPlanetSnapshot {
    pub energy_cells: usize,
    pub charged_cells: usize,
    pub total_resources_generated: usize,
    pub explorer_arrivals: usize,
    pub explorer_departures: usize,
    pub rockets_built: usize,
    pub asteroids_deflected: usize,
    pub combinations_attempted: usize,
    pub combinations_succeeded: usize,
    pub errors_encountered: usize,
}

/// Individual energy cell state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellState {
    pub index: u8,
    pub charged: bool,
}

/// Generation history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationEntry {
    pub resource: String,
}

/// Trait that all planet types must implement for telemetry support.
pub trait PlanetTelemetry: Send + Sync {
    /// Returns the current telemetry snapshot.
    fn snapshot(&self) -> CommonPlanetSnapshot;
    
    /// Returns the number of explorers currently present.
    fn explorer_count(&self) -> usize;
    
    /// Returns the number of energy cells.
    fn energy_cells(&self) -> usize;
    
    /// Returns the number of charged energy cells.
    fn charged_cells(&self) -> usize;
    
    /// Returns the state of each energy cell.
    fn cell_states(&self) -> Vec<CellState>;
    
    /// Returns generation history.
    fn generation_history(&self) -> Vec<GenerationEntry>;
}

/// Wrapper for Skycartel state (existing implementation).
pub struct SkycartelTelemetry {
    pub state: Arc<std::sync::Mutex<crate::planet_ai::state::SkycartelState>>,
    pub ai_running: Arc<AtomicBool>,
}

impl PlanetTelemetry for SkycartelTelemetry {
    fn snapshot(&self) -> CommonPlanetSnapshot {
        let state = self.state.lock().unwrap();
        CommonPlanetSnapshot {
            energy_cells: state.total_cells,
            charged_cells: state.charged_cells,
            total_resources_generated: state.stats.total_resources_generated,
            explorer_arrivals: state.stats.explorer_arrivals,
            explorer_departures: state.stats.explorer_departures,
            rockets_built: state.stats.rockets_built,
            asteroids_deflected: state.stats.asteroids_deflected,
            combinations_attempted: state.stats.combinations_attempted,
            combinations_succeeded: state.stats.combinations_succeeded,
            errors_encountered: state.stats.errors_encountered,
        }
    }
    
    fn explorer_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.present_explorers.len()
    }
    
    fn energy_cells(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.total_cells
    }
    
    fn charged_cells(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.charged_cells
    }
    
    fn cell_states(&self) -> Vec<CellState> {
        let state = self.state.lock().unwrap();
        let total = state.total_cells;
        let charged = state.charged_cells;
        (0..total)
            .map(|i| CellState {
                index: i as u8,
                charged: i < charged,
            })
            .collect()
    }
    
    fn generation_history(&self) -> Vec<GenerationEntry> {
        let state = self.state.lock().unwrap();
        state.generation_history
            .iter()
            .map(|r| GenerationEntry { resource: format!("{:?}", r) })
            .collect()
    }
}

/// Shared stats updated by planet AI hooks; read by [GenericPlanetTelemetry].
#[derive(Debug, Default)]
pub struct SharedPlanetStats {
    pub total_cells: usize,
    pub charged_cells: usize,
    pub total_resources_generated: usize,
    pub explorer_arrivals: usize,
    pub explorer_departures: usize,
    pub rockets_built: usize,
    pub asteroids_deflected: usize,
    pub combinations_attempted: usize,
    pub combinations_succeeded: usize,
    pub errors_encountered: usize,
    pub present_explorers: usize,
}

/// Generic telemetry implementation backed by [SharedPlanetStats].
pub struct GenericPlanetTelemetry {
    pub stats: Arc<Mutex<SharedPlanetStats>>,
}

impl PlanetTelemetry for GenericPlanetTelemetry {
    fn snapshot(&self) -> CommonPlanetSnapshot {
        let s = self.stats.lock().unwrap();
        CommonPlanetSnapshot {
            energy_cells: s.total_cells,
            charged_cells: s.charged_cells,
            total_resources_generated: s.total_resources_generated,
            explorer_arrivals: s.explorer_arrivals,
            explorer_departures: s.explorer_departures,
            rockets_built: s.rockets_built,
            asteroids_deflected: s.asteroids_deflected,
            combinations_attempted: s.combinations_attempted,
            combinations_succeeded: s.combinations_succeeded,
            errors_encountered: s.errors_encountered,
        }
    }
    fn explorer_count(&self) -> usize { self.stats.lock().unwrap().present_explorers }
    fn energy_cells(&self) -> usize { self.stats.lock().unwrap().total_cells }
    fn charged_cells(&self) -> usize { self.stats.lock().unwrap().charged_cells }
    fn cell_states(&self) -> Vec<CellState> {
        let s = self.stats.lock().unwrap();
        (0..s.total_cells)
            .map(|i| CellState { index: i as u8, charged: i < s.charged_cells })
            .collect()
    }
    fn generation_history(&self) -> Vec<GenerationEntry> { vec![] }
}

/// Wraps any [PlanetAI] and intercepts calls to populate [SharedPlanetStats].
pub struct StatsTrackingAI<Inner> {
    pub inner: Inner,
    pub stats: Arc<Mutex<SharedPlanetStats>>,
}

impl<Inner: PlanetAI + Send + 'static> PlanetAI for StatsTrackingAI<Inner> {
    fn handle_sunray(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator, sunray: Sunray) {
        self.inner.handle_sunray(state, gen, comb, sunray);
        let mut s = self.stats.lock().unwrap();
        s.total_cells = state.cells_iter().count();
        s.charged_cells = state.cells_iter().filter(|c| c.is_charged()).count();
    }

    fn handle_asteroid(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator) -> Option<Rocket> {
        let rocket = self.inner.handle_asteroid(state, gen, comb);
        if rocket.is_some() {
            let mut s = self.stats.lock().unwrap();
            s.asteroids_deflected += 1;
            s.rockets_built += 1;
        }
        rocket
    }

    fn handle_internal_state_req(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator) -> DummyPlanetState {
        self.inner.handle_internal_state_req(state, gen, comb)
    }

    fn handle_explorer_msg(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator, msg: ExplorerToPlanet) -> Option<PlanetToExplorer> {
        let is_generate = matches!(msg, ExplorerToPlanet::GenerateResourceRequest { .. });
        let is_combine = matches!(msg, ExplorerToPlanet::CombineResourceRequest { .. });
        let response = self.inner.handle_explorer_msg(state, gen, comb, msg);
        if let Some(ref resp) = response {
            let mut s = self.stats.lock().unwrap();
            if is_generate {
                if let PlanetToExplorer::GenerateResourceResponse { resource: Some(_) } = resp {
                    s.total_resources_generated += 1;
                }
            } else if is_combine {
                if let PlanetToExplorer::CombineResourceResponse { complex_response } = resp {
                    s.combinations_attempted += 1;
                    if complex_response.is_ok() {
                        s.combinations_succeeded += 1;
                    }
                }
            }
        }
        response
    }

    fn on_explorer_arrival(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator, explorer_id: ID) {
        {
            let mut s = self.stats.lock().unwrap();
            s.explorer_arrivals += 1;
            s.present_explorers += 1;
        }
        self.inner.on_explorer_arrival(state, gen, comb, explorer_id);
    }

    fn on_explorer_departure(&mut self, state: &mut PlanetState, gen: &Generator, comb: &Combinator, explorer_id: ID) {
        {
            let mut s = self.stats.lock().unwrap();
            s.explorer_departures += 1;
            s.present_explorers = s.present_explorers.saturating_sub(1);
        }
        self.inner.on_explorer_departure(state, gen, comb, explorer_id);
    }

    fn on_start(&mut self, state: &PlanetState, gen: &Generator, comb: &Combinator) {
        self.inner.on_start(state, gen, comb);
    }

    fn on_stop(&mut self, state: &PlanetState, gen: &Generator, comb: &Combinator) {
        self.inner.on_stop(state, gen, comb);
    }
}
