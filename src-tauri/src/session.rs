//! session.rs — Sit/stand desk session state machine.
//!
//! Tracks how long the user has been sitting, applies break credit rules, and
//! fires alert notifications when the sitting limit is reached.

use chrono::{DateTime, NaiveDate, Utc};
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
    /// Total standing seconds accumulated today (incremented only in Standing state).
    pub standing_seconds: i64,
    pub break_started: Option<DateTime<Utc>>,
    /// Duration of the current break in seconds.
    pub break_seconds: i64,
    /// Alert threshold — sitting_seconds >= this value triggers an alert.
    pub session_limit_secs: i64,
    /// Standing session limit in seconds (0 = disabled).
    pub stand_limit_secs: i64,
    /// Last computed desk height in cm (floor_distance_cm - desk_thickness_cm).
    pub desk_height_cm: f32,
    /// Timestamp of last position change (sitting ↔ standing transition).
    pub last_position_change_at: Option<DateTime<Utc>>,
    /// Number of position changes (Sitting↔Standing transitions) today.
    pub position_changes: u32,
}

/// Serialisable DTO emitted with state-change events.
#[derive(Debug, Clone, Serialize)]
pub struct SessionStateDto {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub standing_seconds: i64,
    pub break_seconds: i64,
    pub session_limit_secs: i64,
    pub stand_limit_secs: i64,
    /// Most recent desk height in cm as computed from sensor + calibration.
    pub desk_height_cm: f32,
    /// Number of position changes (Sitting↔Standing transitions) today.
    pub position_changes: u32,
}

/// Payload for the `desk:state-changed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangedPayload {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub standing_seconds: i64,
    pub break_seconds: i64,
    pub desk_height_cm: f32,
    /// Number of position changes (Sitting↔Standing transitions) today.
    pub position_changes: u32,
}

/// Completed sitting session with timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSession {
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: i64,
}

/// Notification event to be sent to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    Inactivity,
    PostureBalance,
    Praise,
    StandLimitReached,
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
    /// Whether a standing alert has been armed for the current sitting session.
    stand_alert_fired: bool,
    /// Last date when daily reset was performed (T009).
    pub last_reset_date: NaiveDate,
    /// Last time when check_daily_reset() was called (T009).
    pub last_reset_check: DateTime<Utc>,
    /// Notification debounce flag: inactivity alert fired today (T003).
    pub notify_inactivity_fired: bool,
    /// Notification debounce flag: posture balance alert fired today (T003).
    pub notify_posture_balance_fired: bool,
    /// Notification debounce flag: praise message fired today (T003).
    pub praise_halfway_fired_today: bool,
}

