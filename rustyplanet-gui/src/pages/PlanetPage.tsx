import { Card, CardContent, Chip, CircularProgress, Divider, Stack, Typography } from '@mui/material'
import { useEffect } from 'react'

import { EnergyCellsView } from '../components/EnergyCellsView'
import { usePlanetStore } from '../store/planetStore'
import type { PlanetDetails } from '../types/planet'

type Mode = 'player' | 'debug'

interface PlanetPageProps {
  mode: Mode
}

export function PlanetPage({ mode }: PlanetPageProps) {
  const { selectedPlanet, selectedPlanetId, isLoadingDetails, planets, loadPlanets } = usePlanetStore()

  useEffect(() => {
    if (!planets.length) {
      void loadPlanets()
    }
  }, [loadPlanets, planets.length])

  const renderSkycartelStats = (planet: PlanetDetails) => {
    if (!planet.skycartelStats) return null

    const s = planet.skycartelStats
    return (
      <Stack spacing={0.5}>
        <Typography variant="subtitle2">Skycartel stats</Typography>
        <Typography variant="body2">Total resources generated: {s.totalResourcesGenerated}</Typography>
        <Typography variant="body2">Explorer arrivals: {s.explorerArrivals}</Typography>
        <Typography variant="body2">Explorer departures: {s.explorerDepartures}</Typography>
        <Typography variant="body2">Rockets built: {s.rocketsBuilt}</Typography>
        <Typography variant="body2">Asteroids deflected: {s.asteroidsDeflected}</Typography>
        <Typography variant="body2">Errors encountered: {s.errorsEncountered}</Typography>
      </Stack>
    )
  }

  return (
    <Stack spacing={2}>
      <Card>
        <CardContent>
          <Typography variant="h6">Selected planet</Typography>
          {isLoadingDetails && (
            <Stack direction="row" spacing={1} alignItems="center" sx={{ mt: 1 }}>
              <CircularProgress size={16} />
              <Typography variant="body2" color="text.secondary">
                Loading planet details...
              </Typography>
            </Stack>
          )}
          {!isLoadingDetails && !selectedPlanet && (
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              {selectedPlanetId == null ? 'Select a planet from the list on the Overview tab.' : 'Planet details not available.'}
            </Typography>
          )}
          {selectedPlanet && (
            <Stack spacing={1} sx={{ mt: 1 }}>
              <Stack direction="row" spacing={1} alignItems="center">
                <Typography variant="h6" component="span">
                  {selectedPlanet.name}
                </Typography>
                <Chip size="small" label={selectedPlanet.typeLabel} />
              </Stack>
              <Typography variant="body2" color="text.secondary">
                State: {selectedPlanet.runState} · Energy: {selectedPlanet.energyCellsCharged}/
                {selectedPlanet.energyCellsTotal} · Errors: {selectedPlanet.errors}
              </Typography>
              <Divider sx={{ my: 1 }} />
              <Stack spacing={0.5}>
                <Typography variant="subtitle2">Resources</Typography>
                {Object.entries(selectedPlanet.resources).map(([name, value]) => (
                  <Typography key={name} variant="body2">
                    {name}: {value}
                  </Typography>
                ))}
              </Stack>
              {renderSkycartelStats(selectedPlanet)}
            </Stack>
          )}
        </CardContent>
      </Card>
      <Card>
        <CardContent>
          <Typography variant="h6">Energy cells</Typography>
          {selectedPlanet ? (
            <EnergyCellsView
              total={selectedPlanet.energyCellsTotal}
              charged={selectedPlanet.energyCellsCharged}
              resourceTotal={selectedPlanet.resourceCellsTotal}
              resourceCharged={selectedPlanet.resourceCellsCharged}
              defenseTotal={selectedPlanet.defenseCellsTotal}
              defenseCharged={selectedPlanet.defenseCellsCharged}
            />
          ) : (
            <Typography variant="body2" color="text.secondary">
              Select a planet to see its energy cell layout.
            </Typography>
          )}
        </CardContent>
      </Card>
      {mode === 'debug' && (
        <Card>
          <CardContent>
            <Typography variant="h6">Debug details</Typography>
            <Typography variant="body2" color="text.secondary">
              In debug mode, raw planet state and internal diagnostic information can be shown here.
            </Typography>
          </CardContent>
        </Card>
      )}
    </Stack>
  )
}

