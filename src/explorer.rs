use crate::player_log;
use std::thread::{self, JoinHandle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use common_game::protocols::orchestrator_explorer::{OrchestratorToExplorer, ExplorerToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use crossbeam_channel::{Sender, Receiver, RecvTimeoutError};
use common_game::components::resource::{
    BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType,
    GenericResource
};
use common_game::utils::ID;

pub struct Explorer<T> {
    id: ID,
    current_planet_id: ID,
    neighbors: Vec<ID>,
    orchestrator_sender: Sender<ExplorerToOrchestrator<T>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_sender: Sender<ExplorerToPlanet>,
    planet_receiver: Receiver<PlanetToExplorer>,
    bag: Arc<Mutex<Bag>>,
    ai_running: Arc<AtomicBool>,
    ai_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
    // Shared with AI thread so it always knows the current planet id.
    shared_planet_id: Arc<AtomicU32>,
    // Inner channels: main loop → AI thread, to forward orchestrator responses.
    ai_neighbors_tx: Option<Sender<Vec<ID>>>,
    ai_planet_tx: Option<Sender<Option<Sender<ExplorerToPlanet>>>>,
}

#[derive(Debug)]
pub struct Bag {
    pub basic_resources: HashMap<BasicResourceType, u32>,
    pub complex_resources: HashMap<ComplexResourceType, u32>,
    basic_resource_instances: HashMap<BasicResourceType, Vec<BasicResource>>,
    complex_resource_instances: HashMap<ComplexResourceType, Vec<ComplexResource>>,
}

pub struct BagSummary {
    pub basic_resources: HashMap<BasicResourceType, u32>,
    pub complex_resources: HashMap<ComplexResourceType, u32>,
}

impl<T: From<BagSummary> + Send + 'static> Explorer<T> {
    pub fn new(
        id: ID,
        current_planet_id: ID,
        orchestrator_sender: Sender<ExplorerToOrchestrator<T>>,
        orchestrator_receiver: Receiver<OrchestratorToExplorer>,
        planet_sender: Sender<ExplorerToPlanet>,
        planet_receiver: Receiver<PlanetToExplorer>,
    ) -> Self {
        Explorer {
            id,
            current_planet_id,
            neighbors: Vec::new(),
            orchestrator_sender,
            orchestrator_receiver,
            planet_sender,
            planet_receiver,
            bag: Arc::new(Mutex::new(Bag::new())),
            ai_running: Arc::new(AtomicBool::new(false)),
            ai_thread: Arc::new(Mutex::new(None)),
            shared_planet_id: Arc::new(AtomicU32::new(current_planet_id)),
            ai_neighbors_tx: None,
            ai_planet_tx: None,
        }
    }

    pub fn telemetry_handles(&self) -> (Arc<Mutex<Bag>>, Arc<AtomicBool>) {
        (Arc::clone(&self.bag), Arc::clone(&self.ai_running))
    }

    pub fn run(mut self) {
        let _ = thread::spawn(move || {
            while let Ok(mssg) = self.orchestrator_receiver.recv() {
                match mssg {
                    OrchestratorToExplorer::BagContentRequest => {
                        let response = self.handle_bag_content_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::CurrentPlanetRequest => {
                        let response = self.handle_current_planet_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::MoveToPlanet { sender_to_new_planet, planet_id } => {
                        // Update shared planet id before notifying AI thread so the AI
                        // reads the correct value as soon as it wakes from planet_rx.
                        self.shared_planet_id.store(planet_id, Ordering::SeqCst);
                        let response = self.handle_move_to_planet(sender_to_new_planet.clone(), planet_id);
                        // Forward new sender (or None) to the AI thread.
                        if let Some(ref tx) = self.ai_planet_tx {
                            let _ = tx.try_send(sender_to_new_planet);
                        }
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::StartExplorerAI => {
                        let response = self.handle_start_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::StopExplorerAI => {
                        let response = self.handle_stop_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::ResetExplorerAI => {
                        let response = self.handle_reset_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::SupportedResourceRequest => {
                        let response = self.handle_supported_resource_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::SupportedCombinationRequest => {
                        let response = self.handle_supported_combination_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::GenerateResourceRequest { to_generate } => {
                        let response = self.handle_generate_resource_request(to_generate);
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::CombineResourceRequest { to_generate } => {
                        let response = self.handle_combine_resource_request(to_generate);
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::NeighborsResponse { neighbors } => {
                        // Forward to AI thread before storing locally.
                        if let Some(ref tx) = self.ai_neighbors_tx {
                            let _ = tx.try_send(neighbors.clone());
                        }
                        self.handle_neighbors_response(neighbors);
                    }

                    OrchestratorToExplorer::KillExplorer => {
                        let response = ExplorerToOrchestrator::KillExplorerResult { explorer_id: self.id };
                        let _ = self.orchestrator_sender.send(response);
                        break;
                    }
                }
            }
        });
    }

    fn handle_neighbors_response(&mut self, neighbors: Vec<ID>) {
        self.neighbors = neighbors;
    }

    fn handle_combine_resource_request(&mut self, to_generate: ComplexResourceType) -> ExplorerToOrchestrator<T> {
        if self.ai_running.load(Ordering::SeqCst) {
            return ExplorerToOrchestrator::CombineResourceResponse {
                explorer_id: self.id,
                generated: Err("Explorer AI is running; manual combination not permitted".to_string()),
            };
        }
        let req = match to_generate {
            ComplexResourceType::Water    => self.make_water(),
            ComplexResourceType::Diamond  => self.make_diamond(),
            ComplexResourceType::Life     => self.make_life(),
            ComplexResourceType::Robot    => self.make_robot(),
            ComplexResourceType::Dolphin  => self.make_dolphin(),
            ComplexResourceType::AIPartner => self.make_aipartner(),
        };

        let req = match req {
            None => return ExplorerToOrchestrator::CombineResourceResponse {
                explorer_id: self.id,
                generated: Err("Not enough resources in bag".to_string()),
            },
            Some(r) => r,
        };

        let msg = ExplorerToPlanet::CombineResourceRequest { explorer_id: self.id, msg: req };
        let _ = self.planet_sender.send(msg);

        if let Ok(PlanetToExplorer::CombineResourceResponse { complex_response }) = self.planet_receiver.recv() {
            match complex_response {
                Ok(cr) => {
                    self.bag.lock().expect("Bag mutex poisoned").insert_complex_resource_in_bag(cr);
                    ExplorerToOrchestrator::CombineResourceResponse { explorer_id: self.id, generated: Ok(()) }
                }
                Err((e, r1, r2)) => {
                    self.bag.lock().expect("Bag mutex poisoned").restore_resource(r1);
                    self.bag.lock().expect("Bag mutex poisoned").restore_resource(r2);
                    ExplorerToOrchestrator::CombineResourceResponse { explorer_id: self.id, generated: Err(e) }
                }
            }
        } else {
            ExplorerToOrchestrator::CombineResourceResponse {
                explorer_id: self.id,
                generated: Err("No response from planet".to_string()),
            }
        }
    }

    fn make_aipartner(&mut self) -> Option<ComplexResourceRequest> {
        let r = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Robot)?;
        let d = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Diamond)?;
        Some(ComplexResourceRequest::AIPartner(r.to_robot().ok()?, d.to_diamond().ok()?))
    }

    fn make_dolphin(&mut self) -> Option<ComplexResourceRequest> {
        let w = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Water)?;
        let l = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Life)?;
        Some(ComplexResourceRequest::Dolphin(w.to_water().ok()?, l.to_life().ok()?))
    }

    fn make_robot(&mut self) -> Option<ComplexResourceRequest> {
        let s = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Silicon)?;
        let l = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Life)?;
        Some(ComplexResourceRequest::Robot(s.to_silicon().ok()?, l.to_life().ok()?))
    }

    fn make_life(&mut self) -> Option<ComplexResourceRequest> {
        let w = self.bag.lock().expect("Bag mutex poisoned").take_complex_resource(ComplexResourceType::Water)?;
        let c = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Carbon)?;
        Some(ComplexResourceRequest::Life(w.to_water().ok()?, c.to_carbon().ok()?))
    }

    fn make_diamond(&mut self) -> Option<ComplexResourceRequest> {
        let c1 = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Carbon)?;
        let c2 = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Carbon)?;
        Some(ComplexResourceRequest::Diamond(c1.to_carbon().ok()?, c2.to_carbon().ok()?))
    }

    fn make_water(&mut self) -> Option<ComplexResourceRequest> {
        let h = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Hydrogen)?;
        let o = self.bag.lock().expect("Bag mutex poisoned").take_basic_resource(BasicResourceType::Oxygen)?;
        Some(ComplexResourceRequest::Water(h.to_hydrogen().ok()?, o.to_oxygen().ok()?))
    }

    fn handle_generate_resource_request(&mut self, to_generate: BasicResourceType) -> ExplorerToOrchestrator<T> {
        if self.ai_running.load(Ordering::SeqCst) {
            return ExplorerToOrchestrator::GenerateResourceResponse {
                explorer_id: self.id,
                generated: Err("Explorer AI is running; manual generation not permitted".to_string()),
            };
        }
        let msg = ExplorerToPlanet::GenerateResourceRequest { explorer_id: self.id, resource: to_generate };
        let _ = self.planet_sender.send(msg);

        if let Ok(PlanetToExplorer::GenerateResourceResponse { resource }) = self.planet_receiver.recv() {
            match resource {
                Some(r) => {
                    self.bag.lock().unwrap().insert_basic_resource_in_bag(r);
                    ExplorerToOrchestrator::GenerateResourceResponse { explorer_id: self.id, generated: Ok(()) }
                }
                None => ExplorerToOrchestrator::GenerateResourceResponse {
                    explorer_id: self.id,
                    generated: Err("Planet failed to generate".to_string()),
                },
            }
        } else {
            ExplorerToOrchestrator::GenerateResourceResponse {
                explorer_id: self.id,
                generated: Err("No response from Planet".to_string()),
            }
        }
    }

    fn handle_supported_combination_request(&self) -> ExplorerToOrchestrator<T> {
        if self.ai_running.load(Ordering::SeqCst) {
            return ExplorerToOrchestrator::SupportedCombinationResult {
                explorer_id: self.id,
                combination_list: HashSet::new(),
            };
        }
        let _ = self.planet_sender.send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: self.id });
        if let Ok(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = self.planet_receiver.recv() {
            ExplorerToOrchestrator::SupportedCombinationResult { explorer_id: self.id, combination_list }
        } else {
            ExplorerToOrchestrator::SupportedCombinationResult { explorer_id: self.id, combination_list: HashSet::new() }
        }
    }

    fn handle_supported_resource_request(&self) -> ExplorerToOrchestrator<T> {
        if self.ai_running.load(Ordering::SeqCst) {
            return ExplorerToOrchestrator::SupportedResourceResult {
                explorer_id: self.id,
                supported_resources: HashSet::new(),
            };
        }
        let _ = self.planet_sender.send(ExplorerToPlanet::SupportedResourceRequest { explorer_id: self.id });
        if let Ok(PlanetToExplorer::SupportedResourceResponse { resource_list }) = self.planet_receiver.recv() {
            ExplorerToOrchestrator::SupportedResourceResult { explorer_id: self.id, supported_resources: resource_list }
        } else {
            ExplorerToOrchestrator::SupportedResourceResult { explorer_id: self.id, supported_resources: HashSet::new() }
        }
    }

    fn handle_reset_explorer_ai(&mut self) -> ExplorerToOrchestrator<T> {
        player_log!("[Explorer #{}] Resetting autonomous AI", self.id);
        self.ai_running.store(false, Ordering::SeqCst);
        if let Ok(mut handle) = self.ai_thread.lock() {
            if let Some(h) = handle.take() {
                let _ = h.join();
            }
        }
        self.bag.lock().unwrap().reset();
        self.ai_running.store(true, Ordering::SeqCst);
        self.spawn_ai_thread();
        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id: self.id }
    }

    fn handle_stop_explorer_ai(&self) -> ExplorerToOrchestrator<T> {
        self.ai_running.store(false, Ordering::SeqCst);
        // Join before returning so the main thread can't touch planet_receiver
        // until the AI thread has fully exited it.
        if let Some(h) = self.ai_thread.lock().unwrap().take() {
            let _ = h.join();
        }
        player_log!("[Explorer #{}] Stopping autonomous AI", self.id);
        ExplorerToOrchestrator::StopExplorerAIResult { explorer_id: self.id }
    }

    fn handle_start_explorer_ai(&mut self) -> ExplorerToOrchestrator<T> {
        if self.ai_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            player_log!("[Explorer #{}] Starting autonomous AI", self.id);
            self.spawn_ai_thread();
        } else {
            log::debug!("[Explorer #{}] AI already running — ignoring start", self.id);
        }
        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id: self.id }
    }

    fn spawn_ai_thread(&mut self) {
        let id = self.id;
        let planet_sender = self.planet_sender.clone();
        let planet_receiver = self.planet_receiver.clone();
        let bag = Arc::clone(&self.bag);
        let ai_running = Arc::clone(&self.ai_running);
        let orchestrator_sender = self.orchestrator_sender.clone();
        let shared_planet_id = Arc::clone(&self.shared_planet_id);

        let (neighbors_tx, neighbors_rx) = crossbeam_channel::unbounded::<Vec<ID>>();
        let (planet_tx, planet_rx) = crossbeam_channel::unbounded::<Option<Sender<ExplorerToPlanet>>>();

        self.ai_neighbors_tx = Some(neighbors_tx);
        self.ai_planet_tx = Some(planet_tx);

        let handle = thread::spawn(move || {
            autonomous_ai(
                id,
                planet_sender,
                planet_receiver,
                bag,
                ai_running,
                orchestrator_sender,
                shared_planet_id,
                neighbors_rx,
                planet_rx,
            );
        });

        *self.ai_thread.lock().unwrap() = Some(handle);
    }

    fn handle_move_to_planet(
        &mut self,
        sender_to_new_planet: Option<Sender<ExplorerToPlanet>>,
        planet_id: ID,
    ) -> ExplorerToOrchestrator<T> {
        if let Some(new_planet_sender) = sender_to_new_planet {
            self.planet_sender = new_planet_sender;
        }
        self.current_planet_id = planet_id;
        // shared_planet_id is updated in the run() match arm before this call.
        ExplorerToOrchestrator::MovedToPlanetResult { explorer_id: self.id, planet_id: self.current_planet_id }
    }

    fn handle_current_planet_request(&self) -> ExplorerToOrchestrator<T> {
        ExplorerToOrchestrator::CurrentPlanetResult { explorer_id: self.id, planet_id: self.current_planet_id }
    }

    fn handle_bag_content_request(&self) -> ExplorerToOrchestrator<T> {
        ExplorerToOrchestrator::BagContentResponse {
            explorer_id: self.id,
            bag_content: T::from(self.bag.lock().unwrap().to_summary()),
        }
    }
}

