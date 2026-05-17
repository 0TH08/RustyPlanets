import { create } from 'zustand'

import type { LogEntry, LogLevel } from '../types/logs'

const WS_URL = 'ws://localhost:8080/ws/logs'

interface BackendLogEntry {
  timestamp: string
  level: string
  target: string
  message: string
}

interface LogState {
  logs: LogEntry[]
  levelFilter: LogLevel | 'all'
  isStreaming: boolean
  startStream: () => void
  stopStream: () => void
  setLevelFilter: (level: LogState['levelFilter']) => void
}

let ws: WebSocket | null = null

export const useLogStore = create<LogState>((set) => ({
  logs: [],
  levelFilter: 'all',
  isStreaming: false,

  startStream: () => {
    if (ws && ws.readyState === WebSocket.OPEN) return
    ws = new WebSocket(WS_URL)
    set({ isStreaming: true })

    ws.onmessage = (event) => {
      try {
        const raw: BackendLogEntry = JSON.parse(event.data as string)
        const entry: LogEntry = {
          id: `${raw.timestamp}-${Math.random().toString(36).slice(2, 6)}`,
          timestamp: raw.timestamp,
          level: raw.level.toLowerCase() as LogLevel,
          source: raw.target,
          message: raw.message,
        }
        set((state) => ({ logs: [...state.logs.slice(-4999), entry] }))
      } catch {
        // ignore malformed frames
      }
    }

    ws.onclose = () => {
      ws = null
      set({ isStreaming: false })
    }

    ws.onerror = () => {
      ws?.close()
    }
  },

  stopStream: () => {
    ws?.close()
    ws = null
    set({ isStreaming: false })
  },

  setLevelFilter: (levelFilter) => set({ levelFilter }),
}))
