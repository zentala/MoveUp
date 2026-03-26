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

    let totals = load_today_totals(&conn).unwrap();
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
