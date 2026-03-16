//! session.rs — Sit/stand desk session state machine.
//!
//! Tracks how long the user has been sitting, applies break credit rules, and
//! fires alert notifications when the sitting limit is reached.

use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of consecutive readings at a new height before state transitions.
const DEBOUNCE_COUNT: u8 = 5;
/// Default session sitting limit (45 minutes in seconds).
pub const DEFAULT_SESSION_LIMIT_SECS: i64 = 2700;
/// Minimum break duration for any credit (5 minutes).
const BREAK_SHORT_SECS: i64 = 300;
/// Break duration threshold for partial credit (10 minutes).
const BREAK_LONG_SECS: i64 = 600;
/// Sitting seconds subtracted for a short break (20 minutes).
const SHORT_BREAK_CREDIT_SECS: i64 = 1200;

// ─── Types ───────────────────────────────────────────────────────────────────

/// The ergonomic state the user is currently in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Last computed desk height in cm (floor_distance_cm - desk_thickness_cm).
    pub desk_height_cm: f32,
}

/// Serialisable DTO emitted with state-change events.
#[derive(Debug, Clone, Serialize)]
pub struct SessionStateDto {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub break_seconds: i64,
    pub session_limit_secs: i64,
    /// Most recent desk height in cm as computed from sensor + calibration.
    pub desk_height_cm: f32,
}

/// Payload for the `desk:state-changed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangedPayload {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub break_seconds: i64,
    pub desk_height_cm: f32,
}

/// Completed sitting session with timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSession {
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: i64,
}

/// Result of processing a sensor reading.
#[derive(Debug, Clone)]
pub struct ReadingResult {
    pub state_change: Option<StateChangedPayload>,
    pub completed_session: Option<CompletedSession>,
}

// ─── SessionManager ───────────────────────────────────────────────────────────

/// Owns `SessionState` and drives state transitions.
pub struct SessionManager {
    state: SessionState,
    /// Pending candidate state (needs `debounce_count` confirmations).
    pending_state: Option<DeskState>,
    pending_count: u8,
    /// Calibrated sitting desk height in cm (default 72.0).
    pub sitting_height_cm: f32,
    /// Calibrated standing desk height in cm (default 105.0).
    pub standing_height_cm: f32,
    /// Desk surface thickness in cm to subtract from raw sensor reading (default 3.0).
    pub desk_thickness_cm: f32,
    /// Whether `should_alert()` has been armed for the current sitting session.
    alert_fired: bool,
}

