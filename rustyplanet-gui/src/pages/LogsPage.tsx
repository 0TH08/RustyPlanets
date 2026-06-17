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
    TextField,
    Typography,
    styled,
} from "@mui/material";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLogStore } from "../store/logStore";

type Mode = "player" | "debug";

interface LogsPageProps {
    mode: Mode;
}

type LogLevel = "debug" | "info" | "warn" | "error";
type LogCategory = "all" | "explorer" | "planet" | "sunray" | "asteroid" | "error";

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

const CATEGORY_STYLES = {
    explorer: {
        label: "Explorer",
        bg: "rgba(245,158,11,0.06)",
        border: "1px solid rgba(245,158,11,0.16)",
        color: "#fbbf24",
    },
    planet: {
        label: "Planet",
        bg: "rgba(99,102,241,0.06)",
        border: "1px solid rgba(99,102,241,0.16)",
        color: "#818cf8",
    },
    sunray: {
        label: "Sunray",
        bg: "rgba(250,204,21,0.06)",
        border: "1px solid rgba(250,204,21,0.18)",
        color: "#facc15",
    },
    asteroid: {
        label: "Asteroid",
        bg: "rgba(248,113,113,0.06)",
        border: "1px solid rgba(248,113,113,0.18)",
        color: "#f87171",
    },
    error: {
        label: "Error",
        bg: "rgba(239,68,68,0.08)",
        border: "1px solid rgba(239,68,68,0.2)",
        color: "#f87171",
    },
    system: {
        label: "System",
        bg: "rgba(255,255,255,0.02)",
        border: "1px solid rgba(255,255,255,0.05)",
        color: "#9ca3af",
    },
};

const levels = ["all", "debug", "info", "warn", "error"];

const categories: { key: LogCategory; label: string }[] = [
    { key: "all", label: "All" },
    { key: "explorer", label: "Explorer" },
    { key: "planet", label: "Planet" },
    { key: "sunray", label: "Sunray" },
    { key: "asteroid", label: "Asteroid" },
    { key: "error", label: "Error" },
];

function getLogCategory(message: string, level: string) {
    const text = message.toLowerCase();

    if (
        level === "error" ||
        text.includes("error") ||
        text.includes("failed") ||
        text.includes("panic")
    ) {
        return "error";
    }

    if (text.includes("asteroid")) return "asteroid";
    if (text.includes("sunray")) return "sunray";
    if (text.includes("explorer")) return "explorer";
    if (text.includes("planet") || text.includes("ai")) return "planet";

    return "system";
}

