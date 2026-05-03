use std::thread::{self, JoinHandle};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc,Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use common_game::protocols::orchestrator_explorer::{OrchestratorToExplorer, ExplorerToOrchestrator};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};
use crossbeam_channel::{Sender, Receiver};
use common_game::components::resource::{
    BasicResource, BasicResourceType, ComplexResource, ComplexResourceRequest, ComplexResourceType
};

use common_game::utils::ID;

pub struct Explorer{
    id: ID,
    current_planet_id: ID,
    neighbors: Vec<ID>,
    orchestrator_sender: Sender<ExplorerToOrchestrator<Bag>>,
    orchestrator_receiver: Receiver<OrchestratorToExplorer>,
    planet_sender: Sender<ExplorerToPlanet>,
    planet_receiver: Receiver<PlanetToExplorer>,
    bag: Arc<Mutex<Bag>>,
    ai_running: Arc<AtomicBool>,
    ai_thread: Arc<Mutex<Option<JoinHandle<()>>>>,
}

pub struct Bag {
    pub basic_resources : HashMap<BasicResourceType, u32>,
    pub complex_resources: HashMap<ComplexResourceType, u32>,
    basic_resource_instances: HashMap<BasicResourceType , Vec<BasicResource>>,
    complex_resource_instances : HashMap<ComplexResourceType , Vec<ComplexResource>>,
}

impl Explorer{
    pub fn new(
        id: ID,
        current_planet_id: ID,
        orchestrator_sender: Sender<ExplorerToOrchestrator<Bag>>,
        orchestrator_receiver: Receiver<OrchestratorToExplorer>,
        planet_sender: Sender<ExplorerToPlanet>,
        planet_receiver: Receiver<PlanetToExplorer>,
    )->Self{
        let bag= Arc::new(Mutex::new(Bag::new()));
        let neighbors= Vec::new();
        let ai_running=Arc::new(AtomicBool::new(false));
        let ai_thread = Arc::new(Mutex::new(None));
    
        Explorer{
            id,
            current_planet_id,
            neighbors,
            orchestrator_sender,
            orchestrator_receiver,
            planet_sender,
            planet_receiver,
            bag,
            ai_running,
            ai_thread,
        }
    }

