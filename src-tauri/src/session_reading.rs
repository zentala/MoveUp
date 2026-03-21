//! session_reading.rs — Sensor reading processing and state transitions.

use chrono::DateTime;
use chrono::Utc;
use log::info;

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

    /// Handles state exit logic, returning any completed session.
    fn handle_state_exit(
        &mut self,
        candidate: &DeskState,
        now: DateTime<Utc>,
    ) -> Option<CompletedSession> {
        let mut completed_session = None;

        match &self.state.state {
            DeskState::Sitting => {
                if let Some(started) = self.state.sitting_started.take() {
                    let elapsed = (now - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;
                    self.state.current_session_secs += elapsed;
                    self.state.last_sitting_secs = elapsed;
                    if *candidate != DeskState::Sitting {
                        completed_session = Some(CompletedSession {
                            started_at: started.to_rfc3339(),
                            ended_at: now.to_rfc3339(),
                            duration_secs: elapsed,
                        });
                    }
                }
                if *candidate != DeskState::Sitting {
                    self.state.break_started = Some(now);
                    self.state.break_seconds = 0;
                    self.alert_fired = false;
                    self.stand_alert_fired = false;
                    self.state.last_position_change_at = Some(now);
                    self.state.last_break_credit = BreakCredit::None;
                }
            }
            DeskState::Standing => {
                if *candidate != DeskState::Standing {
                    self.state.standing_session_secs = 0;
                    self.state.lap_bonus_awarded_for_lap = 0;
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        self.state.standing_seconds += break_dur;
                        self.state.last_break_secs = break_dur;
                        if *candidate == DeskState::Sitting {
                            self.apply_break_credit(break_dur);
                            // current_session_secs = sitting_seconds after credit
                            self.state.current_session_secs =
                                self.state.sitting_seconds;
                        }
                        self.state.break_seconds = 0;
                    }
                    if *candidate == DeskState::Sitting {
                        self.state.sitting_started = Some(now);
                    }
                    self.state.last_position_change_at = Some(now);
                }
            }
            DeskState::Walking | DeskState::Away => {
                if *candidate == DeskState::Sitting {
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        self.state.last_break_secs = break_dur;
                        self.apply_break_credit(break_dur);
                        // current_session_secs = sitting_seconds after credit
                        self.state.current_session_secs =
                            self.state.sitting_seconds;
                        self.state.break_seconds = 0;
                    }
                    self.state.sitting_started = Some(now);
                    self.state.last_position_change_at = Some(now);
                }
            }
        }

        completed_session
    }
    /// Accumulates time while remaining in the current state (no transition).
    pub(crate) fn accumulate_ongoing(&mut self, now: DateTime<Utc>) {
        match self.state.state {
            DeskState::Sitting => {} // committed on transition; live via snapshot()
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if let Some(bs) = self.state.break_started {
                    self.state.break_seconds = (now - bs).num_seconds().max(0);
                }
            }
        }
    }
    /// Checks notification conditions and returns events that should fire.
    pub fn check_notification_conditions(
        &mut self,
        config: &crate::config::AppConfig,
    ) -> Vec<NotificationEvent> {
        let now = Utc::now();
        let mut events = Vec::new();
        if config.notify_inactivity && !self.notify_inactivity_fired {
            if let Some(last_change) = self.state.last_position_change_at {
                if (now - last_change).num_seconds() >= 60 * 60 {
                    self.notify_inactivity_fired = true;
                    events.push(NotificationEvent::Inactivity);
                }
            }
        }
        if config.notify_daily_posture_balance && !self.notify_posture_balance_fired {
            if self.state.sitting_seconds > self.state.standing_seconds * 2 {
                self.notify_posture_balance_fired = true;
                events.push(NotificationEvent::PostureBalance);
            }
        }
        if !self.standing_target_reached_fired && self.state.stand_limit_secs > 0
            && self.state.standing_seconds >= self.state.stand_limit_secs {
            self.standing_target_reached_fired = true;
            events.push(NotificationEvent::StandingTargetReached);
        }
        events
    }
    /// Checks if praise-halfway notification should fire.
    pub fn should_send_praise_halfway(
        &mut self,
        config: &crate::config::AppConfig,
    ) -> bool {
        if config.notify_praise_halfway
            && !self.praise_halfway_fired_today
            && self.state.stand_limit_secs > 0
            && self.state.standing_seconds >= self.state.stand_limit_secs / 2
        {
            self.praise_halfway_fired_today = true;
            return true;
        }
        false
    }
    /// Checks if a stand limit alert should fire.
    pub fn should_stand_alert(&mut self) -> bool {
        if self.state.state == DeskState::Standing
            && !self.stand_alert_fired
            && self.state.stand_limit_secs > 0
            && self.state.break_seconds >= self.state.stand_limit_secs
        {
            self.stand_alert_fired = true;
            return true;
        }
        false
    }
    /// Applies break credit rules when returning to sitting.
    pub fn apply_break_credit(&mut self, break_secs: i64) {
        if break_secs < BREAK_SHORT_SECS {
            // Less than 5 minutes — no credit.
            self.state.last_break_credit = BreakCredit::None;
        } else if break_secs < BREAK_LONG_SECS {
            self.state.sitting_seconds =
                (self.state.sitting_seconds - SHORT_BREAK_CREDIT_SECS).max(0);
            self.state.last_break_credit = BreakCredit::Partial;
        } else {
            self.state.sitting_seconds = 0;
            self.state.last_break_credit = BreakCredit::Full;
        }
    }
}
