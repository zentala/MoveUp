//! Unit tests for `commands_analyst` — kept in a sibling file to respect the
//! project's ≤250-lines-per-file rule.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::commands_analyst::{
    parse_event_line, t_collect_events, t_collect_snapshots, t_parse_range,
};

// ─── parse_event_line ────────────────────────────────────────────────────────

#[test]
fn parses_state_event() {
    let row = parse_event_line("2026-05-16", "10:15:42 STATE Sitting->Standing").unwrap();
    assert_eq!(row.ts, "2026-05-16 10:15:42");
    assert_eq!(row.kind, "STATE");
    assert_eq!(row.detail, "Sitting->Standing");
}

#[test]
fn parses_all_known_kinds() {
    for kind in &[
        "STATE", "DEVICE", "ALERT", "RESET", "CREDIT", "START", "NOTIF", "AUTOSTART",
    ] {
        let line = format!("09:00:00 {} sample detail", kind);
        let row = parse_event_line("2026-05-16", &line).unwrap();
        assert_eq!(row.kind, *kind);
        assert_eq!(row.ts, "2026-05-16 09:00:00");
        assert_eq!(row.detail, "sample detail");
    }
}

#[test]
fn parses_event_with_no_detail() {
    let row = parse_event_line("2026-05-16", "08:00:00 START").unwrap();
    assert_eq!(row.kind, "START");
    assert_eq!(row.detail, "");
}

#[test]
fn rejects_malformed_lines() {
    assert!(parse_event_line("2026-05-16", "").is_none());
    assert!(parse_event_line("2026-05-16", "   ").is_none());
    assert!(parse_event_line("2026-05-16", "nope").is_none());
    assert!(parse_event_line("2026-05-16", "1015 STATE x").is_none());
    assert!(parse_event_line("2026-05-16", "10:15:42").is_none());
}

// ─── parse_range ─────────────────────────────────────────────────────────────

#[test]
fn range_inclusive_bounds() {
    let r = t_parse_range("2026-05-14", "2026-05-16").unwrap();
    assert_eq!(r, vec!["2026-05-14", "2026-05-15", "2026-05-16"]);
}

#[test]
fn range_single_day() {
    let r = t_parse_range("2026-05-16", "2026-05-16").unwrap();
    assert_eq!(r, vec!["2026-05-16"]);
}

#[test]
fn range_reversed_yields_empty() {
    let r = t_parse_range("2026-05-16", "2026-05-14").unwrap();
    assert!(r.is_empty());
}

#[test]
fn range_rejects_bad_date() {
    assert!(t_parse_range("not-a-date", "2026-05-16").is_err());
    assert!(t_parse_range("2026-05-16", "also-bad").is_err());
}

// ─── collect_snapshots ───────────────────────────────────────────────────────

fn write_snapshot(dir: &Path, date: &str, file: &str, ts: &str) {
    let day = dir.join(date);
    fs::create_dir_all(&day).unwrap();
    let json = format!(
        r#"{{
          "ts": "{}",
          "state": "Sitting",
          "sitting_seconds": 120,
          "standing_seconds": 60,
          "break_seconds": 30,
          "desk_height_cm": 72.5,
          "idle_secs": 0,
          "continuous_computer_secs": 1800,
          "position_changes": 3,
          "daily_score": 4.2
        }}"#,
        ts
    );
    fs::write(day.join(file), json).unwrap();
}

#[test]
fn collects_snapshots_within_range_only() {
    let tmp = TempDir::new().unwrap();
    write_snapshot(tmp.path(), "2026-05-14", "10-00.json", "2026-05-14T10:00:00Z");
    write_snapshot(tmp.path(), "2026-05-15", "11-00.json", "2026-05-15T11:00:00Z");
    write_snapshot(tmp.path(), "2026-05-17", "12-00.json", "2026-05-17T12:00:00Z");

    let dates = t_parse_range("2026-05-14", "2026-05-15").unwrap();
    let rows = t_collect_snapshots(tmp.path(), &dates);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].ts, "2026-05-14T10:00:00Z");
    assert_eq!(rows[1].ts, "2026-05-15T11:00:00Z");
}

#[test]
fn empty_range_returns_empty_snapshots() {
    let tmp = TempDir::new().unwrap();
    let rows = t_collect_snapshots(tmp.path(), &[]);
    assert!(rows.is_empty());
}

#[test]
fn missing_day_is_skipped() {
    let tmp = TempDir::new().unwrap();
    let dates = t_parse_range("2026-05-14", "2026-05-16").unwrap();
    let rows = t_collect_snapshots(tmp.path(), &dates);
    assert!(rows.is_empty());
}

#[test]
fn malformed_json_does_not_abort_walk() {
    let tmp = TempDir::new().unwrap();
    write_snapshot(tmp.path(), "2026-05-15", "10-00.json", "2026-05-15T10:00:00Z");
    let day = tmp.path().join("2026-05-15");
    fs::write(day.join("10-01.json"), "{ not valid json").unwrap();
    write_snapshot(tmp.path(), "2026-05-15", "10-02.json", "2026-05-15T10:02:00Z");

    let dates = t_parse_range("2026-05-15", "2026-05-15").unwrap();
    let rows = t_collect_snapshots(tmp.path(), &dates);
    assert_eq!(rows.len(), 2);
}

// ─── collect_events ──────────────────────────────────────────────────────────

#[test]
fn collects_events_across_days() {
    let tmp = TempDir::new().unwrap();
    for date in &["2026-05-14", "2026-05-15"] {
        let day = tmp.path().join(date);
        fs::create_dir_all(&day).unwrap();
        fs::write(
            day.join("events.log"),
            "08:00:00 START app launched\n09:00:00 STATE Sitting->Standing\nbroken line\n",
        )
        .unwrap();
    }
    let dates = t_parse_range("2026-05-14", "2026-05-15").unwrap();
    let rows = t_collect_events(tmp.path(), &dates);
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].ts, "2026-05-14 08:00:00");
    assert_eq!(rows[0].kind, "START");
}

#[test]
fn collect_events_empty_when_no_logs() {
    let tmp = TempDir::new().unwrap();
    let dates = t_parse_range("2026-05-14", "2026-05-15").unwrap();
    let rows = t_collect_events(tmp.path(), &dates);
    assert!(rows.is_empty());
}
