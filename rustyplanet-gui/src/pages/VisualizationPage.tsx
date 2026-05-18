import {
  Box,
  Card,
  Chip,
  Stack,
  Typography,
  styled,
} from "@mui/material";

import { useEffect, useRef, useState } from "react";
import type { PlanetSummary } from "../types/planet";

const API_BASE = "http://localhost:8080/api";

type Mode = "player" | "debug";

interface VisualizationPageProps {
  mode: Mode;
}

interface OrbitParams {
  radius: number;
  speed: number;
  color: string;
}

interface ExplorerPos {
  id: number;
  currentPlanetId: number | null;
}

// Fixed visual params per planet id — kept stable so the animation doesn't jump on refetch
const PLANET_VISUAL: Record<number, OrbitParams> = {
  1: { radius: 40,  speed: 0.5,  color: "#90caf9" },
  2: { radius: 70,  speed: 0.3,  color: "#ffb74d" },
  3: { radius: 55,  speed: 0.4,  color: "#81c784" },
  4: { radius: 85,  speed: 0.2,  color: "#ce93d8" },
  5: { radius: 65,  speed: 0.6,  color: "#ffcc02" },
  6: { radius: 50,  speed: 0.7,  color: "#ef5350" },
  7: { radius: 75,  speed: 0.35, color: "#26c6da" },
  8: { radius: 95,  speed: 0.15, color: "#f472b6" },
};

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

