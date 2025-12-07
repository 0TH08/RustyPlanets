//! # Planet module
//!
//! This module provides the common definitions for planets and their associated types
//! that groups need to implement their own planet behavior.
//!
//! The main type is [Planet], which represents an actual planet and contains all the
//! logic and state ([PlanetState]) necessary to run as a planet. The orchestrator
//! interacts with instances of [Planet] through the channels provided at construction.
//!
//! Groups provide a struct implementing [PlanetAI] to define the planet behavior.
//! The AI receives messages from the orchestrator and explorers and can mutate the
//! planet state. See [PlanetAI] for handler semantics.
//!
//! NOTE: Planet type constraints follow the project specification (see PDF, section 3.7.2).
//! The table in the specification is reproduced here for clarity:
//!
//! 3.7.2 Planet Types
//!
//! Type | Energy Cells | Generation Recipes | Rockets | Combination Recipes
//! -----|--------------|--------------------|---------|---------------------
//! A    | 5 cells      | At most 1 type     | Allowed | None
//! B    | 1 cell       | Unlimited types    | Not allowed | 1 type
//! C    | 1 cell       | At most 1 type     | Allowed | All 6 types
//! D    | 5 cells      | Unlimited types    | Not allowed | None
//!
//! The implementation below encodes these constraints in PlanetType::constraints().

use crate::components::energy_cell::EnergyCell;
use crate::components::resource::{BasicResourceType, Combinator, ComplexResourceType, Generator};
use crate::components::rocket::Rocket;
use crate::components::sunray::Sunray;
use crate::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::HashMap;
use std::slice::{Iter, IterMut};

/// The trait that defines the behaviour of a planet.
///
/// Structs implementing this trait are intended to be passed to the
/// [Planet] constructor, so that the handlers can be invoked by the planet
/// internal logic when certain messages are received on any of the planet channels.
///
/// The handlers can alter the planet state by accessing the
/// `state` parameter, which is passed to the methods as a mutable borrow.
/// A response can be sent by returning an optional message of the correct type,
/// that will be forwarded to the associated channel passed on planet construction.
pub trait PlanetAI: Send {
    /// Handler for messages received from the orchestrator (receiver end of the [OrchestratorToPlanet] channel).
    ///
    /// The following orchestrator messages are handled specially by the planet core and
    /// will not be dispatched to this handler:
    /// - [OrchestratorToPlanet::StartPlanetAI] (handled by [PlanetAI::start])
    /// - [OrchestratorToPlanet::StopPlanetAI] (handled by [PlanetAI::stop])
    /// - [OrchestratorToPlanet::Asteroid] (handled by [PlanetAI::handle_asteroid])
    /// - [OrchestratorToPlanet::IncomingExplorerRequest] (handled directly by Planet)
    /// - [OrchestratorToPlanet::OutgoingExplorerRequest] (handled directly by Planet)
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator>;

    /// Handler for all messages received from explorers (receiver end of the [ExplorerToPlanet] channel).
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer>;

    /// Handler invoked when an asteroid hits the planet (OrchestratorToPlanet::Asteroid).
    ///
    /// To survive, the AI must return Some(Rocket). Returning None indicates the planet
    /// failed to defend itself and will be considered destroyed by the orchestrator.
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket>;

    /// Called when a StartPlanetAI message is received and the planet is currently stopped.
    fn start(&mut self, state: &PlanetState);

    /// Called when a StopPlanetAI message is received and the planet is currently running.
    fn stop(&mut self, state: &PlanetState);
}

/// Contains planet rules constraints (see [PlanetType]).
pub struct PlanetConstraints {
    n_energy_cells: usize,
    unbounded_gen_rules: bool,
    can_have_rocket: bool,
    n_comb_rules: usize,
}

/// Planet types definitions, intended to be passed to the planet constructor.
/// Identifies the planet rules constraints, with each type having its own.
#[derive(Debug, Clone, Copy)]
pub enum PlanetType {
    A,
    B,
    C,
    D,
}

