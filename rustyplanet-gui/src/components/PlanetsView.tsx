import { usePlanetStore } from "../store/planetStore";
import {
  Avatar,
  Box,
  Card,
  Chip,
  CircularProgress,
  Grid,
  Stack,
  styled,
  Typography,
} from "@mui/material";
import {
  Person as PersonIcon,
  Rocket as RocketIcon,
  Bolt as BoltIcon,
} from "@mui/icons-material";

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

const PlanetCard = styled(Box)({
  display: "grid",
  gridTemplateColumns: "auto 1fr auto auto",
  alignItems: "center",
  gap: "14px",
  padding: "14px 16px",
  background: "rgba(255,255,255,0.02)",
  border: "1px solid rgba(255,255,255,0.06)",
  borderRadius: "12px",
  cursor: "pointer",
  transition: "all 0.2s",
  "&:hover": {
    background: "rgba(255,255,255,0.05)",
    borderColor: "rgba(249,115,22,0.4)",
  },
});

const PlanetOrb = styled(Avatar)({
  width: "40px",
  height: "40px",
  fontSize: "14px",
  fontWeight: 600,
  position: "relative",
  "&::after": {
    content: '""',
    position: "absolute",
    inset: "-2px",
    borderRadius: "50%",
    background: "inherit",
    filter: "blur(8px)",
    opacity: 0.4,
    zIndex: -1,
  },
});

export function PlanetsView() {
  const {
    planets,
    selectedPlanetId,
    isLoadingList,
    error,
    selectPlanet,
  } = usePlanetStore();

  return (
    <Grid size={{ xs: 12, md: 8 }}>
      <StyledCard>
        <Box
          display="flex"
          alignItems="center"
          justifyContent="space-between"
          mb={2.5}
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
            Planets
          </Typography>
          {/* Legend */}
          <Box
            sx={{
              display: "flex",
              alignItems: "center",
              gap: 2,
              flexWrap: "wrap",
              px: 1,
              py: 1,
              border: "1px solid rgba(255,255,255,0.06)",
              borderRadius: "12px",
              background: "rgba(255,255,255,0.02)",
            }}
          >
            <Typography
              sx={{
                fontSize: "11px",
                fontWeight: 600,
                color: "#6b7280",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              Legend
            </Typography>

            <Box
              display="flex"
              alignItems="center"
              gap={0.5}
              sx={{ fontSize: "12px", color: "#9ca3af" }}
            >
              <PersonIcon sx={{ fontSize: "14px" }} />
              Explorers
            </Box>

            <Box
              display="flex"
              alignItems="center"
              gap={0.5}
              sx={{ fontSize: "12px", color: "#9ca3af" }}
            >
              <BoltIcon sx={{ fontSize: "14px" }} />
              Resources Generated
            </Box>

            <Box
              display="flex"
              alignItems="center"
              gap={0.5}
              sx={{ fontSize: "12px", color: "#9ca3af" }}
            >
              <RocketIcon sx={{ fontSize: "14px" }} />
              Rockets Built
            </Box>
          </Box>
        </Box>

        <Stack spacing={2}>
          

          {/* Planet List */}
          <Stack spacing={1.25}>
            {isLoadingList && (
              <Stack
                direction="row"
                alignItems="center"
                spacing={1}
                sx={{ py: 2 }}
              >
                <CircularProgress size={16} />

                <Typography
                  sx={{
                    fontSize: "13px",
                    color: "#9ca3af",
                  }}
                >
                  Loading planets...
                </Typography>
              </Stack>
            )}

            {error && (
              <Typography
                sx={{
                  fontSize: "13px",
                  color: "#ef4444",
                }}
              >
                {error}
              </Typography>
            )}

            {!isLoadingList && !error && planets.length === 0 && (
              <Typography
                sx={{
                  fontSize: "13px",
                  color: "#6b7280",
                }}
              >
                No planets available.
              </Typography>
            )}

            {!isLoadingList &&
              !error &&
              planets.map((planet) => {
                const isSelected = planet.id === selectedPlanetId;

                return (
                  <PlanetCard
                    key={planet.id}
                    onClick={() => void selectPlanet(planet.id)}
                    sx={{
                      background: isSelected
                        ? "rgba(99,102,241,0.12)"
                        : "rgba(255,255,255,0.02)",

                      border: isSelected
                        ? "1px solid rgba(99,102,241,0.45)"
                        : "1px solid rgba(255,255,255,0.06)",

                      "&:hover": {
                        background: "rgba(255,255,255,0.05)",
                        borderColor: "rgba(99,102,241,0.4)",
                      },
                    }}
                  >
                    <PlanetOrb
                      sx={{
                        background: "#282828",
                        color: "#aaaaaa",
                      }}
                    >
                      {planet.name
                        .split(" ")
                        .map((part) => part[0])
                        .join("")
                        .slice(0, 2)
                        .toUpperCase()}
                    </PlanetOrb>

                    <Box minWidth={0}>
                      <Typography
                        sx={{
                          fontSize: "15px",
                          fontWeight: 500,
                          color: "#fff",
                          mb: 0.25,
                        }}
                      >
                        {planet.name}
                      </Typography>

                      <Typography
                        sx={{
                          fontSize: "11px",
                          color: "#6b7280",
                          fontFamily: "monospace",
                        }}
                      >
                        {planet.kind}
                      </Typography>
                    </Box>

                    <Box display="flex" gap={1.25}>
                      <Box
                        display="flex"
                        alignItems="center"
                        gap={0.5}
                        sx={{
                          fontSize: "11px",
                          color: "#9ca3af",
                        }}
                      >
                        <PersonIcon sx={{ fontSize: "14px" }} />
                        {planet.explorerCount}
                      </Box>

                      <Box
                        display="flex"
                        alignItems="center"
                        gap={0.5}
                        sx={{
                          fontSize: "11px",
                          color: "#9ca3af",
                        }}
                      >
                        <BoltIcon sx={{ fontSize: "14px" }} />
                        {planet.totalResourcesGenerated}
                      </Box>

                      <Box
                        display="flex"
                        alignItems="center"
                        gap={0.5}
                        sx={{
                          fontSize: "11px",
                          color: "#9ca3af",
                        }}
                      >
                        <RocketIcon sx={{ fontSize: "14px" }} />
                        {planet.rocketsBuilt}
                      </Box>
                    </Box>

                    <Chip
                      label={planet.aiRunning ? "AI Running" : "AI Stopped"}
                      size="small"
                      sx={{
                        background: planet.aiRunning
                          ? "rgba(34,197,94,0.12)"
                          : "rgba(107,114,128,0.2)",

                        color: planet.aiRunning ? "#4ade80" : "#9ca3af",

                        border: planet.aiRunning
                          ? "1px solid rgba(34,197,94,0.3)"
                          : "1px solid rgba(107,114,128,0.3)",

                        fontSize: "11px",
                        fontWeight: 500,
                        height: "24px",
                      }}
                    />
                  </PlanetCard>
                );
              })}
          </Stack>
        </Stack>
      </StyledCard>
    </Grid>
  );
}
