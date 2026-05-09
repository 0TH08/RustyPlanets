import { Button, Card, CardContent, Chip, CircularProgress, Divider, Stack, Typography } from '@mui/material'
import { useEffect } from 'react'

import { EnergyCellsView } from '../components/EnergyCellsView'
import { usePlanetStore } from '../store/planetStore'
import { startPlanet, stopPlanet } from '../services/planetService'
import { sendSunray, sendAsteroid } from '../services/simulationService'

type Mode = 'player' | 'debug'

interface PlanetPageProps {
  mode?: Mode
}

export function PlanetPage({ }: PlanetPageProps) {
  const { selectedPlanet, selectedPlanetId, isLoadingDetails, planets, selectPlanet, loadPlanets } = usePlanetStore()

  useEffect(() => {
    if (!planets.length) {
      void loadPlanets()
    }
  }, [loadPlanets, planets.length])

  const handleStart = async () => {
    if (selectedPlanetId) {
      await startPlanet(selectedPlanetId)
      await selectPlanet(selectedPlanetId)
    }
  }

  const handleStop = async () => {
    if (selectedPlanetId) {
      await stopPlanet(selectedPlanetId)
      await selectPlanet(selectedPlanetId)
    }
  }

  const handleSunray = async () => {
    if (selectedPlanetId) {
      await sendSunray(selectedPlanetId)
    }
  }

  const handleAsteroid = async () => {
    if (selectedPlanetId) {
      await sendAsteroid(selectedPlanetId)
    }
  }

  const p = selectedPlanet

  return (
    <Stack spacing={2}>
      <Card>
        <CardContent>
          <Typography variant="h6">Planet Details</Typography>
          {isLoadingDetails && (
            <Stack direction="row" spacing={1} alignItems="center" sx={{ mt: 1 }}>
              <CircularProgress size={16} />
              <Typography variant="body2" color="text.secondary">
                Loading...
              </Typography>
            </Stack>
          )}
          {!isLoadingDetails && !p && (
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {selectedPlanetId == null ? 'Select a planet from the Overview tab.' : 'Planet details not available.'}
            </Typography>
          )}
          {p && (
            <Stack spacing={1} sx={{ mt: 1 }}>
              <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
                <Typography variant="h5" component="span">
                  {p.summary.name}
                </Typography>
                <Chip size="small" label={p.summary.kind} color="primary" variant="outlined" />
                <Chip 
                  size="small" 
                  label={p.summary.aiRunning ? 'AI Running' : 'AI Stopped'} 
                  color={p.summary.aiRunning ? 'success' : 'default'} 
                />
              </Stack>
              <Typography variant="body2" color="text.secondary">
                ID: {p.summary.id} · Type: {p.summary.kind}
              </Typography>
              <Divider sx={{ my: 1 }} />
              
              <Stack direction="row" spacing={1} flexWrap="wrap">
                <Button variant="contained" color="primary" size="small" onClick={handleStart}>
                  Start AI
                </Button>
                <Button variant="outlined" color="warning" size="small" onClick={handleStop}>
                  Stop AI
                </Button>
                <Button variant="outlined" color="info" size="small" onClick={handleSunray}>
                  ☀️ Send Sunray
                </Button>
                <Button variant="outlined" color="error" size="small" onClick={handleAsteroid}>
                  ☄️ Send Asteroid
                </Button>
              </Stack>
            </Stack>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <Typography variant="h6">Energy Cells</Typography>
          {p ? (
            <EnergyCellsView
              total={p.cells.length}
              charged={p.cells.filter(c => c.charged).length}
              resourceTotal={Math.min(3, p.cells.length)}
              resourceCharged={p.cells.slice(0, Math.min(3, p.cells.length)).filter(c => c.charged).length}
              defenseTotal={Math.max(0, p.cells.length - 3)}
              defenseCharged={p.cells.slice(3).filter(c => c.charged).length}
            />
          ) : (
            <Typography variant="body2" color="text.secondary">
              Select a planet to view energy cells.
            </Typography>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <Typography variant="h6">Statistics</Typography>
          {p ? (
            <Stack spacing={0.5}>
              <Typography variant="body2">
                Explorers present: <strong>{p.summary.explorerCount}</strong>
              </Typography>
              <Typography variant="body2">
                Resources generated: <strong>{p.summary.totalResourcesGenerated}</strong>
              </Typography>
              <Typography variant="body2">
                Rockets built: <strong>{p.summary.rocketsBuilt}</strong>
              </Typography>
              <Typography variant="body2">
                Asteroids deflected: <strong>{p.summary.asteroidsDeflected}</strong>
              </Typography>
              <Typography variant="body2">
                Errors: <strong>{p.summary.errorsEncountered}</strong>
              </Typography>
              <Divider sx={{ my: 1 }} />
              <Typography variant="body2">
                Explorer arrivals: <strong>{p.explorerArrivals}</strong>
              </Typography>
              <Typography variant="body2">
                Explorer departures: <strong>{p.explorerDepartures}</strong>
              </Typography>
            </Stack>
          ) : (
            <Typography variant="body2" color="text.secondary">
              Select a planet to view statistics.
            </Typography>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <Typography variant="h6">Generation History</Typography>
          {p && p.generationHistory.length > 0 ? (
            <Stack spacing={0.5} sx={{ maxHeight: 200, overflow: 'auto' }}>
              {p.generationHistory.slice(-10).reverse().map((entry, i) => (
                <Typography key={i} variant="body2" color="text.secondary">
                  {entry.resource}
                </Typography>
              ))}
            </Stack>
          ) : (
            <Typography variant="body2" color="text.secondary">
              {p ? 'No resources generated yet.' : 'Select a planet to view history.'}
            </Typography>
          )}
        </CardContent>
      </Card>
    </Stack>
  )
}