impl From<BagSummary> for Bag {
    fn from(summary: BagSummary) -> Self {
        Bag {
            basic_resources: summary.basic_resources,
            complex_resources: summary.complex_resources,
            basic_resource_instances: HashMap::new(),
            complex_resource_instances: HashMap::new(),
        }
    }
}

impl Default for Bag {
    fn default() -> Self { Self::new() }
}

impl Bag {
    pub fn new() -> Bag {
        Bag {
            basic_resources: HashMap::new(),
            complex_resources: HashMap::new(),
            basic_resource_instances: HashMap::new(),
            complex_resource_instances: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.basic_resources.clear();
        self.complex_resources.clear();
        self.basic_resource_instances.clear();
        self.complex_resource_instances.clear();
    }

    pub fn insert_basic_resource_in_bag(&mut self, resource: BasicResource) {
        let rt = resource.get_type();
        self.basic_resources.entry(rt).and_modify(|c| *c += 1).or_insert(1);
        self.basic_resource_instances.entry(rt).or_default().push(resource);
    }

    pub fn insert_complex_resource_in_bag(&mut self, resource: ComplexResource) {
        let rt = resource.get_type();
        self.complex_resources.entry(rt).and_modify(|c| *c += 1).or_insert(1);
        self.complex_resource_instances.entry(rt).or_default().push(resource);
    }

    pub fn take_basic_resource(&mut self, resource_type: BasicResourceType) -> Option<BasicResource> {
        let instance = self.basic_resource_instances.get_mut(&resource_type)?.pop()?;
        self.basic_resources.entry(resource_type).and_modify(|c| *c -= 1);
        Some(instance)
    }

    pub fn take_complex_resource(&mut self, resource_type: ComplexResourceType) -> Option<ComplexResource> {
        let instance = self.complex_resource_instances.get_mut(&resource_type)?.pop()?;
        self.complex_resources.entry(resource_type).and_modify(|c| *c -= 1);
        Some(instance)
    }

    pub fn to_summary(&self) -> BagSummary {
        BagSummary {
            basic_resources: self.basic_resources.clone(),
            complex_resources: self.complex_resources.clone(),
        }
    }

    pub fn restore_resource(&mut self, resource: GenericResource) {
        match resource {
            GenericResource::BasicResources(br) => self.insert_basic_resource_in_bag(br),
            GenericResource::ComplexResources(cr) => self.insert_complex_resource_in_bag(cr),
        }
    }

    pub fn is_basic_full(&self, rt: BasicResourceType) -> bool {
        self.basic_resources.get(&rt).copied().unwrap_or(0) >= basic_bag_limit(rt)
    }

    pub fn is_complex_full(&self, rt: ComplexResourceType) -> bool {
        self.complex_resources.get(&rt).copied().unwrap_or(0) >= complex_bag_limit(rt)
    }
}

// ───── Autonomous AI ────────────────────────────────────────────────────────

/// Receive from `planet_receiver`, polling every 100 ms so the AI thread exits
/// promptly when `ai_running` is set to false. Returns `None` on stop or disconnect.
fn recv_planet(rx: &Receiver<PlanetToExplorer>, ai_running: &AtomicBool) -> Option<PlanetToExplorer> {
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(msg) => return Some(msg),
            Err(RecvTimeoutError::Timeout) => {
                if !ai_running.load(Ordering::SeqCst) { return None; }
            }
            Err(RecvTimeoutError::Disconnected) => return None,
        }
    }
}

