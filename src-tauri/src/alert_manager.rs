//! alert_manager.rs — Pure state machine for sit-too-long alert escalation.
//!
//! Tracks sitting progress and emits [`AlertAction`]s when thresholds are crossed.
//! Has no WinAPI or UI dependencies — callers execute the returned actions.
//!
//! ## State transitions
//! ```text
//! Idle ──(progress >= threshold)──→ Stage1 (bar pulses)
//! Stage1 ──(elapsed >= stage2_delay)──→ Stage2 (popup shown)
//! Stage1/Stage2 ──(on_standing)──→ Idle (snooze_index reset)
//! Stage2 ──(dismiss)──→ Snoozed (duration from snooze_durations[snooze_index])
//! Snoozed ──(expired)──→ Stage1
//! Snoozed ──(progress < threshold)──→ Idle (problem resolved)
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

// ─── State Machine ────────────────────────────────────────────────────────────

/// Alert escalation state machine — no UI, no WinAPI.
///
/// Call [`tick`](AlertManager::tick) on every `desk:distance` event (~1/s).
/// Execute the returned [`AlertAction`]s in [`tray_controller.rs`].
pub struct AlertManager {
    stage: AlertStage,
    stage_entered_at: Instant,
    /// Set to `Instant::now() + snooze_duration` on dismiss; `None` when not snoozed.
    snoozed_until: Option<Instant>,
    config: AlertConfig,
    /// Incremented on each dismiss; reset to 0 on standing.
    /// Controls snooze duration and popup message tone.
    snooze_index: usize,
}

impl AlertManager {
    /// Creates a new [`AlertManager`] with the given config.
    pub fn new(config: AlertConfig) -> Self {
        Self {
            stage: AlertStage::Idle,
            stage_entered_at: Instant::now(),
            snoozed_until: None,
            config,
            snooze_index: 0,
        }
    }

    /// Returns the current alert stage (for debugging / tests).
    #[allow(dead_code)]
    pub fn stage(&self) -> AlertStage {
        self.stage
    }

