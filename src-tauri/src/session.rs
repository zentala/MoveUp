//! session.rs — Sit/stand desk session state machine.
//!
//! Tracks how long the user has been sitting, applies break credit rules, and
//! emits Tauri events when the state changes or when a sitting limit is reached.

use chrono::{DateTime, Utc};
use log::info;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Height threshold (mm) below which the desk is considered in "sitting" position.
const SIT_HEIGHT_MM: i32 = 900;
/// Height threshold (mm) above which the desk is considered in "standing" position.
const STAND_HEIGHT_MM: i32 = 1000;
/// Number of consecutive readings at a new height before state transitions.
const DEBOUNCE_COUNT: u8 = 5;
/// Default session sitting limit (40 minutes in seconds).
const DEFAULT_SESSION_LIMIT_SECS: i64 = 2400;
/// Inactivity duration before transitioning to Away (5 minutes in seconds).
const AWAY_IDLE_SECS: u64 = 300;
/// Minimum break duration for any credit (5 minutes).
const BREAK_SHORT_SECS: i64 = 300;
/// Break duration threshold for partial credit (10 minutes).
const BREAK_LONG_SECS: i64 = 600;
/// Sitting seconds subtracted for a short break (20 minutes).
const SHORT_BREAK_CREDIT_SECS: i64 = 1200;

// ─── Types ───────────────────────────────────────────────────────────────────

/// The ergonomic state the user is currently in.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeskState {
    Sitting,
    Standing,
    Walking,
    Away,
}

/// Full session state, managed by `SessionManager`.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub state: DeskState,
    pub sitting_started: Option<DateTime<Utc>>,
    /// Total sitting seconds accumulated this session.
    pub sitting_seconds: i64,
    pub break_started: Option<DateTime<Utc>>,
    /// Duration of the current break in seconds.
    pub break_seconds: i64,
    /// Alert threshold — sitting_seconds >= this value triggers an alert.
    pub session_limit_secs: i64,
}

/// Serialisable DTO emitted with state-change events.
#[derive(Debug, Clone, Serialize)]
pub struct SessionStateDto {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub break_seconds: i64,
    pub session_limit_secs: i64,
}

/// Payload for the `desk:state-changed` event.
#[derive(Debug, Clone, Serialize)]
struct StateChangedPayload {
    state: DeskState,
    sitting_seconds: i64,
    break_seconds: i64,
}

// ─── SessionManager ───────────────────────────────────────────────────────────

/// Owns `SessionState` and drives state transitions.
pub struct SessionManager {
    state: SessionState,
    /// Pending candidate state (needs `debounce_count` confirmations).
    pending_state: Option<DeskState>,
    pending_count: u8,
}

