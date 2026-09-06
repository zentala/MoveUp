//! session_live.rs — Live value computation for SessionManager.
//!
//! These methods compute real-time values by combining committed counters
//! with elapsed time from active timestamps. Used by snapshot() and UI.

use chrono::{DateTime, Utc};
use log::warn;

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

    /// Seconds elapsed since the last position change — uncredited.
    ///
    /// Debug-tab diagnostic only (E015 decision D2). Never use it for a
    /// timer, progress bar, colour band or notification; those read
    /// `limit_used_secs`, the credited counter. Returns 0 when no position
    /// change has been recorded yet.
    pub(crate) fn get_secs_since_last_break(&self, now: DateTime<Utc>) -> i64 {
        match self.state.last_position_change_at {
            Some(changed_at) => (now - changed_at).num_seconds().max(0),
            None => 0,
        }
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
    /// Caps duration at `MAX_REASONABLE_SESSION_SECS` to prevent sleep inflation.
    pub fn flush_current_session(&self) -> Option<CompletedSession> {
        let now = Utc::now();
        let started = match self.state.state {
            DeskState::Sitting => self.state.sitting_started,
            DeskState::Standing | DeskState::Walking => self.state.standing_bout_started,
            _ => None,
        }?;
        let raw_elapsed = (now - started).num_seconds().max(0);
        let elapsed = if raw_elapsed > MAX_REASONABLE_SESSION_SECS {
            warn!(
                "Session capped at {}s (was {}s — likely sleep gap)",
                MAX_REASONABLE_SESSION_SECS, raw_elapsed
            );
            MAX_REASONABLE_SESSION_SECS
        } else {
            raw_elapsed
        };
        Some(CompletedSession {
            started_at: started.to_rfc3339(),
            ended_at: now.to_rfc3339(),
            duration_secs: elapsed,
        })
    }
}
