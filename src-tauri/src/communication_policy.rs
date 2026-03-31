//! CommunicationPolicy — evaluates session state once/sec and returns [`Signals`]
//! for each UI channel. Stateless helpers live in [`communication_policy_helpers`].

use std::time::{Duration, Instant};
use crate::communication_types::{NotifySignal, Signals};
use crate::communication_profile::{CommunicationProfile, EscalationStep};
use crate::communication_policy_helpers as helpers;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session_types::DeskState;

/// Snapshot of session data passed to [`CommunicationPolicy::evaluate`] each cycle.
pub struct PolicyInput {
    pub state: DeskState,
    pub elapsed_secs: i64,
    pub sensor_connected: bool,
    pub standing_lap_progress: f32,
    pub standing_lap: u32,
    pub standing_lap_flash: bool,
    pub continuous_computer_secs: i64,
}

/// Stateful policy engine that converts session snapshots into UI signals.
///
/// Call [`evaluate`] once per second. Call [`dismiss`] when the user acknowledges
/// an alert, and [`on_position_changed`] on every sitting↔standing transition.
pub struct CommunicationPolicy {
    comm_profile: CommunicationProfile,
    ergo_profile: ErgonomicProfile,
    snoozed_until: Option<Instant>,
    snooze_index: usize,
    disconnect_notified: bool,
    last_notify_step: Option<usize>,
    last_notify_time: Option<Instant>,
    notify_count: u32,
    screen_break_nudge_fired: bool,
}

impl CommunicationPolicy {
    /// Create a new policy engine with the given profiles.
    pub fn new(comm: CommunicationProfile, ergo: ErgonomicProfile) -> Self {
        Self {
            comm_profile: comm,
            ergo_profile: ergo,
            snoozed_until: None,
            snooze_index: 0,
            disconnect_notified: false,
            last_notify_step: None,
            last_notify_time: None,
            notify_count: 0,
            screen_break_nudge_fired: false,
        }
    }

    /// Evaluate current session state and return UI signals (called ~1/sec).
    pub fn evaluate(&mut self, input: &PolicyInput) -> Signals {
        if !input.sensor_connected {
            let (sigs, fired) = helpers::disconnected_signals(&self.comm_profile, self.disconnect_notified);
            if fired { self.disconnect_notified = true; }
            return sigs;
        }

        if matches!(input.state, DeskState::Away | DeskState::Walking) {
            return helpers::inactive_signals();
        }

        // Check snooze — if still active, return baseline only
        if let Some(until) = self.snoozed_until {
            if Instant::now() < until {
                return self.make_baseline(input);
            } else {
                self.snoozed_until = None;
            }
        }

        let (limit_secs, steps) = match input.state {
            DeskState::Sitting => (
                self.ergo_profile.limits.sitting_secs as i64,
                self.comm_profile.escalation.sitting.clone(),
            ),
            DeskState::Standing => (
                self.ergo_profile.limits.standing_max_secs as i64,
                self.comm_profile.escalation.standing.clone(),
            ),
            _ => return self.make_baseline(input),
        };

        let offset = input.elapsed_secs - limit_secs;
        let active_step = steps.iter().enumerate().filter(|(_, s)| offset >= s.at).last();

        let mut signals = match active_step {
            Some((idx, step)) => self.step_to_signals(step, idx, input),
            None => self.make_baseline(input),
        };

        self.maybe_apply_screen_nudge(input, &mut signals);
        signals
    }

    /// Record a user dismiss — start snooze timer, advance index, reset notify count.
    pub fn dismiss(&mut self) {
        let durations = &self.comm_profile.snooze.durations_mins;
        let idx = self.snooze_index.min(durations.len().saturating_sub(1));
        let mins = durations.get(idx).copied().unwrap_or(5);
        self.snoozed_until = Some(Instant::now() + Duration::from_secs(mins as u64 * 60));
        self.snooze_index = (self.snooze_index + 1).min(durations.len().saturating_sub(1));
        self.last_notify_step = None;
        self.last_notify_time = None;
        self.notify_count = 0;
    }

    /// Reset snooze state and notification tracking on position change.
    pub fn on_position_changed(&mut self) {
        self.snoozed_until = None;
        self.snooze_index = 0;
        self.last_notify_step = None;
        self.last_notify_time = None;
        self.notify_count = 0;
    }

