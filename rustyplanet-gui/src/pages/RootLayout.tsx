import { useEffect, useState } from "react";
import { useLogStore } from "../store/logStore";
import { AlertFeed } from "../components/AlertFeed";

import {
  AppBar,
  Box,
  Chip,
  Container,
  FormControlLabel,
  Stack,
  Switch,
  Tab,
  Tabs,
  Toolbar,
  Typography,
  styled,
} from "@mui/material";

import { ExplorersPage } from "./ExplorersPage";
import { LogsPage } from "./LogsPage";
import { OverviewPage } from "./OverviewPage";
import { PlanetPage } from "./PlanetPage";
import { SimulationPage } from "./SimulationPage";
import { VisualizationPage } from "./VisualizationPage";

type Mode = "player" | "debug";

type TabKey =
  | "overview"
  | "planet"
  | "simulation"
  | "logs"
  | "visualization"
  | "explorers";

const TAB_ORDER: TabKey[] = [
  "overview",
  "planet",
  "simulation",
  "logs",
  "visualization",
  "explorers",
];

const StyledAppBar = styled(AppBar)({
  background:
    "linear-gradient(180deg, rgba(15,15,15,0.96) 0%, rgba(5,5,5,0.96) 100%)",
  backdropFilter: "blur(16px)",
  borderBottom: "1px solid rgba(255,255,255,0.08)",
  boxShadow: "none",
  minWidth: "100vw",
});

const SUN_STYLE = {
  width: 44,
  height: 44,
  borderRadius: "50%",
  background:
    "radial-gradient(circle at 30% 30%, #fff7b0 0%, #facc15 25%, #f97316 60%, #ef4444 100%)",
  boxShadow: `
    0 0 14px rgba(250, 204, 21, 0.6),
    0 0 30px rgba(249, 115, 22, 0.45),
    0 0 60px rgba(239, 68, 68, 0.25)
  `,
  animation: "sunPulse 3.5s ease-in-out infinite",
  "@keyframes sunPulse": {
    "0%, 100%": {
      boxShadow: `
        0 0 12px rgba(250, 204, 21, 0.55),
        0 0 28px rgba(249, 115, 22, 0.4),
        0 0 55px rgba(239, 68, 68, 0.2)
      `,
    },
    "50%": {
      boxShadow: `
        0 0 18px rgba(250, 204, 21, 0.75),
        0 0 40px rgba(249, 115, 22, 0.55),
        0 0 80px rgba(239, 68, 68, 0.3)
      `,
    },
  },
};

export function RootLayout() {
  const [mode, setMode] = useState<Mode>("player");
  const [activeTab, setActiveTab] = useState<TabKey>("overview");

  const { startStream, stopStream } = useLogStore();

  useEffect(() => {
    startStream();
    return () => stopStream();
  }, [startStream, stopStream]);

  const handleChangeTab = (_: React.SyntheticEvent, value: string) => {
    if (TAB_ORDER.includes(value as TabKey)) {
      setActiveTab(value as TabKey);
    }
  };

  const handleToggleMode = (
    _: React.ChangeEvent<HTMLInputElement>,
    checked: boolean
  ) => setMode(checked ? "debug" : "player");

  const renderTabContent = () => {
    switch (activeTab) {
      case "overview":
        return <OverviewPage mode={mode} />;
      case "planet":
        return <PlanetPage mode={mode} />;
      case "simulation":
        return <SimulationPage mode={mode} />;
      case "logs":
        return <LogsPage mode={mode} />;
      case "visualization":
        return <VisualizationPage mode={mode} onChangeTab={setActiveTab} />;
      case "explorers":
        return <ExplorersPage mode={mode} />;
      default:
        return null;
    }
  };

  return (
    <Box
      sx={{
        minHeight: "100vh",
        background:
          "radial-gradient(circle at top, #111827 0%, #050505 55%)",
        color: "#fff",
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* HEADER */}
      <StyledAppBar position="sticky">
        <Toolbar sx={{ minHeight: 72, px: { xs: 2, md: 4 } }}>
          <Stack direction="row" spacing={2} alignItems="center" sx={{ flexGrow: 1 }}>
            {/* ☀️ SUN LOGO */}
            <Box sx={SUN_STYLE} />

            {/* TITLE */}
            <Box>
              <Typography sx={{ fontSize: 18, fontWeight: 700 }}>
                RustyPlanet GUI
              </Typography>
              <Typography sx={{ fontSize: 12, color: "#6b7280" }}>
                Planetary Simulation Interface
              </Typography>
            </Box>
          </Stack>

          {/* MODE */}
          <Stack direction="row" spacing={1.5} alignItems="center">
            <Chip
              label={mode === "debug" ? "DEBUG" : "PLAYER"}
              size="small"
              sx={{
                background:
                  mode === "debug"
                    ? "rgba(239,68,68,0.15)"
                    : "rgba(34,197,94,0.15)",
                color: mode === "debug" ? "#f87171" : "#4ade80",
                border:
                  mode === "debug"
                    ? "1px solid rgba(239,68,68,0.3)"
                    : "1px solid rgba(34,197,94,0.3)",
                fontSize: 11,
                fontWeight: 700,
              }}
            />

            <FormControlLabel
              control={
                <Switch
                  checked={mode === "debug"}
                  onChange={handleToggleMode}
                  color="secondary"
                />
              }
              label={
                <Typography sx={{ fontSize: 13, color: "#9ca3af" }}>
                  {mode === "debug" ? "Debug mode" : "Player mode"}
                </Typography>
              }
            />
          </Stack>
        </Toolbar>

        {/* TABS */}
        <Box sx={{ px: { xs: 1, md: 3 }, borderTop: "1px solid rgba(255,255,255,0.04)" }}>
          <Tabs
            value={activeTab}
            onChange={handleChangeTab}
            centered
            sx={{
              minHeight: 64,
              "& .MuiTabs-indicator": {
                height: 3,
                borderRadius: "999px",
                background:
                  "linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%)",
              },
            }}
          >
            {TAB_ORDER.map((value) => (
              <Tab
                key={value}
                label={value.charAt(0).toUpperCase() + value.slice(1)}
                value={value}
                sx={{
                  minHeight: 64,
                  textTransform: "none",
                  fontSize: 14,
                  fontWeight: 600,
                  color: "#6b7280",
                  "&.Mui-selected": { color: "#fff" },
                  "&:hover": { color: "#d1d5db" },
                }}
              />
            ))}
          </Tabs>
        </Box>
      </StyledAppBar>

      {/* CONTENT */}
      <Container maxWidth={false} sx={{ flexGrow: 1, py: 4, px: { xs: 2, md: 4 } }}>
        {renderTabContent()}
      </Container>

      {/* Real-time simulation event notifications overlay */}
      <AlertFeed />
    </Box>
  );
}