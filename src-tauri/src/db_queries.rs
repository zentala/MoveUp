//! db_queries.rs — Summary and aggregate queries for the database.
//!
//! Provides today's summary (sessions + aggregate times + yesterday comparison).

use log::error;
use rusqlite::Connection;

use crate::db::TodaySummary;
use crate::db_sessions::{get_yesterday_totals, map_session_row};

/// Returns today's complete summary including all sessions and aggregate times.
pub fn get_today_summary(conn: &Connection) -> Result<TodaySummary, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT id, started_at, ended_at, state, duration_seconds FROM sessions WHERE started_at LIKE ? AND ended_at IS NOT NULL ORDER BY started_at",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let sessions = stmt
        .query_map(rusqlite::params![format!("{}%", today)], |row| {
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
            match session.state.as_str() {
                "Sitting" => sitting_secs += duration,
                "Standing" | "Walking" => standing_secs += duration,
                _ => {} // Away/unknown: excluded
            }
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
