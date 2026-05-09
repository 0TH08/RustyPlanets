//! # `BlackAdidasShoe` Planet
//!
//! `BlackAdidasShoe` Planet creation module.
//! `BlackAdidasShoe` is a type D Planet that can produce Hydrogen, Oxygen and Carbon.
use super::ai::BlackAdidasShoe;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::BasicResourceType;
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::{Receiver, Sender};

/// Creates and initializes a new instance of the `BlackAdidasShoe` Planet.
/// -`rx_orchestrator`: Orchestrator to Planet Receiver
/// - `tx_orchestrator`: Planet to Orchestrator Sender
/// - `rx_explorer`: Explorer to Planet Sender
/// - `planet_id`: The unique identifier assigned to this planet.
///
/// # Errors
/// Propagates any error returned by `Planet::new`.
pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: ID
) -> Result<Planet, String> {
    let ai = BlackAdidasShoe::new(false);
    let gen_rules = vec![
        BasicResourceType::Oxygen,
        BasicResourceType::Hydrogen,
        BasicResourceType::Carbon,
    ];
    let comb_rules = vec![];

    // Construct the planet and return it
    Planet::new(
        planet_id,
        PlanetType::D,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    )
}