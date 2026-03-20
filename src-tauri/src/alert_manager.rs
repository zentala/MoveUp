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

pub use crate::alert_config::AlertConfig;
pub use crate::alert_actions::{AlertStage, AlertAction};

use std::time::{Duration, Instant};

// ─── State Machine ────────────────────────────────────────────────────────────

/// Alert escalation state machine — no UI, no WinAPI.
///
/// Call [`tick`](AlertManager::tick) on every `desk:distance` event (~1/s).
/// Execute the returned [`AlertAction`]s in [`tray_controller.rs`].
pub struct AlertManager {
    pub(crate) stage: AlertStage,
    pub(crate) stage_entered_at: Instant,
    /// Set to `Instant::now() + snooze_duration` on dismiss; `None` when not snoozed.
    pub(crate) snoozed_until: Option<Instant>,
    pub(crate) config: AlertConfig,
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

// Tests live in alert_manager_tests.rs (declared in lib.rs under #[cfg(test)]).
