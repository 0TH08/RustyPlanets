use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum ActorType {
    Planet,
    Explorer,
    Orchestrator,
    User,
    Broadcast,
    SelfActor,
}

#[derive(Debug, Clone)]
pub enum Channel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone)]
pub enum EventType {
    MessagePlanetToOrchestrator,
    MessagePlanetToExplorer,
    MessageOrchestratorToExplorer,
    MessageOrchestratorToPlanet,
    MessageExplorerToPlanet,
    MessageExplorerToOrchestrator,
    InternalPlanetAction,
    InternalExplorerAction,
    InternalOrchestratorAction,
    UserToPlanet,
    UserToExplorer,
    UserToOrchestrator,
}

pub type Payload = BTreeMap<String, String>;

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp_unix: i64,
    pub sender_type: ActorType,
    pub sender_id: String,
    pub receiver_type: ActorType,
    pub receiver_id: String,
    pub event_type: EventType,
    pub channel: Channel,
    pub payload: Payload,
}

impl LogEvent {
    /// Helper to create an event with current time..
    pub fn new(
        sender_type: ActorType,
        sender_id: impl Into<String>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
        channel: Channel,
        payload: Payload,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            timestamp_unix: now,
            sender_type,
            sender_id: sender_id.into(),
            receiver_type,
            receiver_id: receiver_id.into(),
            event_type,
            channel,
            payload,
        }
    }

    /// Emit this event using the `log` crate_..
    pub fn emit(&self) {
        use Channel::*;

        // For now we log everything via `Debug` formattin:
        match self.channel {
            Error => log::error!("{:?}", self),
            Warning => log::warn!("{:?}", self),
            Info => log::info!("{:?}", self),
            Debug => log::debug!("{:?}", self),
            Trace => log::trace!("{:?}", self),
        }
    }
}
