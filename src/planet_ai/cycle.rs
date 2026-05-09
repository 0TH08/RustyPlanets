//! Cycle/phase model for Skycartel.
//!
//! Skycartel (RustyPlanet Type A) does not require a multi-phase system,
//! but we keep a phase enum so the AI logic matches the architecture
//! used by other planet crates.

/// Skycartel uses a single stable phase.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum SkycartelPhase {
    #[default]
    Stable,
}