pub struct GoalPlan {
    pub goal: Option<ComplexResourceType>,
    pub basics_needed: Vec<BasicResourceType>,
    pub combination_sequence: Vec<ComplexResourceType>,
}

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Scout,
    Plan,
    Collect,
    Craft,
    Move,
}

fn plan_for_with_bag(
    goal: ComplexResourceType,
    bag_complex: &HashMap<ComplexResourceType, u32>,
) -> GoalPlan {
    use BasicResourceType::*;
    use ComplexResourceType::*;

    let steps: &[ComplexResourceType] = match goal {
        Water     => &[Water],
        Diamond   => &[Diamond],
        Life      => &[Water, Life],
        Dolphin   => &[Water, Life, Water, Dolphin],
        Robot     => &[Water, Life, Robot],
        AIPartner => &[Water, Life, Robot, Diamond, AIPartner],
    };

    let mut available = bag_complex.clone();
    let mut basics_needed = Vec::new();
    let mut combination_sequence = Vec::new();

    for &step in steps {
        let count = available.entry(step).or_insert(0);
        if *count > 0 {
            *count -= 1;
        } else {
            match step {
                Water    => { basics_needed.push(Hydrogen); basics_needed.push(Oxygen); }
                Diamond  => { basics_needed.push(Carbon);   basics_needed.push(Carbon); }
                Life     => { basics_needed.push(Carbon); }
                Robot    => { basics_needed.push(Silicon); }
                Dolphin  => {}
                AIPartner => {}
            }
            combination_sequence.push(step);
            *available.entry(step).or_insert(0) += 1;
        }
    }

    GoalPlan { goal: Some(goal), basics_needed, combination_sequence }
}

