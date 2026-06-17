import {
  Box,
  Card,
  Chip,
  Stack,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableRow,
} from "@mui/material";
import { useEffect, useState } from "react";
import styled from "@emotion/styled";

import { usePlanetStore } from "../store/planetStore";
import { useSimulationStore } from "../store/simulationStore";
import { SimulationBar } from "../components/SimulationBar";
import { PlanetsView } from "../components/PlanetsView";

const API_BASE = "http://localhost:8080/api";

type Mode = "player" | "debug";

interface OverviewPageProps {
  mode: Mode;
}

interface ExplorerEntry {
  id: number;
  currentPlanetId: number | null;
  aiRunning: boolean;
  basicResources?: Record<string, number>;
  complexResources?: Record<string, number>;
}

const StyledCard = styled(Card)({
  background: "linear-gradient(145deg, #0f0f0f 0%, #050505 100%)",
  border: "1px solid rgba(255,255,255,0.08)",
  borderRadius: 16,
  padding: 24,
  position: "relative",
  overflow: "hidden",

  "&::before": {
    content: '""',
    position: "absolute",
    inset: "0 0 auto 0",
    height: 1,
    background:
        "linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.1) 50%, transparent 100%)",
  },
});

const titleStyle = {
  fontSize: "14px",
  fontWeight: 500,
  color: "#9ca3af",
  textTransform: "uppercase",
  letterSpacing: "0.1em",
};

const chipStyle = {
  background: "rgba(239,68,68,0.12)",
  color: "#f87171",
  border: "1px solid rgba(239,68,68,0.25)",
  fontSize: "10px",
  fontWeight: 700,
  letterSpacing: "0.05em",
};

function getResourceTotal(resources?: Record<string, number>) {
  if (!resources) return 0;
  return Object.values(resources).reduce((sum, value) => sum + value, 0);
}

function getExplorerBackpackTotal(explorer: ExplorerEntry) {
  return (
      getResourceTotal(explorer.basicResources) +
      getResourceTotal(explorer.complexResources)
  );
}

function StatBox({
                   label,
                   value,
                   color,
                   subtitle,
                 }: {
  label: string;
  value: string | number;
  color: string;
  subtitle?: string;
}) {
  return (
      <Box
          sx={{
            flex: "1 1 160px",
            p: 1.75,
            borderRadius: "14px",
            background: "rgba(255,255,255,0.018)",
            border: "1px solid rgba(255,255,255,0.06)",
          }}
      >
        <Typography
            sx={{
              fontSize: "11px",
              color: "#6b7280",
              textTransform: "uppercase",
              letterSpacing: "0.08em",
              mb: 0.75,
            }}
        >
          {label}
        </Typography>

        <Typography
            sx={{
              fontSize: "24px",
              fontWeight: 700,
              color,
              lineHeight: 1,
            }}
        >
          {value}
        </Typography>

        {subtitle && (
            <Typography sx={{ fontSize: "11px", color: "#4b5563", mt: 0.75 }}>
              {subtitle}
            </Typography>
        )}
      </Box>
  );
}