    pub fn run(mut self){
        // spawn thread
        let _ = thread::spawn(move || {

        // inside thread: match on orchestrator messages
            while let Ok(mssg) = self.orchestrator_receiver.recv()  {
                match mssg{
                    OrchestratorToExplorer::BagContentRequest => {
                        let response = self.handle_bag_content_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::CurrentPlanetRequest=> {
                        let response = self.handle_current_planet_request();
                        let _ = self.orchestrator_sender.send(response); 
                    }

                    OrchestratorToExplorer::MoveToPlanet { sender_to_new_planet, planet_id } => {
                        let response= self.handle_move_to_planet(sender_to_new_planet, planet_id);
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::StartExplorerAI=> {
                        let response = self.handle_start_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::StopExplorerAI => {
                        let response= self.handle_stop_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::ResetExplorerAI=> {
                        let response= self.handle_reset_explorer_ai();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::SupportedResourceRequest => {
                        let response = self.handle_supported_resource_request();
                        let _ = self.orchestrator_sender.send(response);
                    }

                    OrchestratorToExplorer::SupportedCombinationRequest => {
                        let response= self.handle_supported_combination_request();
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
                        self.handle_neighbors_response(neighbors);
                    }

                    OrchestratorToExplorer::KillExplorer => {
                        let response= ExplorerToOrchestrator::KillExplorerResult { explorer_id: self.id };
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

    fn handle_combine_resource_request(&mut self, to_generate: ComplexResourceType)->ExplorerToOrchestrator<Bag>{
        let req = match to_generate{
            ComplexResourceType::Water => self.make_water(),
            ComplexResourceType::Diamond => self.make_diamond(),
            ComplexResourceType::Life => self.make_life(),
            ComplexResourceType::Robot => self.make_robot(),
            ComplexResourceType::Dolphin => self.make_dolphin(),
            ComplexResourceType::AIPartner => self.make_aipartner(),
        };

        let req = match req{
            None => return ExplorerToOrchestrator::CombineResourceResponse { 
                explorer_id: self.id, 
                generated: Err("Not enough resources in bag".to_string()),
            },
            Some(r) => r,
        };

        let msg = ExplorerToPlanet::CombineResourceRequest { explorer_id: self.id , msg: req };
        let _ = self.planet_sender.send(msg);

        if let Ok(PlanetToExplorer::CombineResourceResponse { complex_response }) = self.planet_receiver.recv(){
            match complex_response{
                Ok(complex_resource)=>{
                    self.bag.lock().unwrap().insert_complex_resource_in_bag(complex_resource);
                    ExplorerToOrchestrator::CombineResourceResponse { explorer_id: self.id, generated: Ok(()) }
                }

                Err((e, _lhs, _rhs))=>{
                    ExplorerToOrchestrator::CombineResourceResponse { explorer_id: self.id, generated: Err(e) }
                }
            }
        }else{
            ExplorerToOrchestrator::CombineResourceResponse { 
                explorer_id: self.id,
                generated: Err("No response from planet".to_string())}
        }
    }

    fn make_aipartner(&mut self)->Option<ComplexResourceRequest>{
        let r= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Robot)?;
        let d= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Diamond)?;
        let r = r.to_robot().ok()?;
        let d = d.to_diamond().ok()?;
        Some(ComplexResourceRequest::AIPartner(r, d))
    }

    fn make_dolphin(&mut self)->Option<ComplexResourceRequest>{
        let w= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Water)?;
        let l= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Life)?;
        let w= w.to_water().ok()?;
        let l = l.to_life().ok()?;
        Some(ComplexResourceRequest::Dolphin(w, l))
    }

    fn make_robot(&mut self)->Option<ComplexResourceRequest>{
        let s= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Silicon)?;
        let l= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Life)?;
        let s= s.to_silicon().ok()?;
        let l = l.to_life().ok()?;
        Some(ComplexResourceRequest::Robot(s, l))
    }

    fn make_life(&mut self)->Option<ComplexResourceRequest>{
        let w= self.bag.lock().unwrap().take_complex_resource(ComplexResourceType::Water)?;
        let c= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Carbon)?;
        let w= w.to_water().ok()?;
        let c = c.to_carbon().ok()?;
        Some(ComplexResourceRequest::Life(w, c))
    }

    fn make_diamond(&mut self)->Option<ComplexResourceRequest>{
        let c1= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Carbon)?;
        let c2= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Carbon)?;
        let c1 = c1.to_carbon().ok()?;
        let c2 = c2.to_carbon().ok()?;
        Some(ComplexResourceRequest::Diamond(c1, c2))
    }
    
    fn make_water(&mut self)->Option<ComplexResourceRequest>{
        let h= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Hydrogen)?;
        let o= self.bag.lock().unwrap().take_basic_resource(BasicResourceType::Oxygen)?;
        let h = h.to_hydrogen().ok()?;
        let o = o.to_oxygen().ok()?;
        Some(ComplexResourceRequest::Water(h, o))

    }

    fn handle_generate_resource_request(&mut self, to_generate: BasicResourceType)->ExplorerToOrchestrator<Bag>{
        let msg= ExplorerToPlanet::GenerateResourceRequest { explorer_id: self.id, resource: to_generate};
        let _ = self.planet_sender.send(msg);

        if let Ok(PlanetToExplorer::GenerateResourceResponse { resource }) = self.planet_receiver.recv(){
            match resource{
                Some(r) => { 
                    self.bag.lock().unwrap().insert_basic_resource_in_bag(r);
                    ExplorerToOrchestrator::GenerateResourceResponse { 
                        explorer_id: self.id,
                        generated: Ok(())
                    }
                }
                None => {
                    ExplorerToOrchestrator::GenerateResourceResponse { 
                        explorer_id: self.id,
                        generated: Err("Planet failed to generate".to_string() ) 
                    }
                }
            }
        } else{
            ExplorerToOrchestrator::GenerateResourceResponse {
                explorer_id: self.id,
                generated: Err("No response from Planet". to_string()) 
            }
        }
    }

    fn handle_supported_combination_request(&self)->ExplorerToOrchestrator<Bag>{
        let msg = ExplorerToPlanet::SupportedCombinationRequest { explorer_id: self.id };
        let _ = self.planet_sender.send(msg);

        if let Ok(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = self.planet_receiver.recv(){
            ExplorerToOrchestrator::SupportedCombinationResult { 
                explorer_id: self.id,
                combination_list,
            }
        } else {
            ExplorerToOrchestrator::SupportedCombinationResult { 
                explorer_id: self.id,
                combination_list : HashSet::new(),
            }
        }
    }

    fn handle_supported_resource_request(&self)->ExplorerToOrchestrator<Bag>{
        let req= ExplorerToPlanet::SupportedResourceRequest { explorer_id: self.id };
        let _ = self.planet_sender.send(req);

        if let Ok(PlanetToExplorer::SupportedResourceResponse { resource_list })= self.planet_receiver.recv(){
            ExplorerToOrchestrator::SupportedResourceResult { 
                explorer_id: self.id,
                supported_resources: resource_list,
            }
        } else{
            ExplorerToOrchestrator::SupportedResourceResult { 
                explorer_id: self.id,
                supported_resources: HashSet::new(),
            }
        }
    }

    /// Stops the AI, wipes the bag, and starts a fresh AI cycle.
    fn handle_reset_explorer_ai(&self)->ExplorerToOrchestrator<Bag>{
        log::info!("[Explorer #{}] Resetting autonomous AI", self.id);

        // Tell running thread to exit
        self.ai_running.store(false, Ordering::SeqCst);
        
        // WAIT for old thread to die
        if let Ok(mut handle) = self.ai_thread.lock(){
           if let Some(h) = handle.take(){
            let _ = h.join(); 
           }
        }

        // Wipe the bag
        *self.bag.lock().unwrap() = Bag::new();

        // Start a fresh thread
        self.ai_running.store(true, Ordering::SeqCst);
        self.spawn_ai_thread();

        ExplorerToOrchestrator::ResetExplorerAIResult { explorer_id: self.id }
    }

    fn handle_stop_explorer_ai(&self)->ExplorerToOrchestrator<Bag>{
        //stop autonomous AI
        self.ai_running.store(false, Ordering::SeqCst);
        log::info!("[Explorer #{}:] Stoping autonomous AI", self.id);
        ExplorerToOrchestrator::StopExplorerAIResult { explorer_id: self.id }
    }

    fn handle_start_explorer_ai(&self)->ExplorerToOrchestrator<Bag>{
        //Check if AI is already running
        if self.ai_running
            .compare_exchange(false, true, Ordering::SeqCst,Ordering::SeqCst)
            .is_ok(){
                //AI not started yet. Start it
                log::info!("[Explorer #{}:] Starting autonomous AI", self.id);
                self.spawn_ai_thread();
        }
        else{ 
            //AI already running
            log::debug!("[Explorer #{}:] AI already running - ignoring start", self.id)
        }
        ExplorerToOrchestrator::StartExplorerAIResult { explorer_id: self.id }
    }

    fn spawn_ai_thread(&self){
        let id = self.id;
        let planet_sender= self.planet_sender.clone();
        let planet_receiver= self.planet_receiver.clone();
        let bag = Arc::clone(&self.bag);
        let ai_running = Arc::clone(&self.ai_running);
        
        let handle = thread::spawn(move || {
            autonomous_ai(id, planet_sender, planet_receiver, bag, ai_running);
        });

        *self.ai_thread.lock().unwrap() = Some(handle);
    }

    fn handle_move_to_planet(
        &mut self,
        sender_to_new_planet: Option<Sender<ExplorerToPlanet>>,
        planet_id: ID,
    )->ExplorerToOrchestrator<Bag>{
        if let Some(new_planet_sender)= sender_to_new_planet{
            self.planet_sender = new_planet_sender;
        }
        self.current_planet_id = planet_id;
        ExplorerToOrchestrator::MovedToPlanetResult { explorer_id: self.id, planet_id : self.current_planet_id}
    }

    fn handle_current_planet_request(&self)->ExplorerToOrchestrator<Bag>{
        ExplorerToOrchestrator::CurrentPlanetResult { explorer_id: self.id, planet_id: self.current_planet_id}
    }

    fn handle_bag_content_request(&self)->ExplorerToOrchestrator<Bag>{
        ExplorerToOrchestrator::BagContentResponse { explorer_id: self.id, bag_content: self.bag.lock().unwrap().clone() }
    }

}

impl Bag{
    pub fn new()->Bag{
        let basic_resources = HashMap::new();
        let complex_resources= HashMap::new();
        let basic_resource_instances= HashMap::new();
        let complex_resource_instances= HashMap::new();
        Bag { 
            basic_resources,
            complex_resources,
            basic_resource_instances,
            complex_resource_instances,
        }
    }

    pub fn insert_basic_resource_in_bag(&mut self, resource: BasicResource){
        //1 — get the type and save it
        let resource_type = resource.get_type();
        //2 — update count
        self.basic_resources.entry(resource_type).and_modify(|c| *c += 1).or_insert(1);
        //3 — push the actual instance
        self.basic_resource_instances.entry(resource_type).or_insert_with(Vec::new).push(resource);
    }

    pub fn insert_complex_resource_in_bag(&mut self, resource: ComplexResource){
        let resource_type = resource.get_type();
        self.complex_resources.entry(resource_type).and_modify(|c| *c += 1).or_insert(1);
        self.complex_resource_instances.entry(resource_type).or_insert_with(Vec::new).push(resource);
    }

    pub fn take_basic_resource(&mut self, resource_type: BasicResourceType)->Option<BasicResource>{
        let instance = self.basic_resource_instances.get_mut(&resource_type)?.pop()?;
        self.basic_resources.entry(resource_type).and_modify(|c| *c -= 1);
        Some(instance) 
    }

    pub fn take_complex_resource(&mut self, resource_type: ComplexResourceType)->Option<ComplexResource>{
        let instance = self.complex_resource_instances.get_mut(&resource_type)?.pop()?;
        self.complex_resources.entry(resource_type).and_modify(|c| *c -= 1);
        Some(instance) 
    }
}

impl Clone for Bag{
    fn clone(&self) -> Self {
        Bag {
            basic_resources: self.basic_resources.clone(),
            complex_resources: self.complex_resources.clone(),
            basic_resource_instances: HashMap::new(),
            complex_resource_instances: HashMap::new(),
        }
    }
}

// ───── Autonomous AI ────────────────────────────────────────────────────────

// Concrete plan for one goal cycle.
pub struct GoalPlan {
    /// Target complex resource, or `None` if the planet has no combination support.
    pub goal: Option<ComplexResourceType>,
    /// Basic resources to collect in order (may contain duplicates, e.g. 3× Carbon).
    pub basics_needed: Vec<BasicResourceType>,
    /// Ordered combination steps that transform the basics into `goal`.
    pub combination_sequence: Vec<ComplexResourceType>,
}

/// AI execution phase.
#[derive(Debug, PartialEq, Eq)]
enum Phase {
    /// Query the planet's capabilities.
    Scout,
    /// Select the most ambitious achievable goal.
    Plan,
    /// Collect all basic resources the goal requires.
    Collect,
    /// Execute the recipe chain in the correct dependency order.
    Craft,
}

fn plan_for(goal: ComplexResourceType) -> GoalPlan {
    use BasicResourceType::{Hydrogen, Oxygen, Carbon, Silicon};
    use ComplexResourceType::{Water, Diamond, Life, Robot, Dolphin, AIPartner};
    let (basics, combos) = match goal {
        ComplexResourceType::Water     => (vec![Hydrogen, Oxygen], vec![Water]),
        ComplexResourceType::Diamond   => (vec![Carbon, Carbon], vec![Diamond]),
        ComplexResourceType::Life      => (vec![Hydrogen, Oxygen, Carbon], vec![Water, Life]),
        ComplexResourceType::Dolphin   => (vec![Hydrogen, Oxygen, Hydrogen, Oxygen, Carbon ], vec![Water, Life, Water, Dolphin]),
        ComplexResourceType::Robot     => (vec![Hydrogen, Oxygen, Carbon, Silicon], vec![Water, Life, Robot]),
        ComplexResourceType::AIPartner => (vec![Hydrogen, Oxygen, Carbon, Silicon, Carbon, Carbon], vec![Water, Life, Robot, Diamond, AIPartner]),
    };
    GoalPlan { goal: Some(goal), basics_needed: basics, combination_sequence: combos }
}

fn pick_best_goal(
    supported_basics: &HashSet<BasicResourceType>,
    supported_combos: &HashSet<ComplexResourceType>,
) -> GoalPlan {
    use ComplexResourceType::{Water, Diamond, Life, Robot, Dolphin, AIPartner};
    for goal in [AIPartner, Robot, Dolphin, Life, Diamond, Water] {
        let plan = plan_for(goal);
        let basics_ok = plan.basics_needed.iter().all(|b| supported_basics.contains(b));
        let combos_ok = plan.combination_sequence.iter().all(|c| supported_combos.contains(c));
        if basics_ok && combos_ok {
            return plan;
        }
    }

    // fallback: no combination goal reachable
    let basics = supported_basics.iter().copied().collect();
    GoalPlan { goal: None, basics_needed: basics, combination_sequence: vec![], }
}

fn autonomous_ai(
    id: ID,
    planet_sender: Sender<ExplorerToPlanet>,
    planet_receiver: Receiver<PlanetToExplorer>,
    bag: Arc<Mutex<Bag>>,
    ai_running: Arc<AtomicBool>,
) {
    let mut phase = Phase::Scout;
    let mut supported_basics= HashSet::new();
    let mut supported_combos = HashSet::new();
    let mut remaining_basics = VecDeque::new();
    let mut remaining_combos = VecDeque::new();

    while ai_running.load(Ordering::SeqCst) {
        match phase {
            Phase::Scout => {
                let msg= ExplorerToPlanet::SupportedResourceRequest { explorer_id: id };
                let _ = planet_sender.send(msg);

                if let Ok(PlanetToExplorer::SupportedResourceResponse { resource_list })
                     = planet_receiver.recv(){
                        supported_basics = resource_list;
                }

                let msg = ExplorerToPlanet::SupportedCombinationRequest { explorer_id: id };
                let _ = planet_sender.send(msg);

                if let Ok(PlanetToExplorer::SupportedCombinationResponse { combination_list })
                    = planet_receiver.recv(){
                        supported_combos = combination_list;
                }
                
                phase = Phase::Plan;     

            }
            Phase::Plan => {
                let plan= pick_best_goal(&supported_basics, &supported_combos);
                remaining_basics = plan.basics_needed.into_iter().collect();
                remaining_combos = plan.combination_sequence.into_iter().collect();
                phase = Phase::Collect;
            }
            Phase::Collect => {
                if let Some(&resource) = remaining_basics.front() {
                    let msg= ExplorerToPlanet::GenerateResourceRequest { explorer_id: id, resource };
                    let _ = planet_sender.send(msg); 

                    if let Ok(PlanetToExplorer::GenerateResourceResponse { resource: Some(r) })
                        = planet_receiver.recv(){
                            bag.lock().unwrap().insert_basic_resource_in_bag(r);
                            remaining_basics.pop_front();
                    }
                }  else {
                        phase = Phase::Craft;
                }    
            }
            Phase::Craft => {
                if let Some(c) = remaining_combos.front(){
                    let req= build_request(&mut bag.lock().unwrap(), *c);

                    match req {
                        None => {
                            // ingredients missing — skip this step
                            remaining_combos.pop_front();
                        }
                        Some(req)=>{
                            let msg = ExplorerToPlanet::CombineResourceRequest { 
                                explorer_id: id,
                                msg: req, 
                            };
                            let _ = planet_sender.send(msg);

                            if let Ok(PlanetToExplorer::CombineResourceResponse { complex_response: Ok(r) })
                            = planet_receiver.recv()
                            {
                                bag.lock().unwrap().insert_complex_resource_in_bag(r);
                                remaining_combos.pop_front();
                            }
                            // if Err response: do nothing, retry next iteration
                            }
                    }

                } else {
                    phase = Phase::Scout; 
                }
            }
        }
    }

}

fn build_request(bag: &mut Bag, combo: ComplexResourceType)->Option<ComplexResourceRequest>{
    use BasicResourceType::*;
    use ComplexResourceType::*;

    match combo {
        Water => {
            let h = bag.take_basic_resource(Hydrogen)?.to_hydrogen().ok()?;
            let o = bag.take_basic_resource(Oxygen)?.to_oxygen().ok()?;
            Some(ComplexResourceRequest::Water(h, o))
        }

        Diamond => {
            let c1 = bag.take_basic_resource(Carbon)?.to_carbon().ok()?; 
            let c2 = bag.take_basic_resource(Carbon)?.to_carbon().ok()?;
            Some(ComplexResourceRequest::Diamond(c1, c2))
        }

        Life => {
            let w = bag.take_complex_resource(Water)?.to_water().ok()?;
            let c = bag.take_basic_resource(Carbon)?.to_carbon().ok()?;
            Some(ComplexResourceRequest::Life(w, c))
        }

        Robot => {
            let s = bag.take_basic_resource(Silicon)?.to_silicon().ok()?;
            let l = bag.take_complex_resource(Life)?.to_life().ok()?;
            Some(ComplexResourceRequest::Robot(s, l))
        }

        Dolphin => {
            let w = bag.take_complex_resource(Water)?.to_water().ok()?;
            let l = bag.take_complex_resource(Life)?.to_life().ok()?;
            Some(ComplexResourceRequest::Dolphin(w, l))
        }

        AIPartner => {
            let r = bag.take_complex_resource(Robot)?.to_robot().ok()?;
            let d = bag.take_complex_resource(Diamond)?.to_diamond().ok()?;
            Some(ComplexResourceRequest::AIPartner(r, d))
        }
    }
}