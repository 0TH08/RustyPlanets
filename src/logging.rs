//! Logging helpers for the Skycartel planet crate.
//!
//! This module provides a small adapter around `log` so the planet can emit
//! structured-ish messages while still using the enums from `common-game`.

use crate::planet_ai::RustyPlanetId;
use common_game::logging::{Channel, EventType};

fn channel_to_level(channel: Channel) -> log::Level {
    match channel {
        Channel::Info => log::Level::Info,
        Channel::Warning => log::Level::Warn,
        Channel::Error => log::Level::Error,
        Channel::Debug => log::Level::Debug,
        Channel::Trace => log::Level::Trace,
    }
}

/// Log an event emitted by the planet.
///
/// `meta` is optional key/value data that will be appended as `k=v` pairs.
/// Keep metadata small; it's meant for debugging, not payload transport.
pub fn log_planet_event<I, K>(
    planet_id: RustyPlanetId,
    event_type: EventType,
    channel: Channel,
    message: &str,
    meta: Option<I>,
) where
    I: IntoIterator<Item = (K, String)>,
    K: AsRef<str>,
{
    let mut line = format!(
        "planet_id={} event_type={:?} msg=\"{}\"",
        planet_id.0, event_type, message
    );

    if let Some(meta) = meta {
        for (k, v) in meta {
            line.push(' ');
            line.push_str(k.as_ref());
            line.push('=');
            line.push_str(&sanitize_value(&v));
        }
    }

    log::log!(channel_to_level(channel), "{}", line);
}

fn sanitize_value(v: &str) -> String {
    v.replace(['\n', '\r', '\t'], " ")
}
