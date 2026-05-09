//! Common trait for planet telemetry across different planet types.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Serialize, Deserialize};

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
