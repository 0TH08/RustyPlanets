import { create } from 'zustand'

import type { SimulationRunState, SimulationStatus } from '../types/simulation'

interface SimulationState extends SimulationStatus {
  isBusy: boolean
  setRunState: (runState: SimulationRunState) => void
  setSpeed: (speed: number) => void
  setTick: (tick: number) => void
  setBusy: (busy: boolean) => void
}

export const useSimulationStore = create<SimulationState>((set) => ({
  runState: 'idle',
  tick: 0,
  speed: 1,
  isBusy: false,
  setRunState: (runState) => set({ runState }),
  setSpeed: (speed) => set({ speed }),
  setTick: (tick) => set({ tick }),
  setBusy: (busy) => set({ isBusy: busy }),
}))