//! Lightweight stats snapshot types for Skycartel.

use crate::planet_ai::state::OperationalStats;

#[derive(Debug, Clone, Default)]
pub struct SkycartelStats {
    pub total_resources_generated: usize,
    pub explorer_arrivals: usize,
    pub explorer_departures: usize,
    pub rockets_built: usize,
    pub asteroids_deflected: usize,
    pub errors_encountered: usize,
}

impl From<&OperationalStats> for SkycartelStats {
    fn from(s: &OperationalStats) -> Self {
        Self {
            total_resources_generated: s.total_resources_generated,
            explorer_arrivals: s.explorer_arrivals,
            explorer_departures: s.explorer_departures,
            rockets_built: s.rockets_built,
            asteroids_deflected: s.asteroids_deflected,
            errors_encountered: s.errors_encountered,
        }
    }
}

impl SkycartelStats {
    pub fn snapshot(stats: &crate::planet_ai::state::OperationalStats) -> Self {
        Self {
            total_resources_generated: stats.total_resources_generated,
            explorer_arrivals: stats.explorer_arrivals,
            explorer_departures: stats.explorer_departures,
            rockets_built: stats.rockets_built,
            asteroids_deflected: stats.asteroids_deflected,
            errors_encountered: stats.errors_encountered,
        }
    }
}
