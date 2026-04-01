//! Error types for the Skycartel planet implementation.
//!
//! All crate-specific failures are funneled through a single error enum to keep
//! construction and runtime handling consistent.

use thiserror::Error;

/// Skycartel-specific errors.
#[derive(Error, Debug)]
pub enum SkycartelError {
    /// Failed to construct the `common_game::Planet` wrapper.
    #[error("Planet construction failed: {0}")]
    PlanetConstruction(String),

    /// Energy cell configuration or accounting error.
    #[error("Energy configuration error: {0}")]
    Energy(String),

    /// Resource generation / validation error.
    #[error("Resource error: {0}")]
    Resource(String),

    /// Generic operational error.
    #[error("Operational error: {0}")]
    Operational(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_are_stable() {
        let e = SkycartelError::Energy("bad cells".to_string());
        assert_eq!(e.to_string(), "Energy configuration error: bad cells");

        let e = SkycartelError::PlanetConstruction("wrap failed".to_string());
        assert_eq!(e.to_string(), "Planet construction failed: wrap failed");
    }

    #[test]
    fn debug_formatting_does_not_panic() {
        let e = SkycartelError::Resource("x".to_string());
        let _ = format!("{e:?}");
    }
}