function ControlCenterDashboard({ mode }: { mode: Mode }) {
  const { planets } = usePlanetStore();

  const [explorers, setExplorers] = useState<ExplorerEntry[]>([]);
  const [topology, setTopology] = useState<Record<string, number[]>>({});
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);
  const [backendOnline, setBackendOnline] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const [explorerRes, topologyRes] = await Promise.all([
          fetch(`${API_BASE}/explorers`),
          fetch(`${API_BASE}/topology`),
        ]);

        const explorerData: ExplorerEntry[] = await explorerRes.json();
        const topologyData: Record<string, number[]> = await topologyRes.json();

        setExplorers(explorerData);
        setTopology(topologyData);
        setLastUpdated(new Date().toLocaleTimeString());
        setBackendOnline(true);
      } catch {
        setBackendOnline(false);
      }
    };

    load();

    const id = window.setInterval(load, 3000);

    return () => window.clearInterval(id);
  }, []);

  const runningAis = planets.filter((planet) => planet.aiRunning).length;

  const totalBackpackItems = explorers.reduce(
      (sum, explorer) => sum + getExplorerBackpackTotal(explorer),
      0
  );

  const totalRoutes = Math.floor(
      Object.values(topology).reduce((sum, routes) => sum + routes.length, 0) / 2
  );

  const totalGenerated = planets.reduce(
      (sum, planet) => sum + planet.totalResourcesGenerated,
      0
  );

  const rocketsBuilt = planets.reduce(
      (sum, planet) => sum + planet.rocketsBuilt,
      0
  );

  return (
      <StyledCard>
        <Box
            display="flex"
            alignItems="center"
            justifyContent="space-between"
            gap={2}
            mb={2}
            flexWrap="wrap"
        >
          <Box>
            <Typography sx={titleStyle}>RustyPlanets Control Center</Typography>

            <Typography sx={{ fontSize: "12px", color: "#6b7280", mt: 0.75 }}>
              Live overview of planets, explorers, resources, routes, and system
              state.
            </Typography>
          </Box>

          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            {lastUpdated && (
                <Typography sx={{ fontSize: "11px", color: "#4b5563" }}>
                  Updated {lastUpdated}
                </Typography>
            )}

            <Chip
                label={backendOnline ? "BACKEND ONLINE" : "BACKEND OFFLINE"}
                size="small"
                sx={{
                  background: backendOnline
                      ? "rgba(34,197,94,0.12)"
                      : "rgba(239,68,68,0.12)",
                  color: backendOnline ? "#4ade80" : "#f87171",
                  border: backendOnline
                      ? "1px solid rgba(34,197,94,0.25)"
                      : "1px solid rgba(239,68,68,0.25)",
                  fontSize: "10px",
                  fontWeight: 700,
                }}
            />

            <Chip
                label={mode === "debug" ? "DEBUG MODE" : "PLAYER MODE"}
                size="small"
                sx={{
                  background:
                      mode === "debug"
                          ? "rgba(239,68,68,0.12)"
                          : "rgba(99,102,241,0.12)",
                  color: mode === "debug" ? "#f87171" : "#818cf8",
                  border:
                      mode === "debug"
                          ? "1px solid rgba(239,68,68,0.25)"
                          : "1px solid rgba(99,102,241,0.25)",
                  fontSize: "10px",
                  fontWeight: 700,
                }}
            />
          </Stack>
        </Box>

        <Box display="flex" gap={1.5} flexWrap="wrap">
          <StatBox
              label="Planets"
              value={planets.length}
              color="#93c5fd"
              subtitle={`${runningAis} AI running`}
          />

          <StatBox
              label="Explorers"
              value={explorers.length}
              color="#fbbf24"
              subtitle={`${
                  explorers.filter((e) => e.currentPlanetId !== null).length
              } stationed`}
          />

          <StatBox
              label="Backpack Items"
              value={totalBackpackItems}
              color="#f59e0b"
              subtitle="All explorers combined"
          />

          <StatBox
              label="Galaxy Routes"
              value={totalRoutes}
              color="#818cf8"
              subtitle="Bidirectional links"
          />

          <StatBox
              label="Generated"
              value={totalGenerated}
              color="#4ade80"
              subtitle="Resources generated"
          />

          <StatBox
              label="Rockets"
              value={rocketsBuilt}
              color="#c084fc"
              subtitle="Built by planets"
          />
        </Box>
      </StyledCard>
  );
}

function DebugPanel() {
  const { planets } = usePlanetStore();
  const { runState, tick, speed } = useSimulationStore();
  const [explorers, setExplorers] = useState<ExplorerEntry[]>([]);

  useEffect(() => {
    const load = () =>
        fetch(`${API_BASE}/explorers`)
            .then((r) => r.json())
            .then((data: ExplorerEntry[]) => setExplorers(data))
            .catch(() => {});

    load();

    const id = window.setInterval(load, 3000);

    return () => window.clearInterval(id);
  }, []);

  const aiRunning = planets.filter((p) => p.aiRunning).length;
  const aiStopped = planets.length - aiRunning;

  const totalBackpackItems = explorers.reduce(
      (sum, explorer) => sum + getExplorerBackpackTotal(explorer),
      0
  );

  return (
      <StyledCard>
        <Box
            display="flex"
            alignItems="center"
            justifyContent="space-between"
            mb={2}
        >
          <Typography sx={titleStyle}>Debug Metrics</Typography>
          <Chip label="DEBUG MODE" size="small" sx={chipStyle} />
        </Box>

        <Table
            size="small"
            sx={{
              "& td": {
                color: "#9ca3af",
                fontSize: 13,
                borderBottom: "1px solid rgba(255,255,255,0.04)",
                py: 0.75,
              },
            }}
        >
          <TableBody>
            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Run State
              </TableCell>
              <TableCell>{runState}</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Tick
              </TableCell>
              <TableCell>{tick}</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Speed
              </TableCell>
              <TableCell>{speed.toFixed(1)}x</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Planets Total
              </TableCell>
              <TableCell>{planets.length}</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                AI Running / Stopped
              </TableCell>
              <TableCell>
                {aiRunning} / {aiStopped}
              </TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Explorers
              </TableCell>
              <TableCell>{explorers.length}</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Backpack Items
              </TableCell>
              <TableCell>{totalBackpackItems}</TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Total Generated
              </TableCell>
              <TableCell>
                {planets.reduce((s, p) => s + p.totalResourcesGenerated, 0)}
              </TableCell>
            </TableRow>

            <TableRow>
              <TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>
                Rockets Built
              </TableCell>
              <TableCell>
                {planets.reduce((s, p) => s + p.rocketsBuilt, 0)}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </StyledCard>
  );
}

export function OverviewPage({ mode }: OverviewPageProps) {
  const { loadPlanets } = usePlanetStore();

  useEffect(() => {
    void loadPlanets();
  }, [loadPlanets]);

  return (
      <Stack spacing={2}>
        <ControlCenterDashboard mode={mode} />
        <SimulationBar />
        <PlanetsView />

        {mode === "debug" && <DebugPanel />}
      </Stack>
  );
}