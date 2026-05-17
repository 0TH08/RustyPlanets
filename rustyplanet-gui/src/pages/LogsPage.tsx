import {
  Box,
  Button,
  Card,
  Chip,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  Typography,
  styled,
} from "@mui/material";

import { useCallback, useEffect, useMemo } from "react";
import { useLogStore } from "../store/logStore";

type Mode = "player" | "debug";

interface LogsPageProps {
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

const LEVEL_STYLES = {
  error: {
    bg: "rgba(239,68,68,0.06)",
    border: "1px solid rgba(239,68,68,0.12)",
    chipBg: "rgba(239,68,68,0.15)",
    chipColor: "#f87171",
    chipBorder: "1px solid rgba(239,68,68,0.3)",
  },

  warn: {
    bg: "rgba(249,115,22,0.06)",
    border: "1px solid rgba(249,115,22,0.12)",
    chipBg: "rgba(249,115,22,0.15)",
    chipColor: "#fb923c",
    chipBorder: "1px solid rgba(249,115,22,0.3)",
  },

  info: {
    bg: "rgba(59,130,246,0.05)",
    border: "1px solid rgba(59,130,246,0.1)",
    chipBg: "rgba(59,130,246,0.15)",
    chipColor: "#60a5fa",
    chipBorder: "1px solid rgba(59,130,246,0.25)",
  },

  debug: {
    bg: "rgba(255,255,255,0.02)",
    border: "1px solid rgba(255,255,255,0.05)",
    chipBg: "rgba(107,114,128,0.2)",
    chipColor: "#9ca3af",
    chipBorder: "1px solid rgba(107,114,128,0.3)",
  },
};

const levels = ["all", "debug", "info", "warn", "error"];

export function LogsPage({ mode }: LogsPageProps) {
  const {
    logs,
    levelFilter,
    isStreaming,
    startStream,
    stopStream,
    setLevelFilter,
  } = useLogStore();

  useEffect(() => {
    startStream();
    return () => stopStream();
  }, []);

  const visibleLogs = useMemo(() => {
    if (levelFilter === "all") return logs;

    return logs.filter((log) => log.level === levelFilter);
  }, [logs, levelFilter]);

  const streamLabel = isStreaming ? "STREAMING" : "PAUSED";

  const downloadLogs = useCallback(() => {
    const lines = logs.map((log) => {
      const time = new Date(log.timestamp).toLocaleTimeString();
      const planet = log.planetId != null ? ` [planet ${log.planetId}]` : "";
      return `[${time}] [${log.level.toUpperCase().padEnd(5)}]${planet} ${log.message}`;
    });
    const blob = new Blob([lines.join("\n")], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `rustyplanets-logs-${new Date()
      .toISOString()
      .slice(0, 19)
      .replace(/:/g, "-")}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  }, [logs]);

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
            fontSize: 14,
            fontWeight: 500,
            color: "#9ca3af",
            textTransform: "uppercase",
            letterSpacing: "0.1em",
          }}
        >
          System Logs
        </Typography>

        <Chip
          label={streamLabel}
          size="small"
          sx={{
            background: isStreaming
              ? "rgba(34,197,94,0.15)"
              : "rgba(107,114,128,0.2)",

            color: isStreaming ? "#4ade80" : "#9ca3af",

            border: isStreaming
              ? "1px solid rgba(34,197,94,0.4)"
              : "1px solid rgba(107,114,128,0.3)",

            fontSize: 11,
            fontWeight: 700,
            letterSpacing: "0.05em",
          }}
        />
      </Box>

      <Stack spacing={2}>
        {/* Controls */}
        <Box
          sx={{
            p: 2,
            borderRadius: 3,
            background: "rgba(255,255,255,0.02)",
            border: "1px solid rgba(255,255,255,0.06)",
          }}
        >
          <Stack
            direction="row"
            spacing={2}
            alignItems="center"
            flexWrap="wrap"
          >
            <FormControl size="small">
              <InputLabel
                id="log-level-label"
                sx={{ color: "#9ca3af" }}
              >
                Level
              </InputLabel>

              <Select
                labelId="log-level-label"
                label="Level"
                value={levelFilter}
                onChange={(e) =>
                  setLevelFilter(
                    e.target.value as typeof levelFilter
                  )
                }
                sx={{
                  minWidth: 140,
                  borderRadius: "10px",
                  color: "#fff",

                  "& .MuiOutlinedInput-notchedOutline": {
                    borderColor: "rgba(255,255,255,0.1)",
                  },

                  "&:hover .MuiOutlinedInput-notchedOutline": {
                    borderColor: "rgba(99,102,241,0.4)",
                  },

                  "&.Mui-focused .MuiOutlinedInput-notchedOutline": {
                    borderColor: "#818cf8",
                  },

                  "& .MuiSvgIcon-root": {
                    color: "#9ca3af",
                  },
                }}
              >
                {levels.map((level) => (
                  <MenuItem
                    key={level}
                    value={level}
                  >
                    {level[0].toUpperCase() +
                      level.slice(1)}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            <Button
              size="small"
              variant={
                isStreaming ? "outlined" : "contained"
              }
              onClick={
                isStreaming
                  ? stopStream
                  : startStream
              }
              sx={{
                background: !isStreaming
                  ? "linear-gradient(135deg, #22c55e 0%, #16a34a 100%)"
                  : "transparent",

                borderColor: isStreaming
                  ? "rgba(249,115,22,0.3)"
                  : undefined,

                color: isStreaming
                  ? "#fb923c"
                  : "#fff",

                borderRadius: "10px",
                textTransform: "none",
                fontWeight: 600,
                boxShadow: "none",

                "&:hover": {
                  background: isStreaming
                    ? "rgba(249,115,22,0.08)"
                    : "linear-gradient(135deg, #16a34a 0%, #15803d 100%)",
                },
              }}
            >
              {isStreaming
                ? "Pause Stream"
                : "Start Stream"}
            </Button>

            <Button
              size="small"
              variant="outlined"
              onClick={downloadLogs}
              disabled={logs.length === 0}
              sx={{
                borderColor: "rgba(99,102,241,0.35)",
                color: "#818cf8",
                borderRadius: "10px",
                textTransform: "none",
                fontWeight: 600,
                boxShadow: "none",
                "&:hover": {
                  borderColor: "rgba(99,102,241,0.6)",
                  background: "rgba(99,102,241,0.08)",
                },
                "&.Mui-disabled": {
                  borderColor: "rgba(255,255,255,0.1)",
                  color: "rgba(255,255,255,0.2)",
                },
              }}
            >
              Download Logs ({logs.length})
            </Button>

            {mode !== "debug" && (
              <Typography
                sx={{
                  fontSize: 12,
                  color: "#fb923c",
                }}
              >
                In player mode, very low-level logs may
                be hidden in the future.
              </Typography>
            )}
          </Stack>
        </Box>

        {/* Terminal */}
        <Box
          sx={{
            p: 2,
            borderRadius: 3,
            background: "#050505",
            border: "1px solid rgba(255,255,255,0.06)",
            fontFamily: "monospace",
            maxHeight: 420,
            overflowY: "auto",

            "&::-webkit-scrollbar": {
              width: 8,
            },

            "&::-webkit-scrollbar-thumb": {
              background: "rgba(255,255,255,0.08)",
              borderRadius: 10,
            },
          }}
        >
          <Stack spacing={1}>
            {visibleLogs.map((log) => {
              const styles =
                LEVEL_STYLES[log.level] ??
                LEVEL_STYLES.debug;

              return (
                <Box
                  key={log.id}
                  sx={{
                    p: 1,
                    borderRadius: "10px",
                    background: styles.bg,
                    border: styles.border,
                  }}
                >
                  <Stack
                    direction="row"
                    spacing={1}
                    alignItems="center"
                    flexWrap="wrap"
                  >
                    <Typography
                      component="span"
                      sx={{
                        fontSize: 11,
                        color: "#6b7280",
                      }}
                    >
                      {new Date(
                        log.timestamp
                      ).toLocaleTimeString()}
                    </Typography>

                    <Chip
                      size="small"
                      label={log.level.toUpperCase()}
                      sx={{
                        height: 18,
                        background: styles.chipBg,
                        color: styles.chipColor,
                        border: styles.chipBorder,

                        "& .MuiChip-label": {
                          px: 0.75,
                          fontWeight: 700,
                          fontSize: 10,
                        },
                      }}
                    />

                    {log.planetId != null && (
                      <Typography
                        component="span"
                        sx={{
                          fontSize: 11,
                          color: "#818cf8",
                        }}
                      >
                        [planet {log.planetId}]
                      </Typography>
                    )}

                    <Typography
                      component="span"
                      sx={{
                        fontSize: 12,
                        color: "#e5e7eb",
                        wordBreak: "break-word",
                      }}
                    >
                      {log.message}
                    </Typography>
                  </Stack>
                </Box>
              );
            })}

            {!visibleLogs.length && (
              <Typography
                sx={{
                  fontSize: 13,
                  color: "#6b7280",
                }}
              >
                No logs yet. The stream will begin
                producing entries shortly.
              </Typography>
            )}
          </Stack>
        </Box>
      </Stack>
    </StyledCard>
  );
}