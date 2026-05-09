//! Planet module root for the RustyPlanets crate.

pub mod cycle;
pub mod energy;
pub mod errors;
pub mod resources;
pub mod state;

// Planet implementation modules
pub mod skycartel;
pub mod luna4;
pub mod blackadidasshoe;
pub mod immutablecosmicborrow;
pub mod crabtorio;
pub mod rusteze;
pub mod orbitron;

// Re-export the common Planet type for convenience.
pub type Planet = common_game::components::planet::Planet;

// Public re-exports used by `crate::create_planet(...)`.
pub use skycartel::{RustyPlanet, RustyPlanetId};
