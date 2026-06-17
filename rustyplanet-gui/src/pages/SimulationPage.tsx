import {
  Alert,
  Box,
  Button,
  Chip,
  Slider,
  Snackbar,
  Stack,
  Typography,
  Card,
  styled,
  CircularProgress,
} from "@mui/material";

import { useEffect, useState } from "react";
import {
  getSimulationStatus,
  setSimulationRunState,
  setSimulationSpeed,
  stepSimulation,
  resetSimulation,
} from "../services/simulationService";

import { useSimulationStore } from "../store/simulationStore";

type Mode = "player" | "debug";

interface SimulationPageProps {
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

const RUN_STATE = {
  running: {
    bg: "rgba(34,197,94,0.15)",
    color: "#4ade80",
    border: "1px solid rgba(34,197,94,0.4)",
  },
  paused: {
    bg: "rgba(249,115,22,0.15)",
    color: "#fb923c",
    border: "1px solid rgba(249,115,22,0.35)",
  },
  idle: {
    bg: "rgba(107,114,128,0.2)",
    color: "#9ca3af",
    border: "1px solid rgba(107,114,128,0.3)",
  },
} as const;

const BUTTONS = [
  {
    label: "▶ Start",
    onClick: "start",
    variant: "contained",
    color: {
      bg: "linear-gradient(135deg, #22c55e 0%, #16a34a 100%)",
      hover: "linear-gradient(135deg, #16a34a 0%, #15803d 100%)",
      border: null,
      text: "#fff",
    },
  },
  {
    label: "⏸ Pause",
    onClick: "pause",
    variant: "outlined",
    color: {
      bg: "transparent",
      hover: "rgba(249,115,22,0.08)",
      border: "rgba(249,115,22,0.3)",
      text: "#fb923c",
      hoverBorder: "#fb923c",
    },
  },
  {
    label: "⏭ Step",
    onClick: "step",
    variant: "outlined",
    color: {
      bg: "transparent",
      hover: "rgba(99,102,241,0.08)",
      border: "rgba(99,102,241,0.3)",
      text: "#818cf8",
      hoverBorder: "#818cf8",
    },
  },
  {
    label: "🔄 Reset",
    onClick: "reset",
    variant: "outlined",
    color: {
      bg: "transparent",
      hover: "rgba(14,165,233,0.08)",
      border: "rgba(14,165,233,0.3)",
      text: "#38bdf8",
      hoverBorder: "#38bdf8",
    },
  },
  {
    label: "⏹ Stop",
    onClick: "idle",
    variant: "outlined",
    color: {
      bg: "transparent",
      hover: "rgba(239,68,68,0.08)",
      border: "rgba(239,68,68,0.3)",
      text: "#f87171",
      hoverBorder: "#f87171",
    },
  },
];

export function SimulationPage({ mode }: SimulationPageProps) {
  const {
    runState,
    tick,
    speed,
    isBusy,
    setRunState,
    setSpeed,
    setTick,
    setBusy,
  } = useSimulationStore();

  const [localSpeed, setLocalSpeed] = useState(speed);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const status = await getSimulationStatus();
        setRunState(status.runState);
        setSpeed(status.speed);
        setTick(status.tick);
      } catch {
        // backend not ready yet, keep existing state
      }
    };

    setBusy(true);
    load().finally(() => setBusy(false));

    const id = window.setInterval(load, 1000);
    return () => window.clearInterval(id);
  }, [setBusy, setRunState, setSpeed, setTick]);

  const withBusy = async (fn: () => Promise<unknown>) => {
    setBusy(true);
    try {
      return await fn();
    } finally {
      setBusy(false);
    }
  };

  const actions = {
    start: () =>
      withBusy(async () => {
        const s = await setSimulationRunState("running");
        setRunState(s.runState);
        setToast("▶ Simulation running");
      }),

    pause: () =>
      withBusy(async () => {
        const s = await setSimulationRunState("paused");
        setRunState(s.runState);
        setToast("⏸ Simulation paused");
      }),

    step: () =>
      withBusy(async () => {
        const s = await stepSimulation();
        setRunState(s.runState);
        setToast("⏭ Stepped one tick");
      }),

    reset: () =>
      withBusy(async () => {
        const s = await resetSimulation();
        setRunState(s.runState);
        setSpeed(s.speed);
        setTick(0);
        setToast("🔄 Simulation reset");
      }),

    idle: () =>
      withBusy(async () => {
        const s = await setSimulationRunState("idle");
        setRunState(s.runState);
        setTick(0);
        setToast("⏹ Simulation stopped");
      }),
  };

  const handleSpeedCommit = (_: unknown, value: number | number[]) =>
    withBusy(async () => {
      const v = Array.isArray(value) ? value[0] : value;
      setLocalSpeed(v);
      const s = await setSimulationSpeed(v);
      setSpeed(s.speed);
    });

  const stateStyle =
    RUN_STATE[runState as keyof typeof RUN_STATE] ?? RUN_STATE.idle;

  return (
    <StyledCard>
      {/* Header */}
      <Box display="flex" justifyContent="space-between" mb={2}>
        <Typography
          sx={{
            fontSize: 14,
            fontWeight: 500,
            color: "#9ca3af",
            textTransform: "uppercase",
            letterSpacing: "0.1em",
          }}
        >
          Simulation Controls
        </Typography>

        <Chip
          label={runState.toUpperCase()}
          size="small"
          sx={{
            background: stateStyle.bg,
            color: stateStyle.color,
            border: stateStyle.border,
            fontSize: 11,
            fontWeight: 700,
          }}
        />
      </Box>

      <Stack spacing={2}>
        {/* Info */}
        <Box
          sx={{
            p: 2,
            borderRadius: 2,
            background: "rgba(255,255,255,0.02)",
            border: "1px solid rgba(255,255,255,0.06)",
          }}
        >
          <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
            Control the simulation state, speed, and tick progression.
          </Typography>

          <Typography sx={{ fontSize: 13, color: "#fff", mt: 1 }}>
            State: <strong>{runState}</strong> · Tick:{" "}
            <strong>{tick}</strong>
          </Typography>
        </Box>

        {/* Busy */}
        {isBusy && (
          <Stack direction="row" spacing={1} alignItems="center">
            <CircularProgress size={16} />
            <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
              Updating simulation...
            </Typography>
          </Stack>
        )}

        {/* Buttons */}
        <Stack direction="row" spacing={1} flexWrap="wrap">
          {BUTTONS.map((btn) => (
            <Button
              key={btn.label}
              variant={btn.variant as "text" | "outlined" | "contained"}
              disabled={
                isBusy ||
                (btn.onClick === "start" && runState === "running") ||
                (btn.onClick === "pause" && runState !== "running")
              }
              onClick={actions[btn.onClick as keyof typeof actions]}
              sx={{
                background:
                  btn.variant === "contained"
                    ? btn.color.bg
                    : "transparent",

                borderColor: btn.color.border ?? undefined,
                color: btn.color.text,

                borderRadius: "10px",
                textTransform: "none",
                fontWeight: 600,

                "&:hover": {
                  background: btn.color.hover,
                  borderColor: btn.color.hoverBorder ?? undefined,
                },
              }}
            >
              {btn.label}
            </Button>
          ))}
        </Stack>

        {/* Slider */}
        <Box
          sx={{
            p: 2,
            borderRadius: 2,
            background: "rgba(255,255,255,0.02)",
            border: "1px solid rgba(255,255,255,0.06)",
          }}
        >
          <Typography sx={{ fontSize: 13, color: "#9ca3af", mb: 2 }}>
            Simulation Speed ({localSpeed.toFixed(1)}x)
          </Typography>

          <Slider
            min={0.1}
            max={5}
            step={0.1}
            value={localSpeed}
            onChange={(_, v) =>
              setLocalSpeed(Array.isArray(v) ? v[0] : v)
            }
            onChangeCommitted={handleSpeedCommit}
            sx={{
              color: "#818cf8",
              "& .MuiSlider-thumb": { width: 14, height: 14 },
              "& .MuiSlider-track": { border: "none" },
            }}
          />
        </Box>

        {/* Debug */}
        {mode === "debug" && (
          <Box
            sx={{
              p: 2,
              borderRadius: 2,
              background: "rgba(239,68,68,0.04)",
              border: "1px solid rgba(239,68,68,0.12)",
            }}
          >
            <Typography sx={{ fontSize: 13, color: "#f87171", fontWeight: 600, mb: 1 }}>
              Debug Diagnostics
            </Typography>
            <Typography sx={{ fontSize: 12, color: "#9ca3af", fontFamily: "monospace", lineHeight: 1.8 }}>
              {`runState: ${runState}`}{'\n'}
              {`tick:      ${tick}`}{'\n'}
              {`speed:     ${speed.toFixed(2)}x`}{'\n'}
              {`speed_ms:  ${(500.0 / Math.max(speed, 0.1)).toFixed(0)} ms/tick`}{'\n'}
              {`isBusy:    ${isBusy}`}
            </Typography>
          </Box>
        )}
      </Stack>

      <Snackbar
        open={toast !== null}
        autoHideDuration={2500}
        onClose={() => setToast(null)}
        anchorOrigin={{ vertical: "bottom", horizontal: "center" }}
      >
        <Alert
          onClose={() => setToast(null)}
          severity="success"
          variant="filled"
          sx={{ width: "100%" }}
        >
          {toast}
        </Alert>
      </Snackbar>
    </StyledCard>
  );
}