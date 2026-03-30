//! Default values for [`CommunicationProfile`] sub-structs.
//!
//! Kept separate to stay under the 250-line file limit. Re-exported through
//! `communication_profile` — callers only need to `use communication_profile::*`.

use std::collections::HashMap;
use super::communication_profile::{
    BaselineChannels, BaselineConfig, BlinkPattern, ChannelConfig,
    EscalationStep, MessageConfig, OverlayPattern, PeriodicNotificationConfig,
    StandingOverlayConfig, TimelineConfig,
};

// ── serde default helpers ────────────────────────────────────────────────────

pub(super) fn default_notification_backend() -> String { "toast".to_string() }
pub(super) fn default_snooze_durations() -> Vec<u32> { vec![5, 15, 30, 60] }
pub(super) fn default_tone_shift() -> u8 { 3 }
/// Escalating cooldown: immediate → 5 min → 15 min → 30 min → silence.
pub(super) fn default_notify_cooldowns() -> Vec<u64> { vec![0, 300, 900, 1800] }

pub(super) fn default_disconnected() -> ChannelConfig {
    ChannelConfig {
        tray: "blink_gray".to_string(),
        overlay: "hidden".to_string(),
        popup_header: "gray".to_string(),
        notify_once: Some("toast".to_string()),
    }
}

pub(super) fn default_inactive() -> ChannelConfig {
    ChannelConfig {
        tray: "none".to_string(),
        overlay: "hidden".to_string(),
        popup_header: "neutral".to_string(),
        notify_once: None,
    }
}

pub(super) fn default_baseline() -> BaselineConfig {
    BaselineConfig {
        sitting: BaselineChannels {
            tray: "none".to_string(),
            overlay: "neutral".to_string(),
            popup: "neutral".to_string(),
        },
        standing: BaselineChannels {
            tray: "none".to_string(),
            overlay: "progress".to_string(),
            popup: "neutral".to_string(),
        },
    }
}

pub(super) fn default_standing_overlay() -> StandingOverlayConfig {
    StandingOverlayConfig {
        show_lap_counter: true,
        flash_on_lap_complete: true,
        bar_color: "#b0a898".to_string(),
        lap_flash_color: "#ffc107".to_string(),
    }
}

pub(super) fn default_timeline() -> TimelineConfig {
    TimelineConfig {
        sitting_within_limit: "neutral".to_string(),
        sitting_over_limit: "red".to_string(),
        standing_within_limit: "subtle".to_string(),
        standing_over_limit: "red".to_string(),
        away: "gray".to_string(),
    }
}

pub(super) fn default_sitting_escalation() -> Vec<EscalationStep> {
    vec![
        EscalationStep {
            at: -600,
            tray: "yellow".to_string(),
            overlay: "yellow".to_string(),
            popup_header: "yellow".to_string(),
            notify: None,
        },
        EscalationStep {
            at: 0,
            tray: "red".to_string(),
            overlay: "red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        },
        // Visual-only escalation at +5 min: blink + pulse, no second notification.
        // Reminder toast fires later via notify_count cooldown (5 min after first).
        EscalationStep {
            at: 300,
            tray: "blink_red".to_string(),
            overlay: "pulse_red".to_string(),
            popup_header: "red".to_string(),
            notify: None,
        },
    ]
}

pub(super) fn default_standing_escalation() -> Vec<EscalationStep> {
    vec![
        EscalationStep {
            at: -300,
            tray: "yellow".to_string(),
            overlay: "yellow".to_string(),
            popup_header: "yellow".to_string(),
            notify: None,
        },
        EscalationStep {
            at: 0,
            tray: "red".to_string(),
            overlay: "red".to_string(),
            popup_header: "red".to_string(),
            notify: Some("toast".to_string()),
        },
        // Visual-only escalation at +5 min: blink + pulse, no second notification.
        // Reminder toast fires later via notify_count cooldown (5 min after first).
        EscalationStep {
            at: 300,
            tray: "blink_red".to_string(),
            overlay: "pulse_red".to_string(),
            popup_header: "red".to_string(),
            notify: None,
        },
    ]
}

/// Default blink patterns keyed by name.
pub(super) fn default_blink_patterns() -> HashMap<String, BlinkPattern> {
    let mut m = HashMap::new();
    m.insert("blink_red".to_string(), BlinkPattern {
        color: "#e53935".to_string(),
        on_ms: 250,
        off_ms: 250,
        count: 3,
        pause_ms: 8500,
    });
    m.insert("blink_gray".to_string(), BlinkPattern {
        color: "#9e9e9e".to_string(),
        on_ms: 250,
        off_ms: 250,
        count: 3,
        pause_ms: 8500,
    });
    m
}

/// Default overlay animation patterns keyed by name.
pub(super) fn default_overlay_patterns() -> HashMap<String, OverlayPattern> {
    let mut m = HashMap::new();
    m.insert("pulse_red".to_string(), OverlayPattern {
        color: "#e53935".to_string(),
        min_brightness: 0.4,
        max_brightness: 1.0,
        cycle_ms: 1000,
    });
    m
}

pub(super) fn default_messages() -> MessageConfig {
    MessageConfig {
        sitting_limit_toast: "Time to stand! You've been sitting for too long.".to_string(),
        sitting_overdue_popup: "Still sitting? Stand up and move around.".to_string(),
        standing_limit_toast: "Great standing session! Consider sitting down now.".to_string(),
        standing_overdue_popup: "You've been standing a long time. Take a seat.".to_string(),
        sensor_disconnected: "Sensor disconnected — desk position unknown.".to_string(),
        neutral_popup_messages: vec![
            "Stay consistent — small breaks add up.".to_string(),
            "Regular position changes keep you sharp.".to_string(),
        ],
        positive_popup_messages: vec![
            "Great work! Keep alternating positions.".to_string(),
            "You're on a roll — excellent posture habits today.".to_string(),
        ],
    }
}

pub(super) fn default_nudge_enabled() -> bool { true }

pub(super) fn default_nudge_messages() -> Vec<String> {
    vec![
        "You've been at the screen a while — perfect moment to grab water".to_string(),
        "Your eyes would love a 5-minute break".to_string(),
        "Quick stretch? 2 minutes is all it takes".to_string(),
        "Step away for a moment — your focus will be sharper when you return".to_string(),
        "Screen break time — look out the window for a minute".to_string(),
    ]
}

pub(super) fn default_periodic() -> PeriodicNotificationConfig {
    PeriodicNotificationConfig {
        inactivity_enabled: true,
        inactivity_after_mins: 60,
        posture_balance_enabled: true,
        praise_halfway_enabled: true,
    }
}