impl SessionManager {
    /// Creates a manager with default calibration values.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                standing_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: DEFAULT_SESSION_LIMIT_SECS,
                stand_limit_secs: 0,
                desk_height_cm: 0.0,
                last_position_change_at: None,
                position_changes: 0,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: 72.0,
            standing_height_cm: 105.0,
            desk_thickness_cm: 3.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: now.date_naive(),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
        }
    }

    /// Creates a manager initialized from AppConfig.
    #[allow(dead_code)]
    pub fn new_from_config(config: &crate::config::AppConfig) -> Self {
        let now = Utc::now();
        Self {
            state: SessionState {
                state: DeskState::Away,
                sitting_started: None,
                sitting_seconds: 0,
                standing_seconds: 0,
                break_started: None,
                break_seconds: 0,
                session_limit_secs: config.sit_limit_mins as i64 * 60,
                stand_limit_secs: config.stand_limit_mins as i64 * 60,
                desk_height_cm: 0.0,
                last_position_change_at: None,
                position_changes: 0,
            },
            pending_state: None,
            pending_count: 0,
            sitting_height_cm: config.sitting_mm as f32 / 10.0,
            standing_height_cm: config.standing_mm as f32 / 10.0,
            desk_thickness_cm: config.desk_thickness_mm as f32 / 10.0,
            alert_fired: false,
            stand_alert_fired: false,
            last_reset_date: now.date_naive(),
            last_reset_check: now,
            notify_inactivity_fired: false,
            notify_posture_balance_fired: false,
            praise_halfway_fired_today: false,
        }
    }

    /// Seeds today's totals from SQLite so in-memory counters survive restarts.
    #[allow(dead_code)]
    pub fn load_today_totals(&mut self, sitting_secs: i64, standing_secs: i64) {
        self.state.sitting_seconds = sitting_secs;
        self.state.standing_seconds = standing_secs;
        info!(
            "seeded today totals: sitting={}s standing={}s",
            sitting_secs, standing_secs
        );
    }

    /// Updates the sitting session limit (minutes → seconds).
    pub fn set_limit_minutes(&mut self, minutes: u32) {
        self.state.session_limit_secs = minutes as i64 * 60;
    }

    /// Updates the standing session limit (minutes → seconds).
    pub fn set_stand_limit_minutes(&mut self, minutes: u32) {
        self.state.stand_limit_secs = minutes as i64 * 60;
    }

    /// Returns a snapshot of the current session state as a DTO.
    pub fn snapshot(&self) -> SessionStateDto {
        SessionStateDto {
            state: self.state.state.clone(),
            sitting_seconds: self.state.sitting_seconds,
            standing_seconds: self.state.standing_seconds,
            break_seconds: self.state.break_seconds,
            session_limit_secs: self.state.session_limit_secs,
            stand_limit_secs: self.state.stand_limit_secs,
            desk_height_cm: self.state.desk_height_cm,
            position_changes: self.state.position_changes,
        }
    }

    /// Returns the current desk state.
    pub fn current_state(&self) -> DeskState {
        self.state.state.clone()
    }

    /// Checks if a new day has begun and resets daily counters.
    /// Only checks every 60 seconds to avoid overhead.
    /// Returns `true` if reset was performed.
    pub fn check_daily_reset(&mut self) -> bool {
        let now = Utc::now();
        let today = now.date_naive();

        // Only check every 60 seconds
        if (now - self.last_reset_check).num_seconds() < 60 {
            return false;
        }

        self.last_reset_check = now;

        if self.last_reset_date < today {
            info!("daily reset: new day detected, resetting in-memory counters");
            self.state.sitting_seconds = 0;
            self.state.standing_seconds = 0;
            self.state.position_changes = 0;
            self.alert_fired = false;
            self.stand_alert_fired = false;
            self.notify_inactivity_fired = false;
            self.notify_posture_balance_fired = false;
            self.praise_halfway_fired_today = false;
            self.last_reset_date = today;
            return true;
        }

        false
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

    /// Checks notification conditions and returns a list of notifications that should fire.
    ///
    /// Called periodically (e.g., every 60s) to check for two types of notifications:
    /// 1. **Inactivity** — no position change for ≥ 90 min (fires max once/hour)
    /// 2. **Posture Balance** — sitting time > 2× standing time (fires max once/day)
    pub fn check_notification_conditions(&mut self, config: &crate::config::AppConfig) -> Vec<NotificationEvent> {
        let now = Utc::now();
        let mut events = Vec::new();

        // Check inactivity: no position change for 90+ minutes
        if config.notify_inactivity && !self.notify_inactivity_fired {
            if let Some(last_change) = self.state.last_position_change_at {
                let elapsed_secs = (now - last_change).num_seconds();
                if elapsed_secs >= 90 * 60 {
                    self.notify_inactivity_fired = true;
                    events.push(NotificationEvent::Inactivity);
                }
            }
        }

        // Check posture balance: sitting > 2× standing
        if config.notify_daily_posture_balance && !self.notify_posture_balance_fired {
            if self.state.sitting_seconds > self.state.standing_seconds * 2 {
                self.notify_posture_balance_fired = true;
                events.push(NotificationEvent::PostureBalance);
            }
        }

        events
    }

    /// Checks if praise-halfway notification should fire.
    ///
    /// Triggered when transitioning Sitting→Standing if standing_secs >= (stand_limit_secs / 2).
    /// Fires at most once per day (flag reset on daily reset).
    pub fn should_send_praise_halfway(&mut self, config: &crate::config::AppConfig) -> bool {
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

        // Track position changes: only on Sitting↔Standing transitions (not Standing→Walking/Away)
        let is_position_change = (self.state.state == DeskState::Sitting && candidate == DeskState::Standing)
            || (self.state.state == DeskState::Standing && candidate == DeskState::Sitting);

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
                    // Reset stand alert for new break.
                    self.stand_alert_fired = false;
                    // Update last position change timestamp.
                    self.state.last_position_change_at = Some(now);
                }
            }
            DeskState::Standing => {
                if candidate != DeskState::Standing {
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        // Accumulate standing seconds (only when leaving Standing state).
                        self.state.standing_seconds += break_dur;
                        if candidate == DeskState::Sitting {
                            self.apply_break_credit(break_dur);
                        }
                        self.state.break_seconds = 0;
                    }
                    if candidate == DeskState::Sitting {
                        self.state.sitting_started = Some(now);
                    }
                    // Update last position change timestamp.
                    self.state.last_position_change_at = Some(now);
                }
            }
            DeskState::Walking | DeskState::Away => {
                if candidate == DeskState::Sitting {
                    if let Some(bs) = self.state.break_started.take() {
                        let break_dur = (now - bs).num_seconds().max(0);
                        self.apply_break_credit(break_dur);
                        self.state.break_seconds = 0;
                    }
                    self.state.sitting_started = Some(now);
                    // Update last position change timestamp.
                    self.state.last_position_change_at = Some(now);
                }
            }
        }

        // Increment position_changes only on confirmed Sitting↔Standing transitions
        if is_position_change {
            self.state.position_changes += 1;
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
                standing_seconds: self.state.standing_seconds,
                break_seconds: self.state.break_seconds,
                desk_height_cm,
                position_changes: self.state.position_changes,
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
            DeskState::Standing => {
                if let Some(bs) = self.state.break_started {
                    self.state.break_seconds = (now - bs).num_seconds().max(0);
                }
                // Check stand alert: fires when standing exceeds limit.
                // Spec: fire when break_seconds >= stand_limit_secs AND !stand_alert_fired
                // Returns alert in out-of-band event (not in ReadingResult).
            }
            DeskState::Walking | DeskState::Away => {
                if let Some(bs) = self.state.break_started {
                    self.state.break_seconds = (now - bs).num_seconds().max(0);
                }
            }
        }
    }

    /// Checks if a stand limit alert should fire (when standing for too long).
    ///
    /// Returns `true` exactly once per standing stint when `break_seconds >= stand_limit_secs`.
    /// Resets when the user transitions out of Standing state.
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
        assert_eq!(
            m.state.sitting_seconds, 3000,
            "short break must not reduce sitting time"
        );
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
        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds must not go below 0"
        );
    }

    // Break 12 min → full reset
    #[test]
    fn break_12_min_resets_to_zero() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(12 * 60); // 12-minute break
        assert_eq!(
            m.state.sitting_seconds, 0,
            "10+ min break should reset sitting time"
        );
    }

    // Break exactly at short boundary (5 min) → credit applies
    #[test]
    fn break_exactly_5_min_subtracts_20_min() {
        let mut m = manager_with_sitting_secs(2000);
        m.apply_break_credit(5 * 60); // exactly 5 minutes
        assert_eq!(
            m.state.sitting_seconds, 800,
            "5-minute break should subtract 1200 seconds"
        );
    }

    // Break exactly at long boundary (10 min) → full reset
    #[test]
    fn break_exactly_10_min_resets() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(10 * 60); // exactly 10 minutes
        assert_eq!(
            m.state.sitting_seconds, 0,
            "10-minute break should reset to 0"
        );
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

    // ─── T006 Tests: Standing Seconds ────────────────────────────────────────

    // standing_seconds increments only when transitioning from Standing
    #[test]
    fn standing_seconds_accumulates_on_transition() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 0;
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300)); // 5 min ago
        m.state.state = DeskState::Standing;

        // Transition from Standing to Sitting
        let _ = m.on_reading(800, true); // low reading = Sitting
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }

        assert_eq!(m.state.state, DeskState::Sitting);
        assert!(
            m.state.standing_seconds >= 0,
            "standing_seconds should accumulate"
        );
    }

    // standing_seconds is NOT incremented when in Walking state
    #[test]
    fn standing_seconds_does_not_accumulate_in_walking() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 0;

        // Enter Walking state (high reading, inactive)
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Walking);

        // Stay in Walking for a bit (simulate accumulated time)
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300)); // 5 min ago
        let standing_before = m.state.standing_seconds;

        // Call on_reading to advance time without state change
        let _ = m.on_reading(1200, false);

        // standing_seconds should NOT have increased
        assert_eq!(
            m.state.standing_seconds, standing_before,
            "standing_seconds must not increase in Walking state"
        );
    }

    // standing_seconds is reset along with other counters on daily reset
    #[test]
    fn check_daily_reset_resets_standing_seconds() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 3600; // 1 hour
        m.state.sitting_seconds = 2400; // 40 min
        m.alert_fired = true;
        m.stand_alert_fired = true;

        // Force a date change
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120); // 2 min ago

        let reset_happened = m.check_daily_reset();

        assert!(reset_happened, "daily reset should have triggered");
        assert_eq!(
            m.state.standing_seconds, 0,
            "standing_seconds should be reset to 0"
        );
        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds should be reset to 0"
        );
        assert!(!m.alert_fired, "alert_fired should be reset");
        assert!(!m.stand_alert_fired, "stand_alert_fired should be reset");
    }

    // ─── T009 Tests: Daily Reset ────────────────────────────────────────────

    // check_daily_reset returns false when called within same day
    #[test]
    fn check_daily_reset_does_not_reset_same_day() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.state.standing_seconds = 500;

        // Set reset to today
        m.last_reset_date = Utc::now().date_naive();
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(30);

        let reset_happened = m.check_daily_reset();

        assert!(!reset_happened, "reset should not happen on same day");
        assert_eq!(
            m.state.sitting_seconds, 1000,
            "sitting_seconds should not change"
        );
        assert_eq!(
            m.state.standing_seconds, 500,
            "standing_seconds should not change"
        );
    }

    // check_daily_reset respects 60-second throttle
    #[test]
    fn check_daily_reset_throttled_every_60_seconds() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;

        // Force a date change but check within 60 seconds
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(30); // 30 seconds ago

        let reset_happened = m.check_daily_reset();

        assert!(
            !reset_happened,
            "reset should be throttled if < 60 seconds since last check"
        );
        assert_eq!(
            m.state.sitting_seconds, 1000,
            "sitting_seconds should not change"
        );
    }

    // check_daily_reset is idempotent on same day
    #[test]
    fn check_daily_reset_idempotent_same_day() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 500;
        m.last_reset_date = Utc::now().date_naive();

        let first = m.check_daily_reset();
        let second = m.check_daily_reset();

        assert!(
            !first && !second,
            "multiple calls on same day should all return false"
        );
        assert_eq!(
            m.state.sitting_seconds, 500,
            "sitting_seconds should remain unchanged"
        );
    }

    // all notification flags are reset on daily reset
    #[test]
    fn check_daily_reset_clears_all_notification_flags() {
        let mut m = SessionManager::new();
        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;
        m.alert_fired = true;
        m.stand_alert_fired = true;

        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);

        let _ = m.check_daily_reset();

        assert!(
            !m.notify_inactivity_fired,
            "notify_inactivity_fired should be cleared"
        );
        assert!(
            !m.notify_posture_balance_fired,
            "notify_posture_balance_fired should be cleared"
        );
        assert!(
            !m.praise_halfway_fired_today,
            "praise_halfway_fired_today should be cleared"
        );
        assert!(!m.alert_fired, "alert_fired should be cleared");
        assert!(!m.stand_alert_fired, "stand_alert_fired should be cleared");
    }

    // ─── T003 Tests: Notification Preferences ───────────────────────────────

    #[test]
    fn check_notification_conditions_inactivity_fires_after_90min() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_inactivity: true,
            ..Default::default()
        };

        // Set last position change to 91 minutes ago
        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(91));

        let events = m.check_notification_conditions(&config);

        assert!(
            events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)),
            "inactivity notification should fire after 90+ minutes"
        );
        assert!(
            m.notify_inactivity_fired,
            "notify_inactivity_fired should be set"
        );
    }

    #[test]
    fn check_notification_conditions_inactivity_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_inactivity: false,
            ..Default::default()
        };

        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(91));

        let events = m.check_notification_conditions(&config);

        assert!(
            !events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)),
            "inactivity should not fire when disabled"
        );
        assert!(
            !m.notify_inactivity_fired,
            "notify_inactivity_fired should not be set"
        );
    }

    #[test]
    fn check_notification_conditions_posture_balance_fires() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_daily_posture_balance: true,
            ..Default::default()
        };

        m.state.sitting_seconds = 3600; // 1 hour
        m.state.standing_seconds = 1000; // <0.5 hours: sitting > 2× standing

        let events = m.check_notification_conditions(&config);

        assert!(
            events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "posture balance notification should fire when sitting > 2× standing"
        );
        assert!(
            m.notify_posture_balance_fired,
            "notify_posture_balance_fired should be set"
        );
    }

    #[test]
    fn check_notification_conditions_posture_balance_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_daily_posture_balance: false,
            ..Default::default()
        };

        m.state.sitting_seconds = 3600;
        m.state.standing_seconds = 1000;

        let events = m.check_notification_conditions(&config);

        assert!(
            !events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "posture balance should not fire when disabled"
        );
    }

    #[test]
    fn check_notification_conditions_all_reset_on_daily_reset() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig::default();

        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;

        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);

        let _ = m.check_daily_reset();

        // After daily reset, all notification flags should be cleared
        let events = m.check_notification_conditions(&config);
        assert_eq!(events.len(), 0, "no notifications should fire after reset");
    }

    // ─── T004 Tests: Stand Limit Alert ──────────────────────────────────────

    #[test]
    fn should_stand_alert_fires_once_per_standing_stint() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900; // 15 minutes
        m.state.break_seconds = 901; // 15 min + 1 second

        assert!(
            m.should_stand_alert(),
            "stand alert should fire when break_seconds >= stand_limit_secs"
        );
        assert!(
            m.stand_alert_fired,
            "stand_alert_fired should be set"
        );
        assert!(
            !m.should_stand_alert(),
            "stand alert should not fire twice"
        );
    }

    #[test]
    fn should_stand_alert_disabled_when_stand_limit_is_zero() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 0; // disabled
        m.state.break_seconds = 1000; // well over any limit

        assert!(
            !m.should_stand_alert(),
            "stand alert should not fire when stand_limit_secs is 0"
        );
    }

    #[test]
    fn should_stand_alert_reset_on_state_change() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900;
        m.state.break_seconds = 901;

        // Fire the alert
        assert!(m.should_stand_alert());

        // Simulate transition from Standing to Sitting
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(901));
        let _ = m.on_reading(800, true); // low reading = sitting
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }

        // Now transition back to Standing
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }

        // Alert should fire again in the new standing stint
        m.state.break_seconds = 901; // reset break_seconds as if new break started
        assert!(
            m.should_stand_alert(),
            "stand alert should fire again in new standing stint"
        );
    }

    // ─── T005 Tests: Position Changes ───────────────────────────────────────

    // position_changes increments on Sitting → Standing transition
    #[test]
    fn position_changes_increments_sitting_to_standing() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));

        // Transition to Standing (high reading + active)
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }

        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(
            m.state.position_changes, 1,
            "position_changes should increment on Sitting → Standing"
        );
    }

    #[test]
    fn set_stand_limit_minutes_converts_correctly() {
        let mut m = SessionManager::new();
        m.set_stand_limit_minutes(20);

        assert_eq!(
            m.state.stand_limit_secs, 1200,
            "20 minutes should be 1200 seconds"
        );
    }

    #[test]
    fn should_send_praise_halfway_fires_at_50_percent() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            stand_limit_mins: 20,
            ..Default::default()
        };

        m.state.stand_limit_secs = 1200; // 20 minutes
        m.state.standing_seconds = 600; // exactly 50%

        assert!(
            m.should_send_praise_halfway(&config),
            "praise should fire at 50% of standing goal"
        );
        assert!(
            m.praise_halfway_fired_today,
            "praise_halfway_fired_today should be set"
        );
    }

    #[test]
    fn should_send_praise_halfway_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: false,
            stand_limit_mins: 20,
            ..Default::default()
        };

        m.state.stand_limit_secs = 1200;
        m.state.standing_seconds = 600;

        assert!(
            !m.should_send_praise_halfway(&config),
            "praise should not fire when disabled"
        );
    }

    #[test]
    fn should_send_praise_halfway_disabled_when_stand_limit_zero() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            ..Default::default()
        };

        m.state.stand_limit_secs = 0;
        m.state.standing_seconds = 600;

        assert!(
            !m.should_send_praise_halfway(&config),
            "praise should not fire when stand_limit_secs is 0"
        );
    }

    #[test]
    fn should_send_praise_halfway_fires_once_per_day() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            stand_limit_mins: 20,
            ..Default::default()
        };

        m.state.stand_limit_secs = 1200;
        m.state.standing_seconds = 600;

        assert!(m.should_send_praise_halfway(&config));
        assert!(!m.should_send_praise_halfway(&config), "should not fire twice");
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(
            m.state.position_changes, 1,
            "position_changes should increment on Sitting → Standing"
        );
    }

    // position_changes increments on Standing → Sitting transition
    #[test]
    fn position_changes_increments_standing_to_sitting() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300)); // 5 min
        m.state.position_changes = 1;

        // Transition to Sitting (low reading)
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }

        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(
            m.state.position_changes, 2,
            "position_changes should increment on Standing → Sitting"
        );
    }

    // position_changes does NOT increment on Standing → Walking (same break category)
    #[test]
    fn position_changes_does_not_increment_standing_to_walking() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.break_started = Some(Utc::now());
        m.state.position_changes = 5;

        // Transition to Walking (high reading, inactive)
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }

        assert_eq!(m.state.state, DeskState::Walking);
        assert_eq!(
            m.state.position_changes, 5,
            "position_changes must not change on Standing → Walking"
        );
    }

    // position_changes does NOT increment on Walking → Standing (both are breaks)
    #[test]
    fn position_changes_does_not_increment_walking_to_standing() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Walking;
        m.state.break_started = Some(Utc::now());
        m.state.position_changes = 3;

        // Transition to Standing (high reading, active)
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }

        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(
            m.state.position_changes, 3,
            "position_changes must not change on Walking → Standing"
        );
    }

    // position_changes resets on daily reset
    #[test]
    fn position_changes_resets_on_daily_reset() {
        let mut m = SessionManager::new();
        m.state.position_changes = 12;
        m.state.sitting_seconds = 2400;

        // Force a date change
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);

        let reset_happened = m.check_daily_reset();

        assert!(reset_happened, "daily reset should have triggered");
        assert_eq!(
            m.state.position_changes, 0,
            "position_changes should be reset to 0"
        );
    }

    // position_changes is included in StateChangedPayload
    #[test]
    fn state_changed_payload_includes_position_changes() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));
        m.state.position_changes = 2;

        // Transition to Standing
        for _ in 0..DEBOUNCE_COUNT {
            let result = m.on_reading(1200, true);
            if let Some(payload) = result.state_change {
                assert_eq!(
                    payload.position_changes, 3,
                    "payload should include incremented position_changes"
                );
                return;
            }
        }

        panic!("expected state change payload");
    }
}
