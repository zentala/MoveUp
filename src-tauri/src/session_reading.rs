//! session_reading.rs — Sensor reading processing and state transitions.

use chrono::{DateTime, Timelike, Utc};
use log::{debug, info};

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Process a sensor reading using the system clock.
    ///
    /// This is the impure boundary: it reads `Utc::now()` once and hands it to
    /// [`SessionManager::on_reading_at`], which holds all the logic. Production
    /// callers that already own a `now` for the current tick should call
    /// `on_reading_at` directly so every counter in that tick agrees on one
    /// instant; tests should always call `on_reading_at`.
    pub fn on_reading(&mut self, mm: i32, active: bool) -> ReadingResult {
        self.on_reading_at(mm, active, Utc::now())
    }

    /// Process a sensor reading at an explicit instant. Returns state change
    /// and/or completed session.
    ///
    /// Pure with respect to the clock: `now` is the only source of time here,
    /// so a caller can replay a whole day of readings deterministically.
    pub fn on_reading_at(&mut self, mm: i32, active: bool, now: DateTime<Utc>) -> ReadingResult {
        // Reject earlier observations before changing debounce, state or any
        // anchor. Equal instants remain valid for deterministic debounce.
        self.last_accumulate_ran = false;
        if self.history_last_reading.into_iter().chain(self.state.last_tick_ts)
            .chain(self.history_span_started)
            .any(|last| now < last) {
            return ReadingResult::default();
        }
        if self.state.first_reading_at.is_none() {
            self.state.first_reading_at = Some(now);
        }

        // A long observation gap closes the last known prefix. Its duration is
        // unknown, so it is neither posture time nor break credit.
        let previous_observation = self.history_last_reading
            .into_iter()
            .chain(self.state.last_tick_ts)
            .max();
        if let Some(last) = previous_observation {
            if (now - last).num_seconds() > SLEEP_GAP_THRESHOLD_SECS {
                info!("Observation gap detected: {}s; preserving the known prefix", (now - last).num_seconds());
                if let Some(started) = self.state.break_started {
                    self.observed_break_secs += (last - started).num_seconds().max(0);
                    self.state.break_started = Some(now);
                    self.state.break_seconds = self.observed_break_secs;
                }
                let interrupted_state = self.current_state();
                if let Some(span) = self.close_observed_boundary(last) {
                    self.queue_history(interrupted_state.clone(), span.clone(), None);
                    self.interrupted_history.push((interrupted_state, span));
                }
                self.open_observed_boundary(now);
            }
        }
        self.history_last_reading = Some(now);

        self.state.last_tick_ts = Some(now);

        // Compute raw calibrated desk height (used for state machine decisions).
        let floor_distance_cm = mm as f32 / 10.0;
        let desk_height_cm = floor_distance_cm - self.desk_thickness_cm;

        // Feed raw reading into stabilizer; use stabilized value for UI display.
        self.height_stabilizer.push(mm);
        let display_height_cm = self
            .height_stabilizer
            .stabilized_height_cm(self.desk_thickness_cm)
            .unwrap_or(desk_height_cm);
        self.state.desk_height_cm = display_height_cm;

        // Midpoint between sitting and standing thresholds.
        let mid_cm = (self.sitting_height_cm + self.standing_height_cm) / 2.0;

        // Determine candidate state.
        // Priority: inactive → Away (regardless of desk height),
        // then desk height determines Sitting vs Standing.
        // Walking is reserved for future use (smartwatch proximity detection).
        let candidate = if !active {
            DeskState::Away
        } else if desk_height_cm <= mid_cm {
            DeskState::Sitting
        } else {
            DeskState::Standing
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
                break_credit: None,
            };
        }

        // Candidate confirmed; reset debounce.
        self.pending_count = 0;

        if candidate == self.state.state {
            // A restored/default state has no history coverage until confirmed
            // by readings in this process. Do not restore history from counters.
            self.history_span_started.get_or_insert(now);
            match self.state.state {
                DeskState::Sitting => { self.state.sitting_started.get_or_insert(now); }
                DeskState::Standing => {
                    self.state.standing_bout_started.get_or_insert(now);
                    self.state.standing_session_started.get_or_insert(now);
                }
                DeskState::Walking | DeskState::Away => {}
            }
            self.accumulate_ongoing(now);
            return ReadingResult {
                state_change: None,
                completed_session: None,
                break_credit: None,
            };
        }

        let break_ended = candidate == DeskState::Sitting && self.state.break_started.is_some();
        let completed_session = self.handle_state_exit(&candidate, now);

        // Capture break credit if one was applied during this transition.
        let break_credit = if break_ended
            && self.state.last_break_credit != BreakCredit::None {
            Some((self.state.last_break_credit.clone(), self.state.last_break_secs))
        } else {
            None
        };

        if let Some(span) = completed_session.as_ref() {
            self.queue_history(self.current_state(), span.clone(),
                break_credit.as_ref().map(|(credit, _)| credit.clone()));
        }

        // Track posture changes: Sitting<->Standing desk transitions AND extended
        // Away resets (see accumulate_ongoing). Both count because the goal is to
        // measure how often the user changes their physical situation — standing up
        // AND leaving the computer are both healthy posture changes.
        // Future: split into desk_changes + away_changes for detailed analytics.
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
        self.open_observed_boundary(now);

        let live_sitting = self.get_live_sitting_seconds(now);
        let live_break = self.get_live_break_seconds(now);
        let live_standing = self.get_live_standing_seconds(now);
        ReadingResult {
            state_change: Some(StateChangedPayload {
                state: self.state.state.clone(),
                standing_seconds: live_standing,
                break_seconds: live_break,
                desk_height_cm: display_height_cm,
                position_changes: self.state.position_changes,
                last_break_secs: self.state.last_break_secs,
                last_sitting_secs: self.state.last_sitting_secs,
                break_credit: self.state.last_break_credit.clone(),
                limit_used_secs: live_sitting,
            }),
            completed_session,
            break_credit,
        }
    }

    /// Accumulates time while remaining in the current state (no transition).
    ///
    /// Throttled to run at most once per second: the sensor may send readings
    /// faster than 1 Hz, but counters like `continuous_computer_secs` and
    /// `away_bout_secs` must grow at wall-clock rate.
    ///
    /// The Away duration that resets the computer timer comes from
    /// [`SessionManager::limits`] (the live ergonomic profile), not from
    /// `SessionState`.
    pub(crate) fn accumulate_ongoing(&mut self, now: DateTime<Utc>) {
        self.last_accumulate_ran = false;
        if let Some(prev) = self.state.last_accumulate_ts {
            if (now - prev).num_seconds() < 1 {
                return;
            }
        }
        self.state.last_accumulate_ts = Some(now);
        self.last_accumulate_ran = true;

        // Refresh system idle time from activity module.
        self.state.idle_secs = crate::activity::get_idle_seconds() as i64;

        // Continuous computer timer and Away bout tracking.
        // Walking is grouped with Away (both = not at computer).
        match self.state.state {
            DeskState::Sitting | DeskState::Standing => {
                self.state.continuous_computer_secs += 1;
                self.state.longest_computer_session_secs = self
                    .state
                    .longest_computer_session_secs
                    .max(self.state.continuous_computer_secs);
                self.state.away_bout_secs = 0;
            }
            DeskState::Away | DeskState::Walking => {
                self.state.away_bout_secs += 1;
                // After configured away duration: reset continuous computer timer
                // and count as a posture change (user left the computer).
                if self.state.away_bout_secs >= self.limits.computer_break_reset_secs as i64
                    && self.state.continuous_computer_secs > 0
                {
                    self.state.continuous_computer_secs = 0;
                    self.state.position_changes += 1;
                }
            }
        }
        // Break seconds accumulation (original logic).
        match self.state.state {
            DeskState::Sitting => {} // committed on transition; live via snapshot()
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if self.state.break_started.is_some() {
                    self.state.break_seconds = self.get_live_break_seconds(now);
                }
            }
        }

        // HourlyBreakTracker: tick per-clock-hour break tracking.
        let hour = now.hour() as u8;
        match self.state.state {
            DeskState::Away | DeskState::Walking => {
                self.hourly_break_tracker.tick_away(hour);
            }
            DeskState::Sitting | DeskState::Standing => {
                self.hourly_break_tracker.tick_active(hour);
            }
        }
        self.state.hourly_breaks_covered = self.hourly_break_tracker.hours_with_break();
        self.state.hourly_breaks_active = self.hourly_break_tracker.hours_active();
    }
}
