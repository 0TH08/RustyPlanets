//! Simple demo for the Type A planet AI.
//!
//! Usage (from crate root where Cargo.toml is located for the `game` crate):
//! `cargo run --example demo_type_a`
//!
//! This example:
//!  - constructs the channels
//!  - creates a Type A planet using the factory `create_planet`
//!  - spawns the planet (`planet.run()` is blocking, so run it in a thread)
//!  - drives the planet with a few orchestrator/explorer messages and prints responses
//!
//! Purpose: a minimal, deterministic demo you can show at the fair.

use std::thread;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError};

use common_game::planet_ai::type_a::create_planet;
use common_game::components::planet::Planet;
use common_game::protocols::messages::{
    OrchestratorToPlanet, PlanetToOrchestrator, ExplorerToPlanet, PlanetToExplorer,
};

use common_game::components::asteroid::Asteroid;
use common_game::components::sunray::Sunray;

/// Print `PlanetToOrchestrator` responses (consumes the enum).
fn print_orch_msg(msg: PlanetToOrchestrator) {
    match msg {
        PlanetToOrchestrator::StartPlanetAIResult { planet_id } => {
            println!("[Orch<-Planet] StartPlanetAIResult {{ planet_id: {} }}", planet_id);
        }
        PlanetToOrchestrator::StopPlanetAIResult { planet_id } => {
            println!("[Orch<-Planet] StopPlanetAIResult {{ planet_id: {} }}", planet_id);
        }
        PlanetToOrchestrator::KillPlanetResult { planet_id } => {
            println!("[Orch<-Planet] KillPlanetResult {{ planet_id: {} }}", planet_id);
        }
        PlanetToOrchestrator::SunrayAck { planet_id } => {
            println!("[Orch<-Planet] SunrayAck {{ planet_id: {} }}", planet_id);
        }
        PlanetToOrchestrator::AsteroidAck { planet_id, rocket } => {
            println!(
                "[Orch<-Planet] AsteroidAck {{ planet_id: {}, rocket: {} }}",
                planet_id,
                if rocket.is_some() { "Some(Rocket)" } else { "None" }
            );
        }
        PlanetToOrchestrator::InternalStateResponse { planet_id, planet_state } => {
            println!(
                "[Orch<-Planet] InternalStateResponse {{ planet_id: {}, state: {:?} }}",
                planet_id, planet_state
            );
        }
        PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res } => {
            println!("[Orch<-Planet] IncomingExplorerResponse {{ planet_id: {}, res: {:?} }}", planet_id, res);
        }
        PlanetToOrchestrator::OutgoingExplorerResponse { planet_id, res } => {
            println!("[Orch<-Planet] OutgoingExplorerResponse {{ planet_id: {}, res: {:?} }}", planet_id, res);
        }
        PlanetToOrchestrator::Stopped { planet_id } => {
            println!("[Orch<-Planet] Stopped {{ planet_id: {} }}", planet_id);
        }
    }
}

/// Print `PlanetToExplorer` responses (consumes the enum).
fn print_explorer_msg(msg: PlanetToExplorer) {
    match msg {
        PlanetToExplorer::SupportedResourceResponse { resource_list } => {
            println!("[Expl<-Planet] SupportedResourceResponse {{ resources: {:?} }}", resource_list);
        }
        PlanetToExplorer::SupportedCombinationResponse { combination_list } => {
            println!("[Expl<-Planet] SupportedCombinationResponse {{ combos: {:?} }}", combination_list);
        }
        PlanetToExplorer::GenerateResourceResponse { resource } => {
            println!(
                "[Expl<-Planet] GenerateResourceResponse {{ resource: {} }}",
                match resource {
                    Some(res) => format!("{:?}", res), // BasicResource doesn't implement Display
                    None => "None".to_string(),
                }
            );
        }
        PlanetToExplorer::CombineResourceResponse { complex_response } => {
            println!("[Expl<-Planet] CombineResourceResponse {{ {:?} }}", complex_response);
        }
        PlanetToExplorer::AvailableEnergyCellResponse { available_cells } => {
            println!("[Expl<-Planet] AvailableEnergyCellResponse {{ available_cells: {} }}", available_cells);
        }
        PlanetToExplorer::Stopped => {
            println!("[Expl<-Planet] Stopped");
        }
    }
}

/// Receive and print a single orchestrator message (consumes it). Non-blocking with timeout.
fn recv_and_print_orch(rx: &Receiver<PlanetToOrchestrator>, timeout_ms: u64) {
    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(msg) => print_orch_msg(msg),
        Err(RecvTimeoutError::Timeout) => println!("[Demo] timed out waiting for Planet->Orchestrator response"),
        Err(RecvTimeoutError::Disconnected) => println!("[Demo] orchestrator channel disconnected"),
    }
}

