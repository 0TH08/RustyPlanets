import {
  Box,
  Button,
  Card,
  Chip,
  Stack,
  Typography,
  styled,
} from "@mui/material";

import { useEffect, useRef, useState } from "react";
import type { PlanetSummary } from "../types/planet";
import { usePlanetStore } from "../store/planetStore";
import {
  startPlanet,
  stopPlanet,
  sendSunray,
  sendAsteroid,
  moveExplorer,
} from "../services/planetService";


const API_BASE = "http://localhost:8080/api";

type Mode = "player" | "debug";
type VisualizationViewMode = "orbit" | "topology";
type DebugStatus =
    | "checking"
    | "online"
    | "offline"
    | "connected"
    | "disconnected";

type PlanetEffectType = "sunray" | "asteroid" | "start" | "stop";

interface PlanetEffect {
  planetId: number;
  type: PlanetEffectType;
}

interface MoveTrailEffect {
  fromPlanetId: number;
  toPlanetId: number;
}

interface DebugApiStatus {
  backend: DebugStatus;
  planets: DebugStatus;
  explorers: DebugStatus;
  topology: DebugStatus;
}

interface VisualizationPageProps {
  mode: Mode;
  onChangeTab?: (
      tab:
          | "overview"
          | "planet"
          | "simulation"
          | "logs"
          | "visualization"
          | "explorers"
  ) => void;
}

interface OrbitParams {
  radius: number;
  speed: number;
  color: string;
}

interface ExplorerPos {
  id: number;
  currentPlanetId: number | null;
  aiRunning?: boolean;
  basicResources?: Record<string, number>;
  complexResources?: Record<string, number>;
}

interface ExplorerApiResponse {
  id: number;
  currentPlanetId?: number | null;
  current_planet_id?: number | null;
  aiRunning?: boolean;
  ai_running?: boolean;
  basicResources?: Record<string, number>;
  basic_resources?: Record<string, number>;
  complexResources?: Record<string, number>;
  complex_resources?: Record<string, number>;
}

// Fixed visual params per planet id — kept stable so the animation doesn't jump on refetch
const PLANET_VISUAL: Record<number, OrbitParams> = {
  1: { radius: 40, speed: 0.5, color: "#90caf9" },
  2: { radius: 70, speed: 0.3, color: "#ffb74d" },
  3: { radius: 55, speed: 0.4, color: "#81c784" },
  4: { radius: 85, speed: 0.2, color: "#ce93d8" },
  5: { radius: 65, speed: 0.6, color: "#ffcc02" },
  6: { radius: 50, speed: 0.7, color: "#ef5350" },
  7: { radius: 75, speed: 0.35, color: "#26c6da" },
  8: { radius: 95, speed: 0.15, color: "#f472b6" },
};