fn pick_best_goal(
    supported_basics: &HashSet<BasicResourceType>,
    known_combos: &HashSet<ComplexResourceType>,
    bag_basics: &HashMap<BasicResourceType, u32>,
    bag_complex: &HashMap<ComplexResourceType, u32>,
    known_basics: &HashSet<BasicResourceType>,
) -> GoalPlan {
    use ComplexResourceType::*;

    for goal in [AIPartner, Robot, Dolphin, Life, Diamond, Water] {
        if bag_complex.get(&goal).copied().unwrap_or(0) >= complex_bag_limit(goal) {
            continue;
        }
        let mut bag_for_plan = bag_complex.clone();
        bag_for_plan.remove(&goal);
        let plan = plan_for_with_bag(goal, &bag_for_plan);
        let combos_ok = plan.combination_sequence.iter().all(|c| known_combos.contains(c));
        if !combos_ok { continue; }

        let mut bag_counts = bag_basics.clone();
        let basics_ok = plan.basics_needed.iter().all(|b| {
            let c = bag_counts.entry(*b).or_insert(0);
            if *c > 0 { *c -= 1; true } else { known_basics.contains(b) }
        });
        if basics_ok {
            if plan.basics_needed.is_empty() && plan.combination_sequence.is_empty() {
                continue;
            }
            return plan;
        }
    }

    let basics_needed = supported_basics
        .iter()
        .copied()
        .flat_map(|b| std::iter::repeat_n(b, basic_bag_limit(b) as usize))
        .collect();
    GoalPlan { goal: None, basics_needed, combination_sequence: vec![] }
}