export function VisualizationPage({ mode }: VisualizationPageProps) {
  const [time, setTime] = useState(0);
  const [planets, setPlanets] = useState<PlanetSummary[]>([]);
  const [topology, setTopology] = useState<Record<string, number[]>>({});
  const [explorers, setExplorers] = useState<ExplorerPos[]>([]);
  const startRef = useRef<number | null>(null);

  // Animation loop
  useEffect(() => {
    let frameId: number;
    const loop = (now: number) => {
      if (startRef.current === null) startRef.current = now;
      setTime((now - startRef.current) / 1000);
      frameId = requestAnimationFrame(loop);
    };
    frameId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(frameId);
  }, []);

  // Poll planet list every 3 s
  useEffect(() => {
    const load = () =>
      fetch(`${API_BASE}/planets`)
        .then((r) => r.json())
        .then(setPlanets)
        .catch(() => {});
    load();
    const id = window.setInterval(load, 3000);
    return () => window.clearInterval(id);
  }, []);

  // Fetch topology once
  useEffect(() => {
    fetch(`${API_BASE}/topology`)
      .then((r) => r.json())
      .then(setTopology)
      .catch(() => {});
  }, []);

  // Real-time explorer tracking:
  //   1. Initial REST call → get current positions for all explorers
  //   2. WebSocket log stream → parse "Explorer X moved to planet Y" lines for instant updates
  useEffect(() => {
    // Initial snapshot
    fetch(`${API_BASE}/explorers`)
      .then((r) => r.json())
      .then((data: ExplorerPos[]) => setExplorers(data))
      .catch(() => {});

    // Derive WebSocket URL from API base (http → ws)
    const wsUrl = API_BASE.replace(/^http/, "ws").replace("/api", "") + "/ws/logs";
    const ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
      try {
        const entry = JSON.parse(event.data as string) as { message?: string };
        const msg = entry.message ?? "";
        // Match: "Explorer 2 moved to planet 5 (was on …)"
        const match = msg.match(/^Explorer (\d+) moved to planet (\d+)/);
        if (match) {
          const explorerId = parseInt(match[1], 10);
          const planetId   = parseInt(match[2], 10);
          setExplorers((prev) =>
            prev.map((e) =>
              e.id === explorerId ? { ...e, currentPlanetId: planetId } : e
            )
          );
        }
      } catch (_) {
        // ignore malformed frames
      }
    };

    ws.onerror = () => { /* silent — simulation may not be running */ };

    return () => ws.close();
  }, []);

  const size = 340;
  const center = size / 2;

  // Compute current x/y for each planet
  const positions: Record<number, { x: number; y: number }> = {};
  for (const planet of planets) {
    const vis = PLANET_VISUAL[planet.id];
    if (!vis) continue;
    const angle = time * vis.speed;
    positions[planet.id] = {
      x: center + vis.radius * Math.cos(angle),
      y: center + vis.radius * Math.sin(angle),
    };
  }

  // Build unique edge list from topology (avoid drawing A→B and B→A)
  const edges: [number, number][] = [];
  const seen = new Set<string>();
  for (const [srcStr, neighbors] of Object.entries(topology) as [string, number[]][]) {
    const src = Number(srcStr);
    for (const dst of neighbors) {
      const key = [Math.min(src, dst), Math.max(src, dst)].join("-");
      if (!seen.has(key)) {
        seen.add(key);
        edges.push([src, dst]);
      }
    }
  }

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
            background: "rgba(99,102,241,0.12)",
            color: "#818cf8",
            border: "1px solid rgba(99,102,241,0.25)",
            fontSize: "11px",
            fontWeight: 700,
            letterSpacing: "0.05em",
          }}
        />
      </Box>

      <Stack
        direction={{ xs: "column", md: "row" }}
        spacing={3}
        alignItems="stretch"
      >
        {/* Orbit Visualization */}
        <Box
          sx={{
            p: 2,
            borderRadius: "16px",
            background: "linear-gradient(180deg, #050712 0%, #02030a 100%)",
            border: "1px solid rgba(255,255,255,0.06)",
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "inset 0 0 80px rgba(99,102,241,0.06)",
          }}
        >
          <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
            <defs>
              <radialGradient id="starGradient" cx="50%" cy="50%" r="50%">
                <stop offset="0%" stopColor="#fff9c4" />
                <stop offset="100%" stopColor="#fbc02d" />
              </radialGradient>
              <filter id="explorerGlow" x="-80%" y="-80%" width="260%" height="260%">
                <feGaussianBlur stdDeviation="2.5" result="blur" />
                <feMerge>
                  <feMergeNode in="blur" />
                  <feMergeNode in="SourceGraphic" />
                </feMerge>
              </filter>
            </defs>

            {/* Central star */}
            <circle cx={center} cy={center} r={12} fill="url(#starGradient)" />
            <circle cx={center} cy={center} r={18} fill="rgba(251,192,45,0.15)" />

            {/* Orbit rings */}
            {planets.map((p) => {
              const vis = PLANET_VISUAL[p.id];
              if (!vis) return null;
              return (
                <circle
                  key={`${p.id}-orbit`}
                  cx={center}
                  cy={center}
                  r={vis.radius}
                  fill="none"
                  stroke="rgba(255,255,255,0.12)"
                  strokeDasharray="4 4"
                  strokeWidth={0.7}
                />
              );
            })}

            {/* Topology edges */}
            {edges.map(([src, dst]) => {
              const a = positions[src];
              const b = positions[dst];
              if (!a || !b) return null;
              return (
                <line
                  key={`edge-${src}-${dst}`}
                  x1={a.x} y1={a.y}
                  x2={b.x} y2={b.y}
                  stroke="rgba(99,102,241,0.35)"
                  strokeWidth={0.8}
                  strokeDasharray="3 3"
                />
              );
            })}

            {/* Orbiting planets */}
            {planets.map((p) => {
              const vis = PLANET_VISUAL[p.id];
              const pos = positions[p.id];
              if (!vis || !pos) return null;
              return (
                <g key={p.id}>
                  <circle cx={pos.x} cy={pos.y} r={8} fill={vis.color} opacity={0.2} />
                  <circle cx={pos.x} cy={pos.y} r={5} fill={vis.color} />
                  <text
                    x={pos.x + 10}
                    y={pos.y - 6}
                    fill="#d1d5db"
                    fontSize="8"
                    style={{ pointerEvents: "none" }}
                  >
                    {p.name}
                  </text>
                </g>
              );
            })}

            {/* Explorer spacecraft */}
            {explorers.map((e, i) => {
              if (!e.currentPlanetId) return null;
              const pos = positions[e.currentPlanetId];
              if (!pos) return null;
              // Spread explorers in a ring 20px out from planet center
              const spreadAngle = i * (Math.PI * 2 / Math.max(explorers.length, 1));
              const ex = pos.x + Math.cos(spreadAngle) * 20;
              const ey = pos.y + Math.sin(spreadAngle) * 20;
              const explorerColor = i === 0 ? "#f59e0b" : "#a78bfa";
              return (
                <g key={`explorer-${e.id}`} filter="url(#explorerGlow)">
                  {/* Outer glow halo */}
                  <circle cx={ex} cy={ey} r={9} fill={explorerColor} opacity={0.18} />
                  {/* Mid glow */}
                  <circle cx={ex} cy={ey} r={6} fill={explorerColor} opacity={0.30} />
                  {/* Diamond body */}
                  <polygon
                    points={`${ex},${ey - 7} ${ex + 5},${ey} ${ex},${ey + 7} ${ex - 5},${ey}`}
                    fill={explorerColor}
                    stroke="#fff"
                    strokeWidth={0.8}
                    opacity={1}
                  />
                  {/* Label */}
                  <text
                    x={ex}
                    y={ey + 17}
                    fill={explorerColor}
                    fontSize="8"
                    fontWeight="bold"
                    textAnchor="middle"
                    style={{ pointerEvents: "none" }}
                  >
                    E{e.id}
                  </text>
                </g>
              );
            })}
          </svg>
        </Box>

        {/* Side Panel */}
        <Stack spacing={2} flex={1}>
          {/* Legend */}
          <Box
            sx={{
              p: 2,
              borderRadius: "12px",
              background: "rgba(255,255,255,0.02)",
              border: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <Typography
              sx={{ fontSize: "13px", fontWeight: 600, color: "#fff", mb: 1.5 }}
            >
              Active Planets
            </Typography>

            <Stack spacing={1}>
              {planets.map((p) => {
                const vis = PLANET_VISUAL[p.id];
                if (!vis) return null;
                return (
                  <Box
                    key={p.id}
                    display="flex"
                    alignItems="center"
                    justifyContent="space-between"
                    sx={{
                      p: 1,
                      borderRadius: "10px",
                      background: "rgba(255,255,255,0.015)",
                      border: "1px solid rgba(255,255,255,0.04)",
                    }}
                  >
                    <Stack direction="row" spacing={1} alignItems="center">
                      <Box
                        sx={{
                          width: 10,
                          height: 10,
                          borderRadius: "50%",
                          background: vis.color,
                          boxShadow: `0 0 12px ${vis.color}`,
                        }}
                      />
                      <Typography sx={{ fontSize: "12px", color: "#e5e7eb" }}>
                        {p.name}
                      </Typography>
                    </Stack>
                    <Stack direction="row" spacing={1} alignItems="center">
                      {p.aiRunning && (
                        <Chip
                          label="AI"
                          size="small"
                          sx={{
                            height: 16,
                            fontSize: "10px",
                            background: "rgba(34,197,94,0.15)",
                            color: "#4ade80",
                            border: "1px solid rgba(34,197,94,0.3)",
                          }}
                        />
                      )}
                      <Typography
                        sx={{ fontSize: "11px", color: "#6b7280", fontFamily: "monospace" }}
                      >
                        r={vis.radius}
                      </Typography>
                    </Stack>
                  </Box>
                );
              })}
            </Stack>
          </Box>

          {/* Explorer legend */}
          {explorers.length > 0 && (
            <Box
              sx={{
                p: 2,
                borderRadius: "12px",
                background: "rgba(245,158,11,0.04)",
                border: "1px solid rgba(245,158,11,0.2)",
              }}
            >
              <Typography
                sx={{ fontSize: "12px", color: "#fbbf24", fontWeight: 600, mb: 0.5 }}
              >
                🛸 Explorers ({explorers.length})
              </Typography>
              <Stack spacing={0.5}>
                {explorers.map((e, i) => {
                  const explorerColor = i === 0 ? "#f59e0b" : "#a78bfa";
                  const pName = planets.find((p) => p.id === e.currentPlanetId)?.name ?? "—";
                  return (
                    <Box key={e.id} display="flex" alignItems="center" gap={0.75}>
                      <Box sx={{ width: 8, height: 8, background: explorerColor, clipPath: "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)" }} />
                      <Typography sx={{ fontSize: "11px", color: "#9ca3af" }}>
                        E{e.id} → <span style={{ color: explorerColor }}>{pName}</span>
                        {e.currentPlanetId ? <span style={{ color: "#4b5563", fontSize: "10px" }}> (#{e.currentPlanetId})</span> : null}
                      </Typography>
                    </Box>
                  );
                })}
              </Stack>
            </Box>
          )}

          {/* Topology info */}
          {Object.keys(topology).length > 0 && (
            <Box
              sx={{
                p: 2,
                borderRadius: "12px",
                background: "rgba(99,102,241,0.04)",
                border: "1px solid rgba(99,102,241,0.15)",
              }}
            >
              <Typography
                sx={{ fontSize: "12px", color: "#818cf8", fontWeight: 600, mb: 0.5 }}
              >
                Galaxy Topology
              </Typography>
              <Typography sx={{ fontSize: "11px", color: "#6b7280", lineHeight: 1.7 }}>
                {edges.length} bidirectional links loaded from galaxy.txt.
                Dashed purple lines show connected planets.
              </Typography>
            </Box>
          )}

          {/* Debug overlay */}
          {mode === "debug" && (
            <Box
              sx={{
                p: 2,
                borderRadius: "12px",
                background: "rgba(239,68,68,0.04)",
                border: "1px solid rgba(239,68,68,0.12)",
              }}
            >
              <Typography
                sx={{ fontSize: "13px", color: "#f87171", lineHeight: 1.7 }}
              >
                Debug mode — topology edges: {edges.length}, planets tracked:{" "}
                {Object.keys(positions).length}
              </Typography>
            </Box>
          )}
        </Stack>
      </Stack>
    </StyledCard>
  );
}