/// Receive and print a single explorer message (consumes it). Non-blocking with timeout.
fn recv_and_print_explorer(rx: &Receiver<PlanetToExplorer>, timeout_ms: u64) {
    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(msg) => print_explorer_msg(msg),
        Err(RecvTimeoutError::Timeout) => println!("[Demo] timed out waiting for Planet->Explorer response"),
        Err(RecvTimeoutError::Disconnected) => println!("[Demo] explorer channel disconnected"),
    }
}

fn main() {
    println!("=== Type A Planet demo ===");

    // 1) Build channels
    // Orchestrator -> Planet (planet receives these)
    let (tx_orch_to_planet, rx_orch_to_planet) = unbounded::<OrchestratorToPlanet>();
    // Planet -> Orchestrator (planet sends these)
    let (tx_planet_to_orch, rx_planet_to_orch) = unbounded::<PlanetToOrchestrator>();
    // Global explorer -> Planet channel (planet receives explorer requests here)
    let (tx_expl_global, rx_expl_global) = unbounded::<ExplorerToPlanet>();

    // 2) Create the planet using the Type A factory
    let mut planet: Planet = match create_planet(7, rx_orch_to_planet, tx_planet_to_orch.clone(), rx_expl_global) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create Type A planet: {}", e);
            return;
        }
    };

    // 3) Spawn the planet run loop in a separate thread (it blocks)
    let handle = thread::spawn(move || {
        if let Err(e) = planet.run() {
            eprintln!("[Planet thread] planet.run() returned error: {}", e);
        } else {
            println!("[Planet thread] planet.run() exited normally");
        }
    });

    // Helper closures for demo sends
    let send_orch = |msg| {
        tx_orch_to_planet.send(msg).expect("failed to send orchestrator->planet msg");
    };
    let send_expl_global = |msg| {
        tx_expl_global.send(msg).expect("failed to send explorer->planet msg");
    };

    // small sleep so the planet thread starts
    thread::sleep(Duration::from_millis(50));

    // 4) Start the planet AI
    println!("\n--- Start Planet AI ---");
    send_orch(OrchestratorToPlanet::StartPlanetAI);
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 5) Send a Sunray -> charges a cell -> expect SunrayAck
    println!("\n--- Send Sunray ---");
    // use Default::default() so we don't call private `new()`
    send_orch(OrchestratorToPlanet::Sunray(Sunray::default()));
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 6) Register a local explorer (IncomingExplorerRequest).
    println!("\n--- Register explorer (incoming) ---");
    let explorer_id: u32 = 314;
    let (tx_local_to_explorer, rx_local_from_planet) = unbounded::<PlanetToExplorer>();
    send_orch(OrchestratorToPlanet::IncomingExplorerRequest {
        explorer_id,
        new_mpsc_sender: tx_local_to_explorer,
    });
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 7) Ask supported resources via global explorer channel and expect reply on the local channel
    println!("\n--- Explorer: SupportedResourceRequest ---");
    send_expl_global(ExplorerToPlanet::SupportedResourceRequest { explorer_id });
    recv_and_print_explorer(&rx_local_from_planet, 500);

    // 8) Ensure we have a charged cell: send another Sunray then request generation.
    println!("\n--- Charge cell (another Sunray) and request Oxygen generation ---");
    send_orch(OrchestratorToPlanet::Sunray(Sunray::default()));
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // Explorer requests generation of Oxygen (Type A is configured to support Oxygen)
    send_expl_global(ExplorerToPlanet::GenerateResourceRequest {
        explorer_id,
        resource: common_game::components::resource::BasicResourceType::Oxygen,
    });
    recv_and_print_explorer(&rx_local_from_planet, 500);

    // 9) Demonstrate asteroid survival: send an Asteroid and expect AsteroidAck with Some(Rocket)
    println!("\n--- Send Asteroid (expect rocket if cell was charged) ---");
    send_orch(OrchestratorToPlanet::Asteroid(Asteroid::default()));
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 10) Stop the planet AI
    println!("\n--- Stop Planet AI ---");
    send_orch(OrchestratorToPlanet::StopPlanetAI);
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 11) While stopped, asking for InternalStateRequest should yield Stopped
    println!("\n--- InternalStateRequest while stopped (expect Stopped) ---");
    send_orch(OrchestratorToPlanet::InternalStateRequest);
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // 12) Kill the planet (cleanup)
    println!("\n--- Kill Planet ---");
    send_orch(OrchestratorToPlanet::KillPlanet);
    recv_and_print_orch(&rx_planet_to_orch, 500);

    // Give the planet thread a moment to exit then join
    thread::sleep(Duration::from_millis(200));
    let _ = handle.join();

    println!("\n=== Demo finished ===");
}
