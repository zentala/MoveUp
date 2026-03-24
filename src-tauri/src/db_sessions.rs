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


// ─── Load operations ─────────────────────────────────────────────────────────

/// Today's totals restored from database on restart.
pub struct TodayTotals {
    pub sitting_secs: i64,
    pub standing_secs: i64,
    pub away_secs: i64,
    /// Number of desk position changes (excludes Away transitions).
    pub position_changes: u32,
}

impl TodayTotals {
    /// Creates totals for testing (away_secs and position_changes default to 0).
    #[cfg(test)]
    pub fn from_secs(sitting: i64, standing: i64) -> Self {
        Self { sitting_secs: sitting, standing_secs: standing, away_secs: 0, position_changes: 0 }
    }
}

/// Loads today's total sitting and standing seconds from the database.
pub fn load_today_totals(conn: &Connection) -> Result<TodayTotals, String> {
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
    let mut away_secs = 0i64;
    let mut desk_state_rows = 0u32;

    for row_result in rows {
        let (state, duration) = row_result.map_err(|e| {
            let msg = format!("Failed to read row: {}", e);
            error!("{}", msg);
            msg
        })?;

        match state.as_str() {
            "Sitting" => sitting_secs += duration,
            "Standing" | "Walking" => standing_secs += duration,
            _ => { away_secs += duration; continue; }
        }
        desk_state_rows += 1;
    }

    // position_changes = transitions between desk states (Sitting↔Standing).
    // Away transitions are excluded — leaving the desk isn't a position change.
    let position_changes = desk_state_rows.saturating_sub(1);

    Ok(TodayTotals { sitting_secs, standing_secs, away_secs, position_changes })
}

/// Loads sitting and standing totals for a given date (YYYY-MM-DD).
/// Away/unknown states are excluded from both totals.
pub fn get_totals_for_date(conn: &Connection, date: &str) -> Result<(i64, i64), String> {
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
        .query_map(rusqlite::params![format!("{}%", date)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| {
            let msg = format!("Failed to query totals for {}: {}", date, e);
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

        match state.as_str() {
            "Sitting" => sitting_secs += duration,
            "Standing" | "Walking" => standing_secs += duration,
            _ => {} // Away/unknown: excluded from both totals
        }
    }

    Ok((sitting_secs, standing_secs))
}

/// Loads yesterday's totals. Convenience wrapper around [`get_totals_for_date`].
pub fn get_yesterday_totals(conn: &Connection) -> Result<(i64, i64), String> {
    let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    get_totals_for_date(conn, &yesterday)
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
