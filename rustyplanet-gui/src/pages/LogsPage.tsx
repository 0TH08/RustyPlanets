import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  Typography,
} from '@mui/material'
import { useEffect, useMemo } from 'react'

import { useLogStore } from '../store/logStore'

type Mode = 'player' | 'debug'

interface LogsPageProps {
  mode: Mode
}

export function LogsPage({ mode }: LogsPageProps) {
  const { logs, levelFilter, isStreaming, startMockStream, stopMockStream, setLevelFilter } = useLogStore()

  useEffect(() => {
    // Автоматически запускаем мок‑стрим при открытии вкладки
    if (!isStreaming) {
      startMockStream()
    }
    return () => {
      // Не останавливаем при уходе со страницы, чтобы логи могли продолжать течь,
      // но можем это поменять позже.
    }
  }, [isStreaming, startMockStream])

  const visibleLogs = useMemo(
    () => (levelFilter === 'all' ? logs : logs.filter((l) => l.level === levelFilter)),
    [levelFilter, logs],
  )

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Logs
        </Typography>
        <Stack direction="row" spacing={2} alignItems="center" sx={{ mb: 2 }}>
          <FormControl size="small">
            <InputLabel id="log-level-label">Level</InputLabel>
            <Select
              labelId="log-level-label"
              label="Level"
              value={levelFilter}
              onChange={(e) => setLevelFilter(e.target.value as typeof levelFilter)}
            >
              <MenuItem value="all">All</MenuItem>
              <MenuItem value="debug">Debug</MenuItem>
              <MenuItem value="info">Info</MenuItem>
              <MenuItem value="warn">Warn</MenuItem>
              <MenuItem value="error">Error</MenuItem>
            </Select>
          </FormControl>
          <Button
            size="small"
            variant={isStreaming ? 'outlined' : 'contained'}
            color={isStreaming ? 'inherit' : 'primary'}
            onClick={isStreaming ? stopMockStream : startMockStream}
          >
            {isStreaming ? 'Pause stream' : 'Start stream'}
          </Button>
          {mode !== 'debug' && (
            <Typography variant="body2" color="warning.main">
              In player mode, very low-level logs may be hidden in the future.
            </Typography>
          )}
        </Stack>
        <Box
          component="div"
          sx={{
            mt: 1,
            p: 1,
            bgcolor: 'background.default',
            borderRadius: 1,
            fontFamily: 'monospace',
            fontSize: 12,
            maxHeight: 360,
            overflowY: 'auto',
          }}
        >
          {visibleLogs.map((log) => (
            <Box key={log.id} sx={{ mb: 0.5 }}>
              <Typography component="span" sx={{ color: 'text.disabled', mr: 1 }}>
                {new Date(log.timestamp).toLocaleTimeString()}
              </Typography>
              <Chip
                size="small"
                label={log.level.toUpperCase()}
                color={
                  log.level === 'error'
                    ? 'error'
                    : log.level === 'warn'
                      ? 'warning'
                      : log.level === 'info'
                        ? 'primary'
                        : 'default'
                }
                sx={{ mr: 1, height: 18, '& .MuiChip-label': { px: 0.5 } }}
              />
              {log.planetId != null && (
                <Typography component="span" sx={{ color: 'text.secondary', mr: 1 }}>
                  [planet {log.planetId}]
                </Typography>
              )}
              <Typography component="span" sx={{ color: 'text.primary' }}>
                {log.message}
              </Typography>
            </Box>
          ))}
          {!visibleLogs.length && (
            <Typography variant="body2" color="text.secondary">
              No logs yet. The mock stream will start producing entries shortly.
            </Typography>
          )}
        </Box>
      </CardContent>
    </Card>
  )
}