const TOPOLOGY_LAYOUT: Record<number, { x: number; y: number }> = {
  1: { x: 0.24, y: 0.42 }, // Skycartel
  2: { x: 0.30, y: 0.18 }, // Luna4
  3: { x: 0.63, y: 0.18 }, // BlackAdidasShoe
  4: { x: 0.10, y: 0.62 }, // ImmutableCosmicBorrow
  5: { x: 0.86, y: 0.40 }, // RustEze
  6: { x: 0.32, y: 0.80 }, // Crabtorio
  7: { x: 0.72, y: 0.74 }, // Orbitron
  8: { x: 0.56, y: 0.92 }, // AstroParrot
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

const normalizeExplorer = (explorer: ExplorerApiResponse): ExplorerPos => {
  return {
    id: explorer.id,
    currentPlanetId:
        explorer.currentPlanetId ?? explorer.current_planet_id ?? null,
    aiRunning: explorer.aiRunning ?? explorer.ai_running ?? false,
    basicResources: explorer.basicResources ?? explorer.basic_resources ?? {},
    complexResources:
        explorer.complexResources ?? explorer.complex_resources ?? {},
  };
};


function MissionRow({
                      icon,
                      label,
                      current,
                      target,
                      accent,
                    }: {
  icon: string;
  label: string;
  current: number;
  target: number;
  accent: string;
}) {
  const safeTarget = Math.max(target, 1);
  const progress = Math.min(100, Math.round((current / safeTarget) * 100));
  const completed = current >= safeTarget;

  return (
      <Box>
        <Box
            display="flex"
            alignItems="center"
            justifyContent="space-between"
            mb={0.5}
        >
          <Typography sx={{ fontSize: "11px", color: "#d1d5db" }}>
            {icon} {label}
          </Typography>

          <Typography
              sx={{
                fontSize: "11px",
                color: completed ? "#4ade80" : "#9ca3af",
                fontFamily: "monospace",
              }}
          >
            {Math.min(current, safeTarget)} / {safeTarget}
          </Typography>
        </Box>

        <Box
            sx={{
              height: 6,
              borderRadius: "999px",
              background: "rgba(255,255,255,0.06)",
              overflow: "hidden",
              border: "1px solid rgba(255,255,255,0.04)",
            }}
        >
          <Box
              sx={{
                width: `${progress}%`,
                height: "100%",
                background: completed
                    ? "linear-gradient(90deg, #22c55e 0%, #4ade80 100%)"
                    : `linear-gradient(90deg, ${accent}88 0%, ${accent} 100%)`,
                boxShadow: `0 0 12px ${completed ? "#4ade80" : accent}`,
                transition: "width 0.3s ease",
              }}
          />
        </Box>
      </Box>
  );
}

function DebugStatusChip({
                           label,
                           status,
                         }: {
  label: string;
  status: DebugStatus;
}) {
  const isGood = status === "online" || status === "connected";
  const isChecking = status === "checking";

  return (
      <Chip
          label={`${label}: ${status.toUpperCase()}`}
          size="small"
          sx={{
            height: 20,
            fontSize: "10px",
            fontWeight: 700,
            background: isGood
                ? "rgba(34,197,94,0.14)"
                : isChecking
                    ? "rgba(99,102,241,0.12)"
                    : "rgba(239,68,68,0.12)",
            color: isGood ? "#4ade80" : isChecking ? "#a5b4fc" : "#f87171",
            border: isGood
                ? "1px solid rgba(34,197,94,0.3)"
                : isChecking
                    ? "1px solid rgba(99,102,241,0.25)"
                    : "1px solid rgba(239,68,68,0.25)",
          }}
      />
  );
}

function DebugInfoRow({ label, value }: { label: string; value: string | number }) {
  return (
      <Box
          display="flex"
          justifyContent="space-between"
          gap={1}
          sx={{
            fontSize: "11px",
            color: "#9ca3af",
            background: "rgba(255,255,255,0.025)",
            border: "1px solid rgba(255,255,255,0.04)",
            borderRadius: "8px",
            px: 1,
            py: 0.5,
          }}
      >
        <span>{label}</span>
        <span style={{ color: "#e5e7eb", fontFamily: "monospace" }}>{value}</span>
      </Box>
  );
}

export function VisualizationPage({
                                    mode,
                                    onChangeTab,
                                  }: VisualizationPageProps) {
  const [time, setTime] = useState(0);
  const [viewMode, setViewMode] = useState<VisualizationViewMode>("orbit");
  const [planets, setPlanets] = useState<PlanetSummary[]>([]);
  const [topology, setTopology] = useState<Record<string, number[]>>({});
  const [explorers, setExplorers] = useState<ExplorerPos[]>([]);
  const [selectedExplorerId, setSelectedExplorerId] = useState<number | null>(
      null
  );
  const [planetActionMessage, setPlanetActionMessage] = useState<string | null>(
      null
  );
  const [planetEffect, setPlanetEffect] = useState<PlanetEffect | null>(null);
  const [moveTrailEffect, setMoveTrailEffect] = useState<MoveTrailEffect | null>(
      null
  );
  const [apiStatus, setApiStatus] = useState<DebugApiStatus>({
    backend: "checking",
    planets: "checking",
    explorers: "checking",
    topology: "checking",
  });
  const [wsConnected, setWsConnected] = useState(false);
  const [debugActionMessage, setDebugActionMessage] = useState<string | null>(
      null
  );
  const [movingExplorerId, setMovingExplorerId] = useState<number | null>(null);

  const startRef = useRef<number | null>(null);

  const { selectedPlanetId, selectPlanet } = usePlanetStore();

  const handlePlanetClick = (id: number) => {
    selectPlanet(id);
    if (onChangeTab) {
      onChangeTab("planet");
    }
  };

  const loadExplorers = () => {
    fetch(`${API_BASE}/explorers`)
        .then((r) => r.json())
        .then((data: ExplorerApiResponse[]) =>
            setExplorers(data.map(normalizeExplorer))
        )
        .catch(() => {});
  };

  const getResourceTotal = (resources?: Record<string, number>) => {
    if (!resources) return 0;

    return Object.values(resources).reduce((total, count) => total + count, 0);
  };

  const getExplorerBackpackTotal = (explorer: ExplorerPos) => {
    return (
        getResourceTotal(explorer.basicResources) +
        getResourceTotal(explorer.complexResources)
    );
  };

  const getExplorerPlanetName = (explorer: ExplorerPos) => {
    return (
        planets.find((planet) => planet.id === explorer.currentPlanetId)?.name ??
        "Unknown"
    );
  };

  const refreshPlanets = () => {
    fetch(`${API_BASE}/planets`)
        .then((r) => r.json())
        .then(setPlanets)
        .catch(() => {});
  };

  const runPlanetAction = async (
      action: "sunray" | "asteroid" | "start" | "stop"
  ) => {
    if (selectedPlanetId === null) return;

    const labelMap = {
      sunray: "Sunray sent",
      asteroid: "Asteroid sent",
      start: "Planet AI started",
      stop: "Planet AI stopped",
    };

    try {
      setPlanetActionMessage("Sending action...");

      // Show the visual effect immediately when the button is clicked.
      // This makes Start AI / Stop AI visible even if the backend responds very fast
      // or if the planet was already in that AI state.
      setPlanetEffect({
        planetId: selectedPlanetId,
        type: action,
      });

      window.setTimeout(() => {
        setPlanetEffect(null);
      }, 1400);

      if (action === "sunray") {
        await sendSunray(selectedPlanetId);
      }

      if (action === "asteroid") {
        await sendAsteroid(selectedPlanetId);
      }

      if (action === "start") {
        await startPlanet(selectedPlanetId);
      }

      if (action === "stop") {
        await stopPlanet(selectedPlanetId);
      }

      setPlanetActionMessage(`${labelMap[action]} to Planet ${selectedPlanetId}.`);

      refreshPlanets();
      loadExplorers();
    } catch {
      setPlanetActionMessage("Action failed. Check if backend is running.");
    }
  };

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

  // Poll explorers every 3 s so backpack/resources update too
  useEffect(() => {
    loadExplorers();

    const id = window.setInterval(loadExplorers, 3000);

    return () => window.clearInterval(id);
  }, []);

  // Debug health check: only runs in debug mode so the normal player view stays lightweight
  useEffect(() => {
    if (mode !== "debug") return;

    const checkEndpoint = async (path: string): Promise<boolean> => {
      try {
        const res = await fetch(`${API_BASE}${path}`);
        return res.ok;
      } catch {
        return false;
      }
    };

    const checkApiHealth = async () => {
      const [planetsOk, explorersOk, topologyOk] = await Promise.all([
        checkEndpoint("/planets"),
        checkEndpoint("/explorers"),
        checkEndpoint("/topology"),
      ]);

      setApiStatus({
        backend: planetsOk || explorersOk || topologyOk ? "online" : "offline",
        planets: planetsOk ? "online" : "offline",
        explorers: explorersOk ? "online" : "offline",
        topology: topologyOk ? "online" : "offline",
      });
    };

    setApiStatus({
      backend: "checking",
      planets: "checking",
      explorers: "checking",
      topology: "checking",
    });

    checkApiHealth();

    const id = window.setInterval(checkApiHealth, 5000);

    return () => window.clearInterval(id);
  }, [mode]);

  // Real-time explorer tracking:
  // WebSocket log stream → parse "Explorer X moved to planet Y" lines for instant updates
  useEffect(() => {
    const wsUrl =
        API_BASE.replace(/^http/, "ws").replace("/api", "") + "/ws/logs";
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setWsConnected(true);
    };

    ws.onclose = () => {
      setWsConnected(false);
    };

    ws.onmessage = (event) => {
      try {
        const entry = JSON.parse(event.data as string) as { message?: string };
        const msg = entry.message ?? "";

        // Match: "Explorer 2 moved to planet 5 (was on …)"
        const match = msg.match(/^Explorer (\d+) moved to planet (\d+)/);

        if (match) {
          const explorerId = parseInt(match[1], 10);
          const planetId = parseInt(match[2], 10);

          setExplorers((prev) => {
            const exists = prev.some((e) => e.id === explorerId);

            if (!exists) {
              return [
                ...prev,
                {
                  id: explorerId,
                  currentPlanetId: planetId,
                  aiRunning: false,
                  basicResources: {},
                  complexResources: {},
                },
              ];
            }

            const previousExplorer = prev.find((e) => e.id === explorerId);

            if (
                previousExplorer?.currentPlanetId &&
                previousExplorer.currentPlanetId !== planetId
            ) {
              setMoveTrailEffect({
                fromPlanetId: previousExplorer.currentPlanetId,
                toPlanetId: planetId,
              });

              window.setTimeout(() => {
                setMoveTrailEffect(null);
              }, 900);
            }

            return prev.map((e) =>
                e.id === explorerId ? { ...e, currentPlanetId: planetId } : e
            );
          });
        }
      } catch {
        // ignore malformed frames
      }
    };

    ws.onerror = () => {
      setWsConnected(false);
      // silent — simulation may not be running
    };

    return () => ws.close();
  }, []);

  const size = 340;
  const center = size / 2;

  // Compute current x/y for each planet
  const positions: Record<number, { x: number; y: number }> = {};

  for (const planet of planets) {
    const vis = PLANET_VISUAL[planet.id];

    if (!vis) continue;

    if (viewMode === "topology") {
      const fixedPos = TOPOLOGY_LAYOUT[planet.id];

      if (!fixedPos) continue;

      positions[planet.id] = {
        x: fixedPos.x * size,
        y: fixedPos.y * size,
      };

      continue;
    }

    const angle = time * vis.speed;

    positions[planet.id] = {
      x: center + vis.radius * Math.cos(angle),
      y: center + vis.radius * Math.sin(angle),
    };
  }

  // Build unique edge list from topology (avoid drawing A→B and B→A)
  const edges: [number, number][] = [];
  const seen = new Set<string>();

  for (const [srcStr, neighbors] of Object.entries(topology) as [
    string,
    number[]
  ][]) {
    const src = Number(srcStr);

    for (const dst of neighbors) {
      const key = [Math.min(src, dst), Math.max(src, dst)].join("-");

      if (!seen.has(key)) {
        seen.add(key);
        edges.push([src, dst]);
      }
    }
  }

  const selectedExplorer =
      selectedExplorerId === null
          ? null
          : explorers.find((explorer) => explorer.id === selectedExplorerId) ??
          null;

  const selectedPlanet =
      selectedPlanetId === null
          ? null
          : planets.find((planet) => planet.id === selectedPlanetId) ?? null;

  const selectedPlanetNeighbors =
      selectedPlanetId === null ? [] : topology[String(selectedPlanetId)] ?? [];

  const hostedExplorers =
      selectedPlanetId === null
          ? []
          : explorers.filter(
              (explorer) => explorer.currentPlanetId === selectedPlanetId
          );

  const selectedExplorerMoveTargets =
      selectedExplorer?.currentPlanetId === null || selectedExplorer === null
          ? []
          : topology[String(selectedExplorer.currentPlanetId)] ?? [];

  const moveSelectedExplorer = async (planetId: number) => {
    if (!selectedExplorer) return;

    try {
      setMovingExplorerId(selectedExplorer.id);
      setDebugActionMessage(
          `Moving Explorer ${selectedExplorer.id} to Planet ${planetId}...`
      );

      await moveExplorer(selectedExplorer.id, planetId);

      setDebugActionMessage(
          `Explorer ${selectedExplorer.id} moved to Planet ${planetId}.`
      );

      if (
          selectedExplorer.currentPlanetId &&
          selectedExplorer.currentPlanetId !== planetId
      ) {
        setMoveTrailEffect({
          fromPlanetId: selectedExplorer.currentPlanetId,
          toPlanetId: planetId,
        });

        window.setTimeout(() => {
          setMoveTrailEffect(null);
        }, 900);
      }

      setExplorers((prev) =>
          prev.map((explorer) =>
              explorer.id === selectedExplorer.id
                  ? { ...explorer, currentPlanetId: planetId }
                  : explorer
          )
      );

      loadExplorers();
      refreshPlanets();
    } catch {
      setDebugActionMessage("Explorer move failed. Check backend logs.");
    } finally {
      setMovingExplorerId(null);
    }
  };

  const exploredPlanetIds = new Set(
      explorers
          .map((explorer) => explorer.currentPlanetId)
          .filter((planetId): planetId is number => planetId !== null)
  );

  const totalBackpackItems = explorers.reduce(
      (sum, explorer) => sum + getExplorerBackpackTotal(explorer),
      0
  );

  const runningAiCount = planets.filter((planet) => planet.aiRunning).length;

  const totalRocketsBuilt = planets.reduce((sum, planet) => {
    const statsPlanet = planet as PlanetSummary & { rocketsBuilt?: number };
    return sum + (statsPlanet.rocketsBuilt ?? 0);
  }, 0);

  const missionExploreTarget = Math.max(planets.length, 1);
  const missionAiTarget = Math.min(3, Math.max(planets.length, 1));
  const missionResourceTarget = 10;
  const missionRocketTarget = 1;

  const completedMissionCount = [
    totalBackpackItems >= missionResourceTarget,
    totalRocketsBuilt >= missionRocketTarget,
    exploredPlanetIds.size >= missionExploreTarget,
    runningAiCount >= missionAiTarget,
  ].filter(Boolean).length;

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

          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            <Button
                size="small"
                onClick={() => setViewMode("orbit")}
                sx={{
                  fontSize: "11px",
                  borderRadius: "999px",
                  textTransform: "none",
                  color: viewMode === "orbit" ? "#c7d2fe" : "#9ca3af",
                  border:
                      viewMode === "orbit"
                          ? "1px solid rgba(129,140,248,0.5)"
                          : "1px solid rgba(255,255,255,0.08)",
                  background:
                      viewMode === "orbit"
                          ? "rgba(99,102,241,0.18)"
                          : "rgba(255,255,255,0.03)",
                }}
            >
              Orbit View
            </Button>

            <Button
                size="small"
                onClick={() => setViewMode("topology")}
                sx={{
                  fontSize: "11px",
                  borderRadius: "999px",
                  textTransform: "none",
                  color: viewMode === "topology" ? "#c7d2fe" : "#9ca3af",
                  border:
                      viewMode === "topology"
                          ? "1px solid rgba(129,140,248,0.5)"
                          : "1px solid rgba(255,255,255,0.08)",
                  background:
                      viewMode === "topology"
                          ? "rgba(99,102,241,0.18)"
                          : "rgba(255,255,255,0.03)",
                }}
            >
              Topology View
            </Button>

            <Chip
                label={viewMode === "orbit" ? "LIVE ORBITS" : "STABLE MAP"}
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
          </Stack>
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
                width: { xs: "100%", md: "520px" },
                minHeight: { xs: "auto", md: "520px" },
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

                <filter
                    id="explorerGlow"
                    x="-80%"
                    y="-80%"
                    width="260%"
                    height="260%"
                >
                  <feGaussianBlur stdDeviation="2.5" result="blur" />
                  <feMerge>
                    <feMergeNode in="blur" />
                    <feMergeNode in="SourceGraphic" />
                  </feMerge>
                </filter>
              </defs>

              {/* Central star */}
              <circle cx={center} cy={center} r={12} fill="url(#starGradient)" />
              <circle
                  cx={center}
                  cy={center}
                  r={18}
                  fill="rgba(251,192,45,0.15)"
              />

              {/* Orbit rings */}
              {viewMode === "orbit" &&
                  planets.map((p) => {
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

                const isSelectedRoute =
                    selectedPlanetId !== null &&
                    (src === selectedPlanetId || dst === selectedPlanetId);

                return (
                    <line
                        key={`edge-${src}-${dst}`}
                        x1={a.x}
                        y1={a.y}
                        x2={b.x}
                        y2={b.y}
                        stroke={
                          isSelectedRoute
                              ? "rgba(129,140,248,0.95)"
                              : "rgba(99,102,241,0.35)"
                        }
                        strokeWidth={isSelectedRoute ? 1.6 : 0.8}
                        strokeDasharray={isSelectedRoute ? "0" : "3 3"}
                    />
                );
              })}

              {/* Explorer movement trail effect */}
              {moveTrailEffect &&
                  positions[moveTrailEffect.fromPlanetId] &&
                  positions[moveTrailEffect.toPlanetId] && (
                      <line
                          x1={positions[moveTrailEffect.fromPlanetId].x}
                          y1={positions[moveTrailEffect.fromPlanetId].y}
                          x2={positions[moveTrailEffect.toPlanetId].x}
                          y2={positions[moveTrailEffect.toPlanetId].y}
                          stroke="#38bdf8"
                          strokeWidth={2.5}
                          strokeLinecap="round"
                          strokeDasharray="8 6"
                          opacity={0.95}
                      >
                        <animate
                            attributeName="opacity"
                            values="1;0"
                            dur="0.9s"
                            fill="freeze"
                        />
                        <animate
                            attributeName="stroke-width"
                            values="3;1"
                            dur="0.9s"
                            fill="freeze"
                        />
                      </line>
                  )}

              {/* Orbiting planets */}
              {planets.map((p) => {
                const vis = PLANET_VISUAL[p.id];
                const pos = positions[p.id];

                if (!vis || !pos) return null;

                const isSelected = selectedPlanetId === p.id;
                const isNeighbor =
                    selectedPlanetId !== null &&
                    selectedPlanetNeighbors.includes(p.id);
                const activePlanetEffect =
                    planetEffect?.planetId === p.id ? planetEffect.type : null;
                const effectColor =
                    activePlanetEffect === "sunray"
                        ? "#facc15"
                        : activePlanetEffect === "asteroid"
                            ? "#f87171"
                            : activePlanetEffect === "start"
                                ? "#4ade80"
                                : activePlanetEffect === "stop"
                                    ? "#9ca3af"
                                    : null;

                return (
                    <g
                        key={p.id}
                        style={{ cursor: "pointer" }}
                        onClick={() => handlePlanetClick(p.id)}
                    >
                      {/* Planet action visual effect */}
                      {effectColor && (
                          <>
                            <circle
                                cx={pos.x}
                                cy={pos.y}
                                r={13}
                                fill="none"
                                stroke={effectColor}
                                strokeWidth={3}
                                opacity={0.95}
                            >
                              <animate
                                  attributeName="r"
                                  values="10;34"
                                  dur="1.2s"
                                  fill="freeze"
                              />
                              <animate
                                  attributeName="opacity"
                                  values="0.95;0"
                                  dur="1.2s"
                                  fill="freeze"
                              />
                            </circle>

                            <circle
                                cx={pos.x}
                                cy={pos.y}
                                r={7}
                                fill={effectColor}
                                opacity={0.28}
                            >
                              <animate
                                  attributeName="r"
                                  values="7;18"
                                  dur="1.2s"
                                  fill="freeze"
                              />
                              <animate
                                  attributeName="opacity"
                                  values="0.28;0"
                                  dur="1.2s"
                                  fill="freeze"
                              />
                            </circle>

                            {activePlanetEffect === "asteroid" && (
                                <text
                                    x={pos.x}
                                    y={pos.y - 18}
                                    fill="#f87171"
                                    fontSize="16"
                                    fontWeight="bold"
                                    textAnchor="middle"
                                    style={{ pointerEvents: "none" }}
                                >
                                  ☄
                                </text>
                            )}

                            {activePlanetEffect === "sunray" && (
                                <text
                                    x={pos.x}
                                    y={pos.y - 18}
                                    fill="#facc15"
                                    fontSize="16"
                                    fontWeight="bold"
                                    textAnchor="middle"
                                    style={{ pointerEvents: "none" }}
                                >
                                  ☀
                                </text>
                            )}

                            {activePlanetEffect === "start" && (
                                <text
                                    x={pos.x}
                                    y={pos.y - 18}
                                    fill="#4ade80"
                                    fontSize="12"
                                    fontWeight="bold"
                                    textAnchor="middle"
                                    style={{ pointerEvents: "none" }}
                                >
                                  ▶ AI
                                </text>
                            )}

                            {activePlanetEffect === "stop" && (
                                <text
                                    x={pos.x}
                                    y={pos.y - 18}
                                    fill="#cbd5e1"
                                    fontSize="12"
                                    fontWeight="bold"
                                    textAnchor="middle"
                                    style={{ pointerEvents: "none" }}
                                >
                                  ⏸ AI
                                </text>
                            )}
                          </>
                      )}

                      {/* Neighbor glow ring */}
                      {isNeighbor && !isSelected && (
                          <circle
                              cx={pos.x}
                              cy={pos.y}
                              r={11}
                              fill="none"
                              stroke="#818cf8"
                              strokeWidth={0.8}
                              strokeDasharray="2 2"
                              opacity={0.55}
                          />
                      )}

                      {/* Selection glow ring */}
                      {isSelected && (
                          <circle
                              cx={pos.x}
                              cy={pos.y}
                              r={12}
                              fill="none"
                              stroke="#fff"
                              strokeWidth={1}
                              strokeDasharray="2 2"
                              opacity={0.8}
                          />
                      )}

                      <circle
                          cx={pos.x}
                          cy={pos.y}
                          r={8}
                          fill={vis.color}
                          opacity={0.2}
                      />

                      <circle cx={pos.x} cy={pos.y} r={5} fill={vis.color} />

                      <text
                          x={pos.x + 10}
                          y={pos.y - 6}
                          fill={isSelected ? "#fff" : "#d1d5db"}
                          fontSize={isSelected ? "9" : "8"}
                          fontWeight={isSelected ? "bold" : "normal"}
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
                const spreadAngle =
                    i * ((Math.PI * 2) / Math.max(explorers.length, 1));
                const ex = pos.x + Math.cos(spreadAngle) * 20;
                const ey = pos.y + Math.sin(spreadAngle) * 20;
                const explorerColor = i === 0 ? "#f59e0b" : "#a78bfa";
                const backpackTotal = getExplorerBackpackTotal(e);

                return (
                    <g
                        key={`explorer-${e.id}`}
                        filter="url(#explorerGlow)"
                        onClick={() => setSelectedExplorerId(e.id)}
                        style={{ cursor: "pointer" }}
                    >
                      {/* Outer glow halo */}
                      <circle
                          cx={ex}
                          cy={ey}
                          r={9}
                          fill={explorerColor}
                          opacity={0.18}
                      />

                      {/* Mid glow */}
                      <circle
                          cx={ex}
                          cy={ey}
                          r={6}
                          fill={explorerColor}
                          opacity={0.3}
                      />

                      {/* Diamond body */}
                      <polygon
                          points={`${ex},${ey - 7} ${ex + 5},${ey} ${ex},${
                              ey + 7
                          } ${ex - 5},${ey}`}
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

                      {/* Backpack count */}
                      <text
                          x={ex}
                          y={ey + 28}
                          fill="#fbbf24"
                          fontSize="8"
                          fontWeight="bold"
                          textAnchor="middle"
                          style={{ pointerEvents: "none" }}
                      >
                        🎒 {backpackTotal}
                      </text>
                    </g>
                );
              })}
            </svg>
          </Box>

          {/* Side Panel */}
          <Stack
              spacing={2}
              flex={1}
              sx={{
                minWidth: { xs: "100%", md: 300 },
                maxHeight: { xs: "none", md: 540 },
                overflowY: { xs: "visible", md: "auto" },
                pr: { xs: 0, md: 0.5 },
                "&::-webkit-scrollbar": {
                  width: 6,
                },
                "&::-webkit-scrollbar-thumb": {
                  background: "rgba(255,255,255,0.12)",
                  borderRadius: 10,
                },
              }}
          >
            {/* Mission Control HUD */}
            <Box
                sx={{
                  p: 2,
                  borderRadius: "12px",
                  background:
                      "linear-gradient(145deg, rgba(34,197,94,0.055) 0%, rgba(99,102,241,0.045) 100%)",
                  border: "1px solid rgba(34,197,94,0.18)",
                  boxShadow: "inset 0 0 32px rgba(34,197,94,0.035)",
                }}
            >
              <Box
                  display="flex"
                  alignItems="center"
                  justifyContent="space-between"
                  gap={1}
                  mb={1.5}
              >
                <Box>
                  <Typography
                      sx={{
                        fontSize: "13px",
                        fontWeight: 700,
                        color: "#4ade80",
                      }}
                  >
                    🎯 Mission Control
                  </Typography>

                  <Typography sx={{ fontSize: "11px", color: "#6b7280", mt: 0.25 }}>
                    Complete objectives by exploring planets, carrying resources,
                    running AI, and building rockets.
                  </Typography>
                </Box>

                <Chip
                    label={`${completedMissionCount}/4 DONE`}
                    size="small"
                    sx={{
                      height: 22,
                      fontSize: "10px",
                      fontWeight: 700,
                      background:
                          completedMissionCount === 4
                              ? "rgba(34,197,94,0.16)"
                              : "rgba(99,102,241,0.12)",
                      color: completedMissionCount === 4 ? "#4ade80" : "#a5b4fc",
                      border:
                          completedMissionCount === 4
                              ? "1px solid rgba(34,197,94,0.35)"
                              : "1px solid rgba(99,102,241,0.25)",
                    }}
                />
              </Box>

              <Stack spacing={1.25}>
                <MissionRow
                    icon="🎒"
                    label="Collect resources"
                    current={totalBackpackItems}
                    target={missionResourceTarget}
                    accent="#f59e0b"
                />

                <MissionRow
                    icon="🚀"
                    label="Build rocket"
                    current={totalRocketsBuilt}
                    target={missionRocketTarget}
                    accent="#c084fc"
                />

                <MissionRow
                    icon="🪐"
                    label="Explore planets"
                    current={exploredPlanetIds.size}
                    target={missionExploreTarget}
                    accent="#60a5fa"
                />

                <MissionRow
                    icon="🤖"
                    label="Keep planet AI running"
                    current={runningAiCount}
                    target={missionAiTarget}
                    accent="#22c55e"
                />
              </Stack>

              {completedMissionCount === 4 && (
                  <Typography
                      sx={{
                        mt: 1.25,
                        fontSize: "11px",
                        color: "#4ade80",
                        fontWeight: 600,
                      }}
                  >
                    Mission completed — galaxy stabilization achieved.
                  </Typography>
              )}
            </Box>

            {/* Active Planets Panel - select a planet first */}
            <Box
                sx={{
                  p: 2,
                  borderRadius: "12px",
                  background: "rgba(255,255,255,0.02)",
                  border: "1px solid rgba(255,255,255,0.06)",
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
                          onClick={() => {
                            selectPlanet(p.id);
                            setPlanetActionMessage(null);
                          }}
                          sx={{
                            p: 1,
                            borderRadius: "10px",
                            cursor: "pointer",
                            background:
                                selectedPlanetId === p.id
                                    ? "rgba(99,102,241,0.12)"
                                    : "rgba(255,255,255,0.015)",
                            border:
                                selectedPlanetId === p.id
                                    ? "1px solid rgba(129,140,248,0.35)"
                                    : "1px solid rgba(255,255,255,0.04)",
                            "&:hover": {
                              background: "rgba(99,102,241,0.08)",
                            },
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
                              sx={{
                                fontSize: "11px",
                                color: "#6b7280",
                                fontFamily: "monospace",
                              }}
                          >
                            r={vis.radius}
                          </Typography>
                        </Stack>
                      </Box>
                  );
                })}
              </Stack>
            </Box>

            {/* Planet Inspector */}
            {selectedPlanet && (
                <Box
                    sx={{
                      p: 2,
                      borderRadius: "12px",
                      background: "rgba(99,102,241,0.04)",
                      border: "1px solid rgba(99,102,241,0.18)",
                    }}
                >
                  <Box
                      display="flex"
                      alignItems="center"
                      justifyContent="space-between"
                      mb={1.5}
                  >
                    <Typography
                        sx={{
                          fontSize: "13px",
                          fontWeight: 600,
                          color: "#818cf8",
                        }}
                    >
                      🪐 Planet Inspector
                    </Typography>

                    <Button
                        size="small"
                        onClick={() => setPlanetActionMessage(null)}
                        sx={{
                          minWidth: 0,
                          color: "#9ca3af",
                          fontSize: "11px",
                          textTransform: "none",
                        }}
                    >
                      Clear
                    </Button>
                  </Box>

                  <Stack spacing={1.2}>
                    <Box display="flex" alignItems="center" gap={1}>
                      <Box
                          sx={{
                            width: 11,
                            height: 11,
                            borderRadius: "50%",
                            background: PLANET_VISUAL[selectedPlanet.id]?.color,
                            boxShadow: `0 0 12px ${
                                PLANET_VISUAL[selectedPlanet.id]?.color ?? "#fff"
                            }`,
                          }}
                      />

                      <Typography sx={{ fontSize: "13px", color: "#e5e7eb" }}>
                        {selectedPlanet.name}
                      </Typography>

                      <Typography
                          sx={{
                            fontSize: "11px",
                            color: "#6b7280",
                            fontFamily: "monospace",
                          }}
                      >
                        #{selectedPlanet.id}
                      </Typography>
                    </Box>

                    <Box display="flex" gap={1} flexWrap="wrap">
                      <Chip
                          label={selectedPlanet.aiRunning ? "AI RUNNING" : "AI STOPPED"}
                          size="small"
                          sx={{
                            height: 20,
                            fontSize: "10px",
                            fontWeight: 700,
                            background: selectedPlanet.aiRunning
                                ? "rgba(34,197,94,0.15)"
                                : "rgba(107,114,128,0.15)",
                            color: selectedPlanet.aiRunning ? "#4ade80" : "#9ca3af",
                            border: selectedPlanet.aiRunning
                                ? "1px solid rgba(34,197,94,0.3)"
                                : "1px solid rgba(107,114,128,0.25)",
                          }}
                      />

                      <Chip
                          label={`${selectedPlanetNeighbors.length} ROUTES`}
                          size="small"
                          sx={{
                            height: 20,
                            fontSize: "10px",
                            fontWeight: 700,
                            background: "rgba(99,102,241,0.12)",
                            color: "#818cf8",
                            border: "1px solid rgba(99,102,241,0.25)",
                          }}
                      />

                      <Chip
                          label={`${hostedExplorers.length} EXPLORER${
                              hostedExplorers.length === 1 ? "" : "S"
                          }`}
                          size="small"
                          sx={{
                            height: 20,
                            fontSize: "10px",
                            fontWeight: 700,
                            background: "rgba(245,158,11,0.12)",
                            color: "#fbbf24",
                            border: "1px solid rgba(245,158,11,0.25)",
                          }}
                      />
                    </Box>

                    <Box>
                      <Typography
                          sx={{
                            fontSize: "11px",
                            color: "#818cf8",
                            fontWeight: 600,
                            mb: 0.5,
                          }}
                      >
                        Connected Planets
                      </Typography>

                      {selectedPlanetNeighbors.length > 0 ? (
                          <Stack direction="row" gap={0.75} flexWrap="wrap">
                            {selectedPlanetNeighbors.map((planetId) => {
                              const neighbor = planets.find((p) => p.id === planetId);

                              return (
                                  <Chip
                                      key={planetId}
                                      label={
                                        neighbor
                                            ? `${neighbor.name} #${planetId}`
                                            : `#${planetId}`
                                      }
                                      size="small"
                                      onClick={() => {
                                        selectPlanet(planetId);
                                        setPlanetActionMessage(null);
                                      }}
                                      sx={{
                                        height: 22,
                                        fontSize: "10px",
                                        background: "rgba(255,255,255,0.04)",
                                        color: "#d1d5db",
                                        border: "1px solid rgba(255,255,255,0.08)",
                                        cursor: "pointer",
                                        "&:hover": {
                                          background: "rgba(99,102,241,0.15)",
                                        },
                                      }}
                                  />
                              );
                            })}
                          </Stack>
                      ) : (
                          <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                            No connected planets found.
                          </Typography>
                      )}
                    </Box>

                    <Box>
                      <Typography
                          sx={{
                            fontSize: "11px",
                            color: "#fbbf24",
                            fontWeight: 600,
                            mb: 0.5,
                          }}
                      >
                        Hosted Explorers
                      </Typography>

                      {hostedExplorers.length > 0 ? (
                          <Stack spacing={0.5}>
                            {hostedExplorers.map((explorer) => (
                                <Typography
                                    key={explorer.id}
                                    sx={{
                                      fontSize: "11px",
                                      color: "#9ca3af",
                                      cursor: "pointer",
                                      "&:hover": {
                                        color: "#fbbf24",
                                      },
                                    }}
                                    onClick={() => setSelectedExplorerId(explorer.id)}
                                >
                                  🛸 Explorer {explorer.id} 🎒{" "}
                                  {getExplorerBackpackTotal(explorer)}
                                </Typography>
                            ))}
                          </Stack>
                      ) : (
                          <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                            No explorer is currently on this planet.
                          </Typography>
                      )}
                    </Box>

                    {mode === "debug" && (
                        <Box>
                          <Typography
                              sx={{
                                fontSize: "11px",
                                color: "#f87171",
                                fontWeight: 600,
                                mb: 0.75,
                              }}
                          >
                            Debug Actions
                          </Typography>

                          <Stack direction="row" gap={0.75} flexWrap="wrap">
                            <Button
                                size="small"
                                onClick={() => runPlanetAction("sunray")}
                                sx={{
                                  fontSize: "10px",
                                  borderRadius: "999px",
                                  textTransform: "none",
                                  color: "#facc15",
                                  border: "1px solid rgba(250,204,21,0.3)",
                                  background: "rgba(250,204,21,0.08)",
                                }}
                            >
                              ☀ Sunray
                            </Button>

                            <Button
                                size="small"
                                onClick={() => runPlanetAction("asteroid")}
                                sx={{
                                  fontSize: "10px",
                                  borderRadius: "999px",
                                  textTransform: "none",
                                  color: "#f87171",
                                  border: "1px solid rgba(248,113,113,0.3)",
                                  background: "rgba(248,113,113,0.08)",
                                }}
                            >
                              ☄ Asteroid
                            </Button>

                            <Button
                                size="small"
                                onClick={() => runPlanetAction("start")}
                                sx={{
                                  fontSize: "10px",
                                  borderRadius: "999px",
                                  textTransform: "none",
                                  color: "#4ade80",
                                  border: "1px solid rgba(74,222,128,0.3)",
                                  background: "rgba(74,222,128,0.08)",
                                }}
                            >
                              ▶ Start AI
                            </Button>

                            <Button
                                size="small"
                                onClick={() => runPlanetAction("stop")}
                                sx={{
                                  fontSize: "10px",
                                  borderRadius: "999px",
                                  textTransform: "none",
                                  color: "#9ca3af",
                                  border: "1px solid rgba(156,163,175,0.3)",
                                  background: "rgba(156,163,175,0.08)",
                                }}
                            >
                              ⏸ Stop AI
                            </Button>
                          </Stack>

                          {planetActionMessage && (
                              <Typography
                                  sx={{
                                    mt: 1,
                                    fontSize: "11px",
                                    color: planetActionMessage.includes("failed")
                                        ? "#f87171"
                                        : "#4ade80",
                                  }}
                              >
                                {planetActionMessage}
                              </Typography>
                          )}
                        </Box>
                    )}
                  </Stack>
                </Box>
            )}

            {/* Explorer List */}
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
                      sx={{
                        fontSize: "12px",
                        color: "#fbbf24",
                        fontWeight: 600,
                        mb: 0.5,
                      }}
                  >
                    🛸 Explorers ({explorers.length})
                  </Typography>

                  <Stack spacing={0.5}>
                    {explorers.map((e, i) => {
                      const explorerColor = i === 0 ? "#f59e0b" : "#a78bfa";
                      const pName =
                          planets.find((p) => p.id === e.currentPlanetId)?.name ??
                          "—";
                      const backpackTotal = getExplorerBackpackTotal(e);

                      return (
                          <Box
                              key={e.id}
                              display="flex"
                              alignItems="center"
                              gap={0.75}
                              onClick={() => setSelectedExplorerId(e.id)}
                              sx={{
                                cursor: "pointer",
                                borderRadius: "8px",
                                px: 0.5,
                                py: 0.25,
                                "&:hover": {
                                  background: "rgba(245,158,11,0.08)",
                                },
                              }}
                          >
                            <Box
                                sx={{
                                  width: 8,
                                  height: 8,
                                  background: explorerColor,
                                  clipPath:
                                      "polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)",
                                }}
                            />

                            <Typography sx={{ fontSize: "11px", color: "#9ca3af" }}>
                              E{e.id} 🎒 {backpackTotal} →{" "}
                              <span style={{ color: explorerColor }}>{pName}</span>
                              {e.currentPlanetId ? (
                                  <span
                                      style={{
                                        color: "#4b5563",
                                        fontSize: "10px",
                                      }}
                                  >
                            {" "}
                                    (#{e.currentPlanetId})
                          </span>
                              ) : null}
                            </Typography>
                          </Box>
                      );
                    })}
                  </Stack>
                </Box>
            )}

            {/* Explorer Backpack */}
            {selectedExplorer && (
                <Box
                    sx={{
                      p: 2,
                      borderRadius: "12px",
                      background: "rgba(245,158,11,0.04)",
                      border: "1px solid rgba(245,158,11,0.2)",
                    }}
                >
                  <Box
                      display="flex"
                      alignItems="center"
                      justifyContent="space-between"
                      mb={1.5}
                  >
                    <Typography
                        sx={{
                          fontSize: "13px",
                          fontWeight: 600,
                          color: "#fbbf24",
                        }}
                    >
                      🎒 Explorer {selectedExplorer.id} Backpack
                    </Typography>

                    <Button
                        size="small"
                        onClick={() => setSelectedExplorerId(null)}
                        sx={{
                          minWidth: 0,
                          color: "#9ca3af",
                          fontSize: "11px",
                          textTransform: "none",
                        }}
                    >
                      Close
                    </Button>
                  </Box>

                  <Stack spacing={1.2}>
                    <Box display="flex" gap={1} flexWrap="wrap">
                      <Chip
                          label={
                            selectedExplorer.aiRunning ? "AI RUNNING" : "AI STOPPED"
                          }
                          size="small"
                          sx={{
                            height: 20,
                            fontSize: "10px",
                            fontWeight: 700,
                            background: selectedExplorer.aiRunning
                                ? "rgba(34,197,94,0.15)"
                                : "rgba(107,114,128,0.15)",
                            color: selectedExplorer.aiRunning
                                ? "#4ade80"
                                : "#9ca3af",
                            border: selectedExplorer.aiRunning
                                ? "1px solid rgba(34,197,94,0.3)"
                                : "1px solid rgba(107,114,128,0.25)",
                          }}
                      />

                      <Chip
                          label={`${getExplorerBackpackTotal(
                              selectedExplorer
                          )} ITEMS`}
                          size="small"
                          sx={{
                            height: 20,
                            fontSize: "10px",
                            fontWeight: 700,
                            background: "rgba(245,158,11,0.12)",
                            color: "#fbbf24",
                            border: "1px solid rgba(245,158,11,0.25)",
                          }}
                      />
                    </Box>

                    <Typography sx={{ fontSize: "11px", color: "#9ca3af" }}>
                      Current Planet:{" "}
                      <span style={{ color: "#e5e7eb" }}>
                    {getExplorerPlanetName(selectedExplorer)}
                  </span>
                    </Typography>

                    <Box>
                      <Typography
                          sx={{
                            fontSize: "11px",
                            color: "#93c5fd",
                            fontWeight: 600,
                            mb: 0.5,
                          }}
                      >
                        Basic Resources
                      </Typography>

                      {selectedExplorer.basicResources &&
                      Object.keys(selectedExplorer.basicResources).length > 0 ? (
                          <Stack spacing={0.5}>
                            {Object.entries(selectedExplorer.basicResources).map(
                                ([name, count]) => (
                                    <Box
                                        key={name}
                                        display="flex"
                                        justifyContent="space-between"
                                        sx={{
                                          fontSize: "11px",
                                          color: "#d1d5db",
                                          background: "rgba(255,255,255,0.025)",
                                          border: "1px solid rgba(255,255,255,0.04)",
                                          borderRadius: "8px",
                                          px: 1,
                                          py: 0.5,
                                        }}
                                    >
                                      <span>{name}</span>
                                      <span>x{count}</span>
                                    </Box>
                                )
                            )}
                          </Stack>
                      ) : (
                          <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                            No basic resources yet.
                          </Typography>
                      )}
                    </Box>

                    <Box>
                      <Typography
                          sx={{
                            fontSize: "11px",
                            color: "#c084fc",
                            fontWeight: 600,
                            mb: 0.5,
                          }}
                      >
                        Complex Resources
                      </Typography>

                      {selectedExplorer.complexResources &&
                      Object.keys(selectedExplorer.complexResources).length > 0 ? (
                          <Stack spacing={0.5}>
                            {Object.entries(selectedExplorer.complexResources).map(
                                ([name, count]) => (
                                    <Box
                                        key={name}
                                        display="flex"
                                        justifyContent="space-between"
                                        sx={{
                                          fontSize: "11px",
                                          color: "#d1d5db",
                                          background: "rgba(255,255,255,0.025)",
                                          border: "1px solid rgba(255,255,255,0.04)",
                                          borderRadius: "8px",
                                          px: 1,
                                          py: 0.5,
                                        }}
                                    >
                                      <span>{name}</span>
                                      <span>x{count}</span>
                                    </Box>
                                )
                            )}
                          </Stack>
                      ) : (
                          <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                            No complex resources yet.
                          </Typography>
                      )}
                    </Box>
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
                      sx={{
                        fontSize: "12px",
                        color: "#818cf8",
                        fontWeight: 600,
                        mb: 0.5,
                      }}
                  >
                    Galaxy Topology
                  </Typography>

                  <Typography
                      sx={{
                        fontSize: "11px",
                        color: "#6b7280",
                        lineHeight: 1.7,
                      }}
                  >
                    {edges.length} bidirectional links loaded from galaxy.txt.
                    Dashed purple lines show connected planets.
                  </Typography>
                </Box>
            )}

            {/* Debug tools */}
            {mode === "debug" && (
                <Box
                    sx={{
                      p: 2,
                      borderRadius: "12px",
                      background: "rgba(239,68,68,0.04)",
                      border: "1px solid rgba(239,68,68,0.14)",
                    }}
                >
                  <Box
                      display="flex"
                      alignItems="center"
                      justifyContent="space-between"
                      gap={1}
                      mb={1.25}
                  >
                    <Typography
                        sx={{
                          fontSize: "13px",
                          color: "#f87171",
                          fontWeight: 700,
                        }}
                    >
                      🧪 Debug Tools
                    </Typography>

                    <Chip
                        label="ADMIN"
                        size="small"
                        sx={{
                          height: 20,
                          fontSize: "10px",
                          fontWeight: 700,
                          background: "rgba(239,68,68,0.12)",
                          color: "#f87171",
                          border: "1px solid rgba(239,68,68,0.25)",
                        }}
                    />
                  </Box>

                  <Typography
                      sx={{
                        fontSize: "11px",
                        color: "#fca5a5",
                        fontWeight: 600,
                        mb: 0.75,
                      }}
                  >
                    System Health
                  </Typography>

                  <Stack direction="row" gap={0.75} flexWrap="wrap" mb={1.25}>
                    <DebugStatusChip label="Backend" status={apiStatus.backend} />
                    <DebugStatusChip label="Planets" status={apiStatus.planets} />
                    <DebugStatusChip label="Explorers" status={apiStatus.explorers} />
                    <DebugStatusChip label="Topology" status={apiStatus.topology} />
                    <DebugStatusChip
                        label="WebSocket"
                        status={wsConnected ? "connected" : "disconnected"}
                    />
                  </Stack>

                  <Typography
                      sx={{
                        fontSize: "11px",
                        color: "#fca5a5",
                        fontWeight: 600,
                        mb: 0.75,
                      }}
                  >
                    Selected State Inspector
                  </Typography>

                  <Stack spacing={0.5} mb={1.25}>
                    <DebugInfoRow
                        label="View mode"
                        value={viewMode === "orbit" ? "Orbit" : "Topology"}
                    />
                    <DebugInfoRow label="Topology edges" value={edges.length} />
                    <DebugInfoRow
                        label="Planets tracked"
                        value={Object.keys(positions).length}
                    />
                    <DebugInfoRow
                        label="Selected planet"
                        value={
                          selectedPlanet
                              ? `${selectedPlanet.name} #${selectedPlanet.id}`
                              : "None"
                        }
                    />
                    {selectedPlanet && (
                        <DebugInfoRow
                            label="Planet routes"
                            value={selectedPlanetNeighbors.join(", ") || "None"}
                        />
                    )}
                    {selectedPlanet && (
                        <DebugInfoRow
                            label="Planet explorers"
                            value={hostedExplorers.length}
                        />
                    )}
                    <DebugInfoRow
                        label="Selected explorer"
                        value={selectedExplorer ? `Explorer ${selectedExplorer.id}` : "None"}
                    />
                    {selectedExplorer && (
                        <DebugInfoRow
                            label="Explorer planet"
                            value={
                              selectedExplorer.currentPlanetId
                                  ? `${getExplorerPlanetName(selectedExplorer)} #${selectedExplorer.currentPlanetId}`
                                  : "None"
                            }
                        />
                    )}
                    {selectedExplorer && (
                        <DebugInfoRow
                            label="Explorer backpack"
                            value={`${getExplorerBackpackTotal(selectedExplorer)} items`}
                        />
                    )}
                  </Stack>

                  {selectedExplorer && (
                      <Box>
                        <Typography
                            sx={{
                              fontSize: "11px",
                              color: "#fca5a5",
                              fontWeight: 600,
                              mb: 0.75,
                            }}
                        >
                          Move Explorer {selectedExplorer.id}
                        </Typography>

                        {selectedExplorerMoveTargets.length > 0 ? (
                            <Stack direction="row" gap={0.75} flexWrap="wrap">
                              {selectedExplorerMoveTargets.map((planetId) => {
                                const targetPlanet = planets.find(
                                    (planet) => planet.id === planetId
                                );

                                return (
                                    <Button
                                        key={planetId}
                                        size="small"
                                        disabled={movingExplorerId === selectedExplorer.id}
                                        onClick={() => moveSelectedExplorer(planetId)}
                                        sx={{
                                          fontSize: "10px",
                                          borderRadius: "999px",
                                          textTransform: "none",
                                          color: "#fbbf24",
                                          border: "1px solid rgba(251,191,36,0.3)",
                                          background: "rgba(251,191,36,0.08)",
                                          "&.Mui-disabled": {
                                            color: "#6b7280",
                                            border: "1px solid rgba(107,114,128,0.2)",
                                          },
                                        }}
                                    >
                                      Move to {targetPlanet?.name ?? `#${planetId}`}
                                    </Button>
                                );
                              })}
                            </Stack>
                        ) : (
                            <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                              Select an explorer on a planet with connected routes to move it.
                            </Typography>
                        )}
                      </Box>
                  )}

                  {debugActionMessage && (
                      <Typography
                          sx={{
                            mt: 1,
                            fontSize: "11px",
                            color: debugActionMessage.includes("failed")
                                ? "#f87171"
                                : "#4ade80",
                          }}
                      >
                        {debugActionMessage}
                      </Typography>
                  )}
                </Box>
            )}
          </Stack>
        </Stack>
      </StyledCard>
  );
}