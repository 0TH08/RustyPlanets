import {
  Box,
  Card,
  Chip,
  Stack,
  Typography,
  styled,
} from "@mui/material";

import { useEffect, useState } from "react";

type Mode = "player" | "debug";

interface VisualizationPageProps {
  mode: Mode;
}

interface OrbitObject {
  id: string;
  radius: number;
  speed: number;
  color: string;
  label: string;
}

const MOCK_OBJECTS: OrbitObject[] = [
  {
    id: "planet-1",
    radius: 40,
    speed: 0.5,
    color: "#90caf9",
    label: "Skycartel Alpha",
  },
  {
    id: "planet-2",
    radius: 70,
    speed: 0.3,
    color: "#ffb74d",
    label: "Luna4 Prime",
  },
  {
    id: "planet-3",
    radius: 55,
    speed: 0.4,
    color: "#81c784",
    label: "Black Adidas Shoe",
  },
  {
    id: "planet-4",
    radius: 85,
    speed: 0.2,
    color: "#ce93d8",
    label: "Immutable Cosmic Borrow",
  },
  {
    id: "planet-5",
    radius: 65,
    speed: 0.6,
    color: "#ffcc02",
    label: "Rust-Eze",
  },
  {
    id: "planet-6",
    radius: 50,
    speed: 0.7,
    color: "#ef5350",
    label: "Crabtorio",
  },
  {
    id: "planet-7",
    radius: 75,
    speed: 0.35,
    color: "#26c6da",
    label: "Orbitron",
  },
  {
    id: "asteroid",
    radius: 95,
    speed: 0.9,
    color: "#e57373",
    label: "Asteroid Stream",
  },
];

const StyledCard = styled(Card)({
  background: "linear-gradient(145deg, #0f0f0f 0%, #050505 100%)",
  border: "1px solid rgba(255,255,255,0.08)",
  borderRadius: "16px",
  padding: "24px",
  position: "relative",
  overflow: "hidden",

  "&::before": {
    content: '""',
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    height: "1px",
    background:
      "linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.1) 50%, transparent 100%)",
  },
});

