//! alert_manager.rs — Pure state machine for sit-too-long alert escalation.
//!
//! Tracks sitting progress and emits [`AlertAction`]s when thresholds are crossed.
//! Has no WinAPI or UI dependencies — callers execute the returned actions.
//!
//! ## State transitions
//! ```text
//! Idle ──(progress >= threshold)──→ Stage1 (bar pulses)
//! Stage1 ──(elapsed >= stage2_delay)──→ Stage2 (popup shown)
//! Stage1/Stage2 ──(on_standing)──→ Idle
//! Stage1/Stage2 ──(dismiss)──→ Snoozed → Stage1 (immediately in T013 stub)
//! ```

use std::time::{Duration, Instant};

// ─── Types ────────────────────────────────────────────────────────────────────

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
    ///
    /// T013 stub: `snoozed_until` is always `None`, so re-enters Stage1 immediately.
    Snoozed,
}

/// Actions the caller must execute after calling [`AlertManager::tick`] or similar.
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

/// Configuration for [`AlertManager`] escalation thresholds and feature flags.
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
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            stage1_threshold: 1.0,
            stage2_delay_secs: 120,
            stage3_enabled: false,
            stage4_enabled: false,
            stage5_enabled: false,
        }
    }
}

// ─── State Machine ────────────────────────────────────────────────────────────

/// Alert escalation state machine — no UI, no WinAPI.
///
/// Call [`tick`](AlertManager::tick) on every `desk:distance` event (~1/s).
/// Execute the returned [`AlertAction`]s in [`tray_controller.rs`].
pub struct AlertManager {
    stage: AlertStage,
    stage_entered_at: Instant,
    /// `None` in T013; T015 will set this to `Instant::now() + snooze_duration`.
    snoozed_until: Option<Instant>,
    config: AlertConfig,
}

impl AlertManager {
    /// Creates a new [`AlertManager`] with the given config.
    pub fn new(config: AlertConfig) -> Self {
        Self {
            stage: AlertStage::Idle,
            stage_entered_at: Instant::now(),
            snoozed_until: None,
            config,
        }
    }

    /// Returns the current alert stage (for debugging / tests).
    #[allow(dead_code)]
    pub fn stage(&self) -> AlertStage {
        self.stage
    }

    /// Processes one tick of sensor progress (~1/s).
    ///
    /// Returns a list of [`AlertAction`]s the caller must execute.
    /// Returns an empty list when no state change occurred.
    pub fn tick(&mut self, progress: f32) -> Vec<AlertAction> {
        match self.stage {
            AlertStage::Idle => {
                if progress >= self.config.stage1_threshold {
                    self.enter_stage1()
                } else {
                    vec![]
                }
            }

            AlertStage::Stage1 => {
                if progress < self.config.stage1_threshold {
                    self.enter_idle()
                } else if self.stage_entered_at.elapsed()
                    >= Duration::from_secs(self.config.stage2_delay_secs)
                {
                    self.enter_stage2()
                } else {
                    vec![]
                }
            }

            AlertStage::Stage2 => {
                if progress < self.config.stage1_threshold {
                    // Safety: user sat down again somehow (or limit changed)
                    self.enter_idle_with_dismiss()
                } else {
                    vec![]
                }
            }

            AlertStage::Snoozed => {
                // T013 stub: snoozed_until is always None → immediately re-enter Stage1
                let snooze_expired = self
                    .snoozed_until
                    .map(|until| Instant::now() >= until)
                    .unwrap_or(true);

                if progress < self.config.stage1_threshold {
                    // Safety cancel — progress dropped while snoozed
                    self.enter_idle()
                } else if snooze_expired {
                    self.enter_stage1()
                } else {
                    vec![]
                }
            }

            // Stages 3-5 not implemented — T017
            AlertStage::Stage3 | AlertStage::Stage4 | AlertStage::Stage5 => vec![],
        }
    }

    /// Called when desk transitions to [`DeskState::Standing`].
    ///
    /// Resets to Idle regardless of current stage. T015 will also reset snooze_index.
    pub fn on_standing(&mut self) -> Vec<AlertAction> {
        self.enter_idle_with_dismiss()
    }

    /// Handles a user dismiss (button click in popup).
    ///
    /// T013 stub: sets stage to Snoozed but `snoozed_until = None`, so the very
    /// next [`tick`](AlertManager::tick) call will immediately re-enter Stage1.
    /// T015 will set actual snooze durations.
    #[allow(dead_code)]
    pub fn dismiss(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Snoozed;
        self.snoozed_until = None; // T013 stub — T015 fills in actual duration
        vec![AlertAction::StopPulse, AlertAction::DismissPopup]
    }

    // ── Private transition helpers ───────────────────────────────────────────

    fn enter_stage1(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Stage1;
        self.stage_entered_at = Instant::now();
        vec![AlertAction::PulseBar]
    }

    fn enter_stage2(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Stage2;
        self.stage_entered_at = Instant::now();
        vec![AlertAction::ShowPopup(default_popup_message())]
    }

    fn enter_idle(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Idle;
        self.stage_entered_at = Instant::now();
        vec![AlertAction::StopPulse]
    }

    fn enter_idle_with_dismiss(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Idle;
        self.stage_entered_at = Instant::now();
        self.snoozed_until = None;
        vec![AlertAction::StopPulse, AlertAction::DismissPopup]
    }
}

