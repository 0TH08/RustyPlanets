import { Box, Card, Chip, Stack, Typography } from "@mui/material";
import { useEffect } from "react";
import styled from "@emotion/styled";

import { usePlanetStore } from "../store/planetStore";
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

const infoBoxStyle = {
  p: 2,
  borderRadius: "12px",
  background: "rgba(255,255,255,0.02)",
  border: "1px solid rgba(255,255,255,0.06)",
};

const descriptionStyle = {
  fontSize: "13px",
  color: "#9ca3af",
  lineHeight: 1.7,
};

function DebugPanel() {
  return (
    <StyledCard>
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Typography sx={titleStyle}>Debug Metrics</Typography>

        <Chip label="DEBUG MODE" size="small" sx={chipStyle} />
      </Box>

      <Box sx={infoBoxStyle}>
        <Typography sx={descriptionStyle}>
          Internal engine metrics and diagnostics shown here in debug mode.
        </Typography>
      </Box>
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