//! Skycartel state management
//!
//! Tracks runtime state for the Skycartel planet implementation:
//! explorer presence, simple phase model, and operational counters.
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::cycle::SkycartelPhase;
use super::RustyPlanetId;
use common_game::components::resource::BasicResourceType;

/// Tracks the operational state of a Skycartel planet.
#[derive(Debug)]
pub struct SkycartelState {
    pub(crate) id: RustyPlanetId,
    pub(crate) phase: SkycartelPhase,
    pub(crate) phase_start_time: Instant,

    pub(crate) generation_history: Vec<BasicResourceType>,
    pub(crate) present_explorers: HashMap<u32, Instant>,

    pub(crate) stats: OperationalStats,
}

impl SkycartelState {
    /// Creates a new Skycartel state.
    pub(crate) fn new(id: u32) -> Self {
        Self {
            id: RustyPlanetId::new(id),
            phase: SkycartelPhase::Stable,
            phase_start_time: Instant::now(),
            generation_history: Vec::new(),
            present_explorers: HashMap::new(),
            stats: OperationalStats::new(),
        }
    }

    /// Skycartel uses a single stable phase.
    /// Returns `None` always, but keeps the same architecture shape as other planets.
    pub(crate) fn update_phase(&mut self) -> Option<SkycartelPhase> {
        None
    }

    pub(crate) fn record_generation(&mut self, resource: BasicResourceType) {
        self.generation_history.push(resource);
        self.stats.total_resources_generated += 1;
    }

    pub(crate) fn register_explorer_arrival(&mut self, explorer_id: u32) {
        self.present_explorers.insert(explorer_id, Instant::now());
        self.stats.explorer_arrivals += 1;
    }

    pub(crate) fn register_explorer_departure(&mut self, explorer_id: u32) {
        self.present_explorers.remove(&explorer_id);
        self.stats.explorer_departures += 1;
    }

    pub(crate) fn record_rocket_built(&mut self) {
        self.stats.rockets_built += 1;
    }

    pub(crate) fn record_asteroid_deflected(&mut self) {
        self.stats.asteroids_deflected += 1;
    }

    pub(crate) fn record_error(&mut self) {
        self.stats.errors_encountered += 1;
    }

    #[allow(dead_code)]
    pub(crate) fn explorer_count(&self) -> usize {
        self.present_explorers.len()
    }

    #[allow(dead_code)]
    pub(crate) fn elapsed_in_phase(&self) -> Duration {
        self.phase_start_time.elapsed()
    }

    #[allow(dead_code)]
    pub(crate) fn display_summary(&self) -> String {
        format!(
            "Skycartel #{id} | Phase: {phase:?} | Explorers: {explorers} | Generated: {gen} | Rockets: {rockets} | Deflections: {defl} | Errors: {err}",
            id = self.id.as_u32(),
            phase = self.phase,
            explorers = self.explorer_count(),
            gen = self.stats.total_resources_generated,
            rockets = self.stats.rockets_built,
            defl = self.stats.asteroids_deflected,
            err = self.stats.errors_encountered,
        )
    }
}

/// Operational counters for Skycartel.
#[derive(Debug, Clone, Default)]
pub(crate) struct OperationalStats {
    pub(crate) total_resources_generated: usize,
    pub(crate) explorer_arrivals: usize,
    pub(crate) explorer_departures: usize,
    pub(crate) rockets_built: usize,
    pub(crate) asteroids_deflected: usize,
    pub(crate) errors_encountered: usize,
}

impl OperationalStats {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_initialization() {
        let s = SkycartelState::new(7);
        assert_eq!(s.id.as_u32(), 7);
        assert_eq!(s.phase, SkycartelPhase::Stable);
        assert_eq!(s.explorer_count(), 0);
        assert_eq!(s.stats.total_resources_generated, 0);
    }

    #[test]
    fn explorer_tracking() {
        let mut s = SkycartelState::new(1);

        s.register_explorer_arrival(10);
        assert_eq!(s.explorer_count(), 1);
        assert_eq!(s.stats.explorer_arrivals, 1);

        s.register_explorer_arrival(20);
        assert_eq!(s.explorer_count(), 2);

        s.register_explorer_departure(10);
        assert_eq!(s.explorer_count(), 1);
        assert_eq!(s.stats.explorer_departures, 1);
    }

    #[test]
    fn counters_increment() {
        let mut s = SkycartelState::new(1);

        s.record_generation(BasicResourceType::Carbon);
        s.record_rocket_built();
        s.record_asteroid_deflected();
        s.record_error();

        assert_eq!(s.stats.total_resources_generated, 1);
        assert_eq!(s.stats.rockets_built, 1);
        assert_eq!(s.stats.asteroids_deflected, 1);
        assert_eq!(s.stats.errors_encountered, 1);
    }

    #[test]
    fn summary_contains_id() {
        let s = SkycartelState::new(99);
        let line = s.display_summary();
        assert!(line.contains("Skycartel #99"));
    }
}
