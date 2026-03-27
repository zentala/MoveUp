//! session_live.rs — Live value computation for SessionManager.
//!
//! These methods compute real-time values by combining committed counters
//! with elapsed time from active timestamps. Used by snapshot() and UI.

use chrono::{DateTime, Utc};

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Computes live standing seconds: accumulated + current standing bout elapsed.
    /// Only counts actual Standing time, not Away time.
    pub(crate) fn get_live_standing_seconds(&self, now: DateTime<Utc>) -> i64 {
        let base = self.state.standing_seconds;
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_bout_started {
                return base + (now - started).num_seconds().max(0);
            }
        }
        base
    }

    /// Computes live current session seconds: committed + elapsed since sitting_started.
    pub(crate) fn get_live_current_session_secs(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.current_session_secs + elapsed;
            }
        }
        self.state.current_session_secs
    }

    /// Computes live sitting seconds: committed + elapsed since sitting_started.
    pub(crate) fn get_live_sitting_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.sitting_seconds + elapsed;
            }
        }
        self.state.sitting_seconds
    }

    /// Computes live raw sitting seconds (never reduced by break credit).
    /// Used by standing_pct metric for accurate KPI calculation.
    pub(crate) fn get_live_sitting_seconds_total(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                let elapsed = (now - started).num_seconds().max(0);
                return self.state.sitting_seconds_total + elapsed;
            }
        }
        self.state.sitting_seconds_total
    }

    /// Computes live standing session seconds from timestamp.
    pub(crate) fn get_live_standing_session_secs(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_session_started {
                return (now - started).num_seconds().max(0);
            }
        }
        self.state.standing_session_secs
    }

    /// Snapshots the current in-progress session as a CompletedSession for DB persistence.
    /// Does NOT modify state — safe to call on shutdown without side effects.
    /// Returns `None` if no session is currently active.
    pub fn flush_current_session(&self) -> Option<CompletedSession> {
        let now = Utc::now();
        match self.state.state {
            DeskState::Sitting => {
                if let Some(started) = self.state.sitting_started {
                    let elapsed = (now - started).num_seconds().max(0);
                    return Some(CompletedSession {
                        started_at: started.to_rfc3339(),
                        ended_at: now.to_rfc3339(),
                        duration_secs: elapsed,
                    });
                }
            }
            DeskState::Standing => {
                if let Some(started) = self.state.standing_bout_started {
                    let elapsed = (now - started).num_seconds().max(0);
                    return Some(CompletedSession {
                        started_at: started.to_rfc3339(),
                        ended_at: now.to_rfc3339(),
                        duration_secs: elapsed,
                    });
                }
            }
            _ => {}
        }
        None
    }
}
