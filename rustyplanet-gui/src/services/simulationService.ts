import type { SimulationRunState, SimulationStatus } from '../types/simulation'

const API_BASE = 'http://localhost:8080/api'

export async function getSimulationStatus(): Promise<SimulationStatus> {
  const res = await fetch(`${API_BASE}/simulation/status`)
  return res.json()
}

export async function setSimulationRunState(runState: SimulationRunState): Promise<SimulationStatus> {
  const res = await fetch(`${API_BASE}/simulation/run-state`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ runState }),
  })
  return res.json()
}

export async function stepSimulation(): Promise<SimulationStatus> {
  const res = await fetch(`${API_BASE}/simulation/step`, { method: 'POST' })
  return res.json()
}

export async function setSimulationSpeed(speed: number): Promise<SimulationStatus> {
  const res = await fetch(`${API_BASE}/simulation/speed`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ speed }),
  })
  return res.json()
}

export async function resetSimulation(): Promise<SimulationStatus> {
  const res = await fetch(`${API_BASE}/simulation/reset`, { method: 'POST' })
  return res.json()
}

export async function sendSunray(planetId: number): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/sunray/${planetId}`, { method: 'POST' })
  return res.json()
}

export async function sendAsteroid(planetId: number): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/asteroid/${planetId}`, { method: 'POST' })
  return res.json()
}