import type { SimulationRunState } from './simulation'

export type PlanetKind = 'skycartel' | 'luna4' | 'blackadidasshoe' | 'immutablecosmicborrow' | 'rusteze' | 'crabtorio' | 'orbitron'

export interface SkycartelStatsSnapshot {
  totalResourcesGenerated: number
  explorerArrivals: number
  explorerDepartures: number
  rocketsBuilt: number
  asteroidsDeflected: number
  errorsEncountered: number
}

export interface PlanetSummary {
  id: number
  name: string
  kind: PlanetKind
  typeLabel: string
  runState: SimulationRunState
  energyCellsTotal: number
  energyCellsCharged: number
  resourceCellsTotal?: number
  resourceCellsCharged?: number
  defenseCellsTotal?: number
  defenseCellsCharged?: number
  errors: number
}

export interface PlanetDetails extends PlanetSummary {
  resources: Record<string, number>
  skycartelStats?: SkycartelStatsSnapshot
}

