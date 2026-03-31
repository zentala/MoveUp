//! db_tests.rs — Unit tests for the database layer.

use rusqlite::Connection;

use crate::db::{init_schema, SessionRow};
use crate::db_sessions::{insert_session, insert_session_with_date, load_today_totals, get_totals_for_date};
use crate::db_queries::get_today_summary;
use crate::db_sessions::get_yesterday_totals;

fn test_conn() -> Connection {
    Connection::open_in_memory().unwrap()
}

#[test]
fn test_schema_init_idempotent() {
    let conn = test_conn();
    assert!(init_schema(&conn).is_ok());
    assert!(init_schema(&conn).is_ok(), "second call should be safe");
}

#[test]
fn test_insert_and_query_today() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let started_at = "2025-03-16T09:00:00Z";
    let ended_at = "2025-03-16T09:30:00Z";
    let duration = 1800;

    assert!(insert_session(&conn, started_at, ended_at, "Sitting", duration).is_ok());

    let _totals = load_today_totals(&conn, None).expect("query should succeed");
}

#[test]
fn test_load_today_totals_empty() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 0);
    assert_eq!(totals.standing_secs, 0);
    assert_eq!(totals.position_changes, 0);
}

#[test]
fn test_get_yesterday_totals_empty() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let (sitting, standing) = get_yesterday_totals(&conn).unwrap();
    assert_eq!(sitting, 0, "no yesterday data should return 0 sitting");
    assert_eq!(standing, 0, "no yesterday data should return 0 standing");
}

#[test]
fn test_get_today_summary_includes_yesterday() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let summary = get_today_summary(&conn).unwrap();
    assert_eq!(summary.yesterday_sitting_secs, 0);
    assert_eq!(summary.yesterday_standing_secs, 0);
}

#[test]
fn test_aggregate_multiple_sitting_sessions() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}09:00:00Z", today), &format!("{}09:15:00Z", today), "Sitting", 900).unwrap();
    insert_session(&conn, &format!("{}10:00:00Z", today), &format!("{}10:20:00Z", today), "Sitting", 1200).unwrap();

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 3900, "sitting totals should aggregate correctly");
    assert_eq!(totals.standing_secs, 0, "no standing sessions should be 0");
    assert_eq!(totals.position_changes, 2, "3 rows - 1 = 2 transitions");
}

#[test]
fn test_aggregate_mixed_states() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}08:30:00Z", today), &format!("{}08:40:00Z", today), "Standing", 600).unwrap();
    insert_session(&conn, &format!("{}09:00:00Z", today), &format!("{}09:30:00Z", today), "Sitting", 1800).unwrap();

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 3600, "sitting total: 1800 + 1800");
    assert_eq!(totals.standing_secs, 600, "standing total");
    assert_eq!(totals.position_changes, 2, "3 rows - 1 = 2 transitions");
}

#[test]
fn test_away_excluded_from_standing_and_sitting() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}08:30:00Z", today), &format!("{}08:40:00Z", today), "Standing", 600).unwrap();
    insert_session(&conn, &format!("{}08:40:00Z", today), &format!("{}09:40:00Z", today), "Away", 3600).unwrap();
    insert_session(&conn, &format!("{}09:40:00Z", today), &format!("{}10:10:00Z", today), "Sitting", 1800).unwrap();

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 3600, "sitting: 1800 + 1800");
    assert_eq!(totals.standing_secs, 600, "standing: only Standing, not Away");
    // 3 desk-state rows (Sitting, Standing, Sitting) → 2 position changes
    // Away row excluded from position_changes count
    assert_eq!(totals.position_changes, 2, "Away doesn't count as position change");
}

#[test]
fn test_incomplete_sessions_ignored() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();

    // Insert an incomplete session (ended_at IS NULL)
    conn.execute(
        "INSERT INTO sessions (started_at, state) VALUES (?, ?)",
        rusqlite::params![format!("{}09:00:00Z", today), "Sitting"],
    ).unwrap();

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 1800, "incomplete sessions should be excluded");
    assert_eq!(totals.standing_secs, 0);
    assert_eq!(totals.position_changes, 0, "1 completed row - 1 = 0 transitions");
}

