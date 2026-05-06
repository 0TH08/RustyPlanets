//! Luna4 AI implementation
//!
//! This module contains the PlanetAI trait implementation for Luna4.
//! It handles all message processing from orchestrators and explorers
//! according to Luna4's lunar cycle and resource availability rules.

use common_game::components::planet::{PlanetAI, PlanetState, DummyPlanetState};
use common_game::components::resource::{Generator, Combinator, BasicResourceType};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::protocols::planet_explorer::*;
use common_game::utils::ID;

use std::collections::HashSet;

/// Luna4 AI implementation
///
/// Handles all game logic for Luna4 planets including:
/// - Lunar phase management and transitions
/// - Resource availability based on current phase
/// - Explorer interactions and message processing
#[allow(dead_code)]
pub struct Luna4AI {
    /// Planet ID
    id: ID,
    /// Current lunar phase
    phase: Luna4Phase,
    /// Statistics tracking
    stats: Luna4Stats,
}

impl Luna4AI {
    /// Creates a new Luna4 AI instance
    pub fn new(id: ID) -> Self {
        Self {
            id,
            phase: Luna4Phase::WaxingCrescent,
            stats: Luna4Stats::new(),
        }
    }
    
    /// Gets current resource availability
    pub fn get_current_resources(&self) -> AvailableResources {
        AvailableResources {
            current_phase: format!("{:?}", self.phase),
            basic_resources: self.get_available_basic(),
            complex_resources: vec![],
        }
    }
    
    fn get_available_basic(&self) -> Vec<BasicResourceType> {
        match self.phase {
            Luna4Phase::NewMoon => vec![],
            Luna4Phase::WaxingCrescent => vec![BasicResourceType::Hydrogen],
            Luna4Phase::FirstQuarter => vec![BasicResourceType::Hydrogen, BasicResourceType::Carbon],
            Luna4Phase::WaxingGibbous => vec![BasicResourceType::Hydrogen, BasicResourceType::Carbon, BasicResourceType::Oxygen],
            Luna4Phase::FullMoon => vec![BasicResourceType::Hydrogen, BasicResourceType::Carbon, BasicResourceType::Oxygen, BasicResourceType::Silicon],
            Luna4Phase::WaningGibbous => vec![BasicResourceType::Hydrogen, BasicResourceType::Carbon, BasicResourceType::Oxygen],
            Luna4Phase::LastQuarter => vec![BasicResourceType::Hydrogen, BasicResourceType::Carbon],
            Luna4Phase::WaningCrescent => vec![BasicResourceType::Hydrogen],
        }
    }
    
    fn update_phase(&mut self) {
        self.phase = match self.phase {
            Luna4Phase::NewMoon => Luna4Phase::WaxingCrescent,
            Luna4Phase::WaxingCrescent => Luna4Phase::FirstQuarter,
            Luna4Phase::FirstQuarter => Luna4Phase::WaxingGibbous,
            Luna4Phase::WaxingGibbous => Luna4Phase::FullMoon,
            Luna4Phase::FullMoon => Luna4Phase::WaningGibbous,
            Luna4Phase::WaningGibbous => Luna4Phase::LastQuarter,
            Luna4Phase::LastQuarter => Luna4Phase::WaningCrescent,
            Luna4Phase::WaningCrescent => Luna4Phase::NewMoon,
        };
    }
}