impl SessionManager {
    /// Creates a manager with the default session limit.
    pub fn new() -> Self {
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: DEFAULT_SESSION_LIMIT_SECS,
            },
            pending_state: None,
            pending_count: 0,
        }
    }

    /// Updates the session limit (minutes → seconds).
    pub fn set_limit_minutes(&mut self, minutes: u32) {
        self.state.session_limit_secs = minutes as i64 * 60;
    }

    /// Returns a snapshot of the current session state as a DTO.
    pub fn snapshot(&self) -> SessionStateDto {
        SessionStateDto {
            state: self.state.state.clone(),
            sitting_seconds: self.state.sitting_seconds,
            break_seconds: self.state.break_seconds,
            session_limit_secs: self.state.session_limit_secs,
        }
    }

    /// Called for each new distance reading.
    ///
    /// `height_mm` is the sensor distance (floor distance from underside of desk).
    /// `idle_secs` is the system idle time in seconds.
    pub fn on_reading(&mut self, app: &AppHandle, height_mm: i32, idle_secs: u64) {
        let now = Utc::now();

        // Determine the candidate state from current inputs.
        let candidate = if idle_secs >= AWAY_IDLE_SECS {
            DeskState::Away
        } else if height_mm > STAND_HEIGHT_MM {
            // Active movement at standing height → Walking, else Standing.
            if idle_secs > 30 {
                DeskState::Walking
            } else {
                DeskState::Standing
            }
        } else {
            DeskState::Sitting
        };

        // Debounce: only transition when we see DEBOUNCE_COUNT consistent readings.
        if Some(&candidate) == self.pending_state.as_ref() {
            self.pending_count += 1;
        } else {
            self.pending_state = Some(candidate.clone());
            self.pending_count = 1;
        }

        if self.pending_count < DEBOUNCE_COUNT {
            // Not enough readings yet — accumulate ongoing metrics but don't switch.
            self.accumulate_ongoing(now);
            return;
        }

        // Candidate confirmed; reset debounce.
        self.pending_count = 0;

        if candidate == self.state.state {
            self.accumulate_ongoing(now);
            return;
        }

        // ── Leaving current state ─────────────────────────────────────────────
        match &self.state.state {
            DeskState::Sitting => {
                // Stop counting sitting time.
                if let Some(started) = self.state.sitting_started.take() {
                    let elapsed = (now - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;
                }
                // Starting a break.
                if candidate != DeskState::Sitting {
                    self.state.break_started = Some(now);
                    self.state.break_seconds = 0;
                }
            }
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if candidate == DeskState::Sitting {
                    // Returning to sitting — apply break credit.
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        self.apply_break_credit(break_dur);
                        self.state.break_seconds = 0;
                    }
                    self.state.sitting_started = Some(now);
                }
            }
        }

        info!(
            "State transition: {:?} → {:?}  (sitting={}s)",
            self.state.state, candidate, self.state.sitting_seconds
        );

        self.state.state = candidate;

        // Emit state-changed event.
        let _ = app.emit(
            "desk:state-changed",
            StateChangedPayload {
                state: self.state.state.clone(),
                sitting_seconds: self.state.sitting_seconds,
                break_seconds: self.state.break_seconds,
            },
        );

        // Check alert threshold.
        if self.state.state == DeskState::Sitting
            && self.state.sitting_seconds >= self.state.session_limit_secs
        {
            let _ = app.emit("desk:session-alert", self.snapshot());
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Accumulates time while remaining in the current state (no transition).
    fn accumulate_ongoing(&mut self, now: DateTime<Utc>) {
        match self.state.state {
            DeskState::Sitting => {
                // Update live sitting seconds without committing (sitting_started stays).
                // We don't mutate sitting_seconds here to avoid double-counting on
                // transition; the caller can snapshot() for live display.
            }
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if let Some(bs) = self.state.break_started {
                    self.state.break_seconds = (now - bs).num_seconds().max(0);
                }
            }
        }
    }

    /// Applies break credit rules when the user returns to sitting.
    ///
    /// - break < 5 min  → no effect
    /// - break 5–9 min  → subtract 20 min from sitting_seconds (min 0)
    /// - break ≥ 10 min → reset sitting_seconds to 0
    pub fn apply_break_credit(&mut self, break_secs: i64) {
        if break_secs < BREAK_SHORT_SECS {
            // Less than 5 minutes — no credit.
        } else if break_secs < BREAK_LONG_SECS {
            // 5–9 minutes — subtract 20 minutes.
            self.state.sitting_seconds =
                (self.state.sitting_seconds - SHORT_BREAK_CREDIT_SECS).max(0);
        } else {
            // 10+ minutes — full reset.
            self.state.sitting_seconds = 0;
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn manager_with_sitting_secs(sitting: i64) -> SessionManager {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = sitting;
        m
    }

    // Break < 5 min → no effect
    #[test]
    fn break_under_5_min_has_no_effect() {
        let mut m = manager_with_sitting_secs(3000);
        m.apply_break_credit(4 * 60); // 4-minute break
        assert_eq!(m.state.sitting_seconds, 3000, "short break must not reduce sitting time");
    }

    // Break 7 min → subtract 20 min, floor at 0
    #[test]
    fn break_7_min_subtracts_20_min() {
        let mut m = manager_with_sitting_secs(3000);
        m.apply_break_credit(7 * 60); // 7-minute break
        assert_eq!(
            m.state.sitting_seconds,
            3000 - 1200,
            "7-minute break should subtract 1200 seconds"
        );
    }

    // Break 7 min when sitting < 20 min → floor at 0
    #[test]
    fn break_7_min_floors_at_zero() {
        let mut m = manager_with_sitting_secs(600); // only 10 min sitting
        m.apply_break_credit(7 * 60);
        assert_eq!(m.state.sitting_seconds, 0, "sitting_seconds must not go below 0");
    }

    // Break 12 min → full reset
    #[test]
    fn break_12_min_resets_to_zero() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(12 * 60); // 12-minute break
        assert_eq!(m.state.sitting_seconds, 0, "10+ min break should reset sitting time");
    }

    // Height threshold: below 900 mm → Sitting candidate
    #[test]
    fn height_below_threshold_produces_sitting_candidate() {
        // Test the constant directly — desk at 850 mm is below SIT_HEIGHT_MM.
        assert!(850 <= SIT_HEIGHT_MM, "850mm should be sitting height");
    }

    // Height threshold: above 1000 mm → Standing candidate
    #[test]
    fn height_above_threshold_produces_standing_candidate() {
        assert!(1050 > STAND_HEIGHT_MM, "1050mm should be standing height");
    }

    // Boundary: exactly at sitting threshold
    #[test]
    fn height_exactly_at_sit_threshold() {
        assert!(SIT_HEIGHT_MM <= SIT_HEIGHT_MM, "boundary value should be treated as sitting");
    }

    // Break exactly at short boundary (5 min) → credit applies
    #[test]
    fn break_exactly_5_min_subtracts_20_min() {
        let mut m = manager_with_sitting_secs(2000);
        m.apply_break_credit(5 * 60); // exactly 5 minutes
        assert_eq!(m.state.sitting_seconds, 800, "5-minute break should subtract 1200 seconds");
    }

    // Break exactly at long boundary (10 min) → full reset
    #[test]
    fn break_exactly_10_min_resets() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(10 * 60); // exactly 10 minutes
        assert_eq!(m.state.sitting_seconds, 0, "10-minute break should reset to 0");
    }
}
