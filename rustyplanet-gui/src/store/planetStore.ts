import { create } from 'zustand'

import { getPlanetDetails, getPlanets } from '../services/planetService'
import type { PlanetDetails, PlanetSummary } from '../types/planet'

interface PlanetState {
  planets: PlanetSummary[]
  selectedPlanetId: number | null
  selectedPlanet: PlanetDetails | null
  isLoadingList: boolean
  isLoadingDetails: boolean
  error?: string
  loadPlanets: () => Promise<void>
  selectPlanet: (id: number) => Promise<void>
}

export const usePlanetStore = create<PlanetState>((set, get) => ({
  planets: [],
  selectedPlanetId: null,
  selectedPlanet: null,
  isLoadingList: false,
  isLoadingDetails: false,
  error: undefined,
  loadPlanets: async () => {
    set({ isLoadingList: true, error: undefined })
    try {
      const planets = await getPlanets()
      set({ planets })
      const currentSelected = get().selectedPlanetId ?? planets[0]?.id ?? null
      if (currentSelected != null) {
        await get().selectPlanet(currentSelected)
      }
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to load planets' })
    } finally {
      set({ isLoadingList: false })
    }
  },
  selectPlanet: async (id: number) => {
    set({ selectedPlanetId: id, isLoadingDetails: true, error: undefined })
    try {
      const details = await getPlanetDetails(id)
      if (!details) {
        set({ error: 'Planet not found', selectedPlanet: null })
        return
      }
      set({ selectedPlanet: details })
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to load planet details', selectedPlanet: null })
    } finally {
      set({ isLoadingDetails: false })
    }
  },
}))