fn basic_bag_limit(_: BasicResourceType) -> u32 { 20 }

fn complex_bag_limit(c: ComplexResourceType) -> u32 {
    use ComplexResourceType::*;
    match c {
        Dolphin | AIPartner => 5,
        _ => 10,
    }
}

#[allow(clippy::too_many_arguments)]
fn autonomous_ai<T: Send + 'static>(
    id: ID,
    mut planet_sender: Sender<ExplorerToPlanet>,
    planet_receiver: Receiver<PlanetToExplorer>,
    bag: Arc<Mutex<Bag>>,
    ai_running: Arc<AtomicBool>,
    orchestrator_sender: Sender<ExplorerToOrchestrator<T>>,
    shared_planet_id: Arc<AtomicU32>,
    neighbors_rx: Receiver<Vec<ID>>,
    planet_rx: Receiver<Option<Sender<ExplorerToPlanet>>>,
) {
    let mut phase = Phase::Scout;
    let mut supported_basics = HashSet::new();
    let mut supported_combos = HashSet::new();
    let mut known_combos: HashSet<ComplexResourceType> = HashSet::new();
    let mut known_basics: HashSet<BasicResourceType> = HashSet::new();
    // Maps each combo type → set of planet IDs known to support it.
    let mut combo_planet_map: HashMap<ComplexResourceType, HashSet<u32>> = HashMap::new();
    let mut remaining_basics = VecDeque::new();
    let mut remaining_combos = VecDeque::new();
    let mut retry_count = 0u32;
    let mut visit_count: HashMap<u32, usize> = HashMap::new();

    while ai_running.load(Ordering::SeqCst) {
        match phase {
            Phase::Scout => {
                let _ = planet_sender.send(ExplorerToPlanet::SupportedResourceRequest { explorer_id: id });
                match recv_planet(&planet_receiver, &ai_running) {
                    Some(PlanetToExplorer::SupportedResourceResponse { resource_list }) => {
                        supported_basics = resource_list;
                    }
                    None => break,
                    _ => {}
                }
                known_basics.extend(supported_basics.iter().copied());

                let _ = planet_sender.send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: id });
                match recv_planet(&planet_receiver, &ai_running) {
                    Some(PlanetToExplorer::SupportedCombinationResponse { combination_list }) => {
                        supported_combos = combination_list;
                    }
                    None => break,
                    _ => {}
                }

                known_combos.extend(supported_combos.iter().copied());
                // Record which planet supports which combos for goal-directed movement.
                let current_pid = shared_planet_id.load(Ordering::SeqCst);
                for &c in &supported_combos {
                    combo_planet_map.entry(c).or_default().insert(current_pid);
                }

                log::info!(
                    "[Explorer #{}] SCOUT planet {}: basics={:?}  combos={:?}  \
                     remaining_basics={:?}  remaining_combos={:?}",
                    id, current_pid,
                    supported_basics, supported_combos,
                    remaining_basics, remaining_combos
                );

                // Resume an in-progress goal rather than re-planning from scratch every visit.
                if remaining_combos.is_empty() && remaining_basics.is_empty() {
                    log::debug!("[Explorer #{}] Scout → Plan (no active goal)", id);
                    phase = Phase::Plan;
                } else {
                    log::debug!("[Explorer #{}] Scout → Collect (resuming goal)", id);
                    phase = Phase::Collect;
                }
            }

            Phase::Plan => {
                let bag_lock = bag.lock().unwrap();
                let bag_basics  = bag_lock.basic_resources.clone();
                let bag_complex = bag_lock.complex_resources.clone();
                drop(bag_lock);

                let plan = pick_best_goal(&supported_basics, &known_combos, &bag_basics, &bag_complex, &known_basics);

                let mut bag_counts = bag_basics.clone();
                remaining_basics = plan.basics_needed.into_iter().filter(|b| {
                    let c = bag_counts.entry(*b).or_insert(0);
                    if *c > 0 { *c -= 1; false } else { true }
                }).collect();
                remaining_combos = plan.combination_sequence.into_iter().collect();

                log::info!(
                    "[Explorer #{}] PLAN: goal={:?}  need_basics={:?}  combo_seq={:?}  \
                     bag_basics={:?}  bag_complex={:?}  known_combos={:?}",
                    id, plan.goal,
                    remaining_basics, remaining_combos,
                    bag_basics, bag_complex, known_combos
                );

                phase = Phase::Collect;
            }

            Phase::Collect => {
                // Find the first basic this planet can actually generate and rotate it to front.
                // Basics for other planets stay in the queue for future visits.
                let pos = remaining_basics.iter().position(|b| supported_basics.contains(b));
                if let Some(idx) = pos {
                    remaining_basics.rotate_left(idx);
                    let resource = *remaining_basics.front().unwrap();
                    if bag.lock().unwrap().is_basic_full(resource) {
                        log::debug!(
                            "[Explorer #{}] Collect: {:?} bag already full, skipping → Craft",
                            id, resource
                        );
                        phase = Phase::Craft;
                        continue;
                    }
                    log::debug!(
                        "[Explorer #{}] Collect: requesting {:?}  still_need={:?}",
                        id, resource, remaining_basics
                    );
                    let _ = planet_sender.send(ExplorerToPlanet::GenerateResourceRequest { explorer_id: id, resource });

                    match recv_planet(&planet_receiver, &ai_running) {
                        Some(PlanetToExplorer::GenerateResourceResponse { resource: Some(r) }) => {
                            log::debug!("[Explorer #{}] Collect: ✓ got {:?}", id, resource);
                            bag.lock().unwrap().insert_basic_resource_in_bag(r);
                            remaining_basics.pop_front();
                            retry_count = 0;
                        }
                        Some(PlanetToExplorer::GenerateResourceResponse { resource: None }) => {
                            retry_count += 1;
                            if retry_count >= 10 {
                                log::warn!(
                                    "[Explorer #{}] Collect: failed to get {:?} after 10 retries, skipping",
                                    id, resource
                                );
                                remaining_basics.pop_front();
                                retry_count = 0;
                            } else {
                                log::debug!(
                                    "[Explorer #{}] Collect: planet has no cell charge for {:?}, retry {}/10",
                                    id, resource, retry_count
                                );
                                thread::sleep(Duration::from_millis(50 * retry_count as u64));
                            }
                        }
                        None => break,
                        Some(_) => {}
                    }
                } else {
                    // Nothing collectible on this planet.
                    if remaining_basics.is_empty() {
                        // All basics are in hand — ready to combine.
                        log::info!(
                            "[Explorer #{}] Collect → Craft: all basics collected, bag={:?}",
                            id,
                            bag.lock().unwrap().basic_resources
                        );
                        phase = Phase::Craft;
                    } else {
                        // Still need basics from other planets — go get them first.
                        log::info!(
                            "[Explorer #{}] Collect → Move: planet supports {:?} but still need {:?}",
                            id, supported_basics, remaining_basics
                        );
                        phase = Phase::Move;
                    }
                }
            }

            Phase::Craft => {
                if let Some(&c) = remaining_combos.front() {
                    if !supported_combos.contains(&c) {
                        // Current planet doesn't support this combo step — move to find one that does.
                        log::info!(
                            "[Explorer #{}] Craft → Move: {:?} not supported here (planet supports {:?})",
                            id, c, supported_combos
                        );
                        phase = Phase::Move;
                    } else if bag.lock().unwrap().is_complex_full(c) {
                        // No room for the output — move before crafting more.
                        log::info!(
                            "[Explorer #{}] Craft → Move: bag full for {:?}",
                            id, c
                        );
                        phase = Phase::Move;
                    } else {
                        let bag_snapshot = {
                            let b = bag.lock().unwrap();
                            format!("basics={:?} complex={:?}", b.basic_resources, b.complex_resources)
                        };
                        log::info!(
                            "[Explorer #{}] Craft: attempting {:?}  {}",
                            id, c, bag_snapshot
                        );
                        let req = build_request(&mut bag.lock().unwrap(), c);
                        match req {
                            None => {
                                // Bag no longer has the ingredients the plan assumed — re-plan
                                // from scratch so we go collect whatever is actually missing.
                                log::warn!(
                                    "[Explorer #{}] Craft → Plan: build_request returned None for {:?} \
                                     (bag drifted from plan — {})",
                                    id, c, bag_snapshot
                                );
                                remaining_basics.clear();
                                remaining_combos.clear();
                                phase = Phase::Plan;
                            }
                            Some(req) => {
                                log::debug!("[Explorer #{}] Craft: sending CombineResourceRequest for {:?}", id, c);
                                let _ = planet_sender.send(ExplorerToPlanet::CombineResourceRequest { explorer_id: id, msg: req });

                                match recv_planet(&planet_receiver, &ai_running) {
                                    Some(PlanetToExplorer::CombineResourceResponse { complex_response: Ok(r) }) => {
                                        player_log!("[Explorer #{}] Crafted {:?}", id, c);
                                        bag.lock().unwrap().insert_complex_resource_in_bag(r);
                                        remaining_combos.pop_front();
                                    }
                                    Some(PlanetToExplorer::CombineResourceResponse { complex_response: Err((msg, r1, r2)) }) => {
                                        log::warn!(
                                            "[Explorer #{}] Craft: planet rejected {:?}: {}  → Move",
                                            id, c, msg
                                        );
                                        bag.lock().unwrap().restore_resource(r1);
                                        bag.lock().unwrap().restore_resource(r2);
                                        // Planet rejected the combo — treat as unsupported and move.
                                        phase = Phase::Move;
                                    }
                                    None => break,
                                    Some(_) => {
                                        log::error!("[Explorer #{}] Craft: unexpected response for {:?}", id, c);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    log::debug!("[Explorer #{}] Craft → Move: no more combos in sequence", id);
                    phase = Phase::Move;
                }
            }

            Phase::Move => {
                // Drain stale responses from previous cycles.
                while neighbors_rx.try_recv().is_ok() {}

                let current = shared_planet_id.load(Ordering::SeqCst);
                let _ = orchestrator_sender.send(ExplorerToOrchestrator::NeighborsRequest {
                    explorer_id: id,
                    current_planet_id: current,
                });

                match neighbors_rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(neighbors) if !neighbors.is_empty() => {
                        // Goal-directed: if we have pending combo work, prefer a neighbor
                        // that is known to support the needed combo type directly.
                        let (dst, move_reason) = if !remaining_basics.is_empty() {
                            // Still collecting basics — round-robin to find a planet that has them.
                            let cnt = visit_count.entry(current).or_insert(0);
                            let i = *cnt % neighbors.len();
                            *cnt += 1;
                            (neighbors[i], format!("round-robin (need basics {:?})", remaining_basics))
                        } else if let Some(&needed_combo) = remaining_combos.front() {
                            let combo_neighbor = neighbors.iter().copied().find(|n| {
                                combo_planet_map
                                    .get(&needed_combo)
                                    .is_some_and(|s| s.contains(n))
                            });
                            if let Some(n) = combo_neighbor {
                                (n, format!("goal-directed → {:?} at planet {}", needed_combo, n))
                            } else {
                                let cnt = visit_count.entry(current).or_insert(0);
                                let i = *cnt % neighbors.len();
                                *cnt += 1;
                                (neighbors[i], format!("round-robin (no known neighbor for {:?})", needed_combo))
                            }
                        } else {
                            let cnt = visit_count.entry(current).or_insert(0);
                            let i = *cnt % neighbors.len();
                            *cnt += 1;
                            (neighbors[i], "round-robin (no active goal)".to_string())
                        };

                        log::info!(
                            "[Explorer #{}] MOVE: planet {} → {}  reason={}  \
                             remaining_basics={:?}  remaining_combos={:?}",
                            id, current, dst, move_reason,
                            remaining_basics, remaining_combos
                        );

                        // Drain stale move confirmations.
                        while planet_rx.try_recv().is_ok() {}

                        let _ = orchestrator_sender.send(ExplorerToOrchestrator::TravelToPlanetRequest {
                            explorer_id: id,
                            current_planet_id: current,
                            dst_planet_id: dst,
                        });

                        match planet_rx.recv_timeout(Duration::from_secs(5)) {
                            Ok(Some(new_sender)) => {
                                planet_sender = new_sender;
                                player_log!("[Explorer #{}] Moved to planet {}", id, dst);
                            }
                            Ok(None) => {
                                log::debug!("[Explorer #{}] Move to planet {} was declined", id, dst);
                            }
                            Err(_) => {
                                log::warn!("[Explorer #{}] Move to planet {} timed out", id, dst);
                            }
                        }
                    }
                    Ok(_) => {
                        log::debug!("[Explorer #{}] No neighbors — staying on planet {}", id, current);
                    }
                    Err(_) => {
                        log::warn!("[Explorer #{}] Neighbors request timed out", id);
                    }
                }

                phase = Phase::Scout;
            }

        }
    }
}


fn build_request(bag: &mut Bag, combo: ComplexResourceType) -> Option<ComplexResourceRequest> {
    use BasicResourceType::*;
    use ComplexResourceType::*;

    match combo {
        Water    => Some(ComplexResourceRequest::Water(
            bag.take_basic_resource(Hydrogen)?.to_hydrogen().ok()?,
            bag.take_basic_resource(Oxygen)?.to_oxygen().ok()?,
        )),
        Diamond  => Some(ComplexResourceRequest::Diamond(
            bag.take_basic_resource(Carbon)?.to_carbon().ok()?,
            bag.take_basic_resource(Carbon)?.to_carbon().ok()?,
        )),
        Life     => Some(ComplexResourceRequest::Life(
            bag.take_complex_resource(Water)?.to_water().ok()?,
            bag.take_basic_resource(Carbon)?.to_carbon().ok()?,
        )),
        Robot    => Some(ComplexResourceRequest::Robot(
            bag.take_basic_resource(Silicon)?.to_silicon().ok()?,
            bag.take_complex_resource(Life)?.to_life().ok()?,
        )),
        Dolphin  => Some(ComplexResourceRequest::Dolphin(
            bag.take_complex_resource(Water)?.to_water().ok()?,
            bag.take_complex_resource(Life)?.to_life().ok()?,
        )),
        AIPartner => Some(ComplexResourceRequest::AIPartner(
            bag.take_complex_resource(Robot)?.to_robot().ok()?,
            bag.take_complex_resource(Diamond)?.to_diamond().ok()?,
        )),
    }
}
