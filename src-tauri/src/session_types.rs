//! session_types.rs — Structs, enums, and DTOs for the desk session state machine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of consecutive readings at a new height before state transitions.
pub const DEBOUNCE_COUNT: u8 = 5;
/// Default session sitting limit (45 minutes in seconds).
pub const DEFAULT_SESSION_LIMIT_SECS: i64 = 2700;
/// Gap between sensor readings that indicates machine sleep/suspend (5 minutes).
pub const SLEEP_GAP_THRESHOLD_SECS: i64 = 300;
/// Maximum reasonable session duration (3 hours). Longer durations indicate
/// the app survived a sleep/suspend without the rewind in on_reading() firing.
pub const MAX_REASONABLE_SESSION_SECS: i64 = 3 * 3600;

// ─── Enums ───────────────────────────────────────────────────────────────────

/// The ergonomic state the user is currently in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub enum DeskState {
    Sitting,
    Standing,
    Walking,
    Away,
}

impl DeskState {
    /// Parses a DB state string. Returns `None` for unknown/Away states.
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "Sitting" => Some(Self::Sitting),
            "Standing" => Some(Self::Standing),
            "Walking" => Some(Self::Walking),
            "Away" => Some(Self::Away),
            _ => None,
        }
    }

    /// Whether this state represents a physical desk position (not Away).
    pub fn is_desk_position(&self) -> bool {
        matches!(self, Self::Sitting | Self::Standing | Self::Walking)
    }

    /// Whether this state counts toward standing/break time.
    pub fn is_standing_like(&self) -> bool {
        matches!(self, Self::Standing | Self::Walking)
    }
}

/// Notification event to be sent to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    Inactivity,
    PostureBalance,
    Praise,
    StandLimitReached,
    StandingTargetReached,
}

// ─── Structs ─────────────────────────────────────────────────────────────────

/// Full session state, managed by `SessionManager`.
///
/// **State only — never configuration.** Ergonomic limits
/// (`break_min_secs`, `break_credit_multiplier`, `day_break_min_secs`,
/// `posture_balance_min_sitting_secs`, `max_continuous_computer_secs`,
/// `computer_break_reset_secs`) used to be copied in here at construction and
/// never refreshed, which silently disabled ergonomic-profile hot-reload. They
/// now live in [`crate::session_manager::SessionManager::limits`], refreshed
/// from the active profile every tick. Do not add a config field here.
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
    /// Timestamp of last position change (sitting <-> standing transition).
    pub last_position_change_at: Option<DateTime<Utc>>,
    /// Number of position changes (Sitting<->Standing transitions) today.
    pub position_changes: u32,
    /// Duration of the last completed break/standing session (seconds).
    pub last_break_secs: i64,
    /// Duration of the last completed sitting session (seconds).
    pub last_sitting_secs: i64,
    /// Break credit applied on the most recent Standing->Sitting transition.
    pub last_break_credit: BreakCredit,
    /// Daily posture score (in-memory, resets at midnight).
    pub daily_score: f32,
    /// Standing seconds in current continuous standing session (resets on sit/away).
    pub standing_session_secs: i64,
    /// Timestamp when current standing session started (for live computation).
    pub standing_session_started: Option<DateTime<Utc>>,
    /// The last lap number for which the +5 bonus was awarded (resets per session).
    pub lap_bonus_awarded_for_lap: u32,
    /// Seconds the user has been at the computer continuously (Sitting + Standing + Walking).
    /// Resets after 5+ continuous minutes of Away.
    pub continuous_computer_secs: i64,
    /// Longest continuous computer session today (daily max, seconds).
    pub longest_computer_session_secs: i64,
    /// Current continuous Away duration (seconds). Resets when user returns.
    pub away_bout_secs: i64,
    /// Timestamp of the first sensor reading of the day (for hours_worked calculation).
    pub first_reading_at: Option<DateTime<Utc>>,
    /// Timestamp of last tick (for gap detection on app restart).
    pub last_tick_ts: Option<DateTime<Utc>>,
    /// Timestamp of last `accumulate_ongoing()` execution (throttle: max 1 Hz).
    /// Sensor readings may arrive faster than 1 Hz; this prevents inflated counters.
    pub last_accumulate_ts: Option<DateTime<Utc>>,
    /// Timestamp when the current standing bout started (set on Standing entry, cleared on exit).
    /// Tracks actual standing time separately from total break time (which includes Away).
    pub standing_bout_started: Option<DateTime<Utc>>,
    /// Number of completed work hours that had a ≥5 min away break (from HourlyBreakTracker).
    pub hourly_breaks_covered: u8,
    /// Number of hours the user was active today (from HourlyBreakTracker).
    pub hourly_breaks_active: u8,
    /// Raw total sitting seconds today — never reduced by break credit.
    /// Used by standing_pct metric for accurate KPI (sitting_seconds is modified by break credit).
    pub sitting_seconds_total: i64,
    /// Current system idle time in seconds (refreshed every tick from activity.rs).
    pub idle_secs: i64,
}