impl PlanetType {
    const N_ENERGY_CELLS: usize = 5;
    const N_RESOURCE_COMB_RULES: usize = 6;

    /// Returns a PlanetConstraints struct with the constraints associated to this planet type,
    /// as specified in the project requirements (see PDF section 3.7.2).
    pub fn constraints(&self) -> PlanetConstraints {
        match self {
            PlanetType::A => PlanetConstraints {
                n_energy_cells: Self::N_ENERGY_CELLS,
                unbounded_gen_rules: false, // at most 1 generation recipe
                can_have_rocket: true,
                n_comb_rules: 0,
            },
            PlanetType::B => PlanetConstraints {
                n_energy_cells: 1,
                unbounded_gen_rules: true, // unlimited generation recipes
                can_have_rocket: false,
                n_comb_rules: 1,
            },
            PlanetType::C => PlanetConstraints {
                n_energy_cells: 1,
                unbounded_gen_rules: false, // at most 1 generation recipe
                can_have_rocket: true,
                n_comb_rules: Self::N_RESOURCE_COMB_RULES, // all 6 combination recipes
            },
            PlanetType::D => PlanetConstraints {
                n_energy_cells: Self::N_ENERGY_CELLS,
                unbounded_gen_rules: true, // unlimited generation recipes
                can_have_rocket: false,
                n_comb_rules: 0,
            },
        }
    }
}

/// Representation of the planet's internal state.
/// Provides access to energy cells, an optional rocket, and flags coming from the planet type.
pub struct PlanetState {
    id: u32,
    energy_cells: Vec<EnergyCell>,
    rocket: Option<Rocket>,
    can_have_rocket: bool,
}

impl PlanetState {
    /// Returns the planet id.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Indexed getter for an immutable reference to the i-th energy cell.
    ///
    /// # Panics
    /// Panics if i is out of bounds. Use [PlanetState::cells_count] beforehand.
    pub fn cell(&self, i: usize) -> &EnergyCell {
        &self.energy_cells[i]
    }

    /// Indexed getter for a mutable reference to the i-th energy cell.
    ///
    /// # Panics
    /// Panics if i is out of bounds. Use [PlanetState::cells_count] beforehand.
    pub fn cell_mut(&mut self, i: usize) -> &mut EnergyCell {
        &mut self.energy_cells[i]
    }

    /// Returns the number of energy cells the planet owns.
    pub fn cells_count(&self) -> usize {
        self.energy_cells.len()
    }

    /// Returns an immutable iterator over the energy cells.
    pub fn cells_iter(&self) -> Iter<'_, EnergyCell> {
        self.energy_cells.iter()
    }

    /// Returns a mutable iterator over the energy cells.
    pub fn cells_iter_mut(&mut self) -> IterMut<'_, EnergyCell> {
        self.energy_cells.iter_mut()
    }

    /// Charges the first empty (discharged) cell.
    /// Returns Some(sunray) if there is no empty cell (sunray not consumed).
    pub fn charge_cell(&mut self, sunray: Sunray) -> Option<Sunray> {
        match self.empty_cell() {
            None => Some(sunray),
            Some((cell, _)) => {
                cell.charge(sunray);
                None
            }
        }
    }

    /// Returns a mutable borrow to the first empty (discharged) cell and its index, or None.
    pub fn empty_cell(&mut self) -> Option<(&mut EnergyCell, usize)> {
        let idx = self.energy_cells.iter().position(|cell| !cell.is_charged());
        idx.map(|i| (&mut self.energy_cells[i], i))
    }

    /// Returns a mutable borrow to the first full (charged) cell and its index, or None.
    pub fn full_cell(&mut self) -> Option<(&mut EnergyCell, usize)> {
        let idx = self.energy_cells.iter().position(|cell| cell.is_charged());
        idx.map(|i| (&mut self.energy_cells[i], i))
    }

    /// Returns true if the planet can have a rocket (based on its type).
    pub fn can_have_rocket(&self) -> bool {
        self.can_have_rocket
    }

    /// Returns true if the planet currently has a rocket stored and ready.
    pub fn has_rocket(&self) -> bool {
        self.rocket.is_some()
    }

    /// Takes ownership of the rocket if present, leaving the planet without one.
    pub fn take_rocket(&mut self) -> Option<Rocket> {
        self.rocket.take()
    }

    /// Builds a rocket consuming the i-th energy cell (must be charged) and stores it in the planet.
    ///
    /// # Panics
    /// Panics if i is out of bounds. Use [PlanetState::cells_count] beforehand.
    ///
    /// # Errors
    /// Errors if:
    /// - Planet type doesn't support rockets.
    /// - Planet already has a rocket.
    /// - The energy cell at index i is not charged.
    pub fn build_rocket(&mut self, i: usize) -> Result<(), String> {
        if !self.can_have_rocket {
            Err("This planet type can't have rockets.".to_string())
        } else if self.has_rocket() {
            Err("This planet already has a rocket.".to_string())
        } else {
            let energy_cell = self.cell_mut(i);
            Rocket::new(energy_cell).map(|rocket| {
                self.rocket = Some(rocket);
            })
        }
    }

    /// Returns a simplified clone of the planet state for external introspection.
    pub fn to_dummy(&self) -> DummyPlanetState {
        DummyPlanetState {
            energy_cells: self
                .energy_cells
                .iter()
                .map(|cell| cell.is_charged())
                .collect(),
            charged_cells_count: self
                .energy_cells
                .iter()
                .filter(|cell| cell.is_charged())
                .count(),
            has_rocket: self.has_rocket(),
        }
    }
}