/// Default message shown in the Stage2 popup.
fn default_popup_message() -> String {
    "You've been sitting too long.\nTime to stand up and stretch!".to_string()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn manager() -> AlertManager {
        AlertManager::new(AlertConfig::default())
    }

    fn manager_fast() -> AlertManager {
        AlertManager::new(AlertConfig {
            stage2_delay_secs: 0, // Instant Stage2 for tests
            ..AlertConfig::default()
        })
    }

    // ── Test 1: progress < 1.0 stays Idle ─────────────────────────────────────

    #[test]
    fn idle_below_threshold_no_actions() {
        let mut m = manager();
        let actions = m.tick(0.5);
        assert!(actions.is_empty());
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    #[test]
    fn idle_at_99pct_no_actions() {
        let mut m = manager();
        let actions = m.tick(0.99);
        assert!(actions.is_empty());
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    // ── Test 2: progress >= 1.0 → Stage1, emits PulseBar ─────────────────────

    #[test]
    fn idle_at_threshold_enters_stage1() {
        let mut m = manager();
        let actions = m.tick(1.0);
        assert_eq!(actions, vec![AlertAction::PulseBar]);
        assert_eq!(m.stage(), AlertStage::Stage1);
    }

    #[test]
    fn idle_above_threshold_enters_stage1() {
        let mut m = manager();
        let actions = m.tick(1.2);
        assert_eq!(actions, vec![AlertAction::PulseBar]);
        assert_eq!(m.stage(), AlertStage::Stage1);
    }

    // ── Test 3: Stage1 for < 2 min → stays Stage1, no new actions ────────────

    #[test]
    fn stage1_before_delay_no_actions() {
        let mut m = manager();
        m.tick(1.0); // enter Stage1
        let actions = m.tick(1.0); // still in Stage1, not enough time passed
        assert!(actions.is_empty());
        assert_eq!(m.stage(), AlertStage::Stage1);
    }

    // ── Test 4: Stage1 for >= 2 min → Stage2, emits ShowPopup ────────────────

    #[test]
    fn stage1_after_delay_enters_stage2() {
        let mut m = manager_fast(); // stage2_delay_secs = 0
        m.tick(1.0); // enter Stage1
        // With delay=0, first tick should already trigger — but entered_at was just set.
        // We need elapsed >= 0, which is immediately true after a moment.
        // Force elapsed by manipulating stage_entered_at.
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        let actions = m.tick(1.0);
        assert_eq!(m.stage(), AlertStage::Stage2);
        assert!(actions.iter().any(|a| matches!(a, AlertAction::ShowPopup(_))));
    }

    // ── Test 5: on_standing from Stage1 → Idle, emits StopPulse ─────────────

    #[test]
    fn on_standing_from_stage1_resets() {
        let mut m = manager();
        m.tick(1.0); // enter Stage1
        let actions = m.on_standing();
        assert!(actions.contains(&AlertAction::StopPulse));
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    // ── Test 6: on_standing from Stage2 → Idle, emits StopPulse + DismissPopup

    #[test]
    fn on_standing_from_stage2_dismisses_popup() {
        let mut m = manager_fast();
        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        m.tick(1.0); // Stage2
        assert_eq!(m.stage(), AlertStage::Stage2);

        let actions = m.on_standing();
        assert!(actions.contains(&AlertAction::StopPulse));
        assert!(actions.contains(&AlertAction::DismissPopup));
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    // ── Test 7: dismiss → Snoozed; next tick → Stage1 (snoozed_until=None) ───

    #[test]
    fn dismiss_then_tick_reenters_stage1() {
        let mut m = manager();
        m.tick(1.0); // Stage1
        let dismiss_actions = m.dismiss();
        assert!(dismiss_actions.contains(&AlertAction::StopPulse));
        assert!(dismiss_actions.contains(&AlertAction::DismissPopup));
        assert_eq!(m.stage(), AlertStage::Snoozed);

        // Next tick with progress >= threshold → immediately back to Stage1
        let actions = m.tick(1.0);
        assert_eq!(m.stage(), AlertStage::Stage1);
        assert!(actions.contains(&AlertAction::PulseBar));
    }

    // ── Test 8: progress < 1.0 while Stage2 → Idle, emits StopPulse + Dismiss

    #[test]
    fn stage2_progress_drops_enters_idle() {
        let mut m = manager_fast();
        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        m.tick(1.0); // Stage2
        assert_eq!(m.stage(), AlertStage::Stage2);

        let actions = m.tick(0.5); // progress drops
        assert!(actions.contains(&AlertAction::StopPulse));
        assert!(actions.contains(&AlertAction::DismissPopup));
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    // ── Test 9: progress oscillation 0.99→1.01→0.99 stays correct state ──────

    #[test]
    fn progress_oscillation_correct_states() {
        let mut m = manager();
        assert!(m.tick(0.99).is_empty()); // Idle
        m.tick(1.01); // Stage1
        assert_eq!(m.stage(), AlertStage::Stage1);

        let drop_actions = m.tick(0.99); // back below threshold
        assert!(drop_actions.contains(&AlertAction::StopPulse));
        assert_eq!(m.stage(), AlertStage::Idle);
    }

    // ── Test 10: rapid sit/stand/sit ends in correct state ────────────────────

    #[test]
    fn rapid_sit_stand_sit_ends_in_stage1() {
        let mut m = manager();

        m.tick(1.0); // Sitting → Stage1
        m.on_standing(); // Standing → Idle
        m.tick(1.0); // Sitting again → Stage1

        assert_eq!(m.stage(), AlertStage::Stage1);
    }
}
