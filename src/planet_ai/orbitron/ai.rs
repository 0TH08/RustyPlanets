//! # Orbitron – Planet AI Module
//!
//! This module implements the [PlanetAI] trait for the Orbitron planet.  
//! It defines how the planet reacts to all types of messages coming from both
//! the Orchestrator and Explorers, and how it handles internal game
//! logic such as resource generation, resource combination, energy management,
//! rocket construction, and survival after asteroid impacts.
//!
//! ## Responsibilities
//!
//! The Orbitron AI controls:
//!
//! - Sunray handling
//!   Charges energy cells when possible, otherwise replies with `SunrayAck`.
//!
//! - Internal state reporting*
//!   Returns a snapshot of the current [PlanetState] when requested.
//!
//! - Explorer interactions
//!   * Supported recipes from the [Generator] and [Combinator]  
//!   * Resource generation requests  
//!   * Resource combination (including error reporting)  
//!   * Energy cell availability
//!
//! - Asteroid survival logic  
//!   Attempts to build a [Rocket] and return it.  
//!   If no rocket is returned, the planet is destroyed.
//!
//! - Lifecycle control  
//!   Handles `StartPlanetAI` and `StopPlanetAI` messages, enabling
//!   or disabling the decision-making logic.
use common_game::components::planet::{DummyPlanetState, PlanetAI, PlanetState};
use common_game::components::resource::{
    BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest, Generator,
    GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::logging::*;
use common_game::protocols::planet_explorer::*;
use common_game::utils::ID;

/// Set channels for incoming/outgoing messages
const RCV_MSG_CHNL: Channel = Channel::Debug;
const ACK_MSG_CHNL: Channel = Channel::Debug;

const ORCHESTRATOR_ID: ID = 0;

/// Helper functions to convert messages and responses into string names
fn explorer_to_planet_name(msg: &ExplorerToPlanet) -> String {
    match msg {
        ExplorerToPlanet::SupportedResourceRequest { .. } => "Supported Resource Request".into(),
        ExplorerToPlanet::SupportedCombinationRequest { .. } => {
            "Supported Combination Request".into()
        }
        ExplorerToPlanet::GenerateResourceRequest { .. } => "Generate Resource Request".into(),
        ExplorerToPlanet::CombineResourceRequest { .. } => "Combine Resource Request".into(),
        ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
            "Available Energy Cell Request".into()
        }
    }
}
/// Helper functions to convert messages and responses into string names
fn planet_to_explorer_name(msg: &PlanetToExplorer) -> String {
    match msg {
        PlanetToExplorer::SupportedResourceResponse { .. } => "Supported Resource Response".into(),
        PlanetToExplorer::SupportedCombinationResponse { .. } => {
            "Supported Combination Response".into()
        }
        PlanetToExplorer::GenerateResourceResponse { .. } => "Generate Resource Response".into(),
        PlanetToExplorer::CombineResourceResponse { .. } => "Combine Resource Response".into(),
        PlanetToExplorer::AvailableEnergyCellResponse { .. } => {
            "Available Energy Cell Response".into()
        }
        _ => "Unexpected Message".into(),
    }
}

/// Represents the AI controller for the Orbitron planet.
///
/// The `is_stopped` flag indicates whether the planet's AI is currently
/// inactive and should ignore incoming logic or requests.
pub struct Orbitron {
    is_stopped: bool,
}

/// Creates a new `Orbitron` AI instance.
///
/// By default, the AI starts in the stopped state and will only
/// begin processing once explicitly started.
impl Orbitron {
    pub fn new(id: ID) -> Self {
        // LOG internal ai creation
        let mut payload = Payload::new();
        payload.insert("Message".into(), "New AI orbitron created".into());
        LogEvent::self_directed(
            Participant::new(ActorType::Planet, id),
            EventType::InternalPlanetAction,
            Channel::Info,
            payload,
        )
        .emit();

        Self { is_stopped: true }
    }
}

impl PlanetAI for Orbitron {
    /// This function is used to handle Sunray msg
    /// # Returns
    /// SunrayAck indicates Sunray is not consumed due to full energy cells
    /// None indicates energy cell received Sunray
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        let mut payload = Payload::new();

        if state.charge_cell(sunray).is_some() {
            payload.insert("Energy Cell State".into(), "Energy Cell full".into());
        } else {
            payload.insert("Energy Cell State".into(), "Energy Cell charged".into());
        }

