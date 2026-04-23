//! Planet module root for the Skycartel crate.
//!
//! This mirrors the “single planet as a library” architecture (like Luna),
//! while keeping implementation details behind a stable public interface.

pub mod cycle;
pub mod energy;
pub mod errors;
pub mod resources;
pub mod state;

// Your planet implementation module (equivalent to Luna's `planet/luna4/`)
pub mod skycartel;

// Re-export the common Planet type for convenience.
pub type Planet = common_game::components::planet::Planet;

// Public re-exports used by `crate::create_planet(...)` and consumers.
pub use skycartel::{RustyPlanet, RustyPlanetId};
