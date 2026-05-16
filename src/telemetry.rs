//! Telemetry hub for bridging the sync simulation engine to the async GUI server.
//!
//! The simulation runs on native threads using crossbeam channels.
//! The GUI server runs on tokio/axum. This module provides a shared state
//! layer that both can access safely.
//!
//! # Design: Pull-on-Demand
//!
//! The telemetry hub stores references to the simulation's internal state
//! (via `Arc<Mutex<>>`). When the GUI requests data via REST, the server
//! locks the relevant state and reads it. This avoids the overhead of
//! pushing state updates on every tick.
//!
//! # Thread Safety
//!
//! - `SimStatus` uses atomic fields for lock-free concurrent access
//! - `TelemetryHub.planets` uses `RwLock` for the planet registry
//! - Individual planet states use dynamic dispatch via `PlanetTelemetry` trait

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use serde::{Serialize, Deserialize};

use crate::planet_telemetry::{CommonPlanetSnapshot, PlanetTelemetry};
use crate::explorer::Bag;

// ── PlanetKind enum ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlanetKind {
    Skycartel,
    Luna4,
    BlackAdidasShoe,
    ImmutableCosmicBorrow,
    Rusteze,
    Crabtorio,
    Orbitron,
}

// ── Run state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RunState {
    Idle,
    Running,
    Paused,
}

impl RunState {
    pub fn from_atomic(flag: &AtomicBool) -> Self {
        if flag.load(Ordering::SeqCst) {
            RunState::Running
        } else {
            RunState::Paused
        }
    }
}

// ── Simulation status ────────────────────────────────────────────────

/// Thread-safe simulation status shared between the orchestrator (writer)
/// and the GUI server (reader).
pub struct SimStatus {
    pub tick: AtomicU64,
    pub run_state: AtomicBool,
    pub speed: AtomicU64, // f32 stored as bits for atomic access
}

impl Default for SimStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl SimStatus {
    pub fn new() -> Self {
        Self {
            tick: AtomicU64::new(0),
            run_state: AtomicBool::new(false),
            speed: AtomicU64::new(f32::to_bits(1.0) as u64),
        }
    }

    pub fn read_tick(&self) -> u64 {
        self.tick.load(Ordering::SeqCst)
    }

    pub fn set_tick(&self, tick: u64) {
        self.tick.store(tick, Ordering::SeqCst);
    }

    pub fn read_speed(&self) -> f32 {
        f32::from_bits(self.speed.load(Ordering::SeqCst) as u32)
    }

    pub fn set_speed(&self, speed: f32) {
        self.speed.store(f32::to_bits(speed) as u64, Ordering::SeqCst);
    }

    pub fn is_idle(&self) -> bool {
        self.read_tick() == 0 && !self.run_state.load(Ordering::SeqCst)
    }

    pub fn to_api_state(&self) -> RunState {
        if self.is_idle() {
            RunState::Idle
        } else {
            RunState::from_atomic(&self.run_state)
        }
    }
}

// ── Planet telemetry ─────────────────────────────────────────────────

#[allow(dead_code)]
pub struct ExplorerEntry {
    pub telemetry: ExplorerTelemetry,
}

impl ExplorerEntry {
    pub fn snapshot(&self) -> ExplorerSnapshot {
        self.telemetry.snapshot()
    }
}

#[allow(dead_code)]
pub struct PlanetEntry {
    pub name: String,
    pub kind: PlanetKind,
    pub telemetry: Box<dyn PlanetTelemetry>,
    pub ai_running: Arc<AtomicBool>,
}

impl PlanetEntry {
    pub fn snapshot(&self) -> PlanetSnapshot {
        let stats = self.telemetry.snapshot();
        PlanetSnapshot {
            name: self.name.clone(),
            kind: self.kind,
            stats,
            explorer_count: self.telemetry.explorer_count(),
            ai_running: self.ai_running.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanetSnapshot {
    pub name: String,
    pub kind: PlanetKind,
    pub stats: CommonPlanetSnapshot,
    pub explorer_count: usize,
    pub ai_running: bool,
}

// ── Explorer telemetry ───────────────────────────────────────────────

#[allow(dead_code)]
pub struct ExplorerTelemetry {
    pub bag: Arc<Mutex<Bag>>,
    pub ai_running: Arc<AtomicBool>,
}

impl ExplorerTelemetry {
    pub fn snapshot(&self) -> ExplorerSnapshot {
        let bag_guard = self.bag.lock().unwrap();
        let basic_resources: HashMap<String, u32> = bag_guard
            .basic_resources
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        let complex_resources: HashMap<String, u32> = bag_guard
            .complex_resources
            .iter()
            .map(|(k, v)| (format!("{:?}", k), *v))
            .collect();
        ExplorerSnapshot {
            basic_resources,
            complex_resources,
            ai_running: self.ai_running.load(Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerSnapshot {
    pub basic_resources: HashMap<String, u32>,
    pub complex_resources: HashMap<String, u32>,
    pub ai_running: bool,
}

// ── TelemetryHub ─────────────────────────────────────────────────────

pub struct TelemetryHub {
    pub sim_status: Arc<SimStatus>,
    pub planets: Arc<RwLock<HashMap<u32, PlanetEntry>>>,
    pub explorers: Arc<RwLock<HashMap<u32, ExplorerEntry>>>,
    pub explorer_planet: Arc<RwLock<HashMap<u32, u32>>>,
}

impl TelemetryHub {
    pub fn new() -> Self {
        Self {
            sim_status: Arc::new(SimStatus::new()),
            planets: Arc::new(RwLock::new(HashMap::new())),
            explorers: Arc::new(RwLock::new(HashMap::new())),
            explorer_planet: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update_explorer_planet(&self, explorer_id: u32, planet_id: u32) {
        self.explorer_planet.write().unwrap().insert(explorer_id, planet_id);
    }

    pub fn set_speed(&self, speed: f32) {
        self.sim_status.set_speed(speed);
    }

    /// Register an explorer for telemetry tracking.
    pub fn register_explorer(&self, id: u32, telemetry: ExplorerTelemetry) {
        let mut explorers = self.explorers.write().unwrap();
        explorers.insert(id, ExplorerEntry { telemetry });
    }

    /// Register a planet for telemetry tracking.
    pub fn register_planet(&self, id: u32, entry: PlanetEntry) {
        let mut planets = self.planets.write().unwrap();
        planets.insert(id, entry);
    }
}

impl Default for TelemetryHub {
    fn default() -> Self {
        Self::new()
    }
}
