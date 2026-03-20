//! alert_actions.rs — AlertStage and AlertAction enums for the alert state machine.
//!
//! These types are consumed by [`super::alert_manager::AlertManager`] and
//! executed by `tray_controller.rs`.

/// Current escalation stage of the alert system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Stage3-5 and Snoozed are T017 stubs, always present for state machine completeness
pub enum AlertStage {
    /// No alert active; user is within sitting limit.
    Idle,
    /// Sitting limit reached — progress bar is pulsing.
    Stage1,
    /// Stage1 has been active for ≥ `stage2_delay_secs` — popup is shown.
    Stage2,
    /// Future escalation stages (T017).
    Stage3,
    Stage4,
    Stage5,
    /// Alert was dismissed by user; waiting for snooze cooldown.
    Snoozed,
}

/// Actions the caller must execute after calling [`super::alert_manager::AlertManager::tick`] or similar.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // ExpandOverlay and FullScreenNudge are T017 stubs
pub enum AlertAction {
    /// Set overlay variant to 2 (pulsing animation).
    PulseBar,
    /// Set overlay variant to 0 (solid).
    StopPulse,
    /// Spawn a WinAPI popup with the given message.
    ShowPopup(String),
    /// Signal the popup thread to exit.
    DismissPopup,
    /// Future: increase bar height (T017).
    ExpandOverlay,
    /// Future: full-screen nudge (T017).
    FullScreenNudge,
}
