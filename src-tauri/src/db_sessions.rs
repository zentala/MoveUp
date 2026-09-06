//! db_sessions.rs — Session CRUD operations for SQLite persistence.
//!
//! Provides insert and load functions for session state tracking.

use log::error;
use rusqlite::Connection;

use crate::db::{break_credit_to_db_str, SessionRow};
use crate::session_types::{BreakCredit, DeskState};

// ─── Insert operations ───────────────────────────────────────────────────────

/// Inserts a completed session into the database with no break credit recorded.
/// `date_local` is set to the LOCAL date at time of insert (timezone-safe queries).
/// This means the date reflects the user's timezone when the session happened.
pub fn insert_session(
    conn: &Connection,
    started_at: &str,
    ended_at: &str,
    state: &str,
    duration_seconds: i64,
) -> Result<(), rusqlite::Error> {
    insert_session_with_credit(conn, started_at, ended_at, state, duration_seconds, None)
}

/// Inserts a completed session, recording the break credit it earned (ADR 008).
///
/// `break_credit` is `None` for spans where credit does not apply (a sitting
/// span, a session flushed on shutdown) — the column then stays NULL, meaning
/// "not recorded" rather than "earned nothing".
pub fn insert_session_with_credit(
    conn: &Connection,
    started_at: &str,
    ended_at: &str,
    state: &str,
    duration_seconds: i64,
    break_credit: Option<&BreakCredit>,
) -> Result<(), rusqlite::Error> {
    let date_local = chrono::Local::now().format("%Y-%m-%d").to_string();
    insert_session_with_date_and_credit(
        conn,
        started_at,
        ended_at,
        state,
        duration_seconds,
        &date_local,
        break_credit,
    )
}

/// Insert with explicit date_local (used by tests and migration).
pub fn insert_session_with_date(
    conn: &Connection,
    started_at: &str,
    ended_at: &str,
    state: &str,
    duration_seconds: i64,
    date_local: &str,
) -> Result<(), rusqlite::Error> {
    insert_session_with_date_and_credit(
        conn,
        started_at,
        ended_at,
        state,
        duration_seconds,
        date_local,
        None,
    )
}

/// Insert with explicit date_local and break credit.
pub fn insert_session_with_date_and_credit(
    conn: &Connection,
    started_at: &str,
    ended_at: &str,
    state: &str,
    duration_seconds: i64,
    date_local: &str,
    break_credit: Option<&BreakCredit>,
) -> Result<(), rusqlite::Error> {
    let credit = break_credit.map(break_credit_to_db_str);
    conn.execute(
        "INSERT INTO sessions (started_at, ended_at, state, duration_seconds, date_local, break_credit) \
         VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![started_at, ended_at, state, duration_seconds, date_local, credit],
    )?;
    Ok(())
}


// ─── Load operations ─────────────────────────────────────────────────────────

/// Today's totals restored from database on restart.
pub struct TodayTotals {
    pub sitting_secs: i64,
    pub standing_secs: i64,
    /// Number of desk position changes (excludes Away transitions).
    pub position_changes: u32,
}

impl TodayTotals {
    /// Creates totals for testing (position_changes defaults to 0).
    #[cfg(test)]
    pub fn from_secs(sitting: i64, standing: i64) -> Self {
        Self { sitting_secs: sitting, standing_secs: standing, position_changes: 0 }
    }
}

/// Accumulates duration into sitting or standing based on state string.
/// Returns `true` if state is a desk position (counted for position changes).
pub fn accumulate_state_duration(
    state_str: &str,
    duration: i64,
    sitting: &mut i64,
    standing: &mut i64,
) -> bool {
    match DeskState::from_db_str(state_str) {
        Some(DeskState::Sitting) => { *sitting += duration; true }
        Some(s) if s.is_standing_like() => { *standing += duration; true }
        _ => false, // Away/Walking-without-standing/unknown: excluded
    }
}

/// Loads today's total sitting and standing seconds from the database.
/// Uses `date_local` column for timezone-safe queries. Falls back to
/// `started_at LIKE` for rows without `date_local` (pre-migration).
///
/// `after` — optional ISO timestamp. If set, only sessions started AFTER
/// this time are counted. Used to exclude pre-daily-reset sessions.
pub fn load_today_totals(conn: &Connection, after: Option<&str>) -> Result<TodayTotals, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    // Use a single query with started_at > ?3. When no after filter,
    // pass empty string "" which is less than any ISO timestamp.
    let after_val = after.unwrap_or("");

    let mut stmt = conn
        .prepare(
            "SELECT state, duration_seconds FROM sessions \
             WHERE (date_local = ?1 OR (date_local IS NULL AND started_at LIKE ?2)) \
             AND ended_at IS NOT NULL \
             AND started_at > ?3",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map(
            rusqlite::params![&today, format!("{}%", today), after_val],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| {
            let msg = format!("Failed to query today's totals: {}", e);
            error!("{}", msg);
            msg
        })?;

    let mut sitting_secs = 0i64;
    let mut standing_secs = 0i64;
    let mut desk_state_rows = 0u32;

    for row_result in rows {
        let (state, duration) = row_result.map_err(|e| {
            let msg = format!("Failed to read row: {}", e);
            error!("{}", msg);
            msg
        })?;

        if accumulate_state_duration(&state, duration, &mut sitting_secs, &mut standing_secs) {
            desk_state_rows += 1;
        }
    }

    // position_changes = transitions between desk states (Sitting↔Standing).
    // Away transitions are excluded — leaving the desk isn't a position change.
    let position_changes = desk_state_rows.saturating_sub(1);

    Ok(TodayTotals { sitting_secs, standing_secs, position_changes })
}

/// Loads sitting and standing totals for a given local date (YYYY-MM-DD).
/// Away/unknown states are excluded from both totals.
pub fn get_totals_for_date(conn: &Connection, date: &str) -> Result<(i64, i64), String> {
    let mut stmt = conn
        .prepare(
            "SELECT state, duration_seconds FROM sessions \
             WHERE (date_local = ?1 OR (date_local IS NULL AND started_at LIKE ?2)) \
             AND ended_at IS NOT NULL",
        )
        .map_err(|e| {
            let msg = format!("Failed to prepare query: {}", e);
            error!("{}", msg);
            msg
        })?;

    let rows = stmt
        .query_map(rusqlite::params![date, format!("{}%", date)], |row| {
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

        accumulate_state_duration(&state, duration, &mut sitting_secs, &mut standing_secs);
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
///
/// Expects the column order used by every `sessions` SELECT in `db_queries.rs`:
/// `id, started_at, ended_at, state, duration_seconds, break_credit`.
pub fn map_session_row(row: &rusqlite::Row) -> rusqlite::Result<SessionRow> {
    let credit: Option<String> = row.get(5)?;
    Ok(SessionRow {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        state: row.get(3)?,
        duration_seconds: row.get(4)?,
        break_credit: credit.as_deref().and_then(crate::db::break_credit_from_db_str),
    })
}
