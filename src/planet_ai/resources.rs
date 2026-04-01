//! Resource management for Skycartel (RustyPlanet, Type A).
//!
//! Skycartel's planet is configured for **basic generation only** and your group chose:
//! - **Carbon** as the only supported basic resource.
//! - **No complex resources** (no combinations).
#![allow(dead_code)]

use std::collections::HashSet;

use super::cycle::SkycartelPhase;
use common_game::components::resource::{BasicResourceType, ComplexResourceType};

/// Resources available during a specific phase.
///
/// Skycartel uses a single stable phase, but we keep a phase-based interface
/// to match the architecture used by other planet crates.
///
///
#[derive(Debug, Clone)]
pub(crate) struct AvailableResources {
    pub(crate) basic: HashSet<BasicResourceType>,
    pub(crate) complex: HashSet<ComplexResourceType>,
}

impl AvailableResources {
    pub(crate) fn carbon_only() -> Self {
        let mut basic = HashSet::new();
        basic.insert(BasicResourceType::Carbon);

        Self {
            basic,
            complex: HashSet::new(),
        }
    }
}

/// Resource manager for Skycartel.
///
/// Even though Skycartel does not have a real cycle, we model a single phase
/// so the rest of the AI can stay clean and predictable.
#[derive(Debug, Clone)]
pub(crate) struct ResourceManager {
    stable: AvailableResources,
}

impl ResourceManager {
    /// Creates a new resource manager.
    pub(crate) fn new() -> Self {
        Self {
            stable: AvailableResources::carbon_only(),
        }
    }

    /// Returns the available resources for the current phase.
    ///
    /// Skycartel has only one phase, so this always returns the same set.
    pub(crate) fn get_available_resources(&self, _phase: SkycartelPhase) -> &AvailableResources {
        &self.stable
    }

    /// Convenience: returns the supported basic resources.
    #[allow(dead_code)]
    pub(crate) fn supported_basic(&self) -> HashSet<BasicResourceType> {
        self.stable.basic.clone()
    }

    /// Convenience: returns the supported complex resources (always empty).
    #[allow(dead_code)]
    pub(crate) fn supported_complex(&self) -> HashSet<ComplexResourceType> {
        self.stable.complex.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carbon_is_the_only_basic_resource() {
        let rm = ResourceManager::new();
        let available = rm.get_available_resources(SkycartelPhase::Stable);

        assert_eq!(available.basic.len(), 1);
        assert!(available.basic.contains(&BasicResourceType::Carbon));
        assert!(available.complex.is_empty());
    }
}
