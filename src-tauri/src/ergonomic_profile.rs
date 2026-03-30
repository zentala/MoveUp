//! Ergonomic profile — configures session limits, scoring, and KPI thresholds.
//!
//! A profile is loaded from a JSON file at startup and can be hot-reloaded.
//! All fields carry serde defaults so a partial or missing file degrades
//! gracefully to the built-in defaults.

use serde::{Deserialize, Serialize};

// ── default helpers ──────────────────────────────────────────────────────────

fn default_sitting_secs() -> u32 { 2400 }
fn default_standing_secs() -> u32 { 1200 }
fn default_standing_target_secs() -> u32 { 900 }
fn default_standing_max_secs() -> u32 { 5400 }
fn default_break_min_secs() -> u32 { 60 }
fn default_break_credit_multiplier() -> f32 { 2.0 }
fn default_day_break_min_secs() -> u32 { 21600 }
fn default_posture_balance_min_sitting_secs() -> u32 { 21600 }

fn default_pts_standing_per_min() -> f32 { 1.0 }
fn default_pts_session_bonus() -> f32 { 5.0 }
fn default_pts_sitting_per_min() -> f32 { -0.5 }

fn default_standing_green_pct() -> f32 { 15.0 }
fn default_standing_yellow_pct() -> f32 { 10.0 }
fn default_changes_green() -> f32 { 1.0 }
fn default_changes_yellow() -> f32 { 0.5 }
fn default_break_yellow_missed() -> u8 { 2 }
fn default_break_red_missed() -> u8 { 3 }
fn default_session_green_mins() -> u32 { 45 }
fn default_session_yellow_mins() -> u32 { 75 }
fn default_early_data_threshold_mins() -> u32 { 30 }

fn default_profile_id() -> String { "default".to_string() }
fn default_profile_name() -> String { "Default Ergonomic Profile".to_string() }
fn default_profile_description() -> String {
    "Standard 40-min sitting limit with 15-min standing target.".to_string()
}

// ── Limits ───────────────────────────────────────────────────────────────────

/// Session time limits in seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    /// Maximum continuous sitting before an alert fires (seconds).
    #[serde(default = "default_sitting_secs")]
    pub sitting_secs: u32,

    /// Minimum standing to count as a break (seconds).
    #[serde(default = "default_standing_secs")]
    pub standing_secs: u32,

    /// Standing duration for a "full" break with bonus points (seconds).
    #[serde(default = "default_standing_target_secs")]
    pub standing_target_secs: u32,

    /// Maximum continuous standing before a "consider sitting" nudge (seconds).
    #[serde(default = "default_standing_max_secs")]
    pub standing_max_secs: u32,

    /// Minimum break duration before any credit applies (seconds).
    /// Breaks shorter than this are ignored. Default: 60 (1 minute).
    #[serde(default = "default_break_min_secs")]
    pub break_min_secs: u32,

    /// Each second of break cancels this many seconds of sitting.
    /// Default 2.0 = 1 min break cancels 2 min sitting.
    #[serde(default = "default_break_credit_multiplier")]
    pub break_credit_multiplier: f32,

    /// Minimum break duration (seconds) to trigger a "day break" — resets
    /// notification flags and daily_score for a fresh motivational start.
    /// Default: 21600 (6 hours). Set 0 to disable.
    #[serde(default = "default_day_break_min_secs")]
    pub day_break_min_secs: u32,

    /// Minimum total sitting seconds today before PostureBalance notification fires.
    /// Prevents misleading "sitting most of today" after short periods.
    /// Default: 21600 (6 hours).
    #[serde(default = "default_posture_balance_min_sitting_secs")]
    pub posture_balance_min_sitting_secs: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            sitting_secs: default_sitting_secs(),
            standing_secs: default_standing_secs(),
            standing_target_secs: default_standing_target_secs(),
            standing_max_secs: default_standing_max_secs(),
            break_min_secs: default_break_min_secs(),
            break_credit_multiplier: default_break_credit_multiplier(),
            day_break_min_secs: default_day_break_min_secs(),
            posture_balance_min_sitting_secs: default_posture_balance_min_sitting_secs(),
        }
    }
}