/// A lightweight overview of a planet's internal state used for introspection requests.
#[derive(Debug, Clone)]
pub struct DummyPlanetState {
    pub energy_cells: Vec<bool>,
    pub charged_cells_count: usize,
    pub has_rocket: bool,
}

/// Main planet type that composes state, type, AI and resource handlers.
pub struct Planet {
    state: PlanetState,
    planet_type: PlanetType,
    pub ai: Box<dyn PlanetAI>,
    generator: Generator,
    combinator: Combinator,

    from_orchestrator: Receiver<OrchestratorToPlanet>,
    to_orchestrator: Sender<PlanetToOrchestrator>,
    from_explorers: Receiver<ExplorerToPlanet>,
    to_explorers: HashMap<u32, Sender<PlanetToExplorer>>,
}

impl Planet {
    const ORCH_DISCONNECT_ERR: &str = "Orchestrator disconnected.";

    /// Constructor for Planet.
    ///
    /// # Arguments
    /// - `id`: planet id.
    /// - `planet_type`: the planet type (constraints are enforced).
    /// - `ai`: boxed PlanetAI implementation.
    /// - `gen_rules`: list of BasicResourceType generation rules.
    /// - `comb_rules`: list of ComplexResourceType combination rules.
    /// - `orchestrator_channels`: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>).
    /// - `explorers_receiver`: Receiver<ExplorerToPlanet> for incoming explorer requests.
    ///
    /// # Errors
    /// Returns Err if construction parameters violate planet type constraints.
    pub fn new(
        id: u32,
        planet_type: PlanetType,
        ai: Box<dyn PlanetAI>,
        gen_rules: Vec<BasicResourceType>,
        comb_rules: Vec<ComplexResourceType>,
        orchestrator_channels: (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>),
        explorers_receiver: Receiver<ExplorerToPlanet>,
    ) -> Result<Planet, String> {
        let PlanetConstraints {
            n_energy_cells,
            unbounded_gen_rules,
            can_have_rocket,
            n_comb_rules,
        } = planet_type.constraints();
        let (from_orchestrator, to_orchestrator) = orchestrator_channels;

        if gen_rules.is_empty() {
            Err("gen_rules is empty".to_string())
        } else if !unbounded_gen_rules && gen_rules.len() > 1 {
            Err(format!(
                "Too many generation rules (Planet type {:?} is limited to 1)",
                planet_type
            ))
        } else if comb_rules.len() > n_comb_rules {
            Err(format!(
                "Too many combination rules (Planet type {:?} is limited to {})",
                planet_type, n_comb_rules
            ))
        } else {
            let mut generator = Generator::new();
            let mut combinator = Combinator::new();

            // add generation and combination rules
            for r in gen_rules {
                let _ = generator.add(r);
            }
            for r in comb_rules {
                let _ = combinator.add(r);
            }

            Ok(Planet {
                state: PlanetState {
                    id,
                    energy_cells: (0..n_energy_cells).map(|_| EnergyCell::new()).collect(),
                    can_have_rocket,
                    rocket: None,
                },
                planet_type,
                ai,
                generator,
                combinator,
                from_orchestrator,
                to_orchestrator,
                from_explorers: explorers_receiver,
                to_explorers: HashMap::new(),
            })
        }
    }

