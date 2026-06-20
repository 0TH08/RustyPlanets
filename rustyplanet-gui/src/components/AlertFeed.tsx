import { Box, Stack, Typography, keyframes, styled } from "@mui/material";
import { useEffect, useRef, useState } from "react";
import { useLogStore } from "../store/logStore";
import type { LogEntry } from "../types/logs";

interface AlertItem {
  log: LogEntry;
  isClosing: boolean;
}

interface SeverityDetails {
  severity: "info" | "warning" | "success" | "error";
  color: string;
  glow: string;
  icon: string;
  title: string;
  iconAnimation?: string;
}

const slideIn = keyframes`
  0% {
    transform: translateX(120%) scale(0.9);
    opacity: 0;
  }
  60% {
    transform: translateX(-8px) scale(1.02);
    opacity: 0.95;
  }
  100% {
    transform: translateX(0) scale(1);
    opacity: 1;
  }
`;

const slideOut = keyframes`
  0% {
    transform: translateX(0) scale(1);
    opacity: 1;
    max-height: 120px;
    margin-bottom: 12px;
    padding-top: 14px;
    padding-bottom: 14px;
  }
  100% {
    transform: translateX(120%) scale(0.85);
    opacity: 0;
    max-height: 0;
    margin-bottom: 0;
    padding-top: 0;
    padding-bottom: 0;
    border-width: 0;
  }
`;

const pulse = keyframes`
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4);
  }
  50% {
    transform: scale(1.1);
    box-shadow: 0 0 12px 4px rgba(239, 68, 68, 0.2);
  }
`;

const rotate = keyframes`
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
`;

const AlertCard = styled(Box, {
  shouldForwardProp: (prop) =>
      prop !== "severityColor" && prop !== "isClosing" && prop !== "glowColor",
})<{ severityColor: string; glowColor: string; isClosing: boolean }>(
    ({ severityColor, glowColor, isClosing }) => ({
      display: "flex",
      alignItems: "flex-start",
      gap: "14px",
      padding: "14px 18px",
      borderRadius: "14px",
      background:
          "linear-gradient(135deg, rgba(11, 15, 28, 0.82) 0%, rgba(20, 27, 45, 0.72) 100%)",
      backdropFilter: "blur(20px) saturate(160%)",
      border: "1px solid rgba(255, 255, 255, 0.07)",
      borderLeft: `4px solid ${severityColor}`,
      boxShadow: `0 12px 40px -6px rgba(0, 0, 0, 0.6), 0 0 20px -2px ${glowColor}, inset 0 1px 0 rgba(255, 255, 255, 0.1)`,
      color: "#fff",
      transition: "all 0.4s cubic-bezier(0.16, 1, 0.3, 1)",
      animation: `${isClosing ? slideOut : slideIn} 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards`,
      overflow: "hidden",
      pointerEvents: "auto",
      width: "100%",
    })
);

const IconBox = styled(Box, {
  shouldForwardProp: (prop) => prop !== "accentColor" && prop !== "anim",
})<{ accentColor: string; anim?: string }>(({ accentColor, anim }) => ({
  width: 32,
  height: 32,
  borderRadius: "8px",
  background: `rgba(${accentColor}, 0.12)`,
  border: `1px solid rgba(${accentColor}, 0.25)`,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  fontSize: "15px",
  flexShrink: 0,
  animation: anim || "none",
}));