export function LogsPage({ mode }: LogsPageProps) {
    const {
        logs,
        levelFilter,
        isStreaming,
        startStream,
        stopStream,
        setLevelFilter,
    } = useLogStore();

    const terminalRef = useRef<HTMLDivElement | null>(null);

    const [searchTerm, setSearchTerm] = useState("");
    const [categoryFilter, setCategoryFilter] = useState<LogCategory>("all");
    const [autoScroll, setAutoScroll] = useState(true);
    const [hiddenLogIds, setHiddenLogIds] = useState<Set<string | number>>(
        new Set()
    );

    const visibleLogs = useMemo(() => {
        let filtered = logs.filter((log) => !hiddenLogIds.has(log.id));

        if (mode !== "debug") {
            filtered = filtered.filter((log) => log.player);
        }

        if (levelFilter !== "all") {
            filtered = filtered.filter((log) => log.level === levelFilter);
        }

        if (categoryFilter !== "all") {
            filtered = filtered.filter(
                (log) => getLogCategory(log.message, log.level) === categoryFilter
            );
        }

        const search = searchTerm.trim().toLowerCase();

        if (search) {
            filtered = filtered.filter((log) => {
                const message = String(log.message ?? "").toLowerCase();
                const level = String(log.level ?? "").toLowerCase();
                const planet = log.planetId != null ? String(log.planetId) : "";

                return (
                    message.includes(search) ||
                    level.includes(search) ||
                    planet.includes(search)
                );
            });
        }

        return filtered;
    }, [logs, hiddenLogIds, mode, levelFilter, categoryFilter, searchTerm]);

    useEffect(() => {
        if (!autoScroll) return;

        const node = terminalRef.current;

        if (!node) return;

        node.scrollTop = node.scrollHeight;
    }, [visibleLogs, autoScroll]);

    const streamLabel = isStreaming ? "STREAMING" : "PAUSED";

    const downloadLogs = useCallback(() => {
        const lines = visibleLogs.map((log) => {
            const time = new Date(log.timestamp).toLocaleTimeString();
            const planet = log.planetId != null ? ` [planet ${log.planetId}]` : "";
            return `[${time}] [${log.level.toUpperCase().padEnd(5)}]${planet} ${
                log.message
            }`;
        });

        const blob = new Blob([lines.join("\n")], { type: "text/plain" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");

        a.href = url;
        a.download = `rustyplanets-visible-logs-${new Date()
            .toISOString()
            .slice(0, 19)
            .replace(/:/g, "-")}.txt`;

        a.click();
        URL.revokeObjectURL(url);
    }, [visibleLogs]);

    const clearVisibleLogs = () => {
        setHiddenLogIds((prev) => {
            const next = new Set(prev);

            visibleLogs.forEach((log) => next.add(log.id));

            return next;
        });
    };

    const restoreLogs = () => {
        setHiddenLogIds(new Set());
    };

    const totalErrors = logs.filter(
        (log) => getLogCategory(log.message, log.level) === "error"
    ).length;

    const totalExplorerLogs = logs.filter(
        (log) => getLogCategory(log.message, log.level) === "explorer"
    ).length;

    const totalAsteroidLogs = logs.filter(
        (log) => getLogCategory(log.message, log.level) === "asteroid"
    ).length;

    const totalSunrayLogs = logs.filter(
        (log) => getLogCategory(log.message, log.level) === "sunray"
    ).length;

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

                <Stack direction="row" spacing={1} alignItems="center">
                    <Chip
                        label={`${visibleLogs.length} VISIBLE`}
                        size="small"
                        sx={{
                            background: "rgba(99,102,241,0.12)",
                            color: "#818cf8",
                            border: "1px solid rgba(99,102,241,0.25)",
                            fontSize: 11,
                            fontWeight: 700,
                            letterSpacing: "0.05em",
                        }}
                    />

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
                </Stack>
            </Box>

            <Stack spacing={2}>
                {/* Log Summary */}
                <Box
                    display="flex"
                    gap={1}
                    flexWrap="wrap"
                    sx={{
                        p: 1.25,
                        borderRadius: 3,
                        background: "rgba(255,255,255,0.015)",
                        border: "1px solid rgba(255,255,255,0.05)",
                    }}
                >
                    <Chip
                        label={`Total: ${logs.length}`}
                        size="small"
                        sx={{
                            background: "rgba(255,255,255,0.04)",
                            color: "#d1d5db",
                            border: "1px solid rgba(255,255,255,0.08)",
                            fontSize: "11px",
                        }}
                    />

                    <Chip
                        label={`Explorer: ${totalExplorerLogs}`}
                        size="small"
                        sx={{
                            background: "rgba(245,158,11,0.1)",
                            color: "#fbbf24",
                            border: "1px solid rgba(245,158,11,0.2)",
                            fontSize: "11px",
                        }}
                    />

                    <Chip
                        label={`Sunray: ${totalSunrayLogs}`}
                        size="small"
                        sx={{
                            background: "rgba(250,204,21,0.1)",
                            color: "#facc15",
                            border: "1px solid rgba(250,204,21,0.2)",
                            fontSize: "11px",
                        }}
                    />

                    <Chip
                        label={`Asteroid: ${totalAsteroidLogs}`}
                        size="small"
                        sx={{
                            background: "rgba(248,113,113,0.1)",
                            color: "#f87171",
                            border: "1px solid rgba(248,113,113,0.2)",
                            fontSize: "11px",
                        }}
                    />

                    <Chip
                        label={`Errors: ${totalErrors}`}
                        size="small"
                        sx={{
                            background: "rgba(239,68,68,0.1)",
                            color: "#f87171",
                            border: "1px solid rgba(239,68,68,0.2)",
                            fontSize: "11px",
                        }}
                    />
                </Box>

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
                            <InputLabel id="log-level-label" sx={{ color: "#9ca3af" }}>
                                Level
                            </InputLabel>

                            <Select
                                labelId="log-level-label"
                                label="Level"
                                value={levelFilter}
                                onChange={(e) =>
                                    setLevelFilter(e.target.value as typeof levelFilter)
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
                                    <MenuItem key={level} value={level}>
                                        {level[0].toUpperCase() + level.slice(1)}
                                    </MenuItem>
                                ))}
                            </Select>
                        </FormControl>

                        <FormControl size="small">
                            <InputLabel id="log-category-label" sx={{ color: "#9ca3af" }}>
                                Category
                            </InputLabel>

                            <Select
                                labelId="log-category-label"
                                label="Category"
                                value={categoryFilter}
                                onChange={(e) => setCategoryFilter(e.target.value as LogCategory)}
                                sx={{
                                    minWidth: 150,
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
                                {categories.map((category) => (
                                    <MenuItem key={category.key} value={category.key}>
                                        {category.label}
                                    </MenuItem>
                                ))}
                            </Select>
                        </FormControl>

                        <TextField
                            size="small"
                            label="Search logs"
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            sx={{
                                minWidth: 220,

                                "& .MuiInputLabel-root": {
                                    color: "#9ca3af",
                                    fontSize: "13px",
                                },

                                "& .MuiOutlinedInput-root": {
                                    borderRadius: "10px",
                                    color: "#fff",
                                    fontSize: "13px",

                                    "& fieldset": {
                                        borderColor: "rgba(255,255,255,0.1)",
                                    },

                                    "&:hover fieldset": {
                                        borderColor: "rgba(99,102,241,0.4)",
                                    },

                                    "&.Mui-focused fieldset": {
                                        borderColor: "#818cf8",
                                    },
                                },
                            }}
                        />

                        <Button
                            size="small"
                            variant={isStreaming ? "outlined" : "contained"}
                            onClick={isStreaming ? stopStream : startStream}
                            sx={{
                                background: !isStreaming
                                    ? "linear-gradient(135deg, #22c55e 0%, #16a34a 100%)"
                                    : "transparent",

                                borderColor: isStreaming ? "rgba(249,115,22,0.3)" : undefined,

                                color: isStreaming ? "#fb923c" : "#fff",

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
                            {isStreaming ? "Pause Stream" : "Start Stream"}
                        </Button>

                        <Button
                            size="small"
                            variant="outlined"
                            onClick={() => setAutoScroll((prev) => !prev)}
                            sx={{
                                borderColor: autoScroll
                                    ? "rgba(34,197,94,0.35)"
                                    : "rgba(107,114,128,0.35)",
                                color: autoScroll ? "#4ade80" : "#9ca3af",
                                borderRadius: "10px",
                                textTransform: "none",
                                fontWeight: 600,
                                boxShadow: "none",
                                "&:hover": {
                                    background: autoScroll
                                        ? "rgba(34,197,94,0.08)"
                                        : "rgba(107,114,128,0.08)",
                                },
                            }}
                        >
                            Auto-scroll {autoScroll ? "ON" : "OFF"}
                        </Button>

                        <Button
                            size="small"
                            variant="outlined"
                            onClick={downloadLogs}
                            disabled={visibleLogs.length === 0}
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
                            Download Visible ({visibleLogs.length})
                        </Button>

                        <Button
                            size="small"
                            variant="outlined"
                            onClick={clearVisibleLogs}
                            disabled={visibleLogs.length === 0}
                            sx={{
                                borderColor: "rgba(248,113,113,0.35)",
                                color: "#f87171",
                                borderRadius: "10px",
                                textTransform: "none",
                                fontWeight: 600,
                                boxShadow: "none",
                                "&:hover": {
                                    borderColor: "rgba(248,113,113,0.6)",
                                    background: "rgba(248,113,113,0.08)",
                                },
                                "&.Mui-disabled": {
                                    borderColor: "rgba(255,255,255,0.1)",
                                    color: "rgba(255,255,255,0.2)",
                                },
                            }}
                        >
                            Clear View
                        </Button>

                        {hiddenLogIds.size > 0 && (
                            <Button
                                size="small"
                                variant="outlined"
                                onClick={restoreLogs}
                                sx={{
                                    borderColor: "rgba(255,255,255,0.2)",
                                    color: "#d1d5db",
                                    borderRadius: "10px",
                                    textTransform: "none",
                                    fontWeight: 600,
                                    boxShadow: "none",
                                    "&:hover": {
                                        background: "rgba(255,255,255,0.06)",
                                    },
                                }}
                            >
                                Restore View
                            </Button>
                        )}

                        {mode !== "debug" ? (
                            <Typography
                                sx={{
                                    fontSize: 12,
                                    color: "#4ade80",
                                }}
                            >
                                Player mode — showing only player-facing logs. Switch to debug
                                mode to see all.
                            </Typography>
                        ) : (
                            <Typography
                                sx={{
                                    fontSize: 12,
                                    color: "#fb923c",
                                }}
                            >
                                Debug mode — showing all logs.
                            </Typography>
                        )}
                    </Stack>
                </Box>

                {/* Terminal */}
                <Box
                    ref={terminalRef}
                    sx={{
                        p: 2,
                        borderRadius: 3,
                        background: "#050505",
                        border: "1px solid rgba(255,255,255,0.06)",
                        fontFamily: "monospace",
                        maxHeight: 480,
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
                            const level = log.level as LogLevel;
                            const styles = LEVEL_STYLES[level] ?? LEVEL_STYLES.debug;
                            const category = getLogCategory(log.message, log.level);
                            const categoryStyles =
                                CATEGORY_STYLES[category] ?? CATEGORY_STYLES.system;

                            return (
                                <Box
                                    key={log.id}
                                    sx={{
                                        p: 1,
                                        borderRadius: "10px",
                                        background:
                                            category === "system" ? styles.bg : categoryStyles.bg,
                                        border:
                                            category === "system" ? styles.border : categoryStyles.border,
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
                                            {new Date(log.timestamp).toLocaleTimeString()}
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

                                        <Chip
                                            size="small"
                                            label={categoryStyles.label.toUpperCase()}
                                            sx={{
                                                height: 18,
                                                background: "rgba(255,255,255,0.025)",
                                                color: categoryStyles.color,
                                                border: categoryStyles.border,

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
                                No logs match the current filters.
                            </Typography>
                        )}
                    </Stack>
                </Box>
            </Stack>
        </StyledCard>
    );
}