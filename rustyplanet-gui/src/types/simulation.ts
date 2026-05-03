export type SimulationRunState = 'idle' | 'running' | 'paused'

export interface SimulationStatus {
  runState: SimulationRunState
  tick: number
  speed: number
}

