//! db_queries.rs — Summary and aggregate queries for the database.
//!
//! Provides today's summary (sessions + aggregate times + yesterday comparison).

use log::error;
use rusqlite::Connection;

use crate::db::{SessionRow, TodaySummary};
use crate::db_sessions::{accumulate_state_duration, get_yesterday_totals, map_session_row};

/// Returns today's complete summary including all sessions and aggregate times.
pub fn get_today_summary(conn: &Connection) -> Result<TodaySummary, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, ended_at, state, duration_seconds, break_credit FROM sessions \
             WHERE (date_local = ?1 OR (date_local IS NULL AND started_at LIKE ?2)) \
             AND ended_at IS NOT NULL ORDER BY started_at",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let sessions = stmt
        .query_map(rusqlite::params![&today, format!("{}%", today)], |row| {
            map_session_row(row)
        })
        .map_err(|e| {
            let msg = format!("Failed to query sessions: {}", e);
            error!("{}", msg);
            msg
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            let msg = format!("Failed to collect rows: {}", e);
            error!("{}", msg);
            msg
        })?;

    let mut sitting_secs = 0i64;
    let mut standing_secs = 0i64;

    for session in &sessions {
        if let Some(duration) = session.duration_seconds {
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
    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, ended_at, state, duration_seconds, break_credit FROM sessions \
             WHERE date_local BETWEEN ?1 AND ?2 \
             AND ended_at IS NOT NULL \
             ORDER BY started_at",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare range query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map(rusqlite::params![from, to], map_session_row)
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
