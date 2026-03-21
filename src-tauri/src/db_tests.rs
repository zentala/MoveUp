//! db_tests.rs — Unit tests for the database layer.

use rusqlite::Connection;

use crate::db::init_schema;
use crate::db_sessions::{insert_session, load_today_totals};
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

    let (_sitting, _standing) = load_today_totals(&conn).expect("query should succeed");
}

#[test]
fn test_load_today_totals_empty() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let (sitting, standing) = load_today_totals(&conn).unwrap();
    assert_eq!(sitting, 0);
    assert_eq!(standing, 0);
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

    let (sitting, standing) = load_today_totals(&conn).unwrap();
    assert_eq!(sitting, 3900, "sitting totals should aggregate correctly");
    assert_eq!(standing, 0, "no standing sessions should be 0");
}

#[test]
fn test_aggregate_mixed_states() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    insert_session(&conn, &format!("{}08:00:00Z", today), &format!("{}08:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}08:30:00Z", today), &format!("{}08:40:00Z", today), "Standing", 600).unwrap();
    insert_session(&conn, &format!("{}09:00:00Z", today), &format!("{}09:30:00Z", today), "Sitting", 1800).unwrap();

    let (sitting, standing) = load_today_totals(&conn).unwrap();
    assert_eq!(sitting, 3600, "sitting total: 1800 + 1800");
    assert_eq!(standing, 600, "standing total");
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

    let (sitting, standing) = load_today_totals(&conn).unwrap();
    assert_eq!(sitting, 1800, "incomplete sessions should be excluded");
    assert_eq!(standing, 0);
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
