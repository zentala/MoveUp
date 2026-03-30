//! CommunicationPolicy — central brain for all UI signal decisions.
//!
//! Evaluates the current session state once per second and returns a [`Signals`]
//! struct that each UI channel renderer interprets independently. No I/O, no
//! parsing — just struct comparisons and state transitions.
//!
//! Signal-parsing and stateless builders live in [`communication_policy_helpers`].

use std::time::{Duration, Instant};
use crate::communication_types::{NotifySignal, Signals};
use crate::communication_profile::{CommunicationProfile, EscalationStep};
use crate::communication_policy_helpers as helpers;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session_types::DeskState;

// ---------------------------------------------------------------------------
// PolicyInput
// ---------------------------------------------------------------------------

/// Snapshot of session data passed to [`CommunicationPolicy::evaluate`] each cycle.
pub struct PolicyInput {
    /// Current desk state (Sitting, Standing, Walking, Away).
    pub state: DeskState,
    /// Elapsed seconds in the current phase (sitting or standing).
    pub elapsed_secs: i64,
    /// Whether the hardware sensor is currently connected.
    pub sensor_connected: bool,
    /// Overlay bar fill for standing progress (0.0–1.0).
    pub standing_lap_progress: f32,
    /// Completed standing lap count today.
    pub standing_lap: u32,
    /// Trigger a flash animation on lap reset.
    pub standing_lap_flash: bool,
}

// ---------------------------------------------------------------------------
// CommunicationPolicy
// ---------------------------------------------------------------------------

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
    /// Index of the last escalation step for which a notification was fired.
    last_notify_step: Option<usize>,
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
        }
    }

    /// Evaluate current session state and return UI signals.
    ///
    /// This is the hot path — called ~1/sec. No I/O, no allocation beyond signals.
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

        match active_step {
            Some((idx, step)) => self.step_to_signals(step, idx, input),
            None => self.make_baseline(input),
        }
    }

    /// Record a user dismiss — start a snooze timer and advance the snooze index.
    pub fn dismiss(&mut self) {
        let durations = &self.comm_profile.snooze.durations_mins;
        let idx = self.snooze_index.min(durations.len().saturating_sub(1));
        let mins = durations.get(idx).copied().unwrap_or(5);
        self.snoozed_until = Some(Instant::now() + Duration::from_secs(mins as u64 * 60));
        self.snooze_index = (self.snooze_index + 1).min(durations.len().saturating_sub(1));
        self.last_notify_step = None;
    }

    /// Reset snooze state and notification tracking on position change.
    pub fn on_position_changed(&mut self) {
        self.snoozed_until = None;
        self.snooze_index = 0;
        self.last_notify_step = None;
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

    fn build_step_notify(
        &mut self,
        step: &EscalationStep,
        idx: usize,
        input: &PolicyInput,
    ) -> Option<NotifySignal> {
        if let Some(notify_type) = &step.notify.clone() {
            if self.last_notify_step != Some(idx) {
                self.last_notify_step = Some(idx);
                let is_firm = self.snooze_index
                    >= self.comm_profile.snooze.tone_shift_after_dismisses as usize;
                let msg = helpers::message_for_state(
                    &input.state, notify_type, &self.comm_profile.messages, is_firm,
                );
                Some(helpers::make_notify_signal(notify_type, &msg))
            } else {
                None
            }
        } else {
            if self.last_notify_step.map_or(true, |prev| prev < idx) {
                self.last_notify_step = Some(idx);
            }
            None
        }
    }
}
