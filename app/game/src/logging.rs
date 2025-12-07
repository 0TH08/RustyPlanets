use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
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

#[derive(Debug, Clone)]
pub struct Payload(pub BTreeMap<String, String>);

impl Payload {
    pub fn new() -> Self {
        Payload(BTreeMap::new())
    }

    pub fn kv(k: impl Into<String>, v: impl Into<String>) -> Self {
        let mut m = BTreeMap::new();
        m.insert(k.into(), v.into());
        Payload(m)
    }
}

impl Default for Payload {
    fn default() -> Self {
        Payload::new()
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub timestamp_unix: i64,
    pub sender_type: ActorType,
    pub sender_id: u64,
    pub receiver_type: ActorType,
    pub receiver_id: String,
    pub event_type: EventType,
    pub channel: Channel,
    pub payload: Payload,
}

impl LogEvent {
    pub fn new(
        sender_type: ActorType,
        sender_id: impl Into<u64>,
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

    pub fn empty(
        sender_type: ActorType,
        sender_id: impl Into<u64>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
    ) -> Self {
        Self::new(
            sender_type,
            sender_id,
            receiver_type,
            receiver_id,
            event_type,
            Channel::Info,
            Payload::new(),
        )
    }

    pub fn info(
        sender_type: ActorType,
        sender_id: impl Into<u64>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
        payload: Payload,
    ) -> Self {
        Self::new(
            sender_type,
            sender_id,
            receiver_type,
            receiver_id,
            event_type,
            Channel::Info,
            payload,
        )
    }

    pub fn error(
        sender_type: ActorType,
        sender_id: impl Into<u64>,
        receiver_type: ActorType,
        receiver_id: impl Into<String>,
        event_type: EventType,
        payload: Payload,
    ) -> Self {
        Self::new(
            sender_type,
            sender_id,
            receiver_type,
            receiver_id,
            event_type,
            Channel::Error,
            payload,
        )
    }

    pub fn id_from_str(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    pub fn emit(&self) {
        use Channel::*;
        match self.channel {
            Error => log::error!("{:?}", self),
            Warning => log::warn!("{:?}", self),
            Info => log::info!("{:?}", self),
            Debug => log::debug!("{:?}", self),
            Trace => log::trace!("{:?}", self),
        }
    }

    pub fn with_kv(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.payload.0.insert(k.into(), v.into());
        self
    }

    pub fn planet_to_orchestrator(planet_id: u32, payload: Payload) -> Self {
        Self::info(
            ActorType::Planet,
            planet_id,
            ActorType::Orchestrator,
            "orchestrator",
            EventType::MessagePlanetToOrchestrator,
            payload,
        )
    }

    pub fn orchestrator_to_planet(planet_id: u32, payload: Payload) -> Self {
        Self::info(
            ActorType::Orchestrator,
            0u64,
            ActorType::Planet,
            planet_id.to_string(),
            EventType::MessageOrchestratorToPlanet,
            payload,
        )
    }

    pub fn explorer_to_planet(explorer_id: u32, planet_id: u32, payload: Payload) -> Self {
        Self::info(
            ActorType::Explorer,
            explorer_id,
            ActorType::Planet,
            planet_id.to_string(),
            EventType::MessageExplorerToPlanet,
            payload,
        )
    }
}

impl fmt::Display for LogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LogEvent {{ ts: {}, sender: {:?}#{}, receiver: {:?}/{}, event: {:?}, channel: {:?}, payload: {:?} }}",
            self.timestamp_unix,
            self.sender_type,
            self.sender_id,
            self.receiver_type,
            self.receiver_id,
            self.event_type,
            self.channel,
            &self.payload.0
        )
    }
}
