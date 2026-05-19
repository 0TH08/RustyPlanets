import {
  Alert,
  Box,
  Button,
  Chip,
  CircularProgress,
  Snackbar,
  Stack,
  Typography,
  Card,
  styled,
} from "@mui/material";

import { useEffect, useState } from "react";
import { EnergyCellsView } from "../components/EnergyCellsView";
import { usePlanetStore } from "../store/planetStore";
import {
  startPlanet,
  stopPlanet,
} from "../services/planetService";
import {
  sendSunray,
  sendAsteroid,
} from "../services/simulationService";

type Mode = "player" | "debug";

interface PlanetPageProps {
  mode?: Mode;
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

const SECTION_TITLE = (text: string) => (
  <Typography
    sx={{
      fontSize: 14,
      fontWeight: 500,
      color: "#9ca3af",
      textTransform: "uppercase",
      letterSpacing: "0.1em",
      mb: 2,
    }}
  >
    {text}
  </Typography>
);

const BTN = {
  base: {
    borderRadius: "10px",
    textTransform: "none",
    fontWeight: 600,
    boxShadow: "none",
  },

  start: {
    bg: "linear-gradient(135deg, #22c55e 0%, #16a34a 100%)",
    hover: "linear-gradient(135deg, #16a34a 0%, #15803d 100%)",
    color: "#fff",
  },

  stop: {
    border: "rgba(249,115,22,0.3)",
    color: "#fb923c",
    hoverBg: "rgba(249,115,22,0.08)",
  },

  sunray: {
    border: "rgba(14,165,233,0.3)",
    color: "#38bdf8",
    hoverBg: "rgba(14,165,233,0.08)",
  },

  asteroid: {
    border: "rgba(239,68,68,0.3)",
    color: "#f87171",
    hoverBg: "rgba(239,68,68,0.08)",
  },
};

export function PlanetPage({ mode }: PlanetPageProps) {
  const {
    planets,
    selectedPlanet,
    selectedPlanetId,
    isLoadingDetails,
    selectPlanet,
    loadPlanets,
  } = usePlanetStore();

  const p = selectedPlanet;

  const [toast, setToast] = useState<{ msg: string; severity: "success" | "error" } | null>(null);

  const showToast = (msg: string, severity: "success" | "error" = "success") =>
    setToast({ msg, severity });

  useEffect(() => {
    loadPlanets().then(() => {
      const { selectedPlanetId, planets, selectPlanet } = usePlanetStore.getState();
      if (selectedPlanetId == null && planets.length > 0) {
        selectPlanet(planets[0].id);
      }
    });
    const id = setInterval(() => void loadPlanets(), 1500);
    return () => clearInterval(id);
  }, [loadPlanets]);

  const runIfPlanet = async (
    fn: (id: number) => Promise<any>,
    successMsg: string
  ) => {
    if (!selectedPlanetId) return;
    try {
      await fn(selectedPlanetId);
      await selectPlanet(selectedPlanetId);
      showToast(successMsg, "success");
    } catch {
      showToast("Action failed — is the backend running?", "error");
    }
  };

  const stats = p && [
    ["Explorers", p.summary.explorerCount],
    ["Resources Generated", p.summary.totalResourcesGenerated],
    ["Rockets Built", p.summary.rocketsBuilt],
    ["Asteroids Deflected", p.summary.asteroidsDeflected],
    ["Errors", p.summary.errorsEncountered],
    ["Arrivals", p.explorerArrivals],
    ["Departures", p.explorerDepartures],
  ];

  return (
    <Stack spacing={2}>
      {/* Planet Details */}
      <StyledCard>
        {SECTION_TITLE("Planet Details")}

        {/* Planet selector */}
        {planets.length > 0 && (
          <Stack direction="row" spacing={1} flexWrap="wrap" mb={2}>
            {planets.map((planet) => (
              <Chip
                key={planet.id}
                label={planet.name}
                size="small"
                onClick={() => selectPlanet(planet.id)}
                variant={selectedPlanetId === planet.id ? "filled" : "outlined"}
                sx={{
                  cursor: "pointer",
                  fontSize: 11,
                  fontWeight: 600,
                  ...(selectedPlanetId === planet.id
                    ? {
                        background: "rgba(99,102,241,0.25)",
                        color: "#a5b4fc",
                        border: "1px solid rgba(99,102,241,0.5)",
                      }
                    : {
                        background: "transparent",
                        color: "#6b7280",
                        border: "1px solid rgba(255,255,255,0.1)",
                        "&:hover": {
                          background: "rgba(255,255,255,0.04)",
                          color: "#d1d5db",
                          border: "1px solid rgba(255,255,255,0.2)",
                        },
                      }),
                }}
              />
            ))}
          </Stack>
        )}

        <Box display="flex" justifyContent="space-between" mb={2}>
          {p && (
            <Chip
              label={p.summary.aiRunning ? "AI Running" : "AI Stopped"}
              size="small"
              sx={{
                background: p.summary.aiRunning
                  ? "rgba(34,197,94,0.15)"
                  : "rgba(107,114,128,0.2)",
                color: p.summary.aiRunning ? "#4ade80" : "#9ca3af",
                border: p.summary.aiRunning
                  ? "1px solid rgba(34,197,94,0.4)"
                  : "1px solid rgba(107,114,128,0.3)",
                fontSize: 11,
                fontWeight: 600,
              }}
            />
          )}
        </Box>

        {isLoadingDetails && (
          <Stack direction="row" spacing={1} alignItems="center">
            <CircularProgress size={16} />
            <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
              Loading...
            </Typography>
          </Stack>
        )}

        {!isLoadingDetails && !p && (
          <Typography sx={{ fontSize: 13, color: "#6b7280" }}>
            {selectedPlanetId == null
              ? "Select a planet from the Overview tab."
              : "Planet details not available."}
          </Typography>
        )}

        {p && (
          <Stack spacing={2}>
            {/* Header */}
            <Box
              sx={{
                p: 2,
                borderRadius: 2,
                background: "rgba(255,255,255,0.02)",
                border: "1px solid rgba(255,255,255,0.06)",
              }}
            >
              <Typography sx={{ fontSize: 24, fontWeight: 700, color: "#fff" }}>
                {p.summary.name}
              </Typography>

              <Chip
                size="small"
                label={p.summary.kind}
                sx={{
                  background: "rgba(99,102,241,0.12)",
                  color: "#818cf8",
                  border: "1px solid rgba(99,102,241,0.25)",
                  fontWeight: 600,
                  mt: 1,
                }}
              />

              <Typography sx={{ fontSize: 12, color: "#6b7280", mt: 1 }}>
                ID: {p.summary.id}
              </Typography>
            </Box>

            {/* Actions */}
            <Stack direction="row" spacing={1} flexWrap="wrap">
              <Button
                variant="contained"
                onClick={() => runIfPlanet(startPlanet, "AI started")}
                sx={{
                  ...BTN.base,
                  background: BTN.start.bg,
                  color: BTN.start.color,
                  "&:hover": { background: BTN.start.hover },
                }}
              >
                Start AI
              </Button>

              <Button
                variant="outlined"
                onClick={() => runIfPlanet(stopPlanet, "AI stopped")}
                sx={{
                  ...BTN.base,
                  borderColor: BTN.stop.border,
                  color: BTN.stop.color,
                  "&:hover": {
                    borderColor: BTN.stop.color,
                    background: BTN.stop.hoverBg,
                  },
                }}
              >
                Stop AI
              </Button>

              <Button
                variant="outlined"
                onClick={() => runIfPlanet(sendSunray, "☀️ Sunray sent!")}
                sx={{
                  ...BTN.base,
                  borderColor: BTN.sunray.border,
                  color: BTN.sunray.color,
                  "&:hover": {
                    borderColor: BTN.sunray.color,
                    background: BTN.sunray.hoverBg,
                  },
                }}
              >
                ☀️ Sunray
              </Button>

              <Button
                variant="outlined"
                onClick={() => runIfPlanet(sendAsteroid, "☄️ Asteroid launched!")}
                sx={{
                  ...BTN.base,
                  borderColor: BTN.asteroid.border,
                  color: BTN.asteroid.color,
                  "&:hover": {
                    borderColor: BTN.asteroid.color,
                    background: BTN.asteroid.hoverBg,
                  },
                }}
              >
                ☄️ Asteroid
              </Button>
            </Stack>
          </Stack>
        )}
      </StyledCard>

      {/* Energy Cells */}
      <StyledCard>
        {SECTION_TITLE("Energy Cells")}

        {p ? (
          <EnergyCellsView
            total={p.cells.length}
            charged={p.cells.filter((c) => c.charged).length}
            resourceTotal={Math.min(3, p.cells.length)}
            resourceCharged={
              p.cells.slice(0, 3).filter((c) => c.charged).length
            }
            defenseTotal={Math.max(0, p.cells.length - 3)}
            defenseCharged={p.cells.slice(3).filter((c) => c.charged).length}
          />
        ) : (
          <Typography sx={{ fontSize: 13, color: "#6b7280" }}>
            Select a planet to view energy cells.
          </Typography>
        )}
      </StyledCard>

      {/* Statistics */}
      <StyledCard>
        {SECTION_TITLE("Statistics")}

        {p ? (
          <Stack spacing={1}>
            {stats!.map(([label, value]) => (
              <Box
                key={label}
                sx={{
                  display: "flex",
                  justifyContent: "space-between",
                  p: 1.25,
                  borderRadius: "10px",
                  background: "rgba(255,255,255,0.02)",
                  border: "1px solid rgba(255,255,255,0.05)",
                }}
              >
                <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
                  {label}
                </Typography>
                <Typography sx={{ fontSize: 13, fontWeight: 700, color: "#fff" }}>
                  {value}
                </Typography>
              </Box>
            ))}
          </Stack>
        ) : (
          <Typography sx={{ fontSize: 13, color: "#6b7280" }}>
            Select a planet to view statistics.
          </Typography>
        )}
      </StyledCard>

      {/* History */}
      <StyledCard>
        {SECTION_TITLE("Generation History")}

        {p?.generationHistory?.length ? (
          <Stack spacing={1}>
            {p.generationHistory.slice(-10).reverse().map((e, i) => (
              <Box
                key={i}
                sx={{
                  p: 1.25,
                  borderRadius: "10px",
                  background: "rgba(255,255,255,0.02)",
                  border: "1px solid rgba(255,255,255,0.05)",
                }}
              >
                <Typography sx={{ fontSize: 12, color: "#9ca3af", fontFamily: "monospace" }}>
                  {e.resource}
                </Typography>
              </Box>
            ))}
          </Stack>
        ) : (
          <Typography sx={{ fontSize: 13, color: "#6b7280" }}>
            {p ? "No resources generated yet." : "Select a planet to view history."}
          </Typography>
        )}
      </StyledCard>

      {/* Debug panel */}
      {mode === "debug" && p && (
        <StyledCard>
          {SECTION_TITLE("Debug — Raw Telemetry")}
          <Box sx={{ p: 1.5, borderRadius: 2, background: "rgba(239,68,68,0.04)", border: "1px solid rgba(239,68,68,0.1)" }}>
            <Typography sx={{ fontSize: 12, color: "#f87171", fontFamily: "monospace", lineHeight: 1.8 }}>
              {`id:            ${p.summary.id}`}{'\n'}
              {`name:          ${p.summary.name}`}{'\n'}
              {`kind:          ${p.summary.kind}`}{'\n'}
              {`aiRunning:     ${p.summary.aiRunning}`}{'\n'}
              {`explorerCount: ${p.summary.explorerCount}`}{'\n'}
              {`cells:         ${p.cells.length} total, ${p.cells.filter(c => c.charged).length} charged`}{'\n'}
              {`errors:        ${p.summary.errorsEncountered}`}
            </Typography>
          </Box>
        </StyledCard>
      )}

      {/* Toast */}
      <Snackbar
        open={toast !== null}
        autoHideDuration={3000}
        onClose={() => setToast(null)}
        anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
      >
        <Alert
          onClose={() => setToast(null)}
          severity={toast?.severity ?? "success"}
          variant="filled"
          sx={{ width: "100%" }}
        >
          {toast?.msg}
        </Alert>
      </Snackbar>
    </Stack>
  );
}