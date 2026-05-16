import {
  Box,
  Card,
  Chip,
  Stack,
  Typography,
  styled,
} from "@mui/material";

import { useEffect, useState } from "react";

const API_BASE = "http://localhost:8080/api";

type Mode = "player" | "debug";

interface ExplorersPageProps {
  mode: Mode;
}

interface ExplorerData {
  id: number;
  currentPlanetId: number | null;
  basicResources: Record<string, number>;
  complexResources: Record<string, number>;
  aiRunning: boolean;
}

const PLANET_NAMES: Record<number, string> = {
  1: "Skycartel",
  2: "Luna4",
  3: "BlackAdidasShoe",
  4: "ImmutableCosmicBorrow",
  5: "RustEze",
  6: "Crabtorio",
  7: "Orbitron",
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

const ExplorerCard = styled(Box)({
  padding: "16px",
  borderRadius: "12px",
  background: "rgba(255,255,255,0.02)",
  border: "1px solid rgba(255,255,255,0.07)",
});

function ResourceRow({
  label,
  value,
}: {
  label: string;
  value: number;
}) {
  return (
    <Box
      display="flex"
      justifyContent="space-between"
      alignItems="center"
      sx={{
        py: 0.4,
        borderBottom: "1px solid rgba(255,255,255,0.04)",
        "&:last-child": { borderBottom: "none" },
      }}
    >
      <Typography sx={{ fontSize: "12px", color: "#9ca3af" }}>
        {label}
      </Typography>
      <Typography
        sx={{ fontSize: "12px", color: "#e5e7eb", fontFamily: "monospace" }}
      >
        {value}
      </Typography>
    </Box>
  );
}

export function ExplorersPage({ mode }: ExplorersPageProps) {
  const [explorers, setExplorers] = useState<ExplorerData[]>([]);
  const [lastUpdated, setLastUpdated] = useState<string | null>(null);

  useEffect(() => {
    const load = () =>
      fetch(`${API_BASE}/explorers`)
        .then((r) => r.json())
        .then((data: ExplorerData[]) => {
          setExplorers(data);
          setLastUpdated(new Date().toLocaleTimeString());
        })
        .catch(() => {});
    load();
    const id = window.setInterval(load, 2000);
    return () => window.clearInterval(id);
  }, []);

  const totalBasic = explorers.reduce(
    (sum, e) =>
      sum + Object.values(e.basicResources).reduce((s, v) => s + v, 0),
    0
  );
  const totalComplex = explorers.reduce(
    (sum, e) =>
      sum + Object.values(e.complexResources).reduce((s, v) => s + v, 0),
    0
  );

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
          Explorer Status
        </Typography>

        <Stack direction="row" spacing={1} alignItems="center">
          {lastUpdated && (
            <Typography sx={{ fontSize: "11px", color: "#4b5563" }}>
              {lastUpdated}
            </Typography>
          )}
          <Chip
            label={`${explorers.length} ACTIVE`}
            size="small"
            sx={{
              background: "rgba(99,102,241,0.12)",
              color: "#818cf8",
              border: "1px solid rgba(99,102,241,0.25)",
              fontSize: "11px",
              fontWeight: 700,
            }}
          />
        </Stack>
      </Box>

      {explorers.length === 0 ? (
        <Box
          display="flex"
          alignItems="center"
          justifyContent="center"
          sx={{ height: 200 }}
        >
          <Typography sx={{ fontSize: "13px", color: "#4b5563" }}>
            No explorer data — is the simulation running?
          </Typography>
        </Box>
      ) : (
        <Stack spacing={2}>
          {/* Summary bar */}
          <Box
            display="flex"
            gap={2}
            flexWrap="wrap"
            sx={{
              p: 1.5,
              borderRadius: "10px",
              background: "rgba(255,255,255,0.015)",
              border: "1px solid rgba(255,255,255,0.05)",
            }}
          >
            <Typography sx={{ fontSize: "12px", color: "#6b7280" }}>
              Total basic resources:{" "}
              <span style={{ color: "#e5e7eb", fontFamily: "monospace" }}>
                {totalBasic}
              </span>
            </Typography>
            <Typography sx={{ fontSize: "12px", color: "#6b7280" }}>
              Total complex resources:{" "}
              <span style={{ color: "#e5e7eb", fontFamily: "monospace" }}>
                {totalComplex}
              </span>
            </Typography>
          </Box>

          {/* Explorer cards */}
          <Stack spacing={2}>
            {explorers.map((explorer) => {
              const basicEntries = Object.entries(explorer.basicResources);
              const complexEntries = Object.entries(explorer.complexResources);
              const planetName = explorer.currentPlanetId
                ? (PLANET_NAMES[explorer.currentPlanetId] ??
                  `Planet ${explorer.currentPlanetId}`)
                : "Unknown";

              return (
                <ExplorerCard key={explorer.id}>
                  {/* Card header */}
                  <Box
                    display="flex"
                    alignItems="center"
                    justifyContent="space-between"
                    mb={1.5}
                  >
                    <Stack direction="row" spacing={1.5} alignItems="center">
                      <Box
                        sx={{
                          width: 32,
                          height: 32,
                          borderRadius: "50%",
                          background:
                            "linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          fontSize: "13px",
                          fontWeight: 700,
                          color: "#fff",
                        }}
                      >
                        {explorer.id}
                      </Box>
                      <Box>
                        <Typography
                          sx={{ fontSize: "13px", fontWeight: 600, color: "#fff" }}
                        >
                          Explorer {explorer.id}
                        </Typography>
                        <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                          {planetName}
                        </Typography>
                      </Box>
                    </Stack>

                    <Stack direction="row" spacing={1}>
                      <Chip
                        label={explorer.aiRunning ? "AI ON" : "AI OFF"}
                        size="small"
                        sx={{
                          height: 20,
                          fontSize: "10px",
                          fontWeight: 700,
                          background: explorer.aiRunning
                            ? "rgba(34,197,94,0.15)"
                            : "rgba(107,114,128,0.15)",
                          color: explorer.aiRunning ? "#4ade80" : "#6b7280",
                          border: explorer.aiRunning
                            ? "1px solid rgba(34,197,94,0.3)"
                            : "1px solid rgba(107,114,128,0.2)",
                        }}
                      />
                    </Stack>
                  </Box>

                  {/* Resources */}
                  <Stack
                    direction={{ xs: "column", sm: "row" }}
                    spacing={2}
                  >
                    {/* Basic resources */}
                    <Box flex={1}>
                      <Typography
                        sx={{
                          fontSize: "11px",
                          color: "#6b7280",
                          fontWeight: 600,
                          textTransform: "uppercase",
                          letterSpacing: "0.08em",
                          mb: 0.5,
                        }}
                      >
                        Basic
                      </Typography>
                      {basicEntries.length === 0 ? (
                        <Typography sx={{ fontSize: "11px", color: "#374151" }}>
                          Empty
                        </Typography>
                      ) : (
                        basicEntries.map(([name, count]) => (
                          <ResourceRow key={name} label={name} value={count} />
                        ))
                      )}
                    </Box>

                    {/* Divider */}
                    <Box
                      sx={{
                        width: "1px",
                        background: "rgba(255,255,255,0.05)",
                        flexShrink: 0,
                        display: { xs: "none", sm: "block" },
                      }}
                    />

                    {/* Complex resources */}
                    <Box flex={1}>
                      <Typography
                        sx={{
                          fontSize: "11px",
                          color: "#6b7280",
                          fontWeight: 600,
                          textTransform: "uppercase",
                          letterSpacing: "0.08em",
                          mb: 0.5,
                        }}
                      >
                        Complex
                      </Typography>
                      {complexEntries.length === 0 ? (
                        <Typography sx={{ fontSize: "11px", color: "#374151" }}>
                          Empty
                        </Typography>
                      ) : (
                        complexEntries.map(([name, count]) => (
                          <ResourceRow key={name} label={name} value={count} />
                        ))
                      )}
                    </Box>
                  </Stack>

                  {/* Debug extras */}
                  {mode === "debug" && (
                    <Box
                      mt={1.5}
                      sx={{
                        p: 1,
                        borderRadius: "8px",
                        background: "rgba(239,68,68,0.04)",
                        border: "1px solid rgba(239,68,68,0.1)",
                      }}
                    >
                      <Typography
                        sx={{ fontSize: "11px", color: "#f87171", fontFamily: "monospace" }}
                      >
                        id={explorer.id} planet={explorer.currentPlanetId ?? "?"}{" "}
                        ai={String(explorer.aiRunning)}
                      </Typography>
                    </Box>
                  )}
                </ExplorerCard>
              );
            })}
          </Stack>
        </Stack>
      )}
    </StyledCard>
  );
}
