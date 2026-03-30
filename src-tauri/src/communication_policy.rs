//! CommunicationPolicy — central brain for all UI signal decisions.
//!
//! Evaluates the current session state once per second and returns a [`Signals`]
//! struct that each UI channel renderer interprets independently. No I/O, no
//! parsing — just struct comparisons and state transitions.

use std::time::{Duration, Instant};
use crate::communication_types::{
    NotifySignal, OverlaySignal, PopupSignal, Signals, TraySignal,
};
use crate::communication_profile::{CommunicationProfile, EscalationStep};
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
    /// Prevents re-firing the same notification on subsequent cycles at the same step.
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
            return self.evaluate_disconnected();
        }

        if matches!(input.state, DeskState::Away | DeskState::Walking) {
            return self.evaluate_inactive();
        }

        // Check snooze — if still active, return baseline only
        if let Some(until) = self.snoozed_until {
            if Instant::now() < until {
                return self.evaluate_baseline(input);
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
            _ => return self.evaluate_baseline(input),
        };

        let offset = input.elapsed_secs - limit_secs;

        // Find highest step whose `at` threshold has been reached
        let active_step = steps
            .iter()
            .enumerate()
            .filter(|(_, s)| offset >= s.at)
            .last();

        match active_step {
            Some((idx, step)) => self.step_to_signals(step, idx, input),
            None => self.evaluate_baseline(input),
        }
    }

    /// Record a user dismiss — start a snooze timer and advance the snooze index.
    pub fn dismiss(&mut self) {
        let durations = &self.comm_profile.snooze.durations_mins;
        let idx = self.snooze_index.min(durations.len().saturating_sub(1));
        let mins = durations.get(idx).copied().unwrap_or(5);
        self.snoozed_until = Some(Instant::now() + Duration::from_secs(mins as u64 * 60));
        self.snooze_index = (self.snooze_index + 1).min(durations.len().saturating_sub(1));
        // Reset notify tracking so the notification fires again after snooze
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

    fn evaluate_disconnected(&mut self) -> Signals {
        let notify = if !self.disconnect_notified {
            self.disconnect_notified = true;
            Some(self.make_notify_signal(
                &self.comm_profile.disconnected.notify_once.clone().unwrap_or_default(),
                &self.comm_profile.messages.sensor_disconnected.clone(),
            ))
        } else {
            None
        };
        Signals {
            tray: self.parse_tray_signal(&self.comm_profile.disconnected.tray.clone()),
            overlay: OverlaySignal::Hidden,
            popup: PopupSignal::Gray,
            notify,
        }
    }

    fn evaluate_inactive(&self) -> Signals {
        Signals {
            tray: TraySignal::None,
            overlay: OverlaySignal::Hidden,
            popup: PopupSignal::Neutral,
            notify: None,
        }
    }

    fn evaluate_baseline(&self, input: &PolicyInput) -> Signals {
        match input.state {
            DeskState::Sitting => {
                let bl = &self.comm_profile.baseline.sitting;
                Signals {
                    tray: self.parse_tray_signal(&bl.tray),
                    overlay: OverlaySignal::Neutral { progress: 0.0 },
                    popup: self.parse_popup_signal(&bl.popup),
                    notify: None,
                }
            }
            DeskState::Standing => {
                let bl = &self.comm_profile.baseline.standing;
                Signals {
                    tray: self.parse_tray_signal(&bl.tray),
                    overlay: OverlaySignal::Progress {
                        progress: input.standing_lap_progress,
                        lap: input.standing_lap,
                        flash: input.standing_lap_flash,
                    },
                    popup: self.parse_popup_signal(&bl.popup),
                    notify: None,
                }
            }
            _ => Signals::default(),
        }
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

        let tray = self.parse_tray_signal(&step.tray.clone());
        let overlay = self.parse_overlay_signal(&step.overlay.clone(), progress);
        let popup = self.parse_popup_signal(&step.popup_header.clone());

        let notify = if let Some(notify_type) = &step.notify.clone() {
            if self.last_notify_step != Some(idx) {
                self.last_notify_step = Some(idx);
                let msg = self.get_message_for_state(input.state.clone(), notify_type);
                Some(self.make_notify_signal(notify_type, &msg))
            } else {
                None
            }
        } else {
            // No notification for this step — still track the step index
            if self.last_notify_step.map_or(true, |prev| prev < idx) {
                self.last_notify_step = Some(idx);
            }
            None
        };

        Signals { tray, overlay, popup, notify }
    }

    fn parse_tray_signal(&self, s: &str) -> TraySignal {
        match s {
            "none" | "" => TraySignal::None,
            "yellow" => TraySignal::Yellow,
            "red" => TraySignal::Red,
            other => TraySignal::Blink(other.to_string()),
        }
    }

    fn parse_overlay_signal(&self, s: &str, progress: f32) -> OverlaySignal {
        match s {
            "hidden" | "" => OverlaySignal::Hidden,
            "neutral" => OverlaySignal::Neutral { progress },
            "yellow" => OverlaySignal::Yellow { progress },
            "red" => OverlaySignal::Red { progress },
            "pulse_red" => OverlaySignal::PulseRed { progress },
            _ => OverlaySignal::Neutral { progress },
        }
    }

    fn parse_popup_signal(&self, s: &str) -> PopupSignal {
        match s {
            "yellow" => PopupSignal::Yellow,
            "red" => PopupSignal::Red,
            "gray" => PopupSignal::Gray,
            _ => PopupSignal::Neutral,
        }
    }

    fn make_notify_signal(&self, notify_type: &str, message: &str) -> NotifySignal {
        match notify_type {
            "popup" => NotifySignal::Popup(message.to_string()),
            _ => NotifySignal::Toast(message.to_string()),
        }
    }

    fn get_message_for_state(&self, state: DeskState, notify_type: &str) -> String {
        let msgs = &self.comm_profile.messages;
        let is_firm = self.snooze_index >= self.comm_profile.snooze.tone_shift_after_dismisses as usize;

        match (state, notify_type) {
            (DeskState::Sitting, "toast") => msgs.sitting_limit_toast.clone(),
            (DeskState::Sitting, "popup") => {
                if is_firm {
                    "You really need to stand up now.".to_string()
                } else {
                    msgs.sitting_overdue_popup.clone()
                }
            }
            (DeskState::Standing, "toast") => msgs.standing_limit_toast.clone(),
            (DeskState::Standing, "popup") => msgs.standing_overdue_popup.clone(),
            _ => msgs.sitting_limit_toast.clone(),
        }
    }
}
