// src/lib.rs
//! Skycartel planet crate (RustyPlanet implementation)

pub mod logging;
pub mod orchestrator;
pub mod planet_ai;
pub mod explorer;

#[cfg(test)]
mod tests;

pub use planet_ai::{RustyPlanet, RustyPlanetId};

pub fn create_planet(
    id: u32,
    rx_orchestrator: crossbeam_channel::Receiver<
        common_game::protocols::orchestrator_planet::OrchestratorToPlanet,
    >,
    tx_orchestrator: crossbeam_channel::Sender<
        common_game::protocols::orchestrator_planet::PlanetToOrchestrator,
    >,
    rx_explorer: crossbeam_channel::Receiver<
        common_game::protocols::planet_explorer::ExplorerToPlanet,
    >,
) -> Result<planet_ai::Planet, String> {
    use common_game::logging::{Channel, EventType};
    use logging::log_planet_event;

    let planet_id = RustyPlanetId::new(id);

    log_planet_event(
        planet_id,
        EventType::InternalPlanetAction,
        Channel::Info,
        "Skycartel initializing",
        None::<[(&str, String); 0]>,
    );

    let planet_impl = RustyPlanet::new(id)
        .map_err(|e| format!("Failed to create Skycartel planet handle: {e}"))?;

    let planet = planet_impl
        .create_planet(rx_orchestrator, tx_orchestrator, rx_explorer)
        .map_err(|e| format!("Failed to create planet: {e}"))?;

    log_planet_event(
        planet_id,
        EventType::InternalPlanetAction,
        Channel::Info,
        "Skycartel initialization complete",
        None::<[(&str, String); 0]>,
    );

    Ok(planet)
}
