//! Visual and message configuration types for the communication profile.
//!
//! Extracted from [`communication_profile`] to keep file sizes under 250 lines.
//! Contains overlay, timeline, blink, and message structs.

use serde::{Deserialize, Serialize};
use crate::communication_profile_defaults as defs;

// ── StandingOverlayConfig ────────────────────────────────────────────────────

/// Visual configuration for the overlay bar during standing sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandingOverlayConfig {
    /// Show a lap counter badge on the overlay.
    pub show_lap_counter: bool,
    /// Flash the bar briefly when a lap completes.
    pub flash_on_lap_complete: bool,
    /// CSS hex colour for the standing progress bar.
    pub bar_color: String,
    /// CSS hex colour for the lap-complete flash.
    pub lap_flash_color: String,
}

impl Default for StandingOverlayConfig {
    fn default() -> Self { defs::default_standing_overlay() }
}

// ── TimelineConfig ───────────────────────────────────────────────────────────

/// Colour keys for the daily timeline view in the popup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineConfig {
    pub sitting_within_limit: String,
    pub sitting_over_limit: String,
    pub standing_within_limit: String,
    pub standing_over_limit: String,
    pub away: String,
}

impl Default for TimelineConfig {
    fn default() -> Self { defs::default_timeline() }
}

// ── BlinkPattern ─────────────────────────────────────────────────────────────

/// Tray icon blink animation definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlinkPattern {
    /// Icon tint colour (CSS hex).
    pub color: String,
    /// Icon-on duration (ms).
    #[serde(default = "BlinkPattern::default_on_ms")]
    pub on_ms: u32,
    /// Icon-off duration (ms).
    #[serde(default = "BlinkPattern::default_off_ms")]
    pub off_ms: u32,
    /// Number of blink cycles before pausing.
    #[serde(default = "BlinkPattern::default_count")]
    pub count: u32,
    /// Pause between blink bursts (ms).
    #[serde(default = "BlinkPattern::default_pause_ms")]
    pub pause_ms: u32,
}

impl BlinkPattern {
    fn default_on_ms() -> u32 { 250 }
    fn default_off_ms() -> u32 { 250 }
    fn default_count() -> u32 { 3 }
    fn default_pause_ms() -> u32 { 8500 }
}

// ── OverlayPattern ───────────────────────────────────────────────────────────

/// Overlay bar pulsing animation definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayPattern {
    /// Bar colour (CSS hex).
    pub color: String,
    /// Minimum brightness during pulse (0.0–1.0).
    #[serde(default = "OverlayPattern::default_min")]
    pub min_brightness: f32,
    /// Maximum brightness during pulse (0.0–1.0).
    #[serde(default = "OverlayPattern::default_max")]
    pub max_brightness: f32,
    /// Full pulse cycle duration (ms).
    #[serde(default = "OverlayPattern::default_cycle")]
    pub cycle_ms: u32,
}

impl OverlayPattern {
    fn default_min() -> f32 { 0.4 }
    fn default_max() -> f32 { 1.0 }
    fn default_cycle() -> u32 { 1000 }
}

// ── MessageConfig ────────────────────────────────────────────────────────────

/// Localizable notification message templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageConfig {
    pub sitting_limit_toast: String,
    pub sitting_overdue_popup: String,
    pub standing_limit_toast: String,
    pub standing_overdue_popup: String,
    pub sensor_disconnected: String,
    /// Pool of neutral encouragement messages (randomly selected).
    pub neutral_popup_messages: Vec<String>,
    /// Pool of positive reinforcement messages (randomly selected).
    pub positive_popup_messages: Vec<String>,
}

impl Default for MessageConfig {
    fn default() -> Self { defs::default_messages() }
}

// ── PeriodicNotificationConfig ───────────────────────────────────────────────

/// Configuration for time-based background notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicNotificationConfig {
    /// Fire an inactivity reminder when idle too long.
    pub inactivity_enabled: bool,
    /// Minutes of inactivity before the reminder fires.
    pub inactivity_after_mins: u32,
    /// Fire a posture-balance summary mid-day.
    pub posture_balance_enabled: bool,
    /// Fire a positive praise notification at the halfway point.
    pub praise_halfway_enabled: bool,
}

impl Default for PeriodicNotificationConfig {
    fn default() -> Self { defs::default_periodic() }
}
