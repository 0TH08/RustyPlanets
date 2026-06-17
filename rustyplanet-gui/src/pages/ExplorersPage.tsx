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

type ExplorerFilter = "all" | "ai" | "carrying" | "empty" | "unstationed";

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
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [filter, setFilter] = useState<ExplorerFilter>("all");
  const [toast, setToast] = useState<{
    msg: string;
    severity: "success" | "error";
  } | null>(null);

  const normalizeExplorers = (data: ExplorerData[]): ExplorerData[] => {
    return data.map((explorer) => ({
      ...explorer,
      basicResources: explorer.basicResources ?? {},
      complexResources: explorer.complexResources ?? {},
      aiRunning: explorer.aiRunning ?? false,
      currentPlanetId: explorer.currentPlanetId ?? null,
    }));
  };

  const getResourceTotal = (resources: Record<string, number>) => {
    return Object.values(resources ?? {}).reduce((sum, value) => sum + value, 0);
  };

  const getExplorerBackpackTotal = (explorer: ExplorerData) => {
    return (
        getResourceTotal(explorer.basicResources) +
        getResourceTotal(explorer.complexResources)
    );
  };

  const getPlanetName = (planetId: number | null) => {
    if (!planetId) return "Unknown";
    return PLANET_NAMES[planetId] ?? `Planet ${planetId}`;
  };

  const getExplorerNeighbors = (explorer: ExplorerData) => {
    if (!explorer.currentPlanetId) return [];
    return topology[String(explorer.currentPlanetId)] ?? [];
  };

  const loadExplorers = async () => {
    setIsRefreshing(true);

    try {
      const response = await fetch(`${API_BASE}/explorers`);
      const data: ExplorerData[] = await response.json();

      setExplorers(normalizeExplorers(data));
      setLastUpdated(new Date().toLocaleTimeString());
    } catch {
      setToast({
        msg: "Could not load explorers — check if backend is running.",
        severity: "error",
      });
    } finally {
      setIsRefreshing(false);
    }
  };

  useEffect(() => {
    loadExplorers();

    const id = window.setInterval(loadExplorers, 2000);

    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    fetch(`${API_BASE}/topology`)
        .then((r) => r.json())
        .then(setTopology)
        .catch(() => {});
  }, []);

  const dispatchExplorer = async (explorerId: number, destId: number) => {
    if (!destId) return;

    setIsDispatching((prev) => ({ ...prev, [explorerId]: true }));

    try {
      const res = await moveExplorer(explorerId, destId);

      if (res && res.status === "ok") {
        setToast({
          msg: `Explorer ${explorerId} dispatched to ${getPlanetName(destId)}.`,
          severity: "success",
        });

        setSelectedDests((prev) => {
          const updated = { ...prev };
          delete updated[explorerId];
          return updated;
        });

        await loadExplorers();
      } else {
        setToast({
          msg: "Failed to dispatch explorer — check destination path.",
          severity: "error",
        });
      }
    } catch {
      setToast({
        msg: "Action failed — check if the backend is running.",
        severity: "error",
      });
    } finally {
      setIsDispatching((prev) => ({ ...prev, [explorerId]: false }));
    }
  };

  const handleDispatch = async (explorerId: number) => {
    const destId = selectedDests[explorerId];

    if (!destId) return;

    await dispatchExplorer(explorerId, destId);
  };

  const totalBasic = explorers.reduce(
      (sum, e) => sum + getResourceTotal(e.basicResources),
      0
  );

  const totalComplex = explorers.reduce(
      (sum, e) => sum + getResourceTotal(e.complexResources),
      0
  );

  const totalBackpack = totalBasic + totalComplex;
  const aiRunningCount = explorers.filter((e) => e.aiRunning).length;
  const stationedCount = explorers.filter((e) => e.currentPlanetId !== null).length;
  const totalRoutes = Math.floor(
      Object.values(topology).reduce((sum, routes) => sum + routes.length, 0) / 2
  );

  const filteredExplorers = explorers.filter((explorer) => {
    const backpackTotal = getExplorerBackpackTotal(explorer);

    if (filter === "ai") return explorer.aiRunning;
    if (filter === "carrying") return backpackTotal > 0;
    if (filter === "empty") return backpackTotal === 0;
    if (filter === "unstationed") return explorer.currentPlanetId === null;

    return true;
  });

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

            <Button
                size="small"
                onClick={loadExplorers}
                disabled={isRefreshing}
                sx={{
                  color: "#9ca3af",
                  border: "1px solid rgba(255,255,255,0.08)",
                  borderRadius: "999px",
                  fontSize: "11px",
                  textTransform: "none",
                  px: 1.25,
                  minWidth: 0,
                }}
            >
              {isRefreshing ? <CircularProgress size={12} /> : "Refresh"}
            </Button>

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
                  gap={1}
                  flexWrap="wrap"
                  sx={{
                    p: 1.5,
                    borderRadius: "10px",
                    background: "rgba(255,255,255,0.015)",
                    border: "1px solid rgba(255,255,255,0.05)",
                  }}
              >
                <Chip
                    label={`Basic: ${totalBasic}`}
                    size="small"
                    sx={{
                      background: "rgba(96,165,250,0.1)",
                      color: "#93c5fd",
                      border: "1px solid rgba(96,165,250,0.2)",
                      fontSize: "11px",
                    }}
                />

                <Chip
                    label={`Complex: ${totalComplex}`}
                    size="small"
                    sx={{
                      background: "rgba(192,132,252,0.1)",
                      color: "#c084fc",
                      border: "1px solid rgba(192,132,252,0.2)",
                      fontSize: "11px",
                    }}
                />

                <Chip
                    label={`Backpack Total: ${totalBackpack}`}
                    size="small"
                    sx={{
                      background: "rgba(245,158,11,0.1)",
                      color: "#fbbf24",
                      border: "1px solid rgba(245,158,11,0.2)",
                      fontSize: "11px",
                    }}
                />

                <Chip
                    label={`AI Running: ${aiRunningCount}`}
                    size="small"
                    sx={{
                      background: "rgba(34,197,94,0.1)",
                      color: "#4ade80",
                      border: "1px solid rgba(34,197,94,0.2)",
                      fontSize: "11px",
                    }}
                />

                <Chip
                    label={`Stationed: ${stationedCount}`}
                    size="small"
                    sx={{
                      background: "rgba(255,255,255,0.04)",
                      color: "#d1d5db",
                      border: "1px solid rgba(255,255,255,0.08)",
                      fontSize: "11px",
                    }}
                />

                <Chip
                    label={`Routes: ${totalRoutes}`}
                    size="small"
                    sx={{
                      background: "rgba(99,102,241,0.1)",
                      color: "#818cf8",
                      border: "1px solid rgba(99,102,241,0.2)",
                      fontSize: "11px",
                    }}
                />
              </Box>

              {/* Filters */}
              <Box
                  display="flex"
                  gap={1}
                  flexWrap="wrap"
                  sx={{
                    p: 1.25,
                    borderRadius: "10px",
                    background: "rgba(255,255,255,0.012)",
                    border: "1px solid rgba(255,255,255,0.04)",
                  }}
              >
                {[
                  { key: "all", label: "All" },
                  { key: "ai", label: "AI ON" },
                  { key: "carrying", label: "Carrying" },
                  { key: "empty", label: "Empty Bag" },
                  { key: "unstationed", label: "Unstationed" },
                ].map((item) => (
                    <Chip
                        key={item.key}
                        label={item.label}
                        size="small"
                        onClick={() => setFilter(item.key as ExplorerFilter)}
                        sx={{
                          cursor: "pointer",
                          fontSize: "11px",
                          fontWeight: 600,
                          background:
                              filter === item.key
                                  ? "rgba(99,102,241,0.18)"
                                  : "rgba(255,255,255,0.03)",
                          color: filter === item.key ? "#a5b4fc" : "#9ca3af",
                          border:
                              filter === item.key
                                  ? "1px solid rgba(129,140,248,0.35)"
                                  : "1px solid rgba(255,255,255,0.06)",
                        }}
                    />
                ))}
              </Box>

              {/* Explorer cards */}
              {filteredExplorers.length === 0 ? (
                  <Box
                      sx={{
                        p: 2,
                        borderRadius: "12px",
                        background: "rgba(255,255,255,0.015)",
                        border: "1px solid rgba(255,255,255,0.05)",
                      }}
                  >
                    <Typography sx={{ fontSize: "13px", color: "#6b7280" }}>
                      No explorers match this filter.
                    </Typography>
                  </Box>
              ) : (
                  <Stack spacing={2}>
                    {filteredExplorers.map((explorer) => {
                      const basicEntries = Object.entries(explorer.basicResources);
                      const complexEntries = Object.entries(explorer.complexResources);
                      const planetName = getPlanetName(explorer.currentPlanetId);
                      const backpackTotal = getExplorerBackpackTotal(explorer);
                      const neighbors = getExplorerNeighbors(explorer);

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
                                      sx={{
                                        fontSize: "13px",
                                        fontWeight: 600,
                                        color: "#fff",
                                      }}
                                  >
                                    Explorer {explorer.id}
                                  </Typography>

                                  <Typography sx={{ fontSize: "11px", color: "#6b7280" }}>
                                    {planetName}
                                  </Typography>
                                </Box>
                              </Stack>

                              <Stack direction="row" spacing={1} flexWrap="wrap">
                                <Chip
                                    label={`🎒 ${backpackTotal}`}
                                    size="small"
                                    sx={{
                                      height: 20,
                                      fontSize: "10px",
                                      fontWeight: 700,
                                      background: "rgba(245,158,11,0.12)",
                                      color: "#fbbf24",
                                      border: "1px solid rgba(245,158,11,0.25)",
                                    }}
                                />

                                <Chip
                                    label={`${neighbors.length} ROUTES`}
                                    size="small"
                                    sx={{
                                      height: 20,
                                      fontSize: "10px",
                                      fontWeight: 700,
                                      background: "rgba(99,102,241,0.12)",
                                      color: "#818cf8",
                                      border: "1px solid rgba(99,102,241,0.25)",
                                    }}
                                />

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
                            <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
                              {/* Basic resources */}
                              <Box flex={1}>
                                <Typography
                                    sx={{
                                      fontSize: "11px",
                                      color: "#93c5fd",
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
                                      color: "#c084fc",
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
                              ) : neighbors.length === 0 ? (
                                  <Typography sx={{ fontSize: "12px", color: "#6b7280" }}>
                                    No neighboring routes found for this planet.
                                  </Typography>
                              ) : (
                                  <Stack spacing={1.5}>
                                    <Stack direction="row" spacing={2} alignItems="center">
                                      <FormControl size="small" sx={{ minWidth: 160 }}>
                                        <InputLabel
                                            id={`dest-label-${explorer.id}`}
                                            sx={{ color: "#9ca3af", fontSize: "12px" }}
                                        >
                                          Destination
                                        </InputLabel>

                                        <Select
                                            labelId={`dest-label-${explorer.id}`}
                                            label="Destination"
                                            value={selectedDests[explorer.id] || ""}
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
                                                <MenuItem
                                                    key={nid}
                                                    value={nid}
                                                    sx={{ fontSize: "13px" }}
                                                >
                                                  {name}
                                                </MenuItem>
                                            );
                                          })}
                                        </Select>
                                      </FormControl>

                                      <Button
                                          variant="contained"
                                          size="small"
                                          disabled={
                                              !selectedDests[explorer.id] ||
                                              isDispatching[explorer.id]
                                          }
                                          onClick={() => handleDispatch(explorer.id)}
                                          sx={{
                                            background:
                                                "linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)",
                                            color: "#fff",
                                            borderRadius: "10px",
                                            textTransform: "none",
                                            fontWeight: 600,
                                            px: 2,
                                            height: 38,
                                            boxShadow: "none",
                                            "&:hover": {
                                              background:
                                                  "linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%)",
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

                                    {/* Quick route buttons */}
                                    <Box>
                                      <Typography
                                          sx={{
                                            fontSize: "11px",
                                            color: "#6b7280",
                                            mb: 0.75,
                                          }}
                                      >
                                        Quick routes
                                      </Typography>

                                      <Stack direction="row" gap={0.75} flexWrap="wrap">
                                        {neighbors.map((nid) => (
                                            <Chip
                                                key={nid}
                                                label={PLANET_NAMES[nid] ?? `Planet ${nid}`}
                                                size="small"
                                                disabled={Boolean(isDispatching[explorer.id])}
                                                onClick={() => dispatchExplorer(explorer.id, nid)}
                                                sx={{
                                                  cursor: "pointer",
                                                  height: 24,
                                                  fontSize: "10px",
                                                  background: "rgba(99,102,241,0.08)",
                                                  color: "#a5b4fc",
                                                  border: "1px solid rgba(99,102,241,0.2)",
                                                  "&:hover": {
                                                    background: "rgba(99,102,241,0.16)",
                                                  },
                                                }}
                                            />
                                        ))}
                                      </Stack>
                                    </Box>
                                  </Stack>
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
                                      sx={{
                                        fontSize: "11px",
                                        color: "#f87171",
                                        fontFamily: "monospace",
                                      }}
                                  >
                                    id={explorer.id} planet=
                                    {explorer.currentPlanetId ?? "?"} ai=
                                    {String(explorer.aiRunning)} backpack={backpackTotal} routes=
                                    {neighbors.join(",") || "none"}
                                  </Typography>
                                </Box>
                            )}
                          </ExplorerCard>
                      );
                    })}
                  </Stack>
              )}
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