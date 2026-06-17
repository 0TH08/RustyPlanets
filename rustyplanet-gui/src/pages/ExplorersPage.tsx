import {
  Box,
  Card,
  Chip,
  Stack,
  Typography,
  styled,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  CircularProgress,
  Snackbar,
  Alert,
} from "@mui/material";

import { useEffect, useState } from "react";
import { moveExplorer } from "../services/planetService";

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
  8: "AstroParrot",
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
  const [topology, setTopology] = useState<Record<string, number[]>>({});
  const [selectedDests, setSelectedDests] = useState<Record<number, number>>({});
  const [isDispatching, setIsDispatching] = useState<Record<number, boolean>>({});
  const [toast, setToast] = useState<{ msg: string; severity: "success" | "error" } | null>(null);

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

  useEffect(() => {
    fetch(`${API_BASE}/topology`)
      .then((r) => r.json())
      .then(setTopology)
      .catch(() => {});
  }, []);

  const handleDispatch = async (explorerId: number) => {
    const destId = selectedDests[explorerId];
    if (!destId) return;

    setIsDispatching((prev) => ({ ...prev, [explorerId]: true }));
    try {
      const res = await moveExplorer(explorerId, destId);
      if (res && res.status === "ok") {
        setToast({ msg: `Explorer ${explorerId} successfully dispatched!`, severity: "success" });
        // Clear selected destination for this explorer
        setSelectedDests((prev) => {
          const updated = { ...prev };
          delete updated[explorerId];
          return updated;
        });
        // Immediately fetch updated explorer positions
        const explRes = await fetch(`${API_BASE}/explorers`);
        const explData = await explRes.json();
        setExplorers(explData);
      } else {
        setToast({ msg: "Failed to dispatch explorer — check destination path.", severity: "error" });
      }
    } catch {
      setToast({ msg: "Action failed — check if the backend is running.", severity: "error" });
    } finally {
      setIsDispatching((prev) => ({ ...prev, [explorerId]: false }));
    }
  };

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

                  {/* Manual Dispatch Section */}
                  <Box
                    mt={2.5}
                    pt={2}
                    sx={{
                      borderTop: "1px solid rgba(255,255,255,0.06)",
                    }}
                  >
                    <Typography
                      sx={{
                        fontSize: "11px",
                        color: "#6b7280",
                        fontWeight: 600,
                        textTransform: "uppercase",
                        letterSpacing: "0.08em",
                        mb: 1.5,
                      }}
                    >
                      Manual Dispatch
                    </Typography>

                    {explorer.currentPlanetId === null ? (
                      <Typography sx={{ fontSize: "12px", color: "#6b7280" }}>
                        Explorer is not stationed on any planet.
                      </Typography>
                    ) : (
                      (() => {
                        const neighbors = topology[explorer.currentPlanetId] ?? [];
                        if (neighbors.length === 0) {
                          return (
                            <Typography sx={{ fontSize: "12px", color: "#6b7280" }}>
                              No neighboring routes found for this planet.
                            </Typography>
                          );
                        }

                        const selectedDest = selectedDests[explorer.id] || "";

                        return (
                          <Stack direction="row" spacing={2} alignItems="center">
                            <FormControl size="small" sx={{ minWidth: 160 }}>
                              <InputLabel id={`dest-label-${explorer.id}`} sx={{ color: "#9ca3af", fontSize: "12px" }}>
                                Destination
                              </InputLabel>
                              <Select
                                labelId={`dest-label-${explorer.id}`}
                                label="Destination"
                                value={selectedDest}
                                onChange={(e) =>
                                  setSelectedDests((prev) => ({
                                    ...prev,
                                    [explorer.id]: Number(e.target.value),
                                  }))
                                }
                                sx={{
                                  borderRadius: "10px",
                                  color: "#fff",
                                  fontSize: "13px",
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
                                {neighbors.map((nid) => {
                                  const name = PLANET_NAMES[nid] ?? `Planet ${nid}`;
                                  return (
                                    <MenuItem key={nid} value={nid} sx={{ fontSize: "13px" }}>
                                      {name}
                                    </MenuItem>
                                  );
                                })}
                              </Select>
                            </FormControl>

                            <Button
                              variant="contained"
                              size="small"
                              disabled={!selectedDest || isDispatching[explorer.id]}
                              onClick={() => handleDispatch(explorer.id)}
                              sx={{
                                background: "linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)",
                                color: "#fff",
                                borderRadius: "10px",
                                textTransform: "none",
                                fontWeight: 600,
                                px: 2,
                                height: 38,
                                boxShadow: "none",
                                "&:hover": {
                                  background: "linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%)",
                                },
                                "&.Mui-disabled": {
                                  background: "rgba(255,255,255,0.05)",
                                  color: "rgba(255,255,255,0.2)",
                                },
                              }}
                            >
                              {isDispatching[explorer.id] ? (
                                <CircularProgress size={16} color="inherit" />
                              ) : (
                                "Dispatch"
                              )}
                            </Button>
                          </Stack>
                        );
                      })()
                    )}
                  </Box>

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

      {/* Dispatch Toast Feedback */}
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
    </StyledCard>
  );
}
