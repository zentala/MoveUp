//! session_breaks.rs — Break credit, state exit, and notification logic.

use chrono::DateTime;
use chrono::Utc;

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Handles state exit logic, returning any completed session.
    pub(crate) fn handle_state_exit(
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
                    self.state.current_session_secs = 0;
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
                            self.state.current_session_secs = 0;
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
                        self.state.current_session_secs = 0;
                        self.state.break_seconds = 0;
                    }
                    self.state.sitting_started = Some(now);
                    self.state.last_position_change_at = Some(now);
                }
                // Away/Walking → Standing: start break timer
                if *candidate == DeskState::Standing
                    && self.state.break_started.is_none()
                {
                    self.state.break_started = Some(now);
                    self.state.break_seconds = 0;
                    self.state.last_position_change_at = Some(now);
                }
            }
        }

        completed_session
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
        if config.notify_daily_posture_balance
            && !self.notify_posture_balance_fired
        {
            if self.state.sitting_seconds > self.state.standing_seconds * 2 {
                self.notify_posture_balance_fired = true;
                events.push(NotificationEvent::PostureBalance);
            }
        }
        if !self.standing_target_reached_fired
            && self.state.stand_limit_secs > 0
            && self.state.standing_seconds >= self.state.stand_limit_secs
        {
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

    /// Computes live break seconds: elapsed since break_started.
    pub(crate) fn get_live_break_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state != DeskState::Sitting {
            if let Some(started) = self.state.break_started {
                return (now - started).num_seconds().max(0);
            }
        }
        self.state.break_seconds
    }
}
