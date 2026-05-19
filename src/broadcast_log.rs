//! Broadcast log sink — pipes `log!()` calls into a tokio broadcast channel.
//!
//! This implements the `log::Log` trait so any `log::info!()`, `log::warn!()`, etc.
//! call from the sync simulation engine is captured and forwarded to async
//! WebSocket clients.
//!
//! # Example
//!
//! ```ignore
//! let (tx, rx) = BroadcastLogger::init().unwrap();
//! log::info!("Hello from the simulation!");
//! // rx will receive this log entry
//! ```

use log::{Level, Metadata, Record};
use tokio::sync::broadcast;

/// Log a player-facing message (appears in player-friendly log view).
#[macro_export]
macro_rules! player_log {
    ($($arg:tt)*) => {
        log::info!("[PLAYER] {}", format_args!($($arg)*))
    };
}

/// Serializable log entry sent to WebSocket clients.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub message: String,
    pub player: bool,
}

/// A log sink that broadcasts log records to all subscribers.
#[allow(dead_code)]
pub struct BroadcastLogger {
    sender: broadcast::Sender<LogEntry>,
}

impl BroadcastLogger {
    /// Creates a new broadcast logger and installs it as the global logger.
    ///
    /// Returns a sender that can be cloned to produce subscribers, and a
    /// receiver for the initial subscriber.
    pub fn init() -> Result<(broadcast::Sender<LogEntry>, broadcast::Receiver<LogEntry>), log::SetLoggerError> {
        let (sender, _) = broadcast::channel::<LogEntry>(1024);
        let receiver = sender.subscribe();

        let logger = Box::new(Self {
            sender: sender.clone(),
        });

        log::set_boxed_logger(logger)?;
        log::set_max_level(log::LevelFilter::Debug);

        Ok((sender, receiver))
    }

    fn record_to_entry(record: &Record) -> LogEntry {
        let now = chrono::Utc::now();
        let raw = record.args().to_string();
        let (player, message) = if raw.starts_with("[PLAYER]") {
            (true, raw[8..].trim().to_string())
        } else {
            (false, raw)
        };
        LogEntry {
            timestamp: now.to_rfc3339(),
            level: record.level().to_string(),
            target: record.target().to_string(),
            module_path: record.module_path().map(String::from),
            file: record.file().map(String::from),
            line: record.line(),
            message,
            player,
        }
    }
}

impl log::Log for BroadcastLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let entry = Self::record_to_entry(record);
            // Ignore send errors — means no subscribers are listening
            let _ = self.sender.send(entry);
        }
    }

    fn flush(&self) {
        // no-op for broadcast channel
    }
}
