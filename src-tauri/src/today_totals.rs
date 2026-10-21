//! today_totals.rs — the single composition point for "today's totals".
//!
//! Two stores hold a piece of today's truth and neither holds all of it:
//!
//! - **SQLite** (`db_queries::get_today_summary`) knows every *completed*
//!   session, so it owns the durations and the session list.
//! - **The in-memory `SessionManager`** knows the span that is still running,
//!   so it owns `position_changes` — the DB count lags by the open session.
//!
//! Every caller that needs a `TodaySummary` for display goes through
//! [`load_today_summary`]. Composing the two stores by hand at each call site
//! is what let `ensure_initialized` seed the cache with `position_changes: 0`
//! while `get_today_summary` returned the live value for the same moment.
//!
//! Precedence rule and its rationale: `.arch/ADR/014-persistence-precedence.md`.

use rusqlite::Connection;

use crate::db::TodaySummary;

/// Loads today's summary from SQLite and overlays the live session counters.
///
/// `live_position_changes` comes from `SessionManager::snapshot().position_changes`.
/// The database value for that field is always discarded — see the module docs.
pub fn load_today_summary(
    conn: &Connection,
    live_position_changes: u32,
) -> Result<TodaySummary, String> {
    let mut summary = crate::db::get_today_summary(conn)?;
    summary.position_changes = live_position_changes;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;
    use crate::db::init_schema;
    use crate::db_sessions::{insert_session_with_date, insert_session};

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn local_noon_today() -> chrono::DateTime<chrono::Utc> {
        // Production summary owns the real "today" clock, so these tests still
        // depend on the host date. Capture one basis and use local noon to keep
        // every endpoint in that day, away from midnight and DST transitions.
        let day = chrono::Local::now().date_naive();
        chrono::Local.from_local_datetime(&day.and_hms_opt(12, 0, 0).unwrap())
            .single().unwrap().with_timezone(&chrono::Utc)
    }

    /// The live counter wins over the zero the DB query hard-codes.
    #[test]
    fn e019_t02_today_totals_live_position_changes_wins() {
        let conn = test_conn();
        let start = local_noon_today();
        insert_session(&conn, &start.to_rfc3339(),
            &(start + chrono::Duration::minutes(30)).to_rfc3339(), "Sitting", 1800)
            .unwrap();

        let from_db = crate::db::get_today_summary(&conn).unwrap();
        assert_eq!(from_db.position_changes, 0, "db query never fills this field");

        let composed = load_today_summary(&conn, 7).unwrap();
        assert_eq!(composed.position_changes, 7);
    }

    /// Durations and the session list still come from the database untouched.
    #[test]
    fn e019_t02_today_totals_durations_come_from_db() {
        let conn = test_conn();
        let start = local_noon_today();
        let sitting_end = start + chrono::Duration::minutes(30);
        let standing_end = sitting_end + chrono::Duration::minutes(10);
        let day = start.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
        insert_session_with_date(
            &conn, &start.to_rfc3339(), &sitting_end.to_rfc3339(), "Sitting", 1800, &day,
        )
        .unwrap();
        insert_session_with_date(
            &conn, &sitting_end.to_rfc3339(), &standing_end.to_rfc3339(), "Standing", 600, &day,
        )
        .unwrap();

        let summary = load_today_summary(&conn, 3).unwrap();
        assert_eq!(summary.sitting_secs, 1800);
        assert_eq!(summary.standing_secs, 600);
        assert_eq!(summary.sessions.len(), 2);
        assert_eq!(summary.position_changes, 3);
    }

    /// Empty day: zeros everywhere, but the live counter is still honoured.
    #[test]
    fn e019_t02_today_totals_empty_day() {
        let conn = test_conn();

        let summary = load_today_summary(&conn, 0).unwrap();
        assert_eq!(summary.sitting_secs, 0);
        assert_eq!(summary.standing_secs, 0);
        assert!(summary.sessions.is_empty());
        assert_eq!(summary.position_changes, 0);
    }

    /// A broken database surfaces as an error, never as a zeroed summary —
    /// "no data" must not be indistinguishable from "a day with no sessions".
    #[test]
    fn e019_t02_today_totals_db_error_propagates() {
        let conn = Connection::open_in_memory().unwrap(); // no schema
        let result = load_today_summary(&conn, 5);
        assert!(result.is_err(), "missing schema must be an Err, not a default summary");
    }
}