        // LOG incoming sunray handle
        LogEvent::broadcast(
            Participant::new(ActorType::Planet, state.id()),
            EventType::InternalPlanetAction,
            RCV_MSG_CHNL,
            payload,
        )
        .emit();
    }

    /// This function is used to handle InternalStateRequest msg
    /// # Returns
    /// DummyPlanetState
    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        let mut payload = Payload::new();

        payload.insert("Planet State".into(), format!("{:?}", state.to_dummy()));

        // LOG internal state response
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
            EventType::MessagePlanetToOrchestrator,
            ACK_MSG_CHNL,
            payload,
        )
        .emit();

        state.to_dummy()
    }

    /// Handles messages from explorers.
    ///
    /// - Provides supported basic and complex resource types
    /// - Generates requested basic resources (Hydrogen or Oxygen).  
    ///   First, we check whether there is any charged cell (the `full_cell` function does this).  
    ///   If there is, we then check whether the requested `BasicResourceType` is Hydrogen or Oxygen.  
    ///   If it is, we generate it; otherwise, the function returns `None`.
    /// - Generates Water as the only supported complex resource.  
    ///   As before, we must check whether there is a charged cell.  
    ///   Since the planet can only generate water, if the requested complex resource type is `Water`,
    ///   we proceed with generation; otherwise, we return an error message.
    /// - Returns the number of available charged energy cells.
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        let explorer_id: ID = msg.explorer_id();

        // LOG incoming explorer message
        let mut in_payload = Payload::new();
        in_payload.insert("Message".into(), explorer_to_planet_name(&msg));

        LogEvent::new(
            Some(Participant::new(ActorType::Orchestrator, explorer_id)),
            Some(Participant::new(ActorType::Planet, state.id())),
            EventType::MessageExplorerToPlanet,
            RCV_MSG_CHNL,
            in_payload,
        )
        .emit();


        // LOG explorer message result
        let mut payload = Payload::new();

        let response = match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _id } => {
                payload.insert(
                    "Supported Resources".into(),
                    format!("{:?}", generator.all_available_recipes()),
                );

                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _id } => {
                payload.insert(
                    "Supported Combinations".into(),
                    format!("{:?}", combinator.all_available_recipes()),
                );

                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _id,
                resource,
            } => {
                let generated_resource = state.full_cell().and_then(|(cell, _)| match resource {
                    BasicResourceType::Hydrogen => generator
                        .make_hydrogen(cell)
                        .ok()
                        .map(|hydrogen| hydrogen.to_basic()),
                    BasicResourceType::Oxygen => generator
                        .make_oxygen(cell)
                        .ok()
                        .map(|oxygen| oxygen.to_basic()),
                    _ => None,
                });
                if generated_resource.is_some() {
                    payload.insert(
                        "Generated Resource".into(),
                        format!("{:?}", generated_resource),
                    );
                } else {
                    payload.insert(
                        "Generated Resource".into(),
                        "Unsupported Resource Generation Request".into(),
                    );
                }

                Some(PlanetToExplorer::GenerateResourceResponse {
                    resource: generated_resource,
                })
            }
            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _id,
                msg,
            } => {
                let cell = state.full_cell();

                let ret: Result<ComplexResource, (String, GenericResource, GenericResource)> =
                    match msg {
                        ComplexResourceRequest::Water(resource_1, resource_2) => match cell {
                            Some((cell, _)) => combinator
                                .make_water(resource_1, resource_2, cell)
                                .map(|water| water.to_complex())
                                .map_err(|(err_str, return_resource_1, return_resource_2)| {
                                    (
                                        err_str,
                                        return_resource_1.to_generic(),
                                        return_resource_2.to_generic(),
                                    )
                                }),
                            None => Err((
                                "No charged energy cell found".to_string(),
                                resource_1.to_generic(),
                                resource_2.to_generic(),
                            )),
                        },

                        other => {
                            let variant_name = format!("{other:?}");

                            let (resource_1, resource_2) = match other {
                                ComplexResourceRequest::Diamond(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Life(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Robot(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Dolphin(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::AIPartner(r1, r2) => {
                                    (r1.to_generic(), r2.to_generic())
                                }
                                ComplexResourceRequest::Water(_, _) => {
                                    unreachable!("Water is handled above")
                                }
                            };

                            Err((
                                format!("There isn't a recipe for {variant_name:?}"),
                                resource_1,
                                resource_2,
                            ))
                        }
                    };
                if ret.is_ok() {
                    payload.insert("Combined Resource".into(), format!("{:?}", ret));
                } else {
                    payload.insert(
                        "Combined Resource".into(),
                        format!("Unsupported Resource Combination Request: {:?}", ret),
                    );
                }

                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: ret,
                })
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _id } => {
                let mut cnt: u32 = 0;
                for cell in state.cells_iter() {
                    if cell.is_charged() {
                        cnt += 1;
                    }
                }
                payload.insert("Available Energy Cells".into(), format!("{:?}", cnt));

                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: cnt,
                })
            }
        };

        // LOG planet response
        if let Some(ref res) = response {
            payload.insert("Response".into(), planet_to_explorer_name(res));
            LogEvent::new(
                Some(Participant::new(ActorType::Planet, state.id())),
                Some(Participant::new(ActorType::Orchestrator, explorer_id)),
                EventType::MessagePlanetToExplorer,
                ACK_MSG_CHNL,
                payload,
            )
            .emit();
        }

        response
    }
    /// This handler will be invoked when a [OrchestratorToPlanet::Asteroid]
    /// message is received.
    ///
    /// # Returns
    /// In order to survice, planet try to build rocket.
    /// After this attempt an owned [Rocket] must be returned from this method;
    /// if `None` is returned instead, the planet will  be destroyed by the orchestrator
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        // LOG incoming asteroid
        let mut payload = Payload::new();
        payload.insert("Message".into(), "Asteroid".into());
        LogEvent::new(
            Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
            Some(Participant::new(ActorType::Planet, state.id())),
            EventType::MessageOrchestratorToPlanet,
            RCV_MSG_CHNL,
            payload,
        )
        .emit();

        // LOG asteroid response
        let mut payload = Payload::new();

        let has_rocket = state.has_rocket();
        if has_rocket {
            payload.insert("Result".into(), "Rocket was Ready".into());
        } else {
            payload.insert("Result".into(), "Rocket was Built".into());
            let _ = state.build_rocket(0);
        }
        let rocket = state.take_rocket();

        if rocket.is_some() {
            payload.insert("Result".into(), "Rocket is Available".into());
        } else {
            payload.insert("Result".into(), "No Rocket Available".into());
        }
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
            EventType::MessagePlanetToOrchestrator,
            ACK_MSG_CHNL,
            payload,
        )
        .emit();

        rocket
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: ID,
    ) {
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StartPlanetAI]
    /// is received, but only if the planet is currently in a stopped state.
    ///
    /// Start messages received when planet is already running are ignored.
    fn on_start(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        self.is_stopped = false;

        let mut payload = Payload::new();
        payload.insert("Message".into(), "Started Planet Orbitron".into());

        LogEvent::new(
            Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
            Some(Participant::new(ActorType::Planet, state.id())),
            EventType::MessageOrchestratorToPlanet,
            RCV_MSG_CHNL,
            payload,
        )
        .emit();
    }

    /// This method will be invoked when a [OrchestratorToPlanet::StopPlanetAI]
    /// is received, but only if the planet is currently in a running state.
    ///
    /// Stop messages received when planet is already stopped are ignored.
    fn on_stop(&mut self, state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
        self.is_stopped = true;

        let mut payload = Payload::new();
        payload.insert("Message".into(), "Stoped Planet Orbitron".into());
        LogEvent::new(
            Some(Participant::new(ActorType::Orchestrator, ORCHESTRATOR_ID)),
            Some(Participant::new(ActorType::Planet, state.id())),
            EventType::MessageOrchestratorToPlanet,
            RCV_MSG_CHNL,
            payload,
        )
        .emit();
    }
}

use common_game::components::planet::{Planet, PlanetType};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use crate::planet_telemetry::{SharedPlanetStats, StatsTrackingAI};

/// Creates and initializes a new Orbitron planet.
///
/// Orbitron is a Type A planet: 5 energy cells, at most 1 generation recipe,
/// rockets allowed, no combination recipes.
pub fn create_planet(
    id: ID,
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
) -> Result<Planet, String> {
    let ai = Orbitron::new(id);
    Planet::new(
        id,
        PlanetType::A,
        Box::new(ai),
        vec![BasicResourceType::Hydrogen],
        vec![],
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    )
}

pub fn create_planet_with_stats(
    id: ID,
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
) -> Result<(Planet, Arc<Mutex<SharedPlanetStats>>), String> {
    let stats = Arc::new(Mutex::new(SharedPlanetStats::default()));
    let ai = StatsTrackingAI { inner: Orbitron::new(id), stats: Arc::clone(&stats) };
    let planet = Planet::new(
        id,
        PlanetType::A,
        Box::new(ai),
        vec![BasicResourceType::Hydrogen],
        vec![],
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    )?;
    Ok((planet, stats))
}