import { Card, CardContent, Chip, CircularProgress, List, ListItemButton, ListItemText, Stack, Typography } from '@mui/material'
import { useEffect } from 'react'

import { usePlanetStore } from '../store/planetStore'
import { useSimulationStore } from '../store/simulationStore'

type Mode = 'player' | 'debug'

interface OverviewPageProps {
  mode: Mode
}

export function OverviewPage({ mode }: OverviewPageProps) {
  const { planets, selectedPlanetId, isLoadingList, error, loadPlanets, selectPlanet } = usePlanetStore()
  const { runState, tick, speed } = useSimulationStore()

  useEffect(() => {
    if (!planets.length) {
      void loadPlanets()
    }
  }, [loadPlanets, planets.length])

  return (
    <Stack spacing={2}>
      <Card>
        <CardContent>
          <Typography variant="h6">Simulation Status</Typography>
          <Stack spacing={0.5} sx={{ mt: 1 }}>
            <Typography variant="body2">
              State: <strong>{runState}</strong>
            </Typography>
            <Typography variant="body2">
              Tick: <strong>{tick}</strong>
            </Typography>
            <Typography variant="body2">
              Speed: <strong>{speed.toFixed(1)}x</strong>
            </Typography>
            <Typography variant="body2">
              Planets: <strong>{planets.length}</strong>
            </Typography>
          </Stack>
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <Typography variant="h6">Planets ({planets.length})</Typography>
          {isLoadingList && (
            <Stack direction="row" alignItems="center" spacing={1} sx={{ mt: 1 }}>
              <CircularProgress size={16} />
              <Typography variant="body2" color="text.secondary">
                Loading planets...
              </Typography>
            </Stack>
          )}
          {error && (
            <Typography variant="body2" color="error" sx={{ mt: 1 }}>
              {error}
            </Typography>
          )}
          {!isLoadingList && !error && (
            <List dense>
              {planets.map((planet) => (
                <ListItemButton
                  key={planet.id}
                  selected={planet.id === selectedPlanetId}
                  onClick={() => void selectPlanet(planet.id)}
                >
                  <ListItemText
                    primary={
                      <Stack direction="row" spacing={1} alignItems="center">
                        <span>{planet.name}</span>
                        <Chip size="small" label={planet.kind} color="primary" variant="outlined" />
                        <Chip 
                          size="small" 
                          label={planet.aiRunning ? 'AI Running' : 'AI Stopped'} 
                          color={planet.aiRunning ? 'success' : 'default'} 
                        />
                      </Stack>
                    }
                    secondary={`Explorers: ${planet.explorerCount} · Generated: ${planet.totalResourcesGenerated} · Rockets: ${planet.rocketsBuilt}`}
                  />
                </ListItemButton>
              ))}
              {!planets.length && (
                <Typography variant="body2" color="text.secondary">
                  No planets available.
                </Typography>
              )}
            </List>
          )}
        </CardContent>
      </Card>

      {mode === 'debug' && (
        <Card>
          <CardContent>
            <Typography variant="h6">Debug Metrics</Typography>
            <Typography variant="body2" color="text.secondary">
              Internal engine metrics and diagnostics shown here in debug mode.
            </Typography>
          </CardContent>
        </Card>
      )}
    </Stack>
  )
}