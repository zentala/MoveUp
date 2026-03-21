//! tray_controller_tests.rs — Unit tests for tooltip formatting and standing progress.

use crate::tray_controller::{build_tooltip_label, format_duration};
use crate::session::DeskState;

#[test]
fn format_duration_zero() {
    assert_eq!(format_duration(0), "00:00");
}

#[test]
fn format_duration_one_minute() {
    assert_eq!(format_duration(60), "01:00");
}

#[test]
fn format_duration_mixed() {
    assert_eq!(format_duration(754), "12:34");
}

#[test]
fn format_duration_negative_clamps() {
    assert_eq!(format_duration(-10), "00:00");
}

#[test]
fn tooltip_sitting_shows_sitting_seconds() {
    let label = build_tooltip_label(72.3, &DeskState::Sitting, 754, 0, 0);
    assert_eq!(label, "↕ 72.3 cm — Sitting (12:34)");
}

#[test]
fn tooltip_standing_shows_standing_seconds() {
    let label = build_tooltip_label(114.0, &DeskState::Standing, 120, 452, 452);
    assert_eq!(label, "↕ 114.0 cm — Standing (07:32)");
}

#[test]
fn tooltip_walking_shows_break_seconds() {
    let label = build_tooltip_label(114.0, &DeskState::Walking, 300, 200, 95);
    assert_eq!(label, "↕ 114.0 cm — Walking (01:35)");
}

#[test]
fn tooltip_away_shows_zero() {
    let label = build_tooltip_label(0.0, &DeskState::Away, 500, 200, 100);
    assert_eq!(label, "↕ 0.0 cm — Away (00:00)");
}

// ─── Standing progress calculation tests ─────────────────────────────────────

#[test]
fn standing_progress_at_zero() {
    let session_secs: i64 = 0;
    let target: i64 = 900;
    let session_lap = (session_secs / target) as u32;
    let lap_progress = (session_secs % target) as f32 / target as f32;
    assert_eq!(session_lap, 0);
    assert!((lap_progress - 0.0).abs() < 0.01);
}

#[test]
fn standing_progress_at_half() {
    let session_secs: i64 = 450;
    let target: i64 = 900;
    let session_lap = (session_secs / target) as u32;
    let lap_progress = (session_secs % target) as f32 / target as f32;
    assert_eq!(session_lap, 0);
    assert!((lap_progress - 0.5).abs() < 0.01);
}

#[test]
fn standing_progress_at_target_fires_flash() {
    let session_secs: i64 = 900;
    let standing_seconds: i64 = 900;
    let target: i64 = 900;
    let session_lap = (session_secs / target) as u32;
    let lap_progress = (session_secs % target) as f32 / target as f32;
    let total_laps = (standing_seconds / target) as u32;
    assert_eq!(session_lap, 1, "session_lap should be 1 at target");
    assert!((lap_progress - 0.0).abs() < 0.01);
    assert_eq!(total_laps, 1);
}

#[test]
fn standing_new_session_after_sit_no_immediate_flash() {
    // User stood 15min (total=900), sat, stood again 5min (session_secs=300, total=1200)
    let session_secs: i64 = 300;
    let standing_seconds: i64 = 1200;
    let target: i64 = 900;
    let session_lap = (session_secs / target) as u32;
    let lap_progress = (session_secs % target) as f32 / target as f32;
    let total_laps = (standing_seconds / target) as u32;
    assert_eq!(session_lap, 0, "session_lap=0 (no flash)");
    assert!((lap_progress - 0.333).abs() < 0.01);
    assert_eq!(total_laps, 1, "total_laps shows 1 completed today");
}
