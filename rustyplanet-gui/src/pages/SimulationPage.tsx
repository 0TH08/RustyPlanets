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

type ToastState = {
  message: string;
  severity: "success" | "error" | "warning" | "info";
};

type ActionKey = "start" | "pause" | "step" | "reset" | "idle";

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

const BUTTONS: {
  label: string;
  onClick: ActionKey;
  variant: "contained" | "outlined";
  color: {
    bg: string;
    hover: string;
    border: string | null;
    text: string;
    hoverBorder?: string;
  };
}[] = [
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

const SPEED_PRESETS = [0.5, 1, 2, 5];

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
  const [toast, setToast] = useState<ToastState | null>(null);
  const [backendOnline, setBackendOnline] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const status = await getSimulationStatus();

        setRunState(status.runState);
        setSpeed(status.speed);
        setLocalSpeed(status.speed);
        setTick(status.tick);
        setBackendOnline(true);
      } catch {
        setBackendOnline(false);
      }
    };

    setBusy(true);
    load().finally(() => setBusy(false));

    const id = window.setInterval(load, 1000);

    return () => window.clearInterval(id);
  }, [setBusy, setRunState, setSpeed, setTick]);

  const withBusy = async (
      fn: () => Promise<unknown>,
      errorMessage = "Action failed — check if the backend is running."
  ) => {
    setBusy(true);

    try {
      setBackendOnline(true);
      return await fn();
    } catch {
      setBackendOnline(false);
      setToast({
        message: errorMessage,
        severity: "error",
      });
    } finally {
      setBusy(false);
    }
  };

  const actions: Record<ActionKey, () => Promise<unknown>> = {
    start: () =>
        withBusy(async () => {
          const s = await setSimulationRunState("running");

          setRunState(s.runState);
          setToast({
            message: "▶ Simulation running",
            severity: "success",
          });
        }),

    pause: () =>
        withBusy(async () => {
          const s = await setSimulationRunState("paused");

          setRunState(s.runState);
          setToast({
            message: "⏸ Simulation paused",
            severity: "info",
          });
        }),

    step: () =>
        withBusy(async () => {
          const s = await stepSimulation();

          setRunState(s.runState);
          setTick(s.tick ?? tick + 1);
          setToast({
            message: "⏭ Stepped one tick",
            severity: "success",
          });
        }),

    reset: async () => {
      if (mode === "debug") {
        const confirmed = window.confirm(
            "Reset the simulation? This will restart simulation state."
        );

        if (!confirmed) {
          setToast({
            message: "Reset cancelled",
            severity: "info",
          });

          return;
        }
      }

      return withBusy(async () => {
        const s = await resetSimulation();

        setRunState(s.runState);
        setSpeed(s.speed);
        setLocalSpeed(s.speed);
        setTick(0);
        setToast({
          message: "🔄 Simulation reset",
          severity: "warning",
        });
      });
    },

    idle: () =>
        withBusy(async () => {
          const s = await setSimulationRunState("idle");

          setRunState(s.runState);
          setTick(0);
          setToast({
            message: "⏹ Simulation stopped",
            severity: "info",
          });
        }),
  };

  const handleSpeedCommit = (_: unknown, value: number | number[]) =>
      withBusy(async () => {
        const v = Array.isArray(value) ? value[0] : value;

        setLocalSpeed(v);

        const s = await setSimulationSpeed(v);

        setSpeed(s.speed);
        setLocalSpeed(s.speed);
        setToast({
          message: `Speed set to ${s.speed.toFixed(1)}x`,
          severity: "success",
        });
      });

  const handleSpeedPreset = (preset: number) =>
      withBusy(async () => {
        setLocalSpeed(preset);

        const s = await setSimulationSpeed(preset);

        setSpeed(s.speed);
        setLocalSpeed(s.speed);
        setToast({
          message: `Speed set to ${s.speed.toFixed(1)}x`,
          severity: "success",
        });
      });

  const stateStyle =
      RUN_STATE[runState as keyof typeof RUN_STATE] ?? RUN_STATE.idle;

  const isIdle = runState === "idle";

  return (
      <StyledCard>
        {/* Header */}
        <Box display="flex" justifyContent="space-between" mb={2} gap={2}>
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

          <Stack direction="row" spacing={1} alignItems="center">
            <Chip
                label={backendOnline ? "BACKEND ONLINE" : "BACKEND OFFLINE"}
                size="small"
                sx={{
                  background: backendOnline
                      ? "rgba(34,197,94,0.15)"
                      : "rgba(239,68,68,0.15)",
                  color: backendOnline ? "#4ade80" : "#f87171",
                  border: backendOnline
                      ? "1px solid rgba(34,197,94,0.4)"
                      : "1px solid rgba(239,68,68,0.4)",
                  fontSize: 11,
                  fontWeight: 700,
                }}
            />

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
          </Stack>
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
              Control the simulation state, speed, and tick progression through
              the backend orchestrator API.
            </Typography>

            <Stack direction="row" spacing={1} flexWrap="wrap" mt={1}>
              <Chip
                  label={`State: ${runState}`}
                  size="small"
                  sx={{
                    background: stateStyle.bg,
                    color: stateStyle.color,
                    border: stateStyle.border,
                    fontSize: "11px",
                    fontWeight: 700,
                  }}
              />

              <Chip
                  label={`Tick: ${tick}`}
                  size="small"
                  sx={{
                    background: "rgba(255,255,255,0.04)",
                    color: "#d1d5db",
                    border: "1px solid rgba(255,255,255,0.08)",
                    fontSize: "11px",
                  }}
              />

              <Chip
                  label={`Speed: ${speed.toFixed(1)}x`}
                  size="small"
                  sx={{
                    background: "rgba(99,102,241,0.1)",
                    color: "#a5b4fc",
                    border: "1px solid rgba(99,102,241,0.2)",
                    fontSize: "11px",
                  }}
              />

              <Chip
                  label={`${(500.0 / Math.max(speed, 0.1)).toFixed(0)} ms/tick`}
                  size="small"
                  sx={{
                    background: "rgba(255,255,255,0.04)",
                    color: "#9ca3af",
                    border: "1px solid rgba(255,255,255,0.08)",
                    fontSize: "11px",
                  }}
              />
            </Stack>
          </Box>

          {/* Backend warning */}
          {!backendOnline && (
              <Alert
                  severity="error"
                  sx={{
                    background: "rgba(239,68,68,0.1)",
                    color: "#fecaca",
                    border: "1px solid rgba(239,68,68,0.25)",
                    "& .MuiAlert-icon": {
                      color: "#f87171",
                    },
                  }}
              >
                Backend is not responding. Make sure Rust server is running on
                localhost:8080.
              </Alert>
          )}

          {/* Idle warning */}
          {isIdle && (
              <Alert
                  severity="warning"
                  sx={{
                    background: "rgba(249,115,22,0.08)",
                    color: "#fed7aa",
                    border: "1px solid rgba(249,115,22,0.25)",
                    "& .MuiAlert-icon": {
                      color: "#fb923c",
                    },
                  }}
              >
                Simulation is idle. Start the simulation or use Step to advance one
                tick manually.
              </Alert>
          )}

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
                    variant={btn.variant}
                    disabled={
                        isBusy ||
                        !backendOnline ||
                        (btn.onClick === "start" && runState === "running") ||
                        (btn.onClick === "pause" && runState !== "running")
                    }
                    onClick={actions[btn.onClick]}
                    sx={{
                      background:
                          btn.variant === "contained" ? btn.color.bg : "transparent",

                      borderColor: btn.color.border ?? undefined,
                      color: btn.color.text,

                      borderRadius: "10px",
                      textTransform: "none",
                      fontWeight: 600,

                      "&:hover": {
                        background: btn.color.hover,
                        borderColor: btn.color.hoverBorder ?? undefined,
                      },

                      "&.Mui-disabled": {
                        background: "rgba(255,255,255,0.05)",
                        color: "rgba(255,255,255,0.25)",
                        borderColor: "rgba(255,255,255,0.08)",
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
            <Box
                display="flex"
                justifyContent="space-between"
                alignItems="center"
                gap={2}
                flexWrap="wrap"
                mb={2}
            >
              <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
                Simulation Speed ({localSpeed.toFixed(1)}x)
              </Typography>

              <Stack direction="row" spacing={1} flexWrap="wrap">
                {SPEED_PRESETS.map((preset) => (
                    <Button
                        key={preset}
                        size="small"
                        variant={speed === preset ? "contained" : "outlined"}
                        disabled={isBusy || !backendOnline}
                        onClick={() => handleSpeedPreset(preset)}
                        sx={{
                          minWidth: 48,
                          borderRadius: "999px",
                          textTransform: "none",
                          fontSize: "11px",
                          fontWeight: 700,
                          background:
                              speed === preset
                                  ? "rgba(99,102,241,0.22)"
                                  : "transparent",
                          color: speed === preset ? "#c7d2fe" : "#818cf8",
                          borderColor:
                              speed === preset
                                  ? "rgba(129,140,248,0.5)"
                                  : "rgba(99,102,241,0.25)",
                          "&:hover": {
                            background: "rgba(99,102,241,0.12)",
                            borderColor: "rgba(129,140,248,0.55)",
                          },
                        }}
                    >
                      {preset.toFixed(preset % 1 === 0 ? 0 : 1)}x
                    </Button>
                ))}
              </Stack>
            </Box>

            <Slider
                min={0.1}
                max={5}
                step={0.1}
                value={localSpeed}
                disabled={isBusy || !backendOnline}
                onChange={(_, v) => setLocalSpeed(Array.isArray(v) ? v[0] : v)}
                onChangeCommitted={handleSpeedCommit}
                sx={{
                  color: "#818cf8",
                  "& .MuiSlider-thumb": { width: 14, height: 14 },
                  "& .MuiSlider-track": { border: "none" },
                  "&.Mui-disabled": {
                    color: "rgba(255,255,255,0.15)",
                  },
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
                <Typography
                    sx={{
                      fontSize: 13,
                      color: "#f87171",
                      fontWeight: 600,
                      mb: 1,
                    }}
                >
                  Debug Diagnostics
                </Typography>

                <Typography
                    sx={{
                      fontSize: 12,
                      color: "#9ca3af",
                      fontFamily: "monospace",
                      lineHeight: 1.8,
                      whiteSpace: "pre-line",
                    }}
                >
                  {`backend:   ${backendOnline ? "online" : "offline"}\n`}
                  {`runState:  ${runState}\n`}
                  {`tick:      ${tick}\n`}
                  {`speed:     ${speed.toFixed(2)}x\n`}
                  {`speed_ms:  ${(500.0 / Math.max(speed, 0.1)).toFixed(0)} ms/tick\n`}
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
              severity={toast?.severity ?? "success"}
              variant="filled"
              sx={{ width: "100%" }}
          >
            {toast?.message}
          </Alert>
        </Snackbar>
      </StyledCard>
  );
}