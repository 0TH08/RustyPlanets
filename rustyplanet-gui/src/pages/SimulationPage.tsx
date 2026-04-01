import { Box, Button, Card, CardActions, CardContent, Slider, Stack, Typography } from '@mui/material'
import { useEffect, useState } from 'react'

import {
  getSimulationStatus,
  setSimulationRunState,
  setSimulationSpeed,
  stepSimulation,
} from '../services/simulationService'
import { useSimulationStore } from '../store/simulationStore'

type Mode = 'player' | 'debug'

interface SimulationPageProps {
  mode: Mode
}

export function SimulationPage({ mode }: SimulationPageProps) {
  const { runState, tick, speed, isBusy, setRunState, setSpeed, incrementTick, setBusy } = useSimulationStore()
  const [localSpeed, setLocalSpeed] = useState(speed)

  useEffect(() => {
    // Initial load from backend (mocked for now)
    ;(async () => {
      setBusy(true)
      try {
        const status = await getSimulationStatus()
        setRunState(status.runState)
        setSpeed(status.speed)
      } finally {
        setBusy(false)
      }
    })()
  }, [setBusy, setRunState, setSpeed])

  const handleStart = async () => {
    setBusy(true)
    try {
      const status = await setSimulationRunState('running')
      setRunState(status.runState)
    } finally {
      setBusy(false)
    }
  }

  const handlePause = async () => {
    setBusy(true)
    try {
      const status = await setSimulationRunState('paused')
      setRunState(status.runState)
    } finally {
      setBusy(false)
    }
  }

  const handleStep = async () => {
    setBusy(true)
    try {
      const status = await stepSimulation()
      incrementTick()
      setRunState(status.runState)
    } finally {
      setBusy(false)
    }
  }

  const handleSpeedCommit = async (_: unknown, value: number | number[]) => {
    const v = Array.isArray(value) ? value[0] : value
    setLocalSpeed(v)
    setBusy(true)
    try {
      const status = await setSimulationSpeed(v)
      setSpeed(status.speed)
    } finally {
      setBusy(false)
    }
  }

  return (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Simulation controls
        </Typography>
        <Stack spacing={2}>
          <Box>
            <Typography variant="body2" color="text.secondary">
              Control starting, pausing and stepping the simulation.
            </Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              State: <strong>{runState}</strong> · Tick: <strong>{tick}</strong>
            </Typography>
          </Box>
          <Box display="flex" gap={1}>
            <Button variant="contained" color="primary" onClick={handleStart} disabled={isBusy || runState === 'running'}>
              Start
            </Button>
            <Button variant="outlined" color="primary" onClick={handlePause} disabled={isBusy || runState !== 'running'}>
              Pause
            </Button>
            <Button variant="outlined" color="secondary" onClick={handleStep} disabled={isBusy}>
              Step
            </Button>
          </Box>
          <Box>
            <Typography gutterBottom>Speed</Typography>
            <Slider
              min={0.1}
              max={5}
              step={0.1}
              value={localSpeed}
              onChange={(_, value) => setLocalSpeed(Array.isArray(value) ? value[0] : value)}
              onChangeCommitted={handleSpeedCommit}
            />
            <Typography variant="body2" color="text.secondary">
              Current speed: <strong>{speed.toFixed(1)}x</strong>
            </Typography>
          </Box>
          {mode === 'debug' && (
            <Box>
              <Typography variant="body2" color="text.secondary">
                In debug mode, additional controls (seed, forced events, etc.) can be added here.
              </Typography>
            </Box>
          )}
        </Stack>
      </CardContent>
      <CardActions />
    </Card>
  )
}

