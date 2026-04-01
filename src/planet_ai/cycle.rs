//! Cycle/phase model for Skycartel.
//!
//! Skycartel (RustyPlanet Type A) does not require a multi-phase system,
//! but we keep a phase enum so the AI logic matches the architecture
//! used by other planet crates.

/// Skycartel uses a single stable phase.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum SkycartelPhase {
    Stable,
}

impl Default for SkycartelPhase {
    fn default() -> Self {
        Self::Stable
    }
}
