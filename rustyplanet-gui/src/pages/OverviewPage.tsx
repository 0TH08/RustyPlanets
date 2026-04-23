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
          <Typography variant="h6">Simulation status</Typography>
          <Stack spacing={0.5} sx={{ mt: 1 }}>
            <Typography variant="body2" color="text.secondary">
              State: <strong>{runState}</strong>
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Tick: <strong>{tick}</strong>
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Speed: <strong>{speed.toFixed(1)}x</strong>
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Planets: <strong>{planets.length}</strong>
            </Typography>
          </Stack>
        </CardContent>
      </Card>
      <Card>
        <CardContent>
          <Typography variant="h6">Planets</Typography>
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
                        <Chip size="small" label={planet.typeLabel} />
                      </Stack>
                    }
                    secondary={`State: ${planet.runState} · Energy: ${planet.energyCellsCharged}/${planet.energyCellsTotal} · Errors: ${planet.errors}`}
                  />
                </ListItemButton>
              ))}
              {!planets.length && (
                <Typography variant="body2" color="text.secondary">
                  No planets available yet.
                </Typography>
              )}
            </List>
          )}
        </CardContent>
      </Card>
      {mode === 'debug' && (
        <Card>
          <CardContent>
            <Typography variant="h6">Debug metrics</Typography>
            <Typography variant="body2" color="text.secondary">
              In debug mode, internal engine metrics and diagnostics can be shown here.
            </Typography>
          </CardContent>
        </Card>
      )}
    </Stack>
  )
}