impl SessionManager {
    /// Creates a manager with default calibration values.
    pub fn new() -> Self {
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: DEFAULT_SESSION_LIMIT_SECS,
                desk_height_cm: 0.0,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: 72.0,
            standing_height_cm: 105.0,
            desk_thickness_cm: 3.0,
            alert_fired: false,
        }
    }

    /// Creates a manager initialized from AppConfig.
    pub fn new_from_config(config: &crate::config::AppConfig) -> Self {
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: config.sit_limit_mins as i64 * 60,
                desk_height_cm: 0.0,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: config.sitting_mm as f32 / 10.0,
            standing_height_cm: config.standing_mm as f32 / 10.0,
            desk_thickness_cm: config.desk_thickness_mm as f32 / 10.0,
            alert_fired: false,
        }
    }

    /// Seeds today's totals from SQLite so in-memory counters survive restarts.
    pub fn load_today_totals(&mut self, sitting_secs: i64, standing_secs: i64) {
        self.state.sitting_seconds = sitting_secs;
        info!(
            "seeded today totals: sitting={}s standing={}s",
            sitting_secs, standing_secs
        );
    }

    /// Updates the sitting session limit (minutes → seconds).
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
            desk_height_cm: self.state.desk_height_cm,
        }
    }

    /// Returns `true` (exactly once per sitting stint) when the user has been
    /// sitting for at least `session_limit_secs` and an alert should be shown.
    ///
    /// Resets when the user transitions out of the Sitting state.
    pub fn should_alert(&mut self) -> bool {
        if self.state.state == DeskState::Sitting
            && !self.alert_fired
            && self.state.sitting_seconds >= self.state.session_limit_secs
        {
            self.alert_fired = true;
            return true;
        }
        false
    }

    /// Called for each new distance reading from the sensor.
    ///
    /// `mm` is the raw sensor reading (floor distance from sensor underside).
    /// `active` is `true` when the user has been active recently (keyboard/mouse).
    ///
    /// Returns a [`ReadingResult`] containing optional state change and completed session info.
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

        // Debounce: only transition when we see DEBOUNCE_COUNT consistent readings.
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

        // ── Leaving current state ─────────────────────────────────────────────
        let mut completed_session = None;

        match &self.state.state {
            DeskState::Sitting => {
                if let Some(started) = self.state.sitting_started.take() {
                    let elapsed = (now - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;

                    // Record completed session when leaving Sitting
                    if candidate != DeskState::Sitting {
                        completed_session = Some(CompletedSession {
                            started_at: started.to_rfc3339(),
                            ended_at: now.to_rfc3339(),
                            duration_secs: elapsed,
                        });
                    }
                }
                if candidate != DeskState::Sitting {
                    self.state.break_started = Some(now);
                    self.state.break_seconds = 0;
                    // Reset alert so it can fire in the next sitting stint.
                    self.alert_fired = false;
                }
            }
            DeskState::Standing | DeskState::Walking | DeskState::Away => {
                if candidate == DeskState::Sitting {
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
            "State transition: {:?} → {:?}  (sitting={}s desk_height={:.1}cm)",
            self.state.state, candidate, self.state.sitting_seconds, desk_height_cm
        );

        self.state.state = candidate;

        ReadingResult {
            state_change: Some(StateChangedPayload {
                state: self.state.state.clone(),
                sitting_seconds: self.state.sitting_seconds,
                break_seconds: self.state.break_seconds,
                desk_height_cm,
            }),
            completed_session,
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Accumulates time while remaining in the current state (no transition).
    fn accumulate_ongoing(&mut self, now: DateTime<Utc>) {
        match self.state.state {
            DeskState::Sitting => {
                // sitting_seconds is committed on transition; live value
                // can be derived from snapshot() + sitting_started elapsed.
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

    // Calibrated height: desk_height_cm = floor_distance_cm - desk_thickness_cm
    #[test]
    fn on_reading_computes_desk_height_correctly() {
        let mut m = SessionManager::new();
        // 1000 mm = 100 cm floor distance, minus 3 cm thickness = 97 cm desk height
        let _result = m.on_reading(1000, true);
        assert!(
            (m.state.desk_height_cm - 97.0).abs() < 0.01,
            "desk_height_cm should be 97.0 but got {}",
            m.state.desk_height_cm
        );
    }

    // Sitting threshold: desk height <= midpoint → Sitting candidate
    #[test]
    fn low_reading_produces_sitting_candidate() {
        let mut m = SessionManager::new();
        // Default: sitting=72, standing=105, midpoint=88.5
        // 800 mm = 80 cm floor → 77 cm desk height → below midpoint → Sitting
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    // Standing threshold: desk height > midpoint + active → Standing candidate
    #[test]
    fn high_reading_active_produces_standing_candidate() {
        let mut m = SessionManager::new();
        // 1200 mm = 120 cm floor → 117 cm desk height → above midpoint → Standing (active)
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // Walking: desk height > midpoint + not active → Walking candidate
    #[test]
    fn high_reading_inactive_produces_walking_candidate() {
        let mut m = SessionManager::new();
        // same height but inactive
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Walking);
    }

    // should_alert fires once when limit reached, not again until break
    #[test]
    fn should_alert_fires_once_then_suppressed() {
        let mut m = SessionManager::new();
        m.state.session_limit_secs = 10;
        m.state.sitting_seconds = 11;
        m.state.state = DeskState::Sitting;
        assert!(m.should_alert(), "first call should return true");
        assert!(!m.should_alert(), "second call should be suppressed");
    }
}
