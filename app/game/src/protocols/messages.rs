//! # Communication protocol messages
//!
//! Defines the types of messages exchanged between the different
//! components using [crossbeam_channel] channels.

use crate::components::asteroid::Asteroid;
use crate::components::planet::DummyPlanetState;
use crate::components::resource::{
    BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType,
    GenericResource,
};
use crate::components::rocket::Rocket;
use crate::components::sunray::Sunray;
use crossbeam_channel::Sender;
use std::collections::HashSet;

pub enum OrchestratorToPlanet {
    Sunray(Sunray),
    Asteroid(Asteroid),
    StartPlanetAI,
    StopPlanetAI,
    KillPlanet,
    InternalStateRequest,
    IncomingExplorerRequest {
        explorer_id: u32,
        new_mpsc_sender: Sender<PlanetToExplorer>,
    },
    OutgoingExplorerRequest {
        explorer_id: u32,
    },
}

pub enum PlanetToOrchestrator {
    SunrayAck {
        planet_id: u32,
    },
    AsteroidAck {
        planet_id: u32,
        rocket: Option<Rocket>,
    },
    StartPlanetAIResult {
        planet_id: u32,
    },
    StopPlanetAIResult {
        planet_id: u32,
    },
    KillPlanetResult {
        planet_id: u32,
    },
    InternalStateResponse {
        planet_id: u32,
        planet_state: DummyPlanetState,
    },
    IncomingExplorerResponse {
        planet_id: u32,
        res: Result<(), String>,
    },
    OutgoingExplorerResponse {
        planet_id: u32,
        res: Result<(), String>,
    },
    Stopped {
        planet_id: u32,
    },
}

impl PlanetToOrchestrator {
    pub fn planet_id(&self) -> u32 {
        match self {
            PlanetToOrchestrator::SunrayAck { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::AsteroidAck { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::StartPlanetAIResult { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::StopPlanetAIResult { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::KillPlanetResult { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::InternalStateResponse { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::IncomingExplorerResponse { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, .. } => *planet_id,
            PlanetToOrchestrator::Stopped { planet_id, .. } => *planet_id,
        }
    }
}

pub enum OrchestratorToExplorer {
    StartExplorerAI,
    ResetExplorerAI,
    KillExplorerAI,
    MoveToPlanet {
        sender_to_new_planet: Option<Sender<ExplorerToPlanet>>,
    },
    CurrentPlanetRequest,
    SupportedResourceRequest,
    SupportedCombinationRequest,
    GenerateResourceRequest {
        to_generate: BasicResourceType,
    },
    CombineResourceRequest(ComplexResourceRequest),
    BagContentRequest,
    NeighborsResponse {
        neighbors: Vec<u32>,
    },
}

pub enum ExplorerToOrchestrator<T> {
    StartExplorerAIResult {
        explorer_id: u32,
    },
    KillExplorerAIResult {
        explorer_id: u32,
    },
    ResetExplorerAIResult {
        explorer_id: u32,
    },
    MovedToPlanetResult {
        explorer_id: u32,
    },
    CurrentPlanetResult {
        explorer_id: u32,
        planet_id: u32,
    },
    SupportedResourceResult {
        explorer_id: u32,
        supported_resources: HashSet<BasicResourceType>,
    },
    SupportedCombinationResult {
        explorer_id: u32,
        combination_list: HashSet<ComplexResourceType>,
    },
    GenerateResourceResponse {
        explorer_id: u32,
        generated: Result<(), ()>,
    },
    CombineResourceResponse {
        explorer_id: u32,
        generated: Result<(), ()>,
    },
    BagContentResponse {
        explorer_id: u32,
        bag_content: T,
    },
    NeighborsRequest {
        explorer_id: u32,
        current_planet_id: u32,
    },
    TravelToPlanetRequest {
        explorer_id: u32,
        current_planet_id: u32,
        dst_planet_id: u32,
    },
}

impl<T> ExplorerToOrchestrator<T> {
    pub fn explorer_id(&self) -> u32 {
        match self {
            Self::StartExplorerAIResult { explorer_id, .. } => *explorer_id,
            Self::KillExplorerAIResult { explorer_id, .. } => *explorer_id,
            Self::ResetExplorerAIResult { explorer_id, .. } => *explorer_id,
            Self::MovedToPlanetResult { explorer_id, .. } => *explorer_id,
            Self::CurrentPlanetResult { explorer_id, .. } => *explorer_id,
            Self::SupportedResourceResult { explorer_id, .. } => *explorer_id,
            Self::SupportedCombinationResult { explorer_id, .. } => *explorer_id,
            Self::GenerateResourceResponse { explorer_id, .. } => *explorer_id,
            Self::CombineResourceResponse { explorer_id, .. } => *explorer_id,
            Self::BagContentResponse { explorer_id, .. } => *explorer_id,
            Self::NeighborsRequest { explorer_id, .. } => *explorer_id,
            Self::TravelToPlanetRequest { explorer_id, .. } => *explorer_id,
        }
    }
}

pub enum ExplorerToPlanet {
    SupportedResourceRequest {
        explorer_id: u32,
    },
    SupportedCombinationRequest {
        explorer_id: u32,
    },
    GenerateResourceRequest {
        explorer_id: u32,
        resource: BasicResourceType,
    },
    CombineResourceRequest {
        explorer_id: u32,
        msg: ComplexResourceRequest,
    },
    AvailableEnergyCellRequest {
        explorer_id: u32,
    },
}

impl ExplorerToPlanet {
    pub fn explorer_id(&self) -> u32 {
        match self {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id, .. } => *explorer_id,
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id, .. } => *explorer_id,
            ExplorerToPlanet::GenerateResourceRequest { explorer_id, .. } => *explorer_id,
            ExplorerToPlanet::CombineResourceRequest { explorer_id, .. } => *explorer_id,
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id, .. } => *explorer_id,
        }
    }
}

pub enum PlanetToExplorer {
    SupportedResourceResponse {
        resource_list: HashSet<BasicResourceType>,
    },
    SupportedCombinationResponse {
        combination_list: HashSet<ComplexResourceType>,
    },
    GenerateResourceResponse {
        resource: Option<BasicResource>,
    },
    CombineResourceResponse {
        complex_response: Result<ComplexResource, (String, GenericResource, GenericResource)>,
    },
    AvailableEnergyCellResponse {
        available_cells: u32,
    },
    Stopped,
}
