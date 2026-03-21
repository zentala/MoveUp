//! session_types.rs — Structs, enums, and DTOs for the desk session state machine.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of consecutive readings at a new height before state transitions.
pub const DEBOUNCE_COUNT: u8 = 5;
/// Default session sitting limit (45 minutes in seconds).
pub const DEFAULT_SESSION_LIMIT_SECS: i64 = 2700;
/// Minimum break duration for any credit (5 minutes).
pub const BREAK_SHORT_SECS: i64 = 300;
/// Break duration threshold for partial credit (10 minutes).
pub const BREAK_LONG_SECS: i64 = 600;
/// Sitting seconds subtracted for a short break (20 minutes).
pub const SHORT_BREAK_CREDIT_SECS: i64 = 1200;

// ─── Enums ───────────────────────────────────────────────────────────────────

/// The ergonomic state the user is currently in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeskState {
    Sitting,
    Standing,
    Walking,
    Away,
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
    /// Number of position changes (Sitting<->Standing transitions) today.
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
    /// Number of position changes (Sitting<->Standing transitions) today.
    pub position_changes: u32,
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
