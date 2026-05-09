use common_game::logging::ActorType;
use common_game::logging::Channel;
use common_game::logging::EventType;
use common_game::logging::LogEvent;
use common_game::logging::Participant;
use common_game::logging::Payload;
use common_game::utils::ID;

const ORCH_ID: ID = 0u32;
pub fn exit_on_stopped_ai(is_on: bool, planet_id: ID) -> bool {
    if is_on {
        false
    } else {
        log_ai_state(String::from("AI field `is_on` is false"), planet_id);
        true
    }
}
// Begin of Log functions
pub fn log_ai_state(msg: String, planet_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("AI state".to_string(), msg)]),
    )
    .emit();
}

pub fn log_explorer_transit(msg: String, planet_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Explorer Transit".to_string(), msg)]),
    )
    .emit();
}

pub fn log_resource_created(msg: String, planet_id: ID, explorer_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessageExplorerToPlanet,
        Channel::Info,
        Payload::from([("Resource created".to_string(), msg)]),
    )
    .emit();
}

pub fn log_not_created_resource(err: String, planet_id: ID, explorer_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessageExplorerToPlanet,
        Channel::Error,
        Payload::from([("Cannot make resource".to_string(), err)]),
    )
    .emit();
}
pub fn log_internal_state(msg: String, planet_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Internal state".to_string(), msg)]),
    )
    .emit();
}
pub fn log_asteroid_impact(msg: String, planet_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessagePlanetToOrchestrator,
        Channel::Info,
        Payload::from([("Asteroid Impact".to_string(), msg)]),
    )
    .emit();
}

pub fn log_cell_charge(msg: String, planet_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Orchestrator, ORCH_ID)),
        EventType::MessageOrchestratorToPlanet,
        Channel::Info,
        Payload::from([("Cell charge".to_string(), msg)]),
    )
    .emit();
}
pub fn log_supported_resources(msg: String, planet_id: ID, explorer_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessagePlanetToExplorer,
        Channel::Info,
        Payload::from([("Supported resources".to_string(), msg)]),
    )
    .emit();
}
pub fn log_generation_rules(msg: String, planet_id: ID, explorer_id: ID) {
    LogEvent::new(
        Some(Participant::new(ActorType::Planet, planet_id)),
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        EventType::MessagePlanetToExplorer,
        Channel::Info,
        Payload::from([("Generation rules".to_string(), msg)]),
    )
    .emit();
}