//! db_sessions.rs — Session CRUD operations for SQLite persistence.
//!
//! Provides insert and load functions for session state tracking.

use rusqlite::Connection;

use crate::db::{break_credit_to_db_str, SessionRow};
use crate::session_types::{BreakCredit, DeskState};

// ─── Insert operations ───────────────────────────────────────────────────────

/// Inserts a completed session into the database with no break credit recorded.
/// `date_local` is derived from the span start, never the save clock.
/// Range queries use intersection so crossings appear in both days.
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
    let date_local = chrono::DateTime::parse_from_rfc3339(started_at)
        .map_err(|_| rusqlite::Error::InvalidParameterName("invalid session start".into()))?
        .with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
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


/// Idempotent adapter insert. One SQLite statement holds the write lock for
/// existence check and insert; replay needs no new index or schema migration.
pub(crate) fn insert_session_once(conn: &Connection, started_at: &str, ended_at: &str,
    state: &str, duration_seconds: i64, break_credit: Option<&BreakCredit>)
    -> Result<Option<i64>, rusqlite::Error> {
    let date = chrono::DateTime::parse_from_rfc3339(started_at)
        .map_err(|_| rusqlite::Error::InvalidParameterName("invalid session start".into()))?
        .with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
    let credit = break_credit.map(break_credit_to_db_str);
    let inserted = conn.execute("INSERT INTO sessions
        (started_at, ended_at, state, duration_seconds, date_local, break_credit, history_kind)
        SELECT ?1, ?2, ?3, ?4, ?5, ?6, 'individual' WHERE NOT EXISTS
        (SELECT 1 FROM sessions WHERE started_at = ?1 AND ended_at = ?2
          AND state = ?3 AND duration_seconds = ?4 AND break_credit IS ?6)",
        rusqlite::params![started_at, ended_at, state, duration_seconds, date, credit])?;
    Ok((inserted == 1).then(|| conn.last_insert_rowid()))
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

/// Loads raw totals clipped to today's local calendar boundaries. An explicit
/// reset cuts the range at its instant, including the tail of a crossing span.
pub fn load_today_totals(conn: &Connection, after: Option<&str>) -> Result<TodayTotals, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let (mut start, end) = local_date_bounds(&chrono::Local, &today, &today)?;
    if let Some(after) = after {
        start = start.max(chrono::DateTime::parse_from_rfc3339(after)
            .map_err(|e| format!("Invalid daily reset instant: {e}"))?
            .with_timezone(&chrono::Utc));
    }
    let rows = crate::db_queries::get_sessions_between(conn, start, end)?;

    let mut sitting_secs = 0i64;
    let mut standing_secs = 0i64;
    let mut desk_state_rows = 0u32;

    for row in rows {
        if let Some(duration) = clipped_duration(&row, start, end) {
            if accumulate_state_duration(&row.state, duration, &mut sitting_secs, &mut standing_secs) {
                desk_state_rows += 1;
            }
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
    get_totals_for_date_in_zone(conn, &chrono::Local, date)
}

/// Loads one local date's totals using an explicit zone, so DST boundaries can
/// be tested without changing the process timezone.
pub(crate) fn get_totals_for_date_in_zone<Tz: chrono::TimeZone>(
    conn: &Connection,
    zone: &Tz,
    date: &str,
) -> Result<(i64, i64), String> {
    let (start, end) = local_date_bounds(zone, date, date)?;
    let rows = crate::db_queries::get_sessions_between(conn, start, end)?;

    let (mut sitting_secs, mut standing_secs) = (0i64, 0i64);

    for row in rows {
        if let Some(duration) = clipped_duration(&row, start, end) {
            accumulate_state_duration(&row.state, duration, &mut sitting_secs, &mut standing_secs);
        }
    }

    Ok((sitting_secs, standing_secs))
}

/// Loads yesterday's totals. Convenience wrapper around [`get_totals_for_date`].
pub fn get_yesterday_totals(conn: &Connection) -> Result<(i64, i64), String> {
    let yesterday = chrono::Local::now().date_naive().pred_opt().ok_or("Invalid prior date")?
        .format("%Y-%m-%d")
        .to_string();
    get_totals_for_date(conn, &yesterday)
}

/// Calendar arithmetic, including 23/25-hour DST days and days whose first
/// local wall-clock instant is missing or ambiguous.
pub(crate) fn local_date_bounds<T: chrono::TimeZone>(zone: &T, from: &str, to: &str)
    -> Result<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>), String> {
    let from = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map_err(|e| format!("Invalid range start: {e}"))?;
    let to = chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map_err(|e| format!("Invalid range end: {e}"))?;
    if from > to { return Err("Range start is after end".into()); }
    let end = to.succ_opt().ok_or("Range end overflow")?;
    let start = crate::session_daily::local_day_start_utc(zone, from)
        .ok_or("Unresolvable local start of day")?;
    let end = crate::session_daily::local_day_start_utc(zone, end)
        .ok_or("Unresolvable local end of day")?;
    Ok((start, end))
}

/// Limit daily raw totals to the intersection without changing stored rows.
/// Mismatched legacy duration/endpoint rows cannot be safely divided by day.
pub(crate) fn clipped_duration(row: &SessionRow, start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>) -> Option<i64> {
    let a = chrono::DateTime::parse_from_rfc3339(&row.started_at).ok()?.with_timezone(&chrono::Utc);
    let b = chrono::DateTime::parse_from_rfc3339(row.ended_at.as_deref()?).ok()?.with_timezone(&chrono::Utc);
    let duration = row.duration_seconds?;
    if b <= a || duration < 0 { return None; }
    if a >= start && b <= end { return Some(duration); }
    if duration != (b - a).num_seconds() { return None; }
    Some((b.min(end) - a.max(start)).num_seconds().max(0))
}

/// Maps a database row to a [`SessionRow`].
///
/// Expects the column order used by every `sessions` SELECT in `db_queries.rs`:
/// `id, started_at, ended_at, state, duration_seconds, break_credit, history_kind`.
pub fn map_session_row(row: &rusqlite::Row) -> rusqlite::Result<SessionRow> {
    let credit: Option<String> = row.get(5)?;
    Ok(SessionRow {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        state: row.get(3)?,
        duration_seconds: row.get(4)?,
        break_credit: credit.as_deref().and_then(crate::db::break_credit_from_db_str),
        history_kind: row.get(6)?,
    })
}