/// Serialisable DTO emitted with state-change events.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct SessionStateDto {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub standing_seconds: i64,
    pub break_seconds: i64,
    pub session_limit_secs: i64,
    pub stand_limit_secs: i64,
    /// Most recent desk height in cm as computed from sensor + calibration.
    pub desk_height_cm: f32,
    /// Number of position changes (Sitting<->Standing transitions) today.
    pub position_changes: u32,
    /// Seconds of sitting limit consumed (accounts for break credits).
    /// 0..limit_secs normally, >limit_secs when overtime.
    ///
    /// **This is the only counter any timer, progress bar, colour band or
    /// notification may read.** It equals the credited `sitting_seconds`
    /// (ADR 008): a break reduces it proportionally, it is never reset by a
    /// return to sitting.
    pub limit_used_secs: i64,
    /// Daily posture score (in-memory, resets at midnight).
    pub daily_score: f32,
    /// Current continuous standing session seconds (resets on sit).
    pub standing_session_secs: i64,
    /// Seconds since the last position change, uncredited.
    ///
    /// **Debug tab only.** No timer, progress bar, colour band or
    /// notification may read this — it ignores break credit entirely and
    /// restarts from zero on every position change, which is exactly the
    /// reset the user complained about (E015, decision D2). Use
    /// `limit_used_secs` for anything the user reads as session progress.
    pub secs_since_last_break: i64,
    /// Seconds at computer continuously (resets after 5+ min Away).
    pub continuous_computer_secs: i64,
    /// Longest continuous computer session today (seconds).
    pub longest_computer_session_secs: i64,
    /// Raw total sitting seconds today (never reduced by break credit).
    pub sitting_seconds_total: i64,
    /// Current system idle time in seconds.
    pub idle_secs: i64,
    /// Current continuous Away duration (seconds).
    pub away_bout_secs: i64,
    /// Maximum continuous computer time limit (seconds, from ergonomic profile).
    pub max_continuous_computer_secs: i64,
}

/// Break credit type applied when returning from standing to sitting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub enum BreakCredit {
    /// Break too short — session continues unchanged.
    None,
    /// Proportional credit — session reduced by break × multiplier.
    Partial,
    /// Credit >= sitting — session reset to 0.
    Full,
}

/// Payload for the `desk:state-changed` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct StateChangedPayload {
    pub state: DeskState,
    pub standing_seconds: i64,
    pub break_seconds: i64,
    pub desk_height_cm: f32,
    /// Number of position changes (Sitting<->Standing transitions) today.
    pub position_changes: u32,
    /// Duration of the last standing/break session in seconds (for transition UI).
    pub last_break_secs: i64,
    /// Duration of the last sitting session in seconds (for transition UI).
    pub last_sitting_secs: i64,
    /// Break credit applied on this transition ("none", "partial", "full").
    pub break_credit: BreakCredit,
    /// Seconds of sitting limit consumed (credited — see `SessionStateDto`).
    pub limit_used_secs: i64,
}

/// Completed sitting session with timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSession {
    pub started_at: String,
    pub ended_at: String,
    pub duration_secs: i64,
}

/// Result of processing a sensor reading.
#[derive(Debug, Clone, Default)]
pub struct ReadingResult {
    pub state_change: Option<StateChangedPayload>,
    pub completed_session: Option<CompletedSession>,
    /// Break credit applied this reading (type + standing duration in secs).
    pub break_credit: Option<(BreakCredit, i64)>,
}
