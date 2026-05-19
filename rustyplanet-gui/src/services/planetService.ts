import type { PlanetSummary, PlanetDetails } from '../types/planet'

const API_BASE = 'http://localhost:8080/api'

export async function getPlanets(): Promise<PlanetSummary[]> {
  const res = await fetch(`${API_BASE}/planets`)
  return res.json()
}

export async function getPlanetDetails(id: number): Promise<PlanetDetails> {
  const res = await fetch(`${API_BASE}/planets/${id}`)
  return res.json()
}

export async function startPlanet(id: number): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/planets/${id}/start`, { method: 'POST' })
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: `HTTP ${res.status}` }))
    throw new Error(body.error || `HTTP ${res.status}`)
  }
  return res.json()
}

export async function stopPlanet(id: number): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/planets/${id}/stop`, { method: 'POST' })
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: `HTTP ${res.status}` }))
    throw new Error(body.error || `HTTP ${res.status}`)
  }
  return res.json()
}

export async function moveExplorer(explorerId: number, planetId: number): Promise<{ status: string }> {
  const res = await fetch(`${API_BASE}/explorers/${explorerId}/move/${planetId}`, { method: 'POST' })
  return res.json()
}