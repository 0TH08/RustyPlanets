import { Box, Card, Chip, Stack, Typography, Table, TableBody, TableCell, TableRow } from "@mui/material";
import { useEffect, useState } from "react";
import styled from "@emotion/styled";

import { usePlanetStore } from "../store/planetStore";
import { useSimulationStore } from "../store/simulationStore";
import { SimulationBar } from "../components/SimulationBar";
import { PlanetsView } from "../components/PlanetsView";

type Mode = "player" | "debug";

interface OverviewPageProps {
  mode: Mode;
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



interface ExplorerEntry {
  id: number;
  currentPlanetId: number | null;
  aiRunning: boolean;
}

function DebugPanel() {
  const { planets } = usePlanetStore();
  const { runState, tick, speed } = useSimulationStore();
  const [explorers, setExplorers] = useState<ExplorerEntry[]>([]);

  useEffect(() => {
    const load = () =>
      fetch("http://localhost:8080/api/explorers")
        .then((r) => r.json())
        .then((data: ExplorerEntry[]) => setExplorers(data))
        .catch(() => {});
    load();
    const id = window.setInterval(load, 3000);
    return () => window.clearInterval(id);
  }, []);

  const aiRunning = planets.filter((p) => p.aiRunning).length;
  const aiStopped = planets.length - aiRunning;

  return (
    <StyledCard>
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Typography sx={titleStyle}>Debug Metrics</Typography>
        <Chip label="DEBUG MODE" size="small" sx={chipStyle} />
      </Box>

      <Table size="small" sx={{ "& td": { color: "#9ca3af", fontSize: 13, borderBottom: "1px solid rgba(255,255,255,0.04)", py: 0.75 } }}>
        <TableBody>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Run State</TableCell><TableCell>{runState}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Tick</TableCell><TableCell>{tick}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Speed</TableCell><TableCell>{speed.toFixed(1)}x</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Planets Total</TableCell><TableCell>{planets.length}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>AI Running / Stopped</TableCell><TableCell>{aiRunning} / {aiStopped}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Explorers</TableCell><TableCell>{explorers.length}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Total Generated (all planets)</TableCell><TableCell>{planets.reduce((s, p) => s + p.totalResourcesGenerated, 0)}</TableCell></TableRow>
          <TableRow><TableCell sx={{ fontWeight: 600, color: "#e5e7eb" }}>Rockets Built (all planets)</TableCell><TableCell>{planets.reduce((s, p) => s + p.rocketsBuilt, 0)}</TableCell></TableRow>
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
      <SimulationBar />
      <PlanetsView />

      {mode === "debug" && <DebugPanel />}
    </Stack>
  );
}