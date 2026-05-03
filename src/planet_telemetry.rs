//! Common trait for planet telemetry across different planet types.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::Serialize;

/// Common telemetry snapshot for all planet types.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CommonPlanetSnapshot {
    pub total_resources_generated: usize,
    pub explorer_arrivals: usize,
    pub explorer_departures: usize,
    pub rockets_built: usize,
    pub asteroids_deflected: usize,
    pub errors_encountered: usize,
}

/// Trait that all planet types must implement for telemetry support.
pub trait PlanetTelemetry: Send + Sync {
    /// Returns the current telemetry snapshot.
    fn snapshot(&self) -> CommonPlanetSnapshot;
    
    /// Returns the number of explorers currently present.
    fn explorer_count(&self) -> usize;
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
            total_resources_generated: state.stats.total_resources_generated,
            explorer_arrivals: state.stats.explorer_arrivals,
            explorer_departures: state.stats.explorer_departures,
            rockets_built: state.stats.rockets_built,
            asteroids_deflected: state.stats.asteroids_deflected,
            errors_encountered: state.stats.errors_encountered,
        }
    }
    
    fn explorer_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.present_explorers.len()
    }
}
