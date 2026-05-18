//! Planet module root for the RustyPlanets crate.

pub mod cycle;
pub mod errors;
pub mod resources;
pub mod state;

// Our planet implementation (Skycartel / Type A)
pub mod skycartel;

// Re-export the common Planet type for convenience.
pub type Planet = common_game::components::planet::Planet;

// Public re-exports used by `crate::create_planet(...)`.
pub use skycartel::{RustyPlanet, RustyPlanetId};
