// src/tests.rs
//! Integration-style smoke tests for the Skycartel crate.

#[cfg(test)]
mod tests {
    use crossbeam_channel::unbounded;

    use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
    use common_game::protocols::planet_explorer::ExplorerToPlanet;

    #[test]
    fn skycartel_create_planet_smoke() {
        let (_tx_orch, rx_orch) = unbounded::<OrchestratorToPlanet>();
        let (tx_planet, _rx_planet) = unbounded::<PlanetToOrchestrator>();
        let (_tx_expl, rx_expl) = unbounded::<ExplorerToPlanet>();

        let planet = crate::create_planet(42, rx_orch, tx_planet, rx_expl);
        assert!(planet.is_ok());
    }
}
