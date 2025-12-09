//! Type A planet AI implementation
//!
//! Implements the PlanetAI trait for Planet Type A.
//! The implementation intentionally does not cache resources — it uses only the
//! provided `PlanetState`, `Generator` and `Combinator` objects.

use crate::components::planet::{Planet, PlanetAI, PlanetState, PlanetType};
use crate::components::resource::{
    BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType,
    Combinator, Generator, GenericResource,
};
use crate::components::rocket::Rocket;
use crate::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};

use crossbeam_channel::{Receiver, Sender};
use std::collections::HashSet;

/// Concrete AI for Planet Type A.
///
/// Notes:
/// - Type A has 5 energy cells (handled by PlanetState).
/// - Type A is limited to a single generation rule (we pick Oxygen in `create_planet`).
/// - Type A has no combination recipes — combine requests are rejected but the
///   original resources are returned inside the error result (per protocol).
pub struct TypeAPlanetAI {
    /// Small telemetry counter; it doesn't store resources.
    sunray_seen: u64,
}

impl TypeAPlanetAI {
    /// Construct a new TypeAPlanetAI.
    pub fn new() -> Self {
        Self { sunray_seen: 0 }
    }
}

impl PlanetAI for TypeAPlanetAI {
    /// Handle messages coming from the orchestrator (Sunray, InternalStateRequest, ...).
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            // Charge a first-empty cell with the incoming sunray.
            OrchestratorToPlanet::Sunray(sunray) => {
                self.sunray_seen = self.sunray_seen.saturating_add(1);
                // charge_cell will return Some(sunray) if there was no empty cell.
                let _leftover = state.charge_cell(sunray);
                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                })
            }

            // Return a non-sensitive overview of the internal state.
            OrchestratorToPlanet::InternalStateRequest => {
                let dummy = state.to_dummy();
                Some(PlanetToOrchestrator::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: dummy,
                })
            }

            // Ignore (resp. handled elsewhere): Start/Stop/Asteroid/Incoming/Outgoing are
            // managed by the outer planet loop or have dedicated handlers.
            _ => None,
        }
    }

    /// Handle messages coming from explorers visiting the planet.
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { .. } => {
                // Return the set of basic resources supported by the planet's generator.
                let supported: HashSet<BasicResourceType> = generator.all_available_recipes();
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: supported,
                })
            }

            ExplorerToPlanet::SupportedCombinationRequest { .. } => {
                // Return the set of complex recipes supported by the planet's combinator.
                let supported: HashSet<ComplexResourceType> = combinator.all_available_recipes();
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: supported,
                })
            }

            ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_count() as u32,
                })
            }

            ExplorerToPlanet::GenerateResourceRequest { explorer_id: _, resource } => {
                // Make sure the generator exposes this resource
                if !generator.contains(resource) {
                    return Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                }

                // Use a charged cell
                match state.full_cell() {
                    None => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                    Some((cell, _idx)) => {
                        // Dispatch to the appropriate generator method.
                        let gen_res = match resource {
                            BasicResourceType::Oxygen => generator.make_oxygen(cell).map(BasicResource::Oxygen),
                            BasicResourceType::Hydrogen => generator.make_hydrogen(cell).map(BasicResource::Hydrogen),
                            BasicResourceType::Carbon => generator.make_carbon(cell).map(BasicResource::Carbon),
                            BasicResourceType::Silicon => generator.make_silicon(cell).map(BasicResource::Silicon),
                        };

                        match gen_res {
                            Ok(basic) => Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(basic),
                            }),
                            Err(_) => Some(PlanetToExplorer::GenerateResourceResponse { resource: None }),
                        }
                    }
                }
            }

            ExplorerToPlanet::CombineResourceRequest { explorer_id: _, msg } => {
                // Type A has no combination recipes: return Err with the original resources.
                let err_msg = "This planet type does not support resource combination.".to_string();

                // Map the ComplexResourceRequest -> the two GenericResource back
                let returned_pair: (GenericResource, GenericResource) = match msg {
                    ComplexResourceRequest::Water(h, o) => {
                        (GenericResource::BasicResources(BasicResource::Hydrogen(h)),
                         GenericResource::BasicResources(BasicResource::Oxygen(o)))
                    }
                    ComplexResourceRequest::Diamond(c1, c2) => {
                        (GenericResource::BasicResources(BasicResource::Carbon(c1)),
                         GenericResource::BasicResources(BasicResource::Carbon(c2)))
                    }
                    ComplexResourceRequest::Life(w, c) => {
                        (GenericResource::ComplexResources(ComplexResource::Water(w)),
                         GenericResource::BasicResources(BasicResource::Carbon(c)))
                    }
                    ComplexResourceRequest::Robot(s, l) => {
                        (GenericResource::BasicResources(BasicResource::Silicon(s)),
                         GenericResource::ComplexResources(ComplexResource::Life(l)))
                    }
                    ComplexResourceRequest::Dolphin(w, l) => {
                        (GenericResource::ComplexResources(ComplexResource::Water(w)),
                         GenericResource::ComplexResources(ComplexResource::Life(l)))
                    }
                    ComplexResourceRequest::AIPartner(r, d) => {
                        (GenericResource::ComplexResources(ComplexResource::Robot(r)),
                         GenericResource::ComplexResources(ComplexResource::Diamond(d)))
                    }
                };

                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((err_msg, returned_pair.0, returned_pair.1)),
                })
            }
        }
    }

    /// Handle asteroid: build a rocket if possible (planet survives), else None.
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        match state.full_cell() {
            None => None,
            Some((_cell, idx)) => {
                if state.build_rocket(idx).is_ok() {
                    state.take_rocket()
                } else {
                    None
                }
            }
        }
    }

    /// Called on StartPlanetAI. No resource modification here.
    fn start(&mut self, _state: &PlanetState) {
        self.sunray_seen = 0;
    }

    /// Called on StopPlanetAI. No resource modification.
    fn stop(&mut self, _state: &PlanetState) {}
}