    /// Returns the current snooze index (for debugging / tests).
    #[allow(dead_code)]
    pub fn snooze_index(&self) -> usize {
        self.snooze_index
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
                if progress < self.config.stage1_threshold {
                    // Problem resolved — cancel snooze silently
                    self.snoozed_until = None;
                    self.enter_idle()
                } else if self
                    .snoozed_until
                    .map(|until| Instant::now() >= until)
                    .unwrap_or(true)
                {
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
    /// Resets to Idle and clears snooze state including the dismiss counter.
    pub fn on_standing(&mut self) -> Vec<AlertAction> {
        self.stage = AlertStage::Idle;
        self.stage_entered_at = Instant::now();
        self.snooze_index = 0;
        self.snoozed_until = None;
        vec![AlertAction::StopPulse, AlertAction::DismissPopup]
    }

    /// Handles a user dismiss (button click in popup).
    ///
    /// Snooze duration is selected from `config.snooze_durations[snooze_index]`,
    /// capped at the last entry. `snooze_index` is incremented after selection.
    pub fn dismiss(&mut self) -> Vec<AlertAction> {
        let idx = self
            .snooze_index
            .min(self.config.snooze_durations.len().saturating_sub(1));
        self.snoozed_until = Some(Instant::now() + self.config.snooze_durations[idx]);
        self.snooze_index += 1;
        self.stage = AlertStage::Snoozed;
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
        vec![AlertAction::ShowPopup(self.popup_message().to_string())]
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

    /// Selects the popup message based on the current `snooze_index`.
    ///
    /// Uses neutral messages before `tone_shift_threshold` dismisses,
    /// then switches to positive messages.
    fn popup_message(&self) -> &str {
        let messages = if self.snooze_index >= self.config.tone_shift_threshold {
            &self.config.positive_messages
        } else {
            &self.config.neutral_messages
        };
        let idx = self.snooze_index % messages.len().max(1);
        &messages[idx]
    }
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

    /// Manager with instant Stage2 and very short snooze durations for snooze expiry tests.
    fn manager_fast_snooze() -> AlertManager {
        AlertManager::new(AlertConfig {
            stage2_delay_secs: 0,
            snooze_durations: vec![
                Duration::from_millis(1),
                Duration::from_millis(1),
                Duration::from_millis(1),
                Duration::from_millis(1),
            ],
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

    // ── Test 7: dismiss → Snoozed; next tick after expiry → Stage1 ───────────

    #[test]
    fn dismiss_then_tick_reenters_stage1_after_snooze_expiry() {
        let mut m = manager_fast_snooze();
        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        m.tick(1.0); // Stage2

        let dismiss_actions = m.dismiss();
        assert!(dismiss_actions.contains(&AlertAction::StopPulse));
        assert!(dismiss_actions.contains(&AlertAction::DismissPopup));
        assert_eq!(m.stage(), AlertStage::Snoozed);
        assert_eq!(m.snooze_index(), 1);

        // Wait for snooze to expire (1ms duration)
        std::thread::sleep(Duration::from_millis(5));

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

    // ── T015 Test 1: dismiss #1 → 5 min snooze, snooze_index=1 ──────────────

    #[test]
    fn dismiss_first_sets_5min_snooze() {
        let mut m = manager();
        m.tick(1.0); // Stage1
        m.dismiss();
        assert_eq!(m.stage(), AlertStage::Snoozed);
        assert_eq!(m.snooze_index(), 1);
        // snoozed_until should be approximately now + 5 min
        let until = m.snoozed_until.expect("snoozed_until must be set");
        let remaining = until.duration_since(Instant::now());
        // Should be between 4:59 and 5:01
        assert!(remaining <= Duration::from_secs(5 * 60 + 1));
        assert!(remaining >= Duration::from_secs(5 * 60 - 1));
    }

    // ── T015 Test 2: dismiss #2 → 15 min snooze, snooze_index=2 ─────────────

    #[test]
    fn dismiss_second_sets_15min_snooze() {
        let mut m = manager();
        m.tick(1.0); // Stage1
        m.dismiss(); // snooze_index → 1, 5 min
        // Re-enter stage to allow dismiss again
        m.stage = AlertStage::Stage2;
        m.dismiss(); // snooze_index → 2, 15 min
        assert_eq!(m.snooze_index(), 2);
        let until = m.snoozed_until.expect("snoozed_until must be set");
        let remaining = until.duration_since(Instant::now());
        assert!(remaining <= Duration::from_secs(15 * 60 + 1));
        assert!(remaining >= Duration::from_secs(15 * 60 - 1));
    }

    // ── T015 Test 3: dismiss #3 → 30 min snooze, snooze_index=3 ─────────────

    #[test]
    fn dismiss_third_sets_30min_snooze() {
        let mut m = manager();
        m.tick(1.0);
        m.dismiss(); // index 0 → 5 min, snooze_index=1
        m.stage = AlertStage::Stage2;
        m.dismiss(); // index 1 → 15 min, snooze_index=2
        m.stage = AlertStage::Stage2;
        m.dismiss(); // index 2 → 30 min, snooze_index=3
        assert_eq!(m.snooze_index(), 3);
        let until = m.snoozed_until.expect("snoozed_until must be set");
        let remaining = until.duration_since(Instant::now());
        assert!(remaining <= Duration::from_secs(30 * 60 + 1));
        assert!(remaining >= Duration::from_secs(30 * 60 - 1));
    }

    // ── T015 Test 4: dismiss #4 → 60 min snooze (capped), snooze_index=4 ────

    #[test]
    fn dismiss_fourth_sets_60min_snooze_capped() {
        let mut m = manager();
        m.tick(1.0);
        m.dismiss(); // index 0 → 5 min
        m.stage = AlertStage::Stage2;
        m.dismiss(); // index 1 → 15 min
        m.stage = AlertStage::Stage2;
        m.dismiss(); // index 2 → 30 min
        m.stage = AlertStage::Stage2;
        m.dismiss(); // index 3 → 60 min (last entry)
        assert_eq!(m.snooze_index(), 4);
        let until = m.snoozed_until.expect("snoozed_until must be set");
        let remaining = until.duration_since(Instant::now());
        assert!(remaining <= Duration::from_secs(60 * 60 + 1));
        assert!(remaining >= Duration::from_secs(60 * 60 - 1));
    }

    // ── T015 Test 5: standing while Snoozed → Idle, snooze_index=0 ───────────

    #[test]
    fn standing_while_snoozed_resets_snooze_index() {
        let mut m = manager();
        m.tick(1.0); // Stage1
        m.dismiss(); // Snoozed, snooze_index=1
        assert_eq!(m.stage(), AlertStage::Snoozed);
        assert_eq!(m.snooze_index(), 1);
        assert!(m.snoozed_until.is_some());

        let actions = m.on_standing();
        assert_eq!(m.stage(), AlertStage::Idle);
        assert_eq!(m.snooze_index(), 0);
        assert!(m.snoozed_until.is_none());
        assert!(actions.contains(&AlertAction::StopPulse));
        assert!(actions.contains(&AlertAction::DismissPopup));
    }

    // ── T015 Test 6: progress < 1.0 while Snoozed → Idle (cancel snooze) ────

    #[test]
    fn progress_drops_while_snoozed_enters_idle() {
        let mut m = manager();
        m.tick(1.0); // Stage1
        m.dismiss(); // Snoozed
        assert_eq!(m.stage(), AlertStage::Snoozed);

        let actions = m.tick(0.5); // progress below threshold
        assert_eq!(m.stage(), AlertStage::Idle);
        assert!(actions.contains(&AlertAction::StopPulse));
        // DismissPopup not expected — popup was already dismissed on dismiss()
        assert!(!actions.contains(&AlertAction::DismissPopup));
    }

    // ── T015 Test 7: dismiss returns [StopPulse, DismissPopup]; expiry → PulseBar

    #[test]
    fn dismiss_actions_and_snooze_expiry_emits_pulse_bar() {
        let mut m = manager_fast_snooze();
        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        m.tick(1.0); // Stage2

        let dismiss_actions = m.dismiss();
        assert_eq!(
            dismiss_actions,
            vec![AlertAction::StopPulse, AlertAction::DismissPopup]
        );
        assert_eq!(m.stage(), AlertStage::Snoozed);

        std::thread::sleep(Duration::from_millis(5));

        let actions = m.tick(1.0);
        // Snooze expired → back to Stage1, not Stage2
        assert_eq!(m.stage(), AlertStage::Stage1);
        assert_eq!(actions, vec![AlertAction::PulseBar]);
    }

    // ── T015 Test 8: snooze expiry → Stage1 (not Stage2 directly) ────────────

    #[test]
    fn snooze_expiry_enters_stage1_not_stage2() {
        let mut m = manager_fast_snooze();
        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        m.tick(1.0); // Stage2
        m.dismiss(); // Snoozed

        std::thread::sleep(Duration::from_millis(5));

        // First tick after expiry: Stage1 (PulseBar), not Stage2
        let actions = m.tick(1.0);
        assert_eq!(m.stage(), AlertStage::Stage1);
        assert!(actions.contains(&AlertAction::PulseBar));
        assert!(!actions.iter().any(|a| matches!(a, AlertAction::ShowPopup(_))));
    }

    // ── T015 Test bonus: tone shift — neutral before #3, positive at #3+ ──────

    fn extract_popup_msg(actions: &[AlertAction]) -> String {
        actions
            .iter()
            .find_map(|a| {
                if let AlertAction::ShowPopup(s) = a {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .expect("ShowPopup must be present")
    }

    #[test]
    fn popup_message_tone_shifts_at_threshold() {
        let mut m = manager_fast();
        // Clone config message lists to avoid borrow conflicts
        let neutral_msgs: Vec<String> = m.config.neutral_messages.clone();
        let positive_msgs: Vec<String> = m.config.positive_messages.clone();

        m.tick(1.0); // Stage1
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        let actions = m.tick(1.0); // Stage2, snooze_index=0 → neutral
        let msg = extract_popup_msg(&actions);
        assert!(
            neutral_msgs.iter().any(|n| n == &msg),
            "expected neutral message, got: {msg}"
        );

        // Dismiss #1 → snooze_index=1, still neutral
        m.dismiss();
        m.stage = AlertStage::Stage1;
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        let actions2 = m.tick(1.0); // Stage2, snooze_index=1
        let msg2 = extract_popup_msg(&actions2);
        assert!(
            neutral_msgs.iter().any(|n| n == &msg2),
            "expected neutral message at index 1, got: {msg2}"
        );

        // Dismiss #2 → snooze_index=2, still neutral
        m.dismiss();
        m.stage = AlertStage::Stage1;
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        let actions3 = m.tick(1.0); // Stage2, snooze_index=2
        let msg3 = extract_popup_msg(&actions3);
        assert!(
            neutral_msgs.iter().any(|n| n == &msg3),
            "expected neutral message at index 2, got: {msg3}"
        );

        // Dismiss #3 → snooze_index=3 = tone_shift_threshold → positive messages
        m.dismiss();
        m.stage = AlertStage::Stage1;
        m.stage_entered_at = Instant::now() - Duration::from_secs(1);
        let actions4 = m.tick(1.0); // Stage2, snooze_index=3
        let msg4 = extract_popup_msg(&actions4);
        assert!(
            positive_msgs.iter().any(|p| p == &msg4),
            "expected positive message after tone shift, got: {msg4}"
        );
    }
}
