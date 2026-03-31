//! db_tests_roundtrip.rs — Multi-cycle DB persistence round-trip tests.

use rusqlite::Connection;

use crate::db::init_schema;
use crate::db_sessions::insert_session;
use crate::db_sessions::load_today_totals;
use crate::db_queries::get_today_summary;

fn test_conn() -> Connection {
    Connection::open_in_memory().unwrap()
}

/// Multi-cycle DB round-trip: insert sessions from a realistic day,
/// then verify load_today_totals and get_today_summary return correct data.
#[test]
fn test_multicycle_db_roundtrip() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    // Sit 25m → Stand 8m → Away 12m → Sit 35m → Stand 5m → Sit
    let sessions = [
        ("08:00:00Z", "08:25:00Z", "Sitting",  1500),
        ("08:25:00Z", "08:33:00Z", "Standing",  480),
        ("08:33:00Z", "08:45:00Z", "Away",      720),
        ("08:45:00Z", "09:20:00Z", "Sitting",  2100),
        ("09:20:00Z", "09:25:00Z", "Standing",  300),
        ("09:25:00Z", "09:25:01Z", "Sitting",     1),
    ];

    for (start, end, state, dur) in &sessions {
        insert_session(
            &conn,
            &format!("{}{}", today, start),
            &format!("{}{}", today, end),
            state,
            *dur,
        ).unwrap();
    }

    let totals = load_today_totals(&conn, None).unwrap();
    assert_eq!(totals.sitting_secs, 3601, "sitting: 1500+2100+1");
    assert_eq!(totals.standing_secs, 780, "standing: 480+300");
    assert_eq!(totals.position_changes, 4, "4 transitions between Sit/Stand rows");

    let summary = get_today_summary(&conn).unwrap();
    assert_eq!(summary.sessions.len(), 6, "all 6 sessions must survive DB round-trip");
    assert_eq!(summary.sitting_secs, 3601);
    assert_eq!(summary.standing_secs, 780);

    for i in 1..summary.sessions.len() {
        assert!(
            summary.sessions[i].started_at >= summary.sessions[i - 1].started_at,
            "sessions must be chronological",
        );
    }

    for (i, s) in summary.sessions.iter().enumerate() {
        assert!(
            s.duration_seconds.unwrap_or(0) > 0 || i == 5,
            "session {} must have positive duration", i
        );
    }
}

/// Verifies that `load_today_totals` with `after` filter excludes pre-reset sessions.
#[test]
fn test_load_today_totals_with_after_filter() {
    let conn = test_conn();
    init_schema(&conn).unwrap();

    let today = chrono::Local::now().format("%Y-%m-%dT").to_string();

    // Pre-reset sessions
    insert_session(&conn, &format!("{}01:00:00Z", today), &format!("{}01:30:00Z", today), "Sitting", 1800).unwrap();
    insert_session(&conn, &format!("{}02:00:00Z", today), &format!("{}02:15:00Z", today), "Standing", 900).unwrap();

    // Post-reset sessions
    insert_session(&conn, &format!("{}05:00:00Z", today), &format!("{}05:10:00Z", today), "Sitting", 600).unwrap();
    insert_session(&conn, &format!("{}05:10:00Z", today), &format!("{}05:20:00Z", today), "Standing", 600).unwrap();

    // Without filter: all sessions
    let all = load_today_totals(&conn, None).unwrap();
    assert_eq!(all.sitting_secs, 2400, "all sitting: 1800+600");
    assert_eq!(all.standing_secs, 1500, "all standing: 900+600");

    // With filter at 04:00 — only post-reset sessions
    let after_reset = format!("{}04:00:00Z", today);
    let filtered = load_today_totals(&conn, Some(&after_reset)).unwrap();
    assert_eq!(filtered.sitting_secs, 600, "filtered sitting: 600 only");
    assert_eq!(filtered.standing_secs, 600, "filtered standing: 600 only");
    assert_eq!(filtered.position_changes, 1, "filtered: 2 desk-state rows - 1");
}
