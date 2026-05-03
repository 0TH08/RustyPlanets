import type { SimulationRunState, SimulationStatus } from '../types/simulation'

// Temporary mock implementation.
// Later this can be replaced with real HTTP/WebSocket calls to the Rust backend.

let currentStatus: SimulationStatus = {
  runState: 'idle',
  tick: 0,
  speed: 1,
}

export async function getSimulationStatus(): Promise<SimulationStatus> {
  // Simulate a tiny network delay
  await new Promise((resolve) => setTimeout(resolve, 50))
  return currentStatus
}

export async function setSimulationRunState(runState: SimulationRunState): Promise<SimulationStatus> {
  await new Promise((resolve) => setTimeout(resolve, 50))
  currentStatus = { ...currentStatus, runState }
  return currentStatus
}

export async function stepSimulation(): Promise<SimulationStatus> {
  await new Promise((resolve) => setTimeout(resolve, 50))
  currentStatus = {
    ...currentStatus,
    tick: currentStatus.tick + 1,
  }
  return currentStatus
}

export async function setSimulationSpeed(speed: number): Promise<SimulationStatus> {
  await new Promise((resolve) => setTimeout(resolve, 50))
  currentStatus = { ...currentStatus, speed }
  return currentStatus
}

