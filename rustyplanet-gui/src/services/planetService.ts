import type { PlanetDetails, PlanetSummary } from '../types/planet'

// Temporary mock data. Later this can be replaced with real data
// coming from the Rust simulation backend.

const mockPlanets: PlanetDetails[] = [
  {
    id: 1,
    name: 'Skycartel Alpha',
    kind: 'skycartel',
    typeLabel: 'Skycartel (Type A)',
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
    name: 'Luna4 Prime',
    kind: 'luna4',
    typeLabel: 'Luna4 (Type D)',
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
      Oxygen: 45,
    },
  },
  {
    id: 3,
    name: 'Black Adidas Shoe',
    kind: 'blackadidasshoe',
    typeLabel: 'BlackAdidasShoe (Type D)',
    runState: 'running',
    energyCellsTotal: 4,
    energyCellsCharged: 4,
    errors: 0,
    resources: {
      Carbon: 200,
      Hydrogen: 150,
    },
  },
  {
    id: 4,
    name: 'Immutable Cosmic Borrow',
    kind: 'immutablecosmicborrow',
    typeLabel: 'ImmutableCosmicBorrow (Type C)',
    runState: 'paused',
    energyCellsTotal: 8,
    energyCellsCharged: 5,
    errors: 1,
    resources: {
      Hydrogen: 300,
    },
  },
  {
    id: 5,
    name: 'Rust-Eze',
    kind: 'rusteze',
    typeLabel: 'RustEze (Type D)',
    runState: 'running',
    energyCellsTotal: 5,
    energyCellsCharged: 2,
    errors: 0,
    resources: {
      Silicon: 100,
      Oxygen: 75,
    },
  },
  {
    id: 6,
    name: 'Crabtorio',
    kind: 'crabtorio',
    typeLabel: 'Crabtorio (Type B)',
    runState: 'paused',
    energyCellsTotal: 7,
    energyCellsCharged: 7,
    resourceCellsTotal: 4,
    resourceCellsCharged: 4,
    errors: 3,
    resources: {
      Water: 50,
    },
  },
  {
    id: 7,
    name: 'Orbitron',
    kind: 'orbitron',
    typeLabel: 'Orbitron (Type B)',
    runState: 'running',
    energyCellsTotal: 6,
    energyCellsCharged: 6,
    errors: 0,
    resources: {
      Hydrogen: 500,
      Oxygen: 500,
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