#[test]
fn test_schema_migration_adds_new_columns() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let result = init_schema(&conn);
    assert!(result.is_ok(), "second schema init should be idempotent");

    let mut stmt = conn.prepare("PRAGMA table_info(sessions)").unwrap();
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    assert!(columns.contains(&"sitting_seconds".to_string()));
    assert!(columns.contains(&"standing_seconds".to_string()));
    assert!(columns.contains(&"position_changes".to_string()));
}

#[test]
fn test_get_today_summary_returns_all_sessions() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}09:00:00Z", today), &format!("{}09:30:00Z", today), "Standing", 1800).unwrap();

    let summary = get_today_summary(&conn).unwrap();
    assert_eq!(summary.sessions.len(), 2, "should return all sessions");
    assert_eq!(summary.sitting_secs, 1800);
    assert_eq!(summary.standing_secs, 1800);
}

#[test]
fn test_today_summary_excludes_away_from_standing() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}08:30:00Z", today), &format!("{}08:40:00Z", today), "Standing", 600).unwrap();
    insert_session(&conn, &format!("{}08:40:00Z", today), &format!("{}09:40:00Z", today), "Away", 3600).unwrap();

    let summary = get_today_summary(&conn).unwrap();
    assert_eq!(summary.sitting_secs, 1800, "sitting: only Sitting rows");
    assert_eq!(summary.standing_secs, 600, "standing: only Standing, not Away");
    assert_eq!(summary.sessions.len(), 3, "all sessions returned including Away");
}

#[test]
fn test_totals_for_date_excludes_away() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let d = "2025-06-15";
    insert_session_with_date(&conn, "2025-06-15T08:00:00Z", "2025-06-15T08:30:00Z", "Sitting", 1800, d).unwrap();
    insert_session_with_date(&conn, "2025-06-15T08:30:00Z", "2025-06-15T08:40:00Z", "Standing", 600, d).unwrap();
    insert_session_with_date(&conn, "2025-06-15T08:40:00Z", "2025-06-15T09:40:00Z", "Away", 3600, d).unwrap();
    insert_session_with_date(&conn, "2025-06-15T09:40:00Z", "2025-06-15T10:10:00Z", "Sitting", 1800, d).unwrap();

    let (sitting, standing) = get_totals_for_date(&conn, d).unwrap();
    assert_eq!(sitting, 3600, "sitting: 1800 + 1800, Away excluded");
    assert_eq!(standing, 600, "standing: only Standing, not Away");
}

#[test]
fn test_completed_session_visible_in_summary() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();
    let started = format!("{}10:00:00Z", today);
    let ended = format!("{}10:45:00Z", today);

    insert_session(&conn, &started, &ended, "Sitting", 2700).unwrap();

    let summary = get_today_summary(&conn).unwrap();
    assert_eq!(summary.sessions.len(), 1, "completed session must appear in summary");
    assert_eq!(summary.sitting_secs, 2700, "sitting seconds must match");
    assert!(summary.sessions[0].ended_at.is_some(), "ended_at must be set");
}

#[test]
fn test_session_row_json_field_names() {
    let row = SessionRow {
        id: 42,
        started_at: "2025-03-16T09:00:00Z".to_string(),
        ended_at: Some("2025-03-16T09:30:00Z".to_string()),
        state: "Sitting".to_string(),
        duration_seconds: Some(1800),
    };
    let json = serde_json::to_value(&row).unwrap();
    assert!(json.get("id").is_none(), "id must be skipped in JSON");
    assert!(json.get("start").is_some(), "started_at must serialize as 'start'");
    assert!(json.get("end").is_some(), "ended_at must serialize as 'end'");
    assert!(json.get("duration_secs").is_some(), "duration_seconds must serialize as 'duration_secs'");
    assert!(json.get("started_at").is_none(), "raw field name must not appear");
    assert!(json.get("ended_at").is_none(), "raw field name must not appear");
    assert!(json.get("duration_seconds").is_none(), "raw field name must not appear");
}

// Multi-cycle round-trip test in db_tests_roundtrip.rs
// After-filter test in db_tests_roundtrip.rs
