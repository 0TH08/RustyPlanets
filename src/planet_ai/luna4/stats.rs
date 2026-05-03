//! Statistics tracking for Luna4
//!
//! This module tracks various operational metrics for Luna4 planets
//! that are used for monitoring and debugging purposes.

/// Statistics tracking for Luna4 operations

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Luna4Stats {
    /// Number of successful resource generations
    pub successful_generations: usize,
    /// Number of failed resource generations
    pub failed_generations: usize,
    /// Number of sunrays received
    pub sunrays_received: usize,
    /// Number of explorer messages processed
    pub explorer_messages_processed: usize,
}

#[allow(dead_code)]
impl Luna4Stats {
    /// Creates new statistics with all counts zero
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a successful resource generation
    pub fn record_successful_generation(&mut self) {
        self.successful_generations += 1;
    }

    /// Records a failed resource generation
    pub fn record_failed_generation(&mut self) {
        self.failed_generations += 1;
    }

    /// Records reception of a sunray
    pub fn record_sunray_received(&mut self) {
        self.sunrays_received += 1;
    }

    /// Records processing of an explorer message
    pub fn record_explorer_message_processed(&mut self) {
        self.explorer_messages_processed += 1;
    }

    /// Calculates the success rate for resource generation
    pub fn generation_success_rate(&self) -> f32 {
        let total_attempts = self.successful_generations + self.failed_generations;
        if total_attempts == 0 {
            return 0.0;
        }
        (self.successful_generations as f32 / total_attempts as f32) * 100.0
    }

    /// Creates a display-friendly summary of the statistics
    pub fn display_summary(&self) -> String {
        format!(
            "Stats: {} successful / {} failed generations ({:.1}% success), {} sunrays received, {} explorer messages processed",
            self.successful_generations,
            self.failed_generations,
            self.generation_success_rate(),
            self.sunrays_received,
            self.explorer_messages_processed
        )
    }
}