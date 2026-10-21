//! db_queries.rs — Summary and aggregate queries for the database.
//!
//! Provides today's summary (sessions + aggregate times + yesterday comparison).

use log::error;
use rusqlite::Connection;

use crate::db::{SessionRow, TodaySummary};
use crate::db_sessions::{accumulate_state_duration, get_yesterday_totals, map_session_row,
    local_date_bounds, clipped_duration};

/// Returns today's complete summary including all sessions and aggregate times.
pub fn get_today_summary(conn: &Connection) -> Result<TodaySummary, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let sessions = get_sessions_range(conn, &today, &today)?;
    let (start, end) = local_date_bounds(&chrono::Local, &today, &today)?;

    let mut sitting_secs = 0i64;
    let mut standing_secs = 0i64;

    for session in &sessions {
        if let Some(duration) = clipped_duration(session, start, end) {
            accumulate_state_duration(&session.state, duration, &mut sitting_secs, &mut standing_secs);
        }
    }

    let (yesterday_sitting_secs, yesterday_standing_secs) =
        get_yesterday_totals(conn)?;

    Ok(TodaySummary {
        sitting_secs,
        standing_secs,
        yesterday_sitting_secs,
        yesterday_standing_secs,
        position_changes: 0, // Will be set by caller from SessionManager
        sessions,
    })
}

/// Returns completed session rows across the `[from, to]` inclusive local-date
/// range, ordered by `started_at`. Both bounds are `YYYY-MM-DD`.
pub fn get_sessions_range(
    conn: &Connection,
    from: &str,
    to: &str,
) -> Result<Vec<SessionRow>, String> {
    let (start, end) = local_date_bounds(&chrono::Local, from, to)?;
    get_sessions_between(conn, start, end)
}

/// Query by instants, leaving rows intact for the consumer's local-day clipping.
pub(crate) fn get_sessions_between(conn: &Connection,
    start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>)
    -> Result<Vec<SessionRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, ended_at, state, duration_seconds, break_credit FROM sessions \
             WHERE julianday(started_at) < julianday(?2) \
             AND julianday(ended_at) > julianday(?1) \
             AND ended_at IS NOT NULL \
             ORDER BY julianday(started_at), id",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare range query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map(rusqlite::params![start.to_rfc3339(), end.to_rfc3339()], map_session_row)
        .map_err(|e| {
            let msg = format!("Failed to query sessions range: {}", e);
            error!("{}", msg);
            msg
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            let msg = format!("Failed to collect rows: {}", e);
            error!("{}", msg);
            msg
        })?;
    Ok(rows)
}
