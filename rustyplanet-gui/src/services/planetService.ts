import type { PlanetDetails, PlanetSummary } from '../types/planet'

// Temporary mock data. Later this can be replaced with real data
// coming from the Rust simulation backend.

const mockPlanets: PlanetDetails[] = [
  {
    id: 1,
    name: 'Skycartel Alpha',
    kind: 'skycartel',
    typeLabel: 'Type A (Skycartel)',
    runState: 'running',
    energyCellsTotal: 5,
    energyCellsCharged: 4,
    resourceCellsTotal: 3,
    resourceCellsCharged: 3,
    defenseCellsTotal: 2,
    defenseCellsCharged: 1,
    errors: 0,
    resources: {
      Carbon: 120,
      Iron: 40,
    },
    skycartelStats: {
      totalResourcesGenerated: 500,
      explorerArrivals: 12,
      explorerDepartures: 8,
      rocketsBuilt: 2,
      asteroidsDeflected: 3,
      errorsEncountered: 1,
    },
  },
  {
    id: 2,
    name: 'Experimental Beta',
    kind: 'other',
    typeLabel: 'Type B (external)',
    runState: 'paused',
    energyCellsTotal: 6,
    energyCellsCharged: 3,
    resourceCellsTotal: 3,
    resourceCellsCharged: 2,
    defenseCellsTotal: 3,
    defenseCellsCharged: 1,
    errors: 2,
    resources: {
      Hydrogen: 80,
    },
  },
]

export async function getPlanets(): Promise<PlanetSummary[]> {
  await new Promise((resolve) => setTimeout(resolve, 50))
  return mockPlanets.map(
    ({ id, name, kind, typeLabel, runState, energyCellsTotal, energyCellsCharged, errors }) => ({
      id,
      name,
      kind,
      typeLabel,
      runState,
      energyCellsTotal,
      energyCellsCharged,
      errors,
    }),
  )
}

export async function getPlanetDetails(id: number): Promise<PlanetDetails | undefined> {
  await new Promise((resolve) => setTimeout(resolve, 50))
  return mockPlanets.find((p) => p.id === id)
}

