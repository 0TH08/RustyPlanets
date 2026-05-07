import { useState } from 'react'
import {
  AppBar,
  Box,
  Container,
  FormControlLabel,
  Switch,
  Tab,
  Tabs,
  Toolbar,
  Typography,
} from '@mui/material'

import { LogsPage } from './LogsPage'
import { OverviewPage } from './OverviewPage'
import { PlanetPage } from './PlanetPage'
import { SimulationPage } from './SimulationPage'
import { VisualizationPage } from './VisualizationPage'

type Mode = 'player' | 'debug'

type TabKey = 'overview' | 'planet' | 'simulation' | 'logs' | 'visualization'

const TAB_ORDER: TabKey[] = ['overview', 'planet', 'simulation', 'logs', 'visualization']

export function RootLayout() {
  const [mode, setMode] = useState<Mode>('player')
  const [activeTab, setActiveTab] = useState<TabKey>('overview')

  const handleChangeTab = (_: React.SyntheticEvent, value: string) => {
    if (TAB_ORDER.includes(value as TabKey)) {
      setActiveTab(value as TabKey)
    }
  }

  const handleToggleMode = (_: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
    setMode(checked ? 'debug' : 'player')
  }

  const renderTabContent = () => {
    switch (activeTab) {
      case 'overview':
        return <OverviewPage mode={mode} />
      case 'planet':
        return <PlanetPage mode={mode} />
      case 'simulation':
        return <SimulationPage mode={mode} />
      case 'logs':
        return <LogsPage mode={mode} />
      case 'visualization':
        return <VisualizationPage mode={mode} />
      default:
        return null
    }
  }

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <AppBar position="static" color="transparent" elevation={1}>
        <Toolbar>
          <Typography variant="h6" sx={{ flexGrow: 1 }}>
            RustyPlanet GUI
          </Typography>
          <FormControlLabel
            control={<Switch color="primary" checked={mode === 'debug'} onChange={handleToggleMode} />}
            label={mode === 'debug' ? 'Debug mode' : 'Player mode'}
          />
        </Toolbar>
        <Tabs
          value={activeTab}
          onChange={handleChangeTab}
          centered
          sx={{minWidth: "100vw"}}
        >
          <Tab label="Overview" value="overview" />
          <Tab label="Planet" value="planet" />
          <Tab label="Simulation" value="simulation" />
          <Tab label="Logs" value="logs" />
          <Tab label="Visualization" value="visualization" />
        </Tabs>
      </AppBar>
      <Container sx={{ flexGrow: 1, py: 3 }}>{renderTabContent()}</Container>
    </Box>
  )
}