/// Factory to create a Type A planet instance. The orchestrator will call this.
pub fn create_planet(
    id: u32,
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorers: Receiver<ExplorerToPlanet>,
) -> Result<Planet, String> {
    let ai = TypeAPlanetAI::new();

    // Type A: choose Oxygen as the single generation recipe.
    let gen_rules = vec![BasicResourceType::Oxygen];

    // Type A: no combination rules.
    let comb_rules: Vec<ComplexResourceType> = vec![];

    Planet::new(
        id,
        PlanetType::A,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorers,
    )
}

#[cfg(test)]
mod tests {
    // -- your test module (unchanged) --
    use super::*;
    use crate::components::resource::{BasicResource, BasicResourceType};
    use crate::protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    };
    use crossbeam_channel::unbounded;
    use std::thread;
    use std::time::Duration;

    fn build_channels() -> (
        (crossbeam_channel::Receiver<OrchestratorToPlanet>, crossbeam_channel::Sender<PlanetToOrchestrator>),
        crossbeam_channel::Receiver<ExplorerToPlanet>,
        crossbeam_channel::Sender<OrchestratorToPlanet>,
        crossbeam_channel::Receiver<PlanetToOrchestrator>,
        crossbeam_channel::Sender<ExplorerToPlanet>,
    ) {
        let (tx_orch_in, rx_orch_in) = unbounded::<OrchestratorToPlanet>();
        let (tx_orch_out, rx_orch_out) = unbounded::<PlanetToOrchestrator>();
        let (tx_expl_global, rx_expl_global) = unbounded::<ExplorerToPlanet>();

        (
            (rx_orch_in, tx_orch_out),
            rx_expl_global,
            tx_orch_in,
            rx_orch_out,
            tx_expl_global,
        )
    }

    #[test]
    fn type_a_planet_smoke_test_start_sunray_asteroid_and_explorer_flow() {
        let (
            orch_half_for_planet,
            expl_rx_for_planet,
            orch_tx_for_test,
            orch_rx_for_test,
            expl_tx_global,
        ) = build_channels();

        let mut planet = create_planet(
            42,
            orch_half_for_planet.0.clone(),
            orch_half_for_planet.1.clone(),
            expl_rx_for_planet,
        )
            .expect("Failed to create Type A planet");

        let handle = thread::spawn(move || {
            let res = planet.run();
            if let Err(e) = res {
                panic!("Planet run returned error: {}", e);
            }
        });

        // Start AI
        orch_tx_for_test
            .send(OrchestratorToPlanet::StartPlanetAI)
            .expect("failed to send StartPlanetAI");

        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::StartPlanetAIResult { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected StartPlanetAIResult from planet");
            }
        } else {
            panic!("Timed out waiting for StartPlanetAIResult");
        }

        // Send Sunray
        orch_tx_for_test
            .send(OrchestratorToPlanet::Sunray(crate::components::sunray::Sunray::new()))
            .expect("failed to send Sunray");

        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::SunrayAck { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected SunrayAck from planet");
            }
        } else {
            panic!("Timed out waiting for SunrayAck");
        }

        // Send Asteroid
        orch_tx_for_test
            .send(OrchestratorToPlanet::Asteroid(
                crate::components::asteroid::Asteroid::new(),
            ))
            .expect("failed to send Asteroid");

        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::AsteroidAck { planet_id, rocket } = msg {
                assert_eq!(planet_id, 42);
                assert!(rocket.is_some(), "Type A planet should survive with Some(Rocket)");
            } else {
                panic!("Expected AsteroidAck from planet");
            }
        } else {
            panic!("Timed out waiting for AsteroidAck");
        }

        // Incoming explorer flow
        let (tx_local_to_planet_resp, rx_local_from_planet) = unbounded::<PlanetToExplorer>();
        let explorer_id: u32 = 777;

        orch_tx_for_test
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id,
                new_mpsc_sender: tx_local_to_planet_resp,
            })
            .expect("failed to send IncomingExplorerRequest");

        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res } = msg {
                assert_eq!(planet_id, 42);
                assert!(res.is_ok(), "IncomingExplorerResponse should be Ok(())");
            } else {
                panic!("Expected IncomingExplorerResponse");
            }
        } else {
            panic!("Timed out waiting for IncomingExplorerResponse");
        }

        // Ask supported resources
        expl_tx_global
            .send(ExplorerToPlanet::SupportedResourceRequest { explorer_id })
            .expect("failed to send SupportedResourceRequest");

        if let Ok(msg) = rx_local_from_planet.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToExplorer::SupportedResourceResponse { resource_list } = msg {
                assert!(
                    resource_list.contains(&BasicResourceType::Oxygen),
                    "Type A planet should expose Oxygen in supported resources"
                );
            } else {
                panic!("Expected SupportedResourceResponse on local explorer channel");
            }
        } else {
            panic!("Timed out waiting for SupportedResourceResponse on explorer channel");
        }

        // Generation: charge and generate Oxygen
        orch_tx_for_test
            .send(OrchestratorToPlanet::Sunray(crate::components::sunray::Sunray::new()))
            .expect("failed to send Sunray for generation");

        // consume the SunrayAck
        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::SunrayAck { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected SunrayAck after charging cell");
            }
        } else {
            panic!("Timed out waiting for SunrayAck after charging cell");
        }

        // Request generation of Oxygen
        expl_tx_global
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource: BasicResourceType::Oxygen,
            })
            .expect("failed to send GenerateResourceRequest");

        if let Ok(msg) = rx_local_from_planet.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToExplorer::GenerateResourceResponse { resource } = msg {
                match resource {
                    Some(BasicResource::Oxygen(_)) => {}
                    Some(_) => panic!("Expected Oxygen resource, got a different BasicResource"),
                    None => panic!("Planet replied with None for GenerateResourceResponse"),
                }
            } else {
                panic!("Expected GenerateResourceResponse on local explorer channel");
            }
        } else {
            panic!("Timed out waiting for GenerateResourceResponse on explorer channel");
        }

        // Stop and kill
        orch_tx_for_test
            .send(OrchestratorToPlanet::StopPlanetAI)
            .expect("failed to send StopPlanetAI");
        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::StopPlanetAIResult { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected StopPlanetAIResult");
            }
        } else {
            panic!("Timed out waiting for StopPlanetAIResult");
        }

        orch_tx_for_test
            .send(OrchestratorToPlanet::InternalStateRequest)
            .expect("failed to send InternalStateRequest while stopped");
        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::Stopped { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected Stopped when asking InternalStateRequest while stopped");
            }
        } else {
            panic!("Timed out waiting for Stopped after InternalStateRequest while stopped");
        }

        orch_tx_for_test
            .send(OrchestratorToPlanet::KillPlanet)
            .expect("failed to send KillPlanet");
        if let Ok(msg) = orch_rx_for_test.recv_timeout(Duration::from_millis(500)) {
            if let PlanetToOrchestrator::KillPlanetResult { planet_id } = msg {
                assert_eq!(planet_id, 42);
            } else {
                panic!("Expected KillPlanetResult");
            }
        } else {
            panic!("Timed out waiting for KillPlanetResult");
        }

        let join_res = handle.join();
        assert!(join_res.is_ok(), "Planet thread panicked");
    }
}


