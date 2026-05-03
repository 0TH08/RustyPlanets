import { Card, CardContent, Stack, Typography } from '@mui/material'
import { useEffect, useState } from 'react'

type Mode = 'player' | 'debug'

interface VisualizationPageProps {
  mode: Mode
}

interface OrbitObject {
  id: string
  radius: number
  speed: number
  color: string
  label: string
}

const MOCK_OBJECTS: OrbitObject[] = [
  { id: 'planet-1', radius: 40, speed: 0.5, color: '#90caf9', label: 'Skycartel Alpha' },
  { id: 'planet-2', radius: 70, speed: 0.3, color: '#ffb74d', label: 'Luna4 Prime' },
  { id: 'planet-3', radius: 55, speed: 0.4, color: '#81c784', label: 'Black Adidas Shoe' },
  { id: 'planet-4', radius: 85, speed: 0.2, color: '#ce93d8', label: 'Immutable Cosmic Borrow' },
  { id: 'planet-5', radius: 65, speed: 0.6, color: '#ffcc02', label: 'Rust-Eze' },
  { id: 'planet-6', radius: 50, speed: 0.7, color: '#ef5350', label: 'Crabtorio' },
  { id: 'planet-7', radius: 75, speed: 0.35, color: '#26c6da', label: 'Orbitron' },
  { id: 'asteroid', radius: 95, speed: 0.9, color: '#e57373', label: 'Asteroid stream' },
]

export function VisualizationPage({ mode }: VisualizationPageProps) {
  const [time, setTime] = useState(0)

  useEffect(() => {
    let frameId: number
    const start = performance.now()

    const loop = (now: number) => {
      const dt = (now - start) / 1000
      setTime(dt)
      frameId = requestAnimationFrame(loop)
    }

    frameId = requestAnimationFrame(loop)
    return () => cancelAnimationFrame(frameId)
  }, [])

  const size = 320
  const center = size / 2

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Trajectory visualization
        </Typography>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={3} alignItems="stretch" sx={{ mt: 2 }}>
          <svg
            width={size}
            height={size}
            viewBox={`0 0 ${size} ${size}`}
            style={{ borderRadius: 8, background: '#050712', flexShrink: 0 }}
          >
            {/* Central star */}
            <defs>
              <radialGradient id="starGradient" cx="50%" cy="50%" r="50%">
                <stop offset="0%" stopColor="#fff9c4" />
                <stop offset="100%" stopColor="#fbc02d" />
              </radialGradient>
            </defs>
            <circle cx={center} cy={center} r={10} fill="url(#starGradient)" />

            {/* Orbits */}
            {MOCK_OBJECTS.map((obj) => (
              <circle
                key={`${obj.id}-orbit`}
                cx={center}
                cy={center}
                r={obj.radius}
                fill="none"
                stroke="#424242"
                strokeDasharray="4 4"
                strokeWidth={0.5}
              />
            ))}

            {/* Objects */}
            {MOCK_OBJECTS.map((obj) => {
              const angle = time * obj.speed
              const x = center + obj.radius * Math.cos(angle)
              const y = center + obj.radius * Math.sin(angle)
              return (
                <g key={obj.id}>
                  <circle cx={x} cy={y} r={5} fill={obj.color} />
                  <text
                    x={x + 8}
                    y={y - 4}
                    fill="#e0e0e0"
                    fontSize="8"
                    style={{ pointerEvents: 'none' }}
                  >
                    {obj.label}
                  </text>
                </g>
              )
            })}
          </svg>

          <Stack spacing={1} flex={1}>
            <Typography variant="body2" color="text.secondary">
              This panel shows a simplified orbital view with mocked trajectories for planets and an asteroid stream.
              Later it can be driven by real simulation data (positions, velocities, orbits).
            </Typography>
            {mode === 'debug' && (
              <Typography variant="body2" color="text.secondary">
                In debug mode we can extend this view with extra overlays: collision zones, thrust vectors, energy cell
                usage, or event markers along the paths.
              </Typography>
            )}
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  )
}