    /// Clear the disconnect-notified flag when the sensor reconnects.
    pub fn on_sensor_connected(&mut self) {
        self.disconnect_notified = false;
    }

    /// Hot-reload the communication profile.
    pub fn set_comm_profile(&mut self, p: CommunicationProfile) {
        self.comm_profile = p;
    }

    /// Hot-reload the ergonomic profile.
    pub fn set_ergo_profile(&mut self, p: ErgonomicProfile) {
        self.ergo_profile = p;
    }

    /// Read access to the current communication profile.
    pub fn comm_profile(&self) -> &CommunicationProfile {
        &self.comm_profile
    }

    /// Read access to the current ergonomic profile.
    pub fn ergo_profile(&self) -> &ErgonomicProfile {
        &self.ergo_profile
    }

    // ── private helpers ───────────────────────────────────────────────────────

    /// Check if a screen break nudge should fire and apply it to signals.
    fn maybe_apply_screen_nudge(&mut self, input: &PolicyInput, signals: &mut Signals) {
        if input.continuous_computer_secs < self.ergo_profile.limits.max_continuous_computer_secs as i64 {
            self.screen_break_nudge_fired = false;
        }

        if input.state == DeskState::Standing
            && signals.notify.is_none()
            && !self.screen_break_nudge_fired
            && self.ergo_profile.limits.max_continuous_computer_secs > 0
            && input.continuous_computer_secs >= self.ergo_profile.limits.max_continuous_computer_secs as i64
            && self.comm_profile.screen_break_nudge.enabled
            && !self.comm_profile.screen_break_nudge.messages.is_empty()
        {
            if let Some(msg) = crate::screen_break_nudge::pick_nudge_message(
                &self.comm_profile.screen_break_nudge.messages,
            ) {
                signals.notify = Some(NotifySignal::Toast(msg));
                self.screen_break_nudge_fired = true;
            }
        }
    }

    fn make_baseline(&self, input: &PolicyInput) -> Signals {
        helpers::baseline_signals(
            &input.state,
            &self.comm_profile.baseline,
            input.standing_lap_progress,
            input.standing_lap,
            input.standing_lap_flash,
        )
    }

    fn step_to_signals(&mut self, step: &EscalationStep, idx: usize, input: &PolicyInput) -> Signals {
        let progress = match input.state {
            DeskState::Sitting => {
                let limit = self.ergo_profile.limits.sitting_secs as f32;
                (input.elapsed_secs as f32 / limit).min(1.5)
            }
            DeskState::Standing => input.standing_lap_progress,
            _ => 0.0,
        };

        let tray = helpers::parse_tray_signal(&step.tray);
        let overlay = helpers::parse_overlay_signal(&step.overlay, progress);
        let popup = helpers::parse_popup_signal(&step.popup_header);
        let notify = self.build_step_notify(step, idx, input);

        Signals { tray, overlay, popup, notify }
    }

    /// Cooldown before next notification, or `None` if all reminders exhausted.
    fn notify_cooldown_secs(&self) -> Option<u64> {
        let cooldowns = &self.comm_profile.snooze.notify_cooldowns_secs;
        cooldowns.get(self.notify_count as usize).copied()
    }

    fn build_step_notify(
        &mut self,
        step: &EscalationStep,
        idx: usize,
        input: &PolicyInput,
    ) -> Option<NotifySignal> {
        if let Some(notify_type) = &step.notify.clone() {
            // Already fired this exact step — skip
            if self.last_notify_step == Some(idx) {
                return None;
            }

            // Check escalating cooldown
            let cooldown = match self.notify_cooldown_secs() {
                Some(cd) => cd,
                None => return None, // max notifications reached, stay silent
            };
            if let Some(last) = self.last_notify_time {
                if last.elapsed() < Duration::from_secs(cooldown) {
                    return None;
                }
            }

            self.last_notify_step = Some(idx);
            self.last_notify_time = Some(Instant::now());
            self.notify_count += 1;

            let is_firm = self.snooze_index
                >= self.comm_profile.snooze.tone_shift_after_dismisses as usize;
            let msg = helpers::message_for_state(
                &input.state, notify_type, &self.comm_profile.messages, is_firm,
            );
            Some(helpers::make_notify_signal(notify_type, &msg))
        } else {
            if self.last_notify_step.map_or(true, |prev| prev < idx) {
                self.last_notify_step = Some(idx);
            }
            None
        }
    }
}
