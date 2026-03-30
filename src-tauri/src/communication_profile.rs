//! Communication profile — configures notification channels, escalation, and UI signals.
//!
//! Loaded from `profiles/communication/<id>.json` in the app data directory.
//! All fields carry serde defaults for graceful degradation on partial JSON.
//!
//! Default escalation, blink patterns, and overlay patterns are in
//! [`communication_profile_defaults`].

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::communication_profile_defaults as defs;

// ── EscalationStep ───────────────────────────────────────────────────────────

/// A single escalation threshold within a sitting or standing phase.
///
/// `at` is seconds relative to the phase limit (negative = before, 0 = at limit,
/// positive = overdue). Channels left empty string mean "no change from previous step".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    /// Seconds offset from limit trigger (-600 = 10 min before, 0 = at limit).
    pub at: i64,
    /// Tray signal name (e.g. `"yellow"`, `"blink_red"`).
    pub tray: String,
    /// Overlay signal name (e.g. `"red"`, `"pulse_red"`).
    pub overlay: String,
    /// Popup header colour name (e.g. `"yellow"`, `"red"`).
    pub popup_header: String,
    /// Optional notification backend to fire (`"toast"` or `"popup"`).
    pub notify: Option<String>,
}

// ── EscalationConfig ─────────────────────────────────────────────────────────

/// Escalation ladders for sitting and standing phases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Steps applied while the user is sitting (relative to sit limit).
    #[serde(default = "defs::default_sitting_escalation")]
    pub sitting: Vec<EscalationStep>,

    /// Steps applied while the user is standing too long (relative to stand max).
    #[serde(default = "defs::default_standing_escalation")]
    pub standing: Vec<EscalationStep>,
}

impl Default for EscalationConfig {
    fn default() -> Self {
        Self {
            sitting: defs::default_sitting_escalation(),
            standing: defs::default_standing_escalation(),
        }
    }
}

// ── SnoozeConfig ─────────────────────────────────────────────────────────────

/// Snooze / dismiss configuration for alert popups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnoozeConfig {
    /// Available snooze durations shown to the user (minutes).
    #[serde(default = "defs::default_snooze_durations")]
    pub durations_mins: Vec<u32>,

    /// Number of dismisses before the communication tone shifts (becomes firmer).
    #[serde(default = "defs::default_tone_shift")]
    pub tone_shift_after_dismisses: u8,
}

impl Default for SnoozeConfig {
    fn default() -> Self {
        Self {
            durations_mins: defs::default_snooze_durations(),
            tone_shift_after_dismisses: defs::default_tone_shift(),
        }
    }
}

// ── ChannelConfig ────────────────────────────────────────────────────────────

/// Channel overrides for a named UI state (e.g. disconnected, inactive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Tray signal name.
    pub tray: String,
    /// Overlay signal name.
    pub overlay: String,
    /// Popup header colour name.
    pub popup_header: String,
    /// One-shot notification backend (only fires once until state clears).
    pub notify_once: Option<String>,
}

// ── BaselineChannels ─────────────────────────────────────────────────────────

/// Channel configuration for a single baseline position (sitting or standing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineChannels {
    /// Tray signal name.
    pub tray: String,
    /// Overlay signal name.
    pub overlay: String,
    /// Popup signal name.
    pub popup: String,
}

// ── BaselineConfig ───────────────────────────────────────────────────────────

/// Channel defaults for the calm/within-limit state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    /// Channels while sitting within the session limit.
    #[serde(default)]
    pub sitting: BaselineChannels,

    /// Channels while standing (normal standing session in progress).
    #[serde(default)]
    pub standing: BaselineChannels,
}

impl Default for BaselineChannels {
    fn default() -> Self {
        Self {
            tray: "none".to_string(),
            overlay: "neutral".to_string(),
            popup: "neutral".to_string(),
        }
    }
}

impl Default for BaselineConfig {
    fn default() -> Self { defs::default_baseline() }
}

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

// ── CommunicationProfile ─────────────────────────────────────────────────────

/// Full communication and notification configuration for SmartDesk.
///
/// Loaded from `profiles/communication/<id>.json` in the app data directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationProfile {
    /// Unique identifier (file stem, e.g. `"default"`).
    #[serde(default = "CommunicationProfile::default_id")]
    pub id: String,

    /// Human-readable name shown in the settings panel.
    #[serde(default = "CommunicationProfile::default_name")]
    pub name: String,

    /// Short description of this profile's intent.
    #[serde(default = "CommunicationProfile::default_description")]
    pub description: String,

    /// Backend used for one-shot notifications (`"toast"` | `"popup"`).
    #[serde(default = "defs::default_notification_backend")]
    pub notification_backend: String,

    /// Escalation ladder for sitting and standing phases.
    #[serde(default)]
    pub escalation: EscalationConfig,

    /// Snooze and dismiss behaviour.
    #[serde(default)]
    pub snooze: SnoozeConfig,

    /// Channels used when the sensor is disconnected.
    #[serde(default = "defs::default_disconnected")]
    pub disconnected: ChannelConfig,

    /// Channels used when the user is inactive (keyboard/mouse idle).
    #[serde(default = "defs::default_inactive")]
    pub inactive: ChannelConfig,

    /// Calm-state channels (within session limits).
    #[serde(default)]
    pub baseline: BaselineConfig,

    /// Overlay-specific standing configuration.
    #[serde(default)]
    pub standing_overlay: StandingOverlayConfig,

    /// Timeline segment colours.
    #[serde(default)]
    pub timeline: TimelineConfig,

    /// Named blink animations referenced by escalation steps.
    #[serde(default = "defs::default_blink_patterns")]
    pub blink_patterns: HashMap<String, BlinkPattern>,

    /// Named overlay animations referenced by escalation steps.
    #[serde(default = "defs::default_overlay_patterns")]
    pub overlay_patterns: HashMap<String, OverlayPattern>,

    /// Localizable message templates.
    #[serde(default)]
    pub messages: MessageConfig,

    /// Periodic background notification settings.
    #[serde(default)]
    pub periodic_notifications: PeriodicNotificationConfig,
}

impl CommunicationProfile {
    fn default_id() -> String { "default".to_string() }
    fn default_name() -> String { "Default Communication Profile".to_string() }
    fn default_description() -> String {
        "Standard toast notifications with progressive escalation.".to_string()
    }
}

impl Default for CommunicationProfile {
    fn default() -> Self {
        Self {
            id: Self::default_id(),
            name: Self::default_name(),
            description: Self::default_description(),
            notification_backend: defs::default_notification_backend(),
            escalation: EscalationConfig::default(),
            snooze: SnoozeConfig::default(),
            disconnected: defs::default_disconnected(),
            inactive: defs::default_inactive(),
            baseline: BaselineConfig::default(),
            standing_overlay: StandingOverlayConfig::default(),
            timeline: TimelineConfig::default(),
            blink_patterns: defs::default_blink_patterns(),
            overlay_patterns: defs::default_overlay_patterns(),
            messages: MessageConfig::default(),
            periodic_notifications: PeriodicNotificationConfig::default(),
        }
    }
}
