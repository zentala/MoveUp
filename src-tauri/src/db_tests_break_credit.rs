//! db_tests_break_credit.rs — E015-T03 tests.
//!
//! Two seams closed by this task:
//! - `sessions.break_credit` persists the credit the engine applied, so the
//!   Analyst reads it instead of guessing from a session's duration.
//! - PostureBalance compares raw sitting against raw standing, so it can fire
//!   for a user who takes breaks.

use rusqlite::Connection;

use crate::communication_profile::CommunicationProfile;
use crate::db::{break_credit_from_db_str, break_credit_to_db_str, init_schema, SessionRow};
use crate::db_queries::get_sessions_range;
use crate::db_sessions::{insert_session, insert_session_with_date_and_credit};
use crate::session_manager::SessionManager;
use crate::session_types::{BreakCredit, DeskState, NotificationEvent};

fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    conn
}

const DAY: &str = "2026-09-06";

fn rows_for_day(conn: &Connection) -> Vec<SessionRow> {
    get_sessions_range(conn, DAY, DAY).unwrap()
}

// ─── break_credit column ─────────────────────────────────────────────────────

#[test]
fn e015_break_credit_persists_applied_credit() {
    let conn = test_conn();
    insert_session_with_date_and_credit(
        &conn,
        "2026-09-06T09:00:00Z",
        "2026-09-06T09:10:00Z",
        "Standing",
        600,
        DAY,
        Some(&BreakCredit::Partial),
    )
    .unwrap();

    let rows = rows_for_day(&conn);
    assert_eq!(rows.len(), 1, "the inserted break session must come back");
    assert_eq!(
        rows[0].break_credit,
        Some(BreakCredit::Partial),
        "the credit the engine applied must survive the round trip"
    );
}

#[test]
fn e015_break_credit_unknown_stays_null_not_none() {
    let conn = test_conn();
    // A sitting span: no credit applies, so nothing is recorded.
    insert_session(
        &conn,
        "2026-09-06T08:00:00Z",
        "2026-09-06T08:45:00Z",
        "Sitting",
        2700,
    )
    .unwrap();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let rows = get_sessions_range(&conn, &today, &today).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].break_credit, None,
        "an unrecorded credit must read as unknown, never as BreakCredit::None"
    );
}

#[test]
fn e015_break_credit_migration_adds_column_to_old_db() {
    // A database created before the column existed.
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE sessions (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at       TEXT NOT NULL,
            ended_at         TEXT,
            state            TEXT NOT NULL,
            duration_seconds INTEGER
        )",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO sessions (started_at, ended_at, state, duration_seconds) \
         VALUES ('2026-09-06T09:00:00Z', '2026-09-06T09:10:00Z', 'Standing', 600)",
        [],
    )
    .unwrap();

    init_schema(&conn).unwrap();

    let rows = rows_for_day(&conn);
    assert_eq!(rows.len(), 1, "the pre-migration row must still be readable");
    assert_eq!(
        rows[0].break_credit, None,
        "a row written before the column existed has an unknown credit"
    );

    // And a new row written after the migration records its credit.
    insert_session_with_date_and_credit(
        &conn,
        "2026-09-06T10:00:00Z",
        "2026-09-06T10:20:00Z",
        "Standing",
        1200,
        DAY,
        Some(&BreakCredit::Full),
    )
    .unwrap();
    let rows = rows_for_day(&conn);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].break_credit, Some(BreakCredit::Full));
}

#[test]
fn e015_break_credit_db_str_round_trips() {
    for credit in [BreakCredit::None, BreakCredit::Partial, BreakCredit::Full] {
        let raw = break_credit_to_db_str(&credit);
        assert_eq!(break_credit_from_db_str(raw), Some(credit));
    }
    assert_eq!(
        break_credit_from_db_str("something-else"),
        None,
        "an unknown value is unknown, not 'none'"
    );
}

#[test]
fn e015_break_credit_reaches_the_wire() {
    let conn = test_conn();
    insert_session_with_date_and_credit(
        &conn,
        "2026-09-06T09:00:00Z",
        "2026-09-06T09:10:00Z",
        "Standing",
        600,
        DAY,
        Some(&BreakCredit::Full),
    )
    .unwrap();

    let json = serde_json::to_value(&rows_for_day(&conn)[0]).unwrap();
    assert_eq!(
        json.get("break_credit").and_then(|v| v.as_str()),
        Some("full"),
        "the Analyst must receive the recorded credit, not derive it"
    );
}

#[test]
fn test_session_row_json_field_names() {
    let row = SessionRow {
        id: 42,
        started_at: "2025-03-16T09:00:00Z".to_string(),
        ended_at: Some("2025-03-16T09:30:00Z".to_string()),
        state: "Sitting".to_string(),
        duration_seconds: Some(1800),
        break_credit: None,
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

// ─── PostureBalance: raw vs raw ──────────────────────────────────────────────

fn posture_profile() -> CommunicationProfile {
    let mut comm = CommunicationProfile::default();
    comm.periodic_notifications.posture_balance_enabled = true;
    comm
}

#[test]
fn e015_posture_balance_fires_for_user_who_takes_breaks() {
    let mut m = SessionManager::new();
    m.state.state = DeskState::Sitting;
    // 7h of sitting and 1h of standing today — clearly unbalanced.
    m.state.sitting_seconds_total = 7 * 3600;
    m.state.standing_seconds = 3600;
    // Breaks have credited the session counter down to nearly nothing. Before
    // E015-T03 this alone suppressed the notification.
    m.state.sitting_seconds = 300;

    let events = m.check_notification_conditions(&posture_profile());
    assert!(
        events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
        "PostureBalance must compare raw sitting with raw standing"
    );
}

#[test]
fn e015_posture_balance_silent_when_balance_is_good() {
    let mut m = SessionManager::new();
    m.state.state = DeskState::Sitting;
    // 7h sitting against 4h standing — not more than twice as much.
    m.state.sitting_seconds_total = 7 * 3600;
    m.state.standing_seconds = 4 * 3600;
    m.state.sitting_seconds = 7 * 3600;

    let events = m.check_notification_conditions(&posture_profile());
    assert!(
        !events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
        "a balanced day must not nag"
    );
}
