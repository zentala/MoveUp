//! alert_config.rs — Configuration struct for the AlertManager escalation thresholds.

use std::time::Duration;

/// Configuration for [`super::alert_manager::AlertManager`] escalation thresholds and feature flags.
#[derive(Debug, Clone)]
#[allow(dead_code)] // stage3/4/5_enabled are T017 stubs, read when those stages are wired
pub struct AlertConfig {
    /// Progress fraction at which Stage1 activates (default: 1.0 = 100%).
    pub stage1_threshold: f32,
    /// Seconds to remain in Stage1 before escalating to Stage2 (default: 120 = 2 min).
    pub stage2_delay_secs: u64,
    /// Whether Stage3 escalation is enabled (T017).
    pub stage3_enabled: bool,
    /// Whether Stage4 escalation is enabled (T017).
    pub stage4_enabled: bool,
    /// Whether Stage5 escalation is enabled (T017).
    pub stage5_enabled: bool,
    /// Snooze durations indexed by dismiss count (default: [5, 15, 30, 60] minutes).
    ///
    /// dismiss #1 → index 0 → 5 min, dismiss #2 → index 1 → 15 min, etc.
    /// The last value is used for all subsequent dismisses.
    pub snooze_durations: Vec<Duration>,
    /// Messages shown when `snooze_index < tone_shift_threshold`.
    pub neutral_messages: Vec<String>,
    /// Messages shown when `snooze_index >= tone_shift_threshold`.
    pub positive_messages: Vec<String>,
    /// Number of dismisses before switching from neutral to positive messages (default: 3).
    pub tone_shift_threshold: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            stage1_threshold: 1.0,
            stage2_delay_secs: 120,
            stage3_enabled: false,
            stage4_enabled: false,
            stage5_enabled: false,
            snooze_durations: vec![
                Duration::from_secs(5 * 60),
                Duration::from_secs(15 * 60),
                Duration::from_secs(30 * 60),
                Duration::from_secs(60 * 60),
            ],
            neutral_messages: vec![
                "Time for a stretch!".to_string(),
                "Your body needs a break".to_string(),
            ],
            positive_messages: vec![
                "Even 2 min standing helps blood flow".to_string(),
                "Quick stand = fresh mind".to_string(),
            ],
            tone_shift_threshold: 3,
        }
    }
}