impl PlanetAI for Luna4AI {
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        self.stats.record_sunray_received();
        self.update_phase();
        state.charge_cell(sunray);
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, 0u32)),
            EventType::MessageOrchestratorToPlanet,
            Channel::Debug,
            Payload::from([
                ("action".to_string(), "sunray_charged".to_string()),
                ("phase".to_string(), format!("{:?}", self.phase)),
            ]),
        ).emit();
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        self.update_phase();
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, 0u32)),
            EventType::InternalPlanetAction,
            Channel::Warning,
            Payload::from([
                ("action".to_string(), "asteroid_no_rocket_type_d".to_string()),
                ("phase".to_string(), format!("{:?}", self.phase)),
            ]),
        ).emit();
        None // Type D: No rockets
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        self.update_phase();
        state.to_dummy()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
         _combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        self.stats.record_explorer_message_processed();
        self.update_phase();
        
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: HashSet::new(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest { explorer_id: _, resource } => {
                let available = self.get_available_basic();
                if !available.contains(&resource) {
                    self.stats.record_failed_generation();
                    LogEvent::new(
                        Some(Participant::new(ActorType::Planet, state.id())),
                        None,
                        EventType::MessageExplorerToPlanet,
                        Channel::Warning,
                        Payload::from([
                            ("action".to_string(), "generate_unavailable_resource".to_string()),
                            ("resource".to_string(), format!("{:?}", resource)),
                            ("phase".to_string(), format!("{:?}", self.phase)),
                        ]),
                    ).emit();
                    return Some(PlanetToExplorer::GenerateResourceResponse {
                        resource: None,
                    });
                }
                
                // Try to generate the resource
                if let Some((cell, _)) = state.full_cell() {
                    if let Ok(generated) = generator.try_make(resource, cell) {
                        self.stats.record_successful_generation();
                        LogEvent::new(
                            Some(Participant::new(ActorType::Planet, state.id())),
                            None,
                            EventType::MessageExplorerToPlanet,
                            Channel::Info,
                            Payload::from([
                                ("action".to_string(), "generate_success".to_string()),
                                ("resource".to_string(), format!("{:?}", resource)),
                            ]),
                        ).emit();
                        Some(PlanetToExplorer::GenerateResourceResponse {
                            resource: Some(generated),
                        })
                    } else {
                        self.stats.record_failed_generation();
                        LogEvent::new(
                            Some(Participant::new(ActorType::Planet, state.id())),
                            None,
                            EventType::MessageExplorerToPlanet,
                            Channel::Error,
                            Payload::from([
                                ("action".to_string(), "generate_failed".to_string()),
                                ("resource".to_string(), format!("{:?}", resource)),
                            ]),
                        ).emit();
                        Some(PlanetToExplorer::GenerateResourceResponse {
                            resource: None,
                        })
                    }
                } else {
                    self.stats.record_failed_generation();
                    LogEvent::new(
                        Some(Participant::new(ActorType::Planet, state.id())),
                        None,
                        EventType::MessageExplorerToPlanet,
                        Channel::Warning,
                        Payload::from([
                            ("action".to_string(), "generate_no_charge".to_string()),
                            ("resource".to_string(), format!("{:?}", resource)),
                        ]),
                    ).emit();
                    Some(PlanetToExplorer::GenerateResourceResponse {
                        resource: None,
                    })
                }
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id: _, msg } => {
                self.stats.record_failed_combination();
                LogEvent::new(
                    Some(Participant::new(ActorType::Planet, state.id())),
                    None,
                    EventType::MessageExplorerToPlanet,
                    Channel::Warning,
                    Payload::from([
                        ("action".to_string(), "combine_not_supported_type_d".to_string()),
                    ]),
                ).emit();
                let basic = |res| common_game::components::resource::GenericResource::BasicResources(res);
                
                // Retrieve the explorer's generic resource
                let (res1, res2) = match msg {
                    common_game::components::resource::ComplexResourceRequest::Water(h, o) => 
                        (basic(common_game::components::resource::BasicResource::Hydrogen(h)), 
                         basic(common_game::components::resource::BasicResource::Oxygen(o))),
                    common_game::components::resource::ComplexResourceRequest::Diamond(c1, c2) =>
                        (basic(common_game::components::resource::BasicResource::Carbon(c1)), 
                         basic(common_game::components::resource::BasicResource::Carbon(c2))),
                    common_game::components::resource::ComplexResourceRequest::Life(w, c) =>
                        (common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Water(w)), 
                         basic(common_game::components::resource::BasicResource::Carbon(c))),
                    common_game::components::resource::ComplexResourceRequest::Robot(s, l) =>
                        (basic(common_game::components::resource::BasicResource::Silicon(s)), 
                         common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Life(l))),
                    common_game::components::resource::ComplexResourceRequest::Dolphin(w, l) =>
                        (common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Water(w)), 
                         common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Life(l))),
                    common_game::components::resource::ComplexResourceRequest::AIPartner(r, d) =>
                        (common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Robot(r)), 
                         common_game::components::resource::GenericResource::ComplexResources(common_game::components::resource::ComplexResource::Diamond(d))),
                };
                
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((
                        "Luna4 (Type D) does not support resource combination".to_string(),
                        res1,
                        res2,
                    )),
                })
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
                let mut charged_cells: u32 = 0;
                state.cells_iter().for_each(|cell|
                    if cell.is_charged() { charged_cells += 1 }
                );
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: charged_cells,
                })
            }
        }
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: u32,
    ) {
        self.stats.record_explorer_message_processed();
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: u32,
    ) {
    }

    fn on_start(
        &mut self,
        state: &PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) {
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, 0u32)),
            EventType::InternalPlanetAction,
            Channel::Info,
            Payload::from([
                ("action".to_string(), "planet_ai_started".to_string()),
            ]),
        ).emit();
    }

    fn on_stop(
        &mut self,
        state: &PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) {
        LogEvent::new(
            Some(Participant::new(ActorType::Planet, state.id())),
            Some(Participant::new(ActorType::Orchestrator, 0u32)),
            EventType::InternalPlanetAction,
            Channel::Info,
            Payload::from([
                ("action".to_string(), "planet_ai_stopped".to_string()),
            ]),
        ).emit();
    }
}

/// Luna4 lunar phases
#[derive(Debug, Clone, Copy)]
enum Luna4Phase {
    NewMoon,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    FullMoon,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

/// Statistics tracking for Luna4
#[derive(Debug, Clone, Default)]
pub struct Luna4Stats {
    pub successful_generations: usize,
    pub failed_generations: usize,
    pub failed_combinations: usize,
    pub sunrays_received: usize,
    pub explorer_messages_processed: usize,
}

impl Luna4Stats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_successful_generation(&mut self) {
        self.successful_generations += 1;
    }
    
    pub fn record_failed_generation(&mut self) {
        self.failed_generations += 1;
    }
    
    pub fn record_failed_combination(&mut self) {
        self.failed_combinations += 1;
    }
    
    pub fn record_sunray_received(&mut self) {
        self.sunrays_received += 1;
    }
    
    pub fn record_explorer_message_processed(&mut self) {
        self.explorer_messages_processed += 1;
    }
}

/// Available resources response
pub struct AvailableResources {
    pub current_phase: String,
    pub basic_resources: Vec<BasicResourceType>,
    pub complex_resources: Vec<common_game::components::resource::ComplexResourceType>,
}
