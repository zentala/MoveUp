//! session_reading.rs — Sensor reading processing and state transitions.

use chrono::DateTime;
use chrono::Utc;
use log::{debug, info};

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Process a sensor reading. Returns state change and/or completed session.
    pub fn on_reading(&mut self, mm: i32, active: bool) -> ReadingResult {
        let now = Utc::now();

        // Compute calibrated desk height.
        let floor_distance_cm = mm as f32 / 10.0;
        let desk_height_cm = floor_distance_cm - self.desk_thickness_cm;
        self.state.desk_height_cm = desk_height_cm;

        // Midpoint between sitting and standing thresholds.
        let mid_cm = (self.sitting_height_cm + self.standing_height_cm) / 2.0;

        // Determine candidate state.
        let candidate = if desk_height_cm <= mid_cm {
            DeskState::Sitting
        } else if active {
            DeskState::Standing
        } else {
            DeskState::Walking
        };

        // Debounce: only transition after DEBOUNCE_COUNT consistent readings.
        if Some(&candidate) == self.pending_state.as_ref() {
            self.pending_count += 1;
        } else {
            if self.pending_count > 1 {
                debug!(
                    "debounce reset after {}x {:?}: new candidate={:?} (height={:.1}cm mid={:.1}cm)",
                    self.pending_count, self.pending_state, candidate, desk_height_cm, mid_cm
                );
            }
            self.pending_state = Some(candidate.clone());
            self.pending_count = 1;
        }

        if self.pending_count < DEBOUNCE_COUNT {
            self.accumulate_ongoing(now);
            return ReadingResult {
                state_change: None,
                completed_session: None,
            };
        }

        // Candidate confirmed; reset debounce.
        self.pending_count = 0;

        if candidate == self.state.state {
            self.accumulate_ongoing(now);
            return ReadingResult {
                state_change: None,
                completed_session: None,
            };
        }

        let completed_session = self.handle_state_exit(&candidate, now);

        // Track position changes: only Sitting<->Standing transitions.
        let is_position_change = (self.state.state == DeskState::Sitting
            && candidate == DeskState::Standing)
            || (self.state.state == DeskState::Standing
                && candidate == DeskState::Sitting);

        if is_position_change {
            self.state.position_changes += 1;
        }

        info!(
            "State transition: {:?} -> {:?}  (sitting={}s desk_height={:.1}cm)",
            self.state.state, candidate, self.state.sitting_seconds, desk_height_cm
        );

        self.state.state = candidate;

        let live_sitting = self.get_live_sitting_seconds(now);
        let live_current = self.get_live_current_session_secs(now);
        ReadingResult {
            state_change: Some(StateChangedPayload {
                state: self.state.state.clone(),
                sitting_seconds: live_sitting,
                standing_seconds: self.state.standing_seconds,
                break_seconds: self.state.break_seconds,
                desk_height_cm,
                position_changes: self.state.position_changes,
                last_break_secs: self.state.last_break_secs,
                last_sitting_secs: self.state.last_sitting_secs,
                break_credit: self.state.last_break_credit.clone(),
                current_session_secs: live_current,
            }),
            completed_session,
        }
    }

    /// Accumulates time while remaining in the current state (no transition).
    pub(crate) fn accumulate_ongoing(&mut self, now: DateTime<Utc>) {
        match self.state.state {
            DeskState::Sitting => {} // committed on transition; live via snapshot()
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if let Some(bs) = self.state.break_started {
                    self.state.break_seconds =
                        (now - bs).num_seconds().max(0);
                }
            }
        }
    }
}
