import { create } from 'zustand'

import type { LogEntry, LogLevel } from '../types/logs'

interface LogState {
  logs: LogEntry[]
  levelFilter: LogLevel | 'all'
  isStreaming: boolean
  startMockStream: () => void
  stopMockStream: () => void
  setLevelFilter: (level: LogState['levelFilter']) => void
}

let intervalId: number | null = null

export const useLogStore = create<LogState>((set) => ({
  logs: [],
  levelFilter: 'all',
  isStreaming: false,
  startMockStream: () => {
    if (intervalId !== null) return
    set({ isStreaming: true })
    intervalId = window.setInterval(() => {
      const now = new Date()
      const levels: LogLevel[] = ['debug', 'info', 'warn', 'error']
      const level = levels[Math.floor(Math.random() * levels.length)]
      const id = `${now.getTime()}-${Math.random().toString(36).slice(2, 8)}`
      const entry: LogEntry = {
        id,
        timestamp: now.toISOString(),
        level,
        source: 'mock-engine',
        planetId: Math.random() > 0.5 ? 1 : undefined,
        message: `Sample ${level} message at ${now.toLocaleTimeString()}`,
      }
      set((state) => ({
        logs: [...state.logs.slice(-199), entry],
      }))
    }, 1000)
  },
  stopMockStream: () => {
    if (intervalId !== null) {
      window.clearInterval(intervalId)
      intervalId = null
    }
    set({ isStreaming: false })
  },
  setLevelFilter: (levelFilter) => set({ levelFilter }),
}))