    /// Start the planet loop. This blocks and should be run in a dedicated thread.
    /// The planet starts in stopped state and waits for StartPlanetAI. Returns Ok when the
    /// planet is killed/destroyed. Errors if orchestrator disconnects.
    pub fn run(&mut self) -> Result<(), String> {
        // run stopped by default and wait for a StartPlanetAI message
        let kill = self.wait_for_start()?;
        if kill {
            return Ok(());
        }

        self.ai.start(&self.state);

        loop {
            select_biased! {
                // orchestrator messages have priority
                recv(self.from_orchestrator) -> msg => match msg {
                    Ok(OrchestratorToPlanet::StartPlanetAI) => {}

                    Ok(OrchestratorToPlanet::StopPlanetAI) => {
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::StopPlanetAIResult {
                                planet_id: self.id(),
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;
                        self.ai.stop(&self.state);

                        let kill = self.wait_for_start()?; // blocking wait
                        if kill { return Ok(()) }

                        // restart AI
                        self.ai.start(&self.state)
                    }

                    Ok(OrchestratorToPlanet::KillPlanet) => {
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::KillPlanetResult { planet_id: self.id() })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;

                        return Ok(())
                    }

                    Ok(OrchestratorToPlanet::Asteroid(_)) => {
                        let rocket =
                            self.ai
                                .handle_asteroid(&mut self.state, &self.generator, &self.combinator);

                        self.to_orchestrator
                            .send(PlanetToOrchestrator::AsteroidAck {
                                planet_id: self.id(),
                                rocket
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;
                    }

                    Ok(OrchestratorToPlanet::IncomingExplorerRequest {
                        explorer_id,
                        new_mpsc_sender,
                    }) => {
                        // register new explorer's dedicated sender
                        self.to_explorers.insert(explorer_id, new_mpsc_sender);

                        // ack back to orchestrator
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::IncomingExplorerResponse {
                                planet_id: self.id(),
                                res: Ok(()),
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;
                    }

                    Ok(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id }) => {
                        // remove explorer channel
                        self.to_explorers.remove(&explorer_id);

                        // ack back to orchestrator
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::OutgoingExplorerResponse {
                                planet_id: self.id(),
                                res: Ok(()),
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;
                    }

                    // default: dispatch to AI handler
                    Ok(msg) => {
                        self.ai
                            .handle_orchestrator_msg(
                                &mut self.state,
                                &self.generator,
                                &self.combinator,
                                msg,
                            )
                            .map(|response| self.to_orchestrator.send(response))
                            .transpose()
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;
                    }

                    Err(_) => {
                        return Err(Self::ORCH_DISCONNECT_ERR.to_string())
                    }
                },

                // explorer messages (ignore disconnections)
                recv(self.from_explorers) -> msg => if let Ok(msg) = msg {
                    let explorer_id = msg.explorer_id();

                    if let Some(to_explorer) = self.to_explorers.get(&explorer_id) {
                        if let Some(response) = self.ai.handle_explorer_msg(
                            &mut self.state,
                            &self.generator,
                            &self.combinator,
                            msg,
                        ) {
                            to_explorer
                                .send(response)
                                .map_err(|_| format!("Explorer {} disconnected.", explorer_id))?;
                        }
                    }
                }
            }
        }
    }

    // Helper: blocks until StartPlanetAI or KillPlanet is received.
    // Returns Ok(true) if KillPlanet was received, Ok(false) if StartPlanetAI,
    // or Err if orchestrator disconnected.
    fn wait_for_start(&self) -> Result<bool, String> {
        loop {
            select_biased! {
                recv(self.from_orchestrator) -> msg => match msg {
                    Ok(OrchestratorToPlanet::StartPlanetAI) => {
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::StartPlanetAIResult {
                                planet_id: self.id(),
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;

                        return Ok(false);
                    }
                    Ok(OrchestratorToPlanet::KillPlanet) => {
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::KillPlanetResult { planet_id: self.id() })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?;

                        return Ok(true)
                    }
                    Ok(_) => {
                        self.to_orchestrator
                            .send(PlanetToOrchestrator::Stopped {
                                planet_id: self.id(),
                            })
                            .map_err(|_| Self::ORCH_DISCONNECT_ERR.to_string())?
                    }

                    Err(_) => return Err(Self::ORCH_DISCONNECT_ERR.to_string()),
                },

                recv(self.from_explorers) -> msg => if let Ok(msg) = msg {
                    if let Some(to_explorer) = self.to_explorers.get(&msg.explorer_id()) {
                        let _ = to_explorer.send(PlanetToExplorer::Stopped);
                    }
                }
            }
        }
    }

    /// Returns the planet id.
    pub fn id(&self) -> u32 {
        self.state.id
    }

    /// Returns the planet type.
    pub fn planet_type(&self) -> PlanetType {
        self.planet_type
    }

    /// Returns an immutable borrow of the planet's state.
    pub fn state(&self) -> &PlanetState {
        &self.state
    }

    /// Returns an immutable borrow of the planet's generator.
    pub fn generator(&self) -> &Generator {
        &self.generator
    }

    /// Returns an immutable borrow of the planet's combinator.
    pub fn combinator(&self) -> &Combinator {
        &self.combinator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use std::thread;
    use std::time::Duration;

    use crate::components::asteroid::{Asteroid, Vec2};
    use crate::components::energy_cell::EnergyCell;
    use crate::components::resource::{
        BasicResourceType, Combinator, ComplexResourceType, Generator,
    };
    use crate::components::rocket::Rocket;
    use crate::components::sunray::Sunray;
    use crate::protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    };

    // --- Mock AI ---
    struct MockAI {
        start_called: bool,
        stop_called: bool,
        sunray_count: u32,
    }

    impl MockAI {
        fn new() -> Self {
            Self {
                start_called: false,
                stop_called: false,
                sunray_count: 0,
            }
        }
    }

    impl PlanetAI for MockAI {
        fn handle_orchestrator_msg(
            &mut self,
            state: &mut PlanetState,
            _generator: &Generator,
            _combinator: &Combinator,
            msg: OrchestratorToPlanet,
        ) -> Option<PlanetToOrchestrator> {
            match msg {
                OrchestratorToPlanet::Sunray(s) => {
                    self.sunray_count += 1;

                    if let Some(cell) = state.cells_iter_mut().next() {
                        cell.charge(s);
                    }

                    Some(PlanetToOrchestrator::SunrayAck {
                        planet_id: state.id(),
                    })
                }
                _ => None,
            }
        }

        fn handle_explorer_msg(
            &mut self,
            _state: &mut PlanetState,
            _generator: &Generator,
            _combinator: &Combinator,
            msg: ExplorerToPlanet,
        ) -> Option<PlanetToExplorer> {
            match msg {
                ExplorerToPlanet::AvailableEnergyCellRequest { .. } => {
                    Some(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: 5 })
                }
                _ => None,
            }
        }

        fn handle_asteroid(
            &mut self,
            state: &mut PlanetState,
            _generator: &Generator,
            _combinator: &Combinator,
        ) -> Option<Rocket> {
            match state.full_cell() {
                None => None,
                Some((_cell, i)) => {
                    let _ = state.build_rocket(i);
                    state.take_rocket()
                }
            }
        }

        fn start(&mut self, _state: &PlanetState) {
            self.start_called = true;
        }

        fn stop(&mut self, _state: &PlanetState) {
            self.stop_called = true;
        }
    }

    // --- Helper for creating dummy channels ---
    type PlanetOrchHalfChannels = (Receiver<OrchestratorToPlanet>, Sender<PlanetToOrchestrator>);
    type PlanetExplHalfChannels = (Receiver<ExplorerToPlanet>, Sender<PlanetToExplorer>);
    type OrchPlanetHalfChannels = (Sender<OrchestratorToPlanet>, Receiver<PlanetToOrchestrator>);
    type ExplPlanetHalfChannels = (Sender<ExplorerToPlanet>, Receiver<PlanetToExplorer>);

    fn get_test_channels() -> (
        PlanetOrchHalfChannels,
        PlanetExplHalfChannels,
        OrchPlanetHalfChannels,
        ExplPlanetHalfChannels,
    ) {
        let (tx_orch_in, rx_orch_in) = unbounded::<OrchestratorToPlanet>();
        let (tx_orch_out, rx_orch_out) = unbounded::<PlanetToOrchestrator>();

        let (tx_expl_in, rx_expl_in) = unbounded::<ExplorerToPlanet>();
        let (tx_expl_out, rx_expl_out) = unbounded::<PlanetToExplorer>();

        (
            (rx_orch_in, tx_orch_out),
            (rx_expl_in, tx_expl_out),
            (tx_orch_in, rx_orch_out),
            (tx_expl_in, rx_expl_out),
        )
    }

    // --- Unit Tests: Planet State Logic ---

    #[test]
    fn test_planet_state_rocket_construction() {
        let mut state = PlanetState {
            id: 0,
            energy_cells: vec![EnergyCell::new()],
            rocket: None,
            can_have_rocket: true,
        };

        let cell = state.cell_mut(0);
        let sunray = Sunray::new();
        cell.charge(sunray);

        // Build Rocket
        let res = state.build_rocket(0);
        assert!(res.is_ok());
        assert!(state.has_rocket());
        assert!(!state.cell(0).is_charged());

        // Take Rocket
        let rocket = state.take_rocket();
        assert!(rocket.is_some());
        assert!(!state.has_rocket());
    }

    #[test]
    fn test_planet_state_type_b_no_rocket() {
        let mut state = PlanetState {
            id: 0,
            energy_cells: vec![EnergyCell::new()],
            rocket: None,
            can_have_rocket: false, // Type B
        };

        let cell = state.cell_mut(0);
        cell.charge(Sunray::new());

        let res = state.build_rocket(0);
        assert!(res.is_err(), "Type B should not be able to build rockets");
    }

    // --- Integration Tests: Constructor ---

    #[test]
    fn test_planet_construction_constraints() {
        // 1. Valid Construction
        let (orch_ch, expl_ch, _, _) = get_test_channels();
        let valid_gen = vec![BasicResourceType::Oxygen];

        let valid_planet = Planet::new(
            1,
            PlanetType::A,
            Box::new(MockAI::new()),
            valid_gen,
            vec![],
            orch_ch,
            expl_ch.0,
        );
        assert!(valid_planet.is_ok());

        // 2. Invalid: Empty Gen Rules
        let (orch_ch, expl_ch, _, _) = get_test_channels();
        let invalid_empty = Planet::new(
            1,
            PlanetType::A,
            Box::new(MockAI::new()),
            vec![], // Error
            vec![],
            orch_ch,
            expl_ch.0,
        );
        assert!(invalid_empty.is_err());

        // 3. Invalid: Too Many Gen Rules for Type A
        let (orch_ch, expl_ch, _, _) = get_test_channels();
        let invalid_gen = Planet::new(
            1,
            PlanetType::A,
            Box::new(MockAI::new()),
            vec![BasicResourceType::Oxygen, BasicResourceType::Hydrogen], // Error for Type A
            vec![],
            orch_ch,
            expl_ch.0,
        );
        assert!(invalid_gen.is_err());
    }

    // --- Integration Tests: Loop ---

    #[test]
    fn test_planet_run_loop_survival() {
        let (planet_orch_ch, planet_expl_ch, orch_planet_ch, _) = get_test_channels();

        let (rx_from_orch, tx_from_planet_orch) = planet_orch_ch;
        let (rx_from_expl, _) = planet_expl_ch;
        let (tx_to_planet_orch, rx_to_orch) = orch_planet_ch;

        // Build Planet
        let mut planet = Planet::new(
            100,
            PlanetType::A,
            Box::new(MockAI::new()),
            vec![BasicResourceType::Oxygen],
            vec![],
            (rx_from_orch, tx_from_planet_orch),
            rx_from_expl,
        )
        .expect("Failed to create planet");

        // Spawn thread
        let handle = thread::spawn(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let res = planet.run();
                match res {
                    Ok(_) => {}
                    Err(err) => {
                        dbg!(err);
                    }
                }
            }));
        });

        // 1. Start AI
        tx_to_planet_orch
            .send(OrchestratorToPlanet::StartPlanetAI)
            .unwrap();
        match rx_to_orch.recv_timeout(Duration::from_millis(50)) {
            Ok(PlanetToOrchestrator::StartPlanetAIResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }
        thread::sleep(Duration::from_millis(50));

        // 2. Send Sunray
        tx_to_planet_orch
            .send(OrchestratorToPlanet::Sunray(Sunray::new()))
            .unwrap();

        // Expect Ack
        if let Ok(PlanetToOrchestrator::SunrayAck { planet_id, .. }) =
            rx_to_orch.recv_timeout(Duration::from_millis(200))
        {
            assert_eq!(planet_id, 100);
        } else {
            panic!("Did not receive SunrayAck");
        }

        // 3. Send Asteroid (AI should build rocket using the charged cell)
        tx_to_planet_orch
            .send(OrchestratorToPlanet::Asteroid(Asteroid::new(
                Vec2::default(),
                Vec2::default(),
                1.0,
                1.0,
            )))
            .unwrap();

        // 4. Expect Survival (Ack with Some(Rocket))
        match rx_to_orch.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::AsteroidAck {
                planet_id, rocket, ..
            }) => {
                assert_eq!(planet_id, 100);
                assert!(rocket.is_some(), "Planet failed to build rocket!");
            }
            Ok(_) => panic!("Wrong message type"),
            Err(_) => panic!("Timeout waiting for AsteroidAck"),
        }

        // 5. Stop
        tx_to_planet_orch
            .send(OrchestratorToPlanet::StopPlanetAI)
            .unwrap();
        match rx_to_orch.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::StopPlanetAIResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // 6. Try to send a request while stopped
        tx_to_planet_orch
            .send(OrchestratorToPlanet::InternalStateRequest)
            .unwrap();
        match rx_to_orch.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::Stopped { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // 7. Kill planet while stopped
        tx_to_planet_orch
            .send(OrchestratorToPlanet::KillPlanet)
            .unwrap();
        match rx_to_orch.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::KillPlanetResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // should return immediately
        assert!(handle.join().is_ok(), "Planet thread exited with an error");
    }

    #[test]
    fn test_resource_creation() {
        let (orch_ch, expl_ch, _, _) = get_test_channels();
        let gen_rules = vec![BasicResourceType::Oxygen, BasicResourceType::Hydrogen];
        let comb_rules = vec![ComplexResourceType::Water];
        let mut planet = Planet::new(
            0,
            PlanetType::B,
            Box::new(MockAI::new()),
            gen_rules,
            comb_rules,
            orch_ch,
            expl_ch.0,
        )
        .unwrap();

        // aliases for planet internals
        let state = &mut planet.state;
        let generator = &planet.generator;
        let combinator = &planet.combinator;

        // gen oxygen
        let cell = state.cell_mut(0);
        cell.charge(Sunray::new());

        let oxygen = generator.make_oxygen(cell);
        assert!(oxygen.is_ok());
        let oxygen = oxygen.unwrap();

        // gen hydrogen
        let cell = state.cell_mut(0);
        cell.charge(Sunray::new());

        let hydrogen = generator.make_hydrogen(cell);
        assert!(hydrogen.is_ok());
        let hydrogen = hydrogen.unwrap();

        // combine the two elements into water
        let cell = state.cell_mut(0);
        cell.charge(Sunray::new());

        let diamond = combinator.make_water(hydrogen, oxygen, cell);
        assert!(diamond.is_ok());

        // try to gen resource not contained in the planet recipes
        let carbon = generator.make_carbon(cell);
        assert!(carbon.is_err());
    }

    #[test]
    fn test_explorer_comms() {
        // 1. Setup Channels using the new helper
        let (
            planet_orch_channels,
            planet_expl_channels,
            (orch_tx, orch_rx),
            (expl_tx_global, _expl_rx_global),
        ) = get_test_channels();

        // 2. Setup Planet
        let (planet_expl_rx, _) = planet_expl_channels;

        let mut planet = Planet::new(
            1,
            PlanetType::A,
            Box::new(MockAI::new()),
            vec![BasicResourceType::Oxygen],
            vec![],
            planet_orch_channels,
            planet_expl_rx,
        )
        .expect("Failed to create planet");

        // Spawn planet thread
        let handle = thread::spawn(move || {
            let res = planet.run();
            match res {
                Ok(_) => {}
                Err(err) => {
                    dbg!(err);
                }
            }
        });

        // 3. Start Planet
        orch_tx.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
        match orch_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(PlanetToOrchestrator::StartPlanetAIResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }
        thread::sleep(Duration::from_millis(50));

        // 4. Setup Local Explorer Channels (Simulating Explorer 101)
        let explorer_id = 101;
        let (expl_tx_local, expl_rx_local) = unbounded::<PlanetToExplorer>();

        // 5. Send IncomingExplorerRequest (Orchestrator -> Planet)
        orch_tx
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id,
                new_mpsc_sender: expl_tx_local,
            })
            .unwrap();

        // 6. Verify Ack from Planet
        match orch_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res }) => {
                assert_eq!(planet_id, 1);
                assert!(res.is_ok());
            }
            _ => panic!("Expected IncomingExplorerResponse"),
        }

        // 7. Test Interaction (Explorer -> Planet -> Explorer)
        expl_tx_global
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id })
            .unwrap();

        // Verify Explorer receives response on the LOCAL channel
        match expl_rx_local.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) => {
                assert_eq!(available_cells, 5);
            }
            _ => panic!("Expected AvailableEnergyCellResponse"),
        }

        // Stop Planet AI
        orch_tx.send(OrchestratorToPlanet::StopPlanetAI).unwrap();
        match orch_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::StopPlanetAIResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // Try to send request from explorer to stopped planet
        expl_tx_global
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id })
            .unwrap();
        match expl_rx_local.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToExplorer::Stopped) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // Restart planet AI
        orch_tx.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
        match orch_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::StartPlanetAIResult { .. }) => {}
            _ => panic!("Planet sent incorrect response"),
        }

        // 8. Send OutgoingExplorerRequest (Orchestrator -> Planet)
        orch_tx
            .send(OrchestratorToPlanet::OutgoingExplorerRequest { explorer_id })
            .unwrap();

        // 9. Verify Ack from Planet
        match orch_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, res }) => {
                assert_eq!(planet_id, 1);
                assert!(res.is_ok());
            }
            _ => panic!("Expected OutgoingExplorerResponse"),
        }

        // 10. Verify Isolation
        expl_tx_global
            .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id })
            .unwrap();

        // We expect NO response on expl_rx_local
        let result = expl_rx_local.recv_timeout(Duration::from_millis(200));
        assert!(
            result.is_err(),
            "Planet responded to explorer after it left!"
        );

        // 11. Cleanup
        drop(orch_tx);
        let _ = handle.join();
    }
}