export function VisualizationPage({
  mode,
}: VisualizationPageProps) {
  const [time, setTime] = useState(0);

  useEffect(() => {
    let frameId: number;

    const start = performance.now();

    const loop = (now: number) => {
      const dt = (now - start) / 1000;

      setTime(dt);

      frameId =
        requestAnimationFrame(loop);
    };

    frameId = requestAnimationFrame(loop);

    return () =>
      cancelAnimationFrame(frameId);
  }, []);

  const size = 340;
  const center = size / 2;

  return (
    <StyledCard sx={{ height: "100%" }}>
      {/* Header */}
      <Box
        display="flex"
        alignItems="center"
        justifyContent="space-between"
        mb={2}
      >
        <Typography
          sx={{
            fontSize: "14px",
            fontWeight: 500,
            color: "#9ca3af",
            textTransform: "uppercase",
            letterSpacing: "0.1em",
          }}
        >
          Trajectory Visualization
        </Typography>

        <Chip
          label="LIVE ORBITS"
          size="small"
          sx={{
            background:
              "rgba(99,102,241,0.12)",

            color: "#818cf8",

            border:
              "1px solid rgba(99,102,241,0.25)",

            fontSize: "11px",
            fontWeight: 700,
            letterSpacing: "0.05em",
          }}
        />
      </Box>

      <Stack
        direction={{
          xs: "column",
          md: "row",
        }}
        spacing={3}
        alignItems="stretch"
      >
        {/* Orbit Visualization */}
        <Box
          sx={{
            p: 2,
            borderRadius: "16px",
            background:
              "linear-gradient(180deg, #050712 0%, #02030a 100%)",

            border:
              "1px solid rgba(255,255,255,0.06)",

            flexShrink: 0,

            display: "flex",
            alignItems: "center",
            justifyContent: "center",

            boxShadow:
              "inset 0 0 80px rgba(99,102,241,0.06)",
          }}
        >
          <svg
            width={size}
            height={size}
            viewBox={`0 0 ${size} ${size}`}
          >
            {/* Star Glow */}
            <defs>
              <radialGradient
                id="starGradient"
                cx="50%"
                cy="50%"
                r="50%"
              >
                <stop
                  offset="0%"
                  stopColor="#fff9c4"
                />

                <stop
                  offset="100%"
                  stopColor="#fbc02d"
                />
              </radialGradient>
            </defs>

            {/* Central Star */}
            <circle
              cx={center}
              cy={center}
              r={12}
              fill="url(#starGradient)"
            />

            {/* Glow */}
            <circle
              cx={center}
              cy={center}
              r={18}
              fill="rgba(251,192,45,0.15)"
            />

            {/* Orbit Rings */}
            {MOCK_OBJECTS.map((obj) => (
              <circle
                key={`${obj.id}-orbit`}
                cx={center}
                cy={center}
                r={obj.radius}
                fill="none"
                stroke="rgba(255,255,255,0.12)"
                strokeDasharray="4 4"
                strokeWidth={0.7}
              />
            ))}

            {/* Orbiting Objects */}
            {MOCK_OBJECTS.map((obj) => {
              const angle =
                time * obj.speed;

              const x =
                center +
                obj.radius *
                  Math.cos(angle);

              const y =
                center +
                obj.radius *
                  Math.sin(angle);

              return (
                <g key={obj.id}>
                  {/* Glow */}
                  <circle
                    cx={x}
                    cy={y}
                    r={8}
                    fill={obj.color}
                    opacity={0.2}
                  />

                  {/* Planet */}
                  <circle
                    cx={x}
                    cy={y}
                    r={5}
                    fill={obj.color}
                  />

                  {/* Label */}
                  <text
                    x={x + 10}
                    y={y - 6}
                    fill="#d1d5db"
                    fontSize="8"
                    style={{
                      pointerEvents:
                        "none",
                    }}
                  >
                    {obj.label}
                  </text>
                </g>
              );
            })}
          </svg>
        </Box>

        {/* Side Panel */}
        <Stack spacing={2} flex={1}>
          <Box
            sx={{
              p: 2,
              borderRadius: "12px",
              background:
                "rgba(255,255,255,0.02)",

              border:
                "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <Typography
              sx={{
                fontSize: "13px",
                color: "#9ca3af",
                lineHeight: 1.8,
              }}
            >
              This panel displays a live
              orbital visualization with
              mocked trajectories for
              planets and asteroid streams.
              The animation can later be
              connected directly to real
              simulation coordinates,
              velocities, and orbital
              physics.
            </Typography>
          </Box>

          {/* Legend */}
          <Box
            sx={{
              p: 2,
              borderRadius: "12px",
              background:
                "rgba(255,255,255,0.02)",

              border:
                "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <Typography
              sx={{
                fontSize: "13px",
                fontWeight: 600,
                color: "#fff",
                mb: 1.5,
              }}
            >
              Active Objects
            </Typography>

            <Stack spacing={1}>
              {MOCK_OBJECTS.map((obj) => (
                <Box
                  key={obj.id}
                  display="flex"
                  alignItems="center"
                  justifyContent="space-between"
                  sx={{
                    p: 1,
                    borderRadius: "10px",
                    background:
                      "rgba(255,255,255,0.015)",

                    border:
                      "1px solid rgba(255,255,255,0.04)",
                  }}
                >
                  <Stack
                    direction="row"
                    spacing={1}
                    alignItems="center"
                  >
                    <Box
                      sx={{
                        width: 10,
                        height: 10,
                        borderRadius: "50%",
                        background:
                          obj.color,

                        boxShadow: `0 0 12px ${obj.color}`,
                      }}
                    />

                    <Typography
                      sx={{
                        fontSize: "12px",
                        color: "#e5e7eb",
                      }}
                    >
                      {obj.label}
                    </Typography>
                  </Stack>

                  <Typography
                    sx={{
                      fontSize: "11px",
                      color: "#6b7280",
                      fontFamily:
                        "monospace",
                    }}
                  >
                    r={obj.radius}
                  </Typography>
                </Box>
              ))}
            </Stack>
          </Box>

          {/* Debug Panel */}
          {mode === "debug" && (
            <Box
              sx={{
                p: 2,
                borderRadius: "12px",
                background:
                  "rgba(239,68,68,0.04)",

                border:
                  "1px solid rgba(239,68,68,0.12)",
              }}
            >
              <Typography
                sx={{
                  fontSize: "13px",
                  color: "#f87171",
                  lineHeight: 1.7,
                }}
              >
                Debug mode enabled.
                Additional overlays such as
                collision zones, thrust
                vectors, energy cell usage,
                gravity wells, and event
                markers can be rendered
                here.
              </Typography>
            </Box>
          )}
        </Stack>
      </Stack>
    </StyledCard>
  );
}