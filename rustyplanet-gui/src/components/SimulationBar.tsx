import { Box, keyframes, Typography } from "@mui/material";



import { usePlanetStore } from "../store/planetStore";
import { useSimulationStore } from "../store/simulationStore";
import styled from "@emotion/styled";

const pulse = keyframes`
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.2);
  }
`;

const SimBar = styled(Box)({
  display: "flex",
  alignItems: "center",
  gap: "20px",
  padding: "16px 20px",
  background: "rgba(249,115,22,0.08)",
  border: "1px solid rgba(249,115,22,0.2)",
  borderRadius: "12px",
  marginTop: "24px",
});

const PulseDot = styled(Box)({
  width: "10px",
  height: "10px",
  borderRadius: "50%",
  animation: `${pulse} 2s ease-in-out infinite`,
});

const SimDivider = styled(Box)({
  width: "1px",
  height: "24px",
  background: "rgba(255,255,255,0.1)",
});

const SimTitles = {
  fontSize: "12px",
  color: "#9ca3af",
  textTransform: "uppercase",
  letterSpacing: "0.1em",
};

export function SimulationBar() {
  const { planets } = usePlanetStore();
  const { runState, tick, speed } = useSimulationStore();
  const stateColor = runState === "running"
    ? "#10b981"
    : runState === "paused"
    ? "#ffffff"
    : "#ef4444";

  return (
    <SimBar>
      <Box
        display="flex"
        alignItems="center"
        justifyContent="space-between"
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
          Simulation Status
        </Typography>
      </Box>

      <Box display="flex" alignItems="center" gap={2}>
        <PulseDot sx={{ background: stateColor }} />
        <Box>
          <Typography sx={SimTitles}>State</Typography>
          <Typography
            sx={{
              fontSize: "16px",
              fontWeight: 600,
              color: stateColor,
              textTransform: "uppercase",
            }}
          >
            {runState}
          </Typography>
        </Box>
      </Box>
      <SimDivider />
      <Box>
        <Typography sx={SimTitles}>Tick</Typography>
        <Typography sx={{ fontSize: "18px", fontWeight: 600 }}>
          {tick}
        </Typography>
      </Box>
      <SimDivider />
      <Box>
        <Typography sx={SimTitles}>Speed</Typography>
        <Typography
          sx={{ fontSize: "18px", fontWeight: 600, color: "#60a5fa" }}
        >
          {speed.toFixed(1)}x
        </Typography>
      </Box>
      <SimDivider />
      <Box>
        <Typography sx={SimTitles}>Planets</Typography>
        <Typography
          sx={{ fontSize: "18px", fontWeight: 500, color: "#fb923c" }}
        >
          {planets.length}
        </Typography>
      </Box>
    </SimBar>
  );
}
