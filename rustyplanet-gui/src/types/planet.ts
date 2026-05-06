import type { } from './simulation'

export type PlanetKind = 'skycartel' | 'luna4' | 'blackadidasshoe' | 'immutablecosmicborrow' | 'rusteze' | 'crabtorio' | 'orbitron'

export interface CellState {
  index: number
  charged: boolean
}

export interface GenerationEntry {
  resource: string
}

export interface PlanetStats {
  totalResourcesGenerated: number
  explorerArrivals: number
  explorerDepartures: number
  rocketsBuilt: number
  asteroidsDeflected: number
  combinationsAttempted: number
  combinationsSucceeded: number
  errorsEncountered: number
}

export interface PlanetSummary {
  id: number
  name: string
  kind: PlanetKind
  aiRunning: boolean
  explorerCount: number
  totalResourcesGenerated: number
  rocketsBuilt: number
  asteroidsDeflected: number
  errorsEncountered: number
}

export interface PlanetDetails {
  summary: PlanetSummary
  explorerArrivals: number
  explorerDepartures: number
  cells: CellState[]
  generationHistory: GenerationEntry[]
}