// ── Scoring ──────────────────────────────────────────────────────────────────

/// Points awarded / deducted per time unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scoring {
    /// Points earned per minute of standing.
    #[serde(default = "default_pts_standing_per_min")]
    pub pts_standing_per_min: f32,

    /// Bonus points for completing a full standing session.
    #[serde(default = "default_pts_session_bonus")]
    pub pts_session_bonus: f32,

    /// Points deducted per minute of sitting (use negative value).
    #[serde(default = "default_pts_sitting_per_min")]
    pub pts_sitting_per_min: f32,
}

impl Default for Scoring {
    fn default() -> Self {
        Self {
            pts_standing_per_min: default_pts_standing_per_min(),
            pts_session_bonus: default_pts_session_bonus(),
            pts_sitting_per_min: default_pts_sitting_per_min(),
        }
    }
}

// ── KpiThresholds ────────────────────────────────────────────────────────────

/// Thresholds that drive KPI badge colours in the popup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiThresholds {
    /// Standing-time % of workday for green KPI badge.
    #[serde(default = "default_standing_green_pct")]
    pub standing_green_pct: f32,

    /// Standing-time % of workday for yellow KPI badge.
    #[serde(default = "default_standing_yellow_pct")]
    pub standing_yellow_pct: f32,

    /// Position changes per hour for green KPI badge.
    #[serde(default = "default_changes_green")]
    pub changes_green: f32,

    /// Position changes per hour for yellow KPI badge.
    #[serde(default = "default_changes_yellow")]
    pub changes_yellow: f32,

    /// Missed breaks before yellow KPI badge.
    #[serde(default = "default_break_yellow_missed")]
    pub break_yellow_missed: u8,

    /// Missed breaks before red KPI badge.
    #[serde(default = "default_break_red_missed")]
    pub break_red_missed: u8,

    /// Session length (mins) for green KPI badge.
    #[serde(default = "default_session_green_mins")]
    pub session_green_mins: u32,

    /// Session length (mins) for yellow KPI badge.
    #[serde(default = "default_session_yellow_mins")]
    pub session_yellow_mins: u32,

    /// Minimum minutes of data before KPI badges are meaningful.
    #[serde(default = "default_early_data_threshold_mins")]
    pub early_data_threshold_mins: u32,
}

impl Default for KpiThresholds {
    fn default() -> Self {
        Self {
            standing_green_pct: default_standing_green_pct(),
            standing_yellow_pct: default_standing_yellow_pct(),
            changes_green: default_changes_green(),
            changes_yellow: default_changes_yellow(),
            break_yellow_missed: default_break_yellow_missed(),
            break_red_missed: default_break_red_missed(),
            session_green_mins: default_session_green_mins(),
            session_yellow_mins: default_session_yellow_mins(),
            early_data_threshold_mins: default_early_data_threshold_mins(),
        }
    }
}

// ── ErgonomicProfile ─────────────────────────────────────────────────────────

/// Full ergonomic configuration for the SmartDesk session engine.
///
/// Loaded from `profiles/ergonomic/<id>.json` in the app data directory.
/// Missing fields fall back to defaults via serde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErgonomicProfile {
    /// Unique identifier (file stem, e.g. `"default"`).
    #[serde(default = "default_profile_id")]
    pub id: String,

    /// Human-readable name shown in the settings panel.
    #[serde(default = "default_profile_name")]
    pub name: String,

    /// Short description of this profile's intent.
    #[serde(default = "default_profile_description")]
    pub description: String,

    /// Session time limits.
    #[serde(default)]
    pub limits: Limits,

    /// Points scoring configuration.
    #[serde(default)]
    pub scoring: Scoring,

    /// KPI badge thresholds.
    #[serde(default)]
    pub kpi: KpiThresholds,
}

impl Default for ErgonomicProfile {
    fn default() -> Self {
        Self {
            id: default_profile_id(),
            name: default_profile_name(),
            description: default_profile_description(),
            limits: Limits::default(),
            scoring: Scoring::default(),
            kpi: KpiThresholds::default(),
        }
    }
}
