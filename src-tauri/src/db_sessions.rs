//! db_sessions.rs — Session CRUD operations for SQLite persistence.
//!
//! Provides insert and load functions for session state tracking.

use log::error;
use rusqlite::Connection;

use crate::db::SessionRow;

// ─── Insert operations ───────────────────────────────────────────────────────

/// Inserts a completed sitting session into the database.
/// Returns an error if the insert fails.
pub fn insert_session(
    conn: &Connection,
    started_at: &str,
    ended_at: &str,
    state: &str,
    duration_seconds: i64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sessions (started_at, ended_at, state, duration_seconds) VALUES (?, ?, ?, ?)",
        rusqlite::params![started_at, ended_at, state, duration_seconds],
    )?;
    Ok(())
}

/// Persists a state change event to the database (for session history).
/// Called whenever SessionManager emits a state change to ensure durable state tracking.
pub fn save_session_state(
    conn: &Connection,
    state_change: &crate::session::StateChangedPayload,
) -> Result<(), String> {
    let now = chrono::Local::now().to_rfc3339();
    let state_str = format!("{:?}", state_change.state);

    conn.execute(
        "INSERT INTO sessions (started_at, state, duration_seconds, sitting_seconds, standing_seconds, position_changes, session_limit_secs) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            &now,
            state_str,
            state_change.break_seconds,
            state_change.sitting_seconds,
            state_change.standing_seconds,
            state_change.position_changes,
            0
        ],
    )
    .map_err(|e| format!("Failed to save session state: {}", e))?;

    Ok(())
}

// ─── Load operations ─────────────────────────────────────────────────────────

/// Loads today's total sitting and standing seconds from the database.
/// Returns (sitting_secs, standing_secs) or an error.
pub fn load_today_totals(conn: &Connection) -> Result<(i64, i64), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn
        .prepare(
            "SELECT state, duration_seconds FROM sessions WHERE started_at LIKE ? AND ended_at IS NOT NULL",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map(rusqlite::params![format!("{}%", today)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| {
            let msg = format!("Failed to query today's totals: {}", e);
            error!("{}", msg);
            msg
        })?;

    let mut sitting_secs = 0i64;
    let mut standing_secs = 0i64;

    for row_result in rows {
        let (state, duration) = row_result.map_err(|e| {
            let msg = format!("Failed to read row: {}", e);
            error!("{}", msg);
            msg
        })?;

        if state == "Sitting" {
            sitting_secs += duration;
        } else {
            standing_secs += duration;
        }
    }

    Ok((sitting_secs, standing_secs))
}

/// Loads yesterday's total sitting and standing seconds from the database.
/// Returns (sitting_secs, standing_secs) or an error.
/// Returns (0, 0) if no sessions exist for yesterday.
pub fn get_yesterday_totals(conn: &Connection) -> Result<(i64, i64), String> {
    let mut stmt = conn
        .prepare(
            "SELECT state, duration_seconds FROM sessions WHERE date(started_at) = date('now', '-1 day') AND ended_at IS NOT NULL",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| {
            let msg = format!("Failed to query yesterday's totals: {}", e);
            error!("{}", msg);
            msg
        })?;

    let (mut sitting_secs, mut standing_secs) = (0i64, 0i64);

    for row_result in rows {
        let (state, duration) = row_result.map_err(|e| {
            let msg = format!("Failed to read row: {}", e);
            error!("{}", msg);
            msg
        })?;

        if state == "Sitting" {
            sitting_secs += duration;
        } else {
            standing_secs += duration;
        }
    }

    Ok((sitting_secs, standing_secs))
}

/// Maps a database row to a [`SessionRow`].
pub fn map_session_row(row: &rusqlite::Row) -> rusqlite::Result<SessionRow> {
    Ok(SessionRow {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        state: row.get(3)?,
        duration_seconds: row.get(4)?,
    })
}
