// src/lib.rs
//! Skycartel planet crate (RustyPlanet implementation)

pub mod logging;
pub mod orchestrator;
pub mod planet_ai;
pub mod explorer;
pub mod telemetry;
pub mod planet_telemetry;
pub mod broadcast_log;
#[cfg(test)]
mod tests;

pub use common_game;

pub use planet_ai::{RustyPlanet, RustyPlanetId};
pub use telemetry::PlanetKind;

/// Planet type selection for creating different planet implementations.
#[derive(Debug, Clone, Copy)]
pub enum PlanetType {
    Skycartel,
    Luna4,
    BlackAdidasShoe,
    ImmutableCosmicBorrow,
    Rusteze,
    Crabtorio,
    Orbitron,
    AstroParrot,
}

/// Creates a planet without telemetry tracking.
///
/// Use this for standalone planet creation when you don't need the GUI
/// to observe planet state.
pub fn create_planet(
    id: u32,
    planet_type: PlanetType,
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
        "Planet initializing",
        None::<[(&str, String); 0]>,
    );

    let planet = match planet_type {
        PlanetType::Skycartel => {
            let planet_impl = RustyPlanet::new(id)
                .map_err(|e| format!("Failed to create Skycartel planet handle: {e}"))?;
            planet_impl
                .create_planet(rx_orchestrator, tx_orchestrator, rx_explorer)
                .map_err(|e| format!("Failed to create planet: {e}"))?
        }
        PlanetType::Luna4 => {
            luna4::create_planet(id, rx_orchestrator, tx_orchestrator, rx_explorer)
                .map_err(|e| format!("Luna4: {e}"))?
        }
        PlanetType::BlackAdidasShoe => {
            black_adidas_shoe::planet::create_planet(
                rx_orchestrator, tx_orchestrator, rx_explorer, id,
            ).map_err(|e| format!("BlackAdidasShoe: {e}"))?
        }
        PlanetType::ImmutableCosmicBorrow => {
            use std::time::Duration;
            immutable_cosmic_borrow::create_planet(
                false,
                1.0,
                1.0,
                Duration::from_secs(30),
                Duration::from_secs(1),
                id,
                (rx_orchestrator, tx_orchestrator),
                rx_explorer,
            ).map_err(|e| format!("ImmutableCosmicBorrow: {e}"))?
        }
        PlanetType::Rusteze => {
            // pub-rust-eze panics internally on error, returns Planet directly
            rusteze::create_planet(id, rx_orchestrator, tx_orchestrator, rx_explorer)
        }
        PlanetType::Crabtorio => {
            // crabtorio panics internally on error, returns Planet directly
            crabtorio::create_planet(id, rx_orchestrator, tx_orchestrator, rx_explorer)
        }
        PlanetType::Orbitron => {
            // orbitron's arg order puts planet_id last
            orbitron::create_planet(rx_orchestrator, tx_orchestrator, rx_explorer, id)
        }
        PlanetType::AstroParrot => {
            // same arg order as orbitron — planet_id last, returns Planet directly
            astro_parrot::create_planet(rx_orchestrator, tx_orchestrator, rx_explorer, id)
        }
    };

    log_planet_event(
        planet_id,
        EventType::InternalPlanetAction,
        Channel::Info,
        "Planet initialization complete",
        None::<[(&str, String); 0]>,
    );

    Ok(planet)
}

/// Creates a planet and registers it with the telemetry hub.
///
/// The planet's internal state is accessible via `TelemetryHub` for
/// real-time observation by the GUI server. The `ai_running` flag
/// allows the GUI to track whether this planet's AI is active.
#[allow(clippy::too_many_arguments)]
pub fn create_planet_with_telemetry(
    id: u32,
    name: String,
    planet_type: PlanetType,
    rx_orchestrator: crossbeam_channel::Receiver<
        common_game::protocols::orchestrator_planet::OrchestratorToPlanet,
    >,
    tx_orchestrator: crossbeam_channel::Sender<
        common_game::protocols::orchestrator_planet::PlanetToOrchestrator,
    >,
    rx_explorer: crossbeam_channel::Receiver<
        common_game::protocols::planet_explorer::ExplorerToPlanet,
    >,
    ai_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    telemetry: &telemetry::TelemetryHub,
) -> Result<planet_ai::Planet, String> {
    use common_game::logging::{Channel, EventType};
    use logging::log_planet_event;

    let planet_id = RustyPlanetId::new(id);

    let kind = match planet_type {
        PlanetType::Skycartel => telemetry::PlanetKind::Skycartel,
        PlanetType::Luna4 => telemetry::PlanetKind::Luna4,
        PlanetType::BlackAdidasShoe => telemetry::PlanetKind::BlackAdidasShoe,
        PlanetType::ImmutableCosmicBorrow => telemetry::PlanetKind::ImmutableCosmicBorrow,
        PlanetType::Rusteze => telemetry::PlanetKind::Rusteze,
        PlanetType::Crabtorio => telemetry::PlanetKind::Crabtorio,
        PlanetType::Orbitron => telemetry::PlanetKind::Orbitron,
        PlanetType::AstroParrot => telemetry::PlanetKind::AstroParrot,
    };

    log_planet_event(
        planet_id,
        EventType::InternalPlanetAction,
        Channel::Info,
        "Planet initializing (with telemetry)",
        None::<[(&str, String); 0]>,
    );

    let (planet, telemetry_obj): (_, Box<dyn planet_telemetry::PlanetTelemetry>) = match planet_type {
        PlanetType::Skycartel => {
            let planet_impl = RustyPlanet::new(id)
                .map_err(|e| format!("Failed to create Skycartel planet handle: {e}"))?;
            let (planet, state) = planet_impl
                .create_planet_with_ai_state(rx_orchestrator, tx_orchestrator, rx_explorer)
                .map_err(|e| format!("Failed to create planet: {e}"))?;
            let tel = Box::new(planet_telemetry::SkycartelTelemetry {
                state,
                ai_running: ai_running.clone(),
            });
            (planet, tel)
        }
        // External planets don't expose internal stats hooks, so we use zeroed telemetry.
        // They still run and respond correctly — only the GUI counters stay at 0.
        _ => {
            let planet = create_planet(id, planet_type, rx_orchestrator, tx_orchestrator, rx_explorer)?;
            let stats = std::sync::Arc::new(std::sync::Mutex::new(
                planet_telemetry::SharedPlanetStats::default(),
            ));
            (planet, Box::new(planet_telemetry::GenericPlanetTelemetry { stats }))
        }
    };

    telemetry.register_planet(
        id,
        telemetry::PlanetEntry {
            name,
            kind,
            telemetry: telemetry_obj,
            ai_running,
        },
    );

    log_planet_event(
        planet_id,
        EventType::InternalPlanetAction,
        Channel::Info,
        "Planet initialization complete (with telemetry)",
        None::<[(&str, String); 0]>,
    );

    Ok(planet)
}