function getSeverityDetails(message: string): SeverityDetails {
  const msg = message.toLowerCase();

  if (
      msg.includes("hit by an asteroid") ||
      msg.includes("destroyed") ||
      msg.includes("killed")
  ) {
    return {
      severity: "error",
      color: "#ef4444",
      glow: "rgba(239, 68, 68, 0.15)",
      icon: "☄️",
      title: "HAZARD STRIKE",
      iconAnimation: `${pulse} 2s infinite ease-in-out`,
    };
  }

  if (msg.includes("deflected")) {
    return {
      severity: "success",
      color: "#10b981",
      glow: "rgba(16, 185, 129, 0.12)",
      icon: "🛡️",
      title: "DEFENSE ENGAGED",
    };
  }

  if (msg.includes("crafted")) {
    return {
      severity: "info",
      color: "#8b5cf6",
      glow: "rgba(139, 92, 246, 0.12)",
      icon: "🧪",
      title: "RESOURCE SYNTHESIS",
    };
  }

  if (
      msg.includes("moved to") ||
      msg.includes("arrived") ||
      msg.includes("left")
  ) {
    return {
      severity: "info",
      color: "#3b82f6",
      glow: "rgba(59, 130, 246, 0.12)",
      icon: "🛸",
      title: "COSMIC DEPARTURE",
    };
  }

  if (
      msg.includes("ai started") ||
      msg.includes("ai stopped") ||
      msg.includes("resetting")
  ) {
    return {
      severity: "warning",
      color: "#f59e0b",
      glow: "rgba(245, 158, 11, 0.12)",
      icon: "⚙️",
      title: "SYSTEM ADJUSTMENT",
      iconAnimation: `${rotate} 6s linear infinite`,
    };
  }

  return {
    severity: "info",
    color: "#6366f1",
    glow: "rgba(99, 102, 241, 0.1)",
    icon: "📢",
    title: "SIMULATION UPDATE",
  };
}

const RGB_ACCENTS = {
  "#ef4444": "239, 68, 68",
  "#10b981": "16, 185, 129",
  "#8b5cf6": "139, 92, 246",
  "#3b82f6": "59, 130, 246",
  "#f59e0b": "245, 158, 11",
  "#6366f1": "99, 102, 241",
};

export function AlertFeed() {
  const { logs } = useLogStore();
  const [alerts, setAlerts] = useState<AlertItem[]>([]);
  const lastProcessedIdRef = useRef<string | number | null>(null);
  const timersRef = useRef<number[]>([]);

  useEffect(() => {
    return () => {
      timersRef.current.forEach((timer) => window.clearTimeout(timer));
    };
  }, []);

  useEffect(() => {
    if (logs.length === 0) return;

    const latest = logs[logs.length - 1];
    if (latest.id === lastProcessedIdRef.current) return;

    lastProcessedIdRef.current = latest.id;

    const details = getSeverityDetails(latest.message);
    const shouldShowPopup =
        latest.player &&
        (latest.level === "error" ||
            latest.level === "warn" ||
            details.severity === "error");

    if (!shouldShowPopup) return;

    const newItem: AlertItem = { log: latest, isClosing: false };
    setAlerts((previous) => [...previous, newItem].slice(-4));

    const closeTimer = window.setTimeout(() => {
      setAlerts((previous) =>
          previous.map((alert) =>
              alert.log.id === latest.id ? { ...alert, isClosing: true } : alert
          )
      );
    }, 3500);

    const removeTimer = window.setTimeout(() => {
      setAlerts((previous) =>
          previous.filter((alert) => alert.log.id !== latest.id)
      );
    }, 4000);

    timersRef.current.push(closeTimer, removeTimer);
  }, [logs]);

  return (
      <Box
          sx={{
            position: "fixed",
            top: 88,
            right: 24,
            zIndex: 1600,
            width: 360,
            pointerEvents: "none",
          }}
      >
        <Stack spacing={1.5}>
          {alerts.map((item) => {
            const details = getSeverityDetails(item.log.message);
            const rgbAccent =
                RGB_ACCENTS[details.color as keyof typeof RGB_ACCENTS] ||
                "99, 102, 241";

            return (
                <AlertCard
                    key={item.log.id}
                    severityColor={details.color}
                    glowColor={details.glow}
                    isClosing={item.isClosing}
                >
                  <IconBox accentColor={rgbAccent} anim={details.iconAnimation}>
                    {details.icon}
                  </IconBox>

                  <Box sx={{ flexGrow: 1 }}>
                    <Typography
                        sx={{
                          fontSize: "10px",
                          fontWeight: 700,
                          letterSpacing: "0.1em",
                          color: details.color,
                          textTransform: "uppercase",
                          mb: 0.25,
                        }}
                    >
                      {details.title}
                    </Typography>

                    <Typography
                        sx={{
                          fontSize: "12px",
                          fontWeight: 500,
                          color: "#f3f4f6",
                          lineHeight: 1.4,
                        }}
                    >
                      {item.log.message}
                    </Typography>
                  </Box>
                </AlertCard>
            );
          })}
        </Stack>
      </Box>
  );
}
