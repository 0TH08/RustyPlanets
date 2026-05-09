import { Box, Button, Card, CardActions, CardContent, Slider, Stack, Typography } from '@mui/material'
import { useEffect, useState } from 'react'

import {
  getSimulationStatus,
  setSimulationRunState,
  setSimulationSpeed,
  stepSimulation,
  resetSimulation,
} from '../services/simulationService'
import { useSimulationStore } from '../store/simulationStore'

type Mode = 'player' | 'debug'

interface SimulationPageProps {
  mode: Mode
}

export function SimulationPage({ mode }: SimulationPageProps) {
  const { runState, tick, speed, isBusy, setRunState, setSpeed, setTick, setBusy } = useSimulationStore()
  const [localSpeed, setLocalSpeed] = useState(speed)

  useEffect(() => {
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

  const handleReset = async () => {
    setBusy(true)
    try {
      const status = await resetSimulation()
      setRunState(status.runState)
      setSpeed(status.speed)
      setTick(0)
    } finally {
      setBusy(false)
    }
  }

  const handleIdle = async () => {
    setBusy(true)
    try {
      const status = await setSimulationRunState('idle')
      setRunState(status.runState)
      setTick(0)
    } finally {
      setBusy(false)
    }
  }

  return (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Simulation Controls
        </Typography>
        <Stack spacing={2}>
          <Box>
            <Typography variant="body2" color="text.secondary">
              Control the simulation state, speed, and tick progression.
            </Typography>
            <Typography variant="body2" sx={{ mt: 1 }}>
              State: <strong>{runState}</strong> · Tick: <strong>{tick}</strong>
            </Typography>
          </Box>
          <Box display="flex" gap={1} flexWrap="wrap">
            <Button variant="contained" color="primary" onClick={handleStart} disabled={isBusy || runState === 'running'}>
              ▶ Start
            </Button>
            <Button variant="outlined" color="primary" onClick={handlePause} disabled={isBusy || runState !== 'running'}>
              ⏸ Pause
            </Button>
            <Button variant="outlined" color="secondary" onClick={handleStep} disabled={isBusy}>
              ⏭ Step
            </Button>
            <Button variant="outlined" color="warning" onClick={handleReset} disabled={isBusy}>
              🔄 Reset
            </Button>
            <Button variant="text" color="error" onClick={handleIdle} disabled={isBusy}>
              ⏹ Stop
            </Button>
          </Box>
          <Box>
            <Typography gutterBottom>Speed ({speed.toFixed(1)}x)</Typography>
            <Slider
              min={0.1}
              max={5}
              step={0.1}
              value={localSpeed}
              onChange={(_, value) => setLocalSpeed(Array.isArray(value) ? value[0] : value)}
              onChangeCommitted={handleSpeedCommit}
            />
          </Box>
          {mode === 'debug' && (
            <Box>
              <Typography variant="body2" color="text.secondary">
                Debug mode: Additional controls for seed, forced events, etc. can be added here.
              </Typography>
            </Box>
          )}
        </Stack>
      </CardContent>
      <CardActions />
    </Card>
  )
}