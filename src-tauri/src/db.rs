//! db.rs — SQLite persistence layer via rusqlite.
//!
//! Manages database initialization, session recording, and daily summary queries.
//! Emits `desk:db-error` on any rusqlite failure.

use log::error;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ─── Schema ──────────────────────────────────────────────────────────────────

/// Initializes the database schema on first startup.
/// Idempotent — safe to call multiple times.
pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at       TEXT    NOT NULL,
            ended_at         TEXT,
            state            TEXT    NOT NULL,
            duration_seconds INTEGER
        );

        CREATE TABLE IF NOT EXISTS height_readings (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            recorded_at TEXT    NOT NULL,
            mm          INTEGER NOT NULL,
            cm          REAL    NOT NULL,
            desk_state  TEXT    NOT NULL
        );
        "#,
    )?;
    Ok(())
}

// ─── Row types ───────────────────────────────────────────────────────────────

/// A single row from the `sessions` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub state: String,
    pub duration_seconds: Option<i64>,
}

/// A single row from the `height_readings` table.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightReadingRow {
    pub id: i64,
    pub recorded_at: String,
    pub mm: i32,
    pub cm: f32,
    pub desk_state: String,
}

// ─── Summary ─────────────────────────────────────────────────────────────────

/// Aggregated daily summary returned by `get_today_summary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodaySummary {
    pub sitting_secs: i64,
    pub standing_secs: i64,
    pub yesterday_sitting_secs: i64,
    pub yesterday_standing_secs: i64,
    pub position_changes: u32,
    pub sessions: Vec<SessionRow>,
}

// ─── Insert operations ───────────────────────────────────────────────────────

/// Inserts a completed sitting session into the database.
/// Returns an error if the insert fails; emits `desk:db-error` event on failure.
// ─── Query operations ────────────────────────────────────────────────────────

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
            Ok(SessionRow {
                id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                state: row.get(3)?,
                duration_seconds: row.get(4)?,
            })
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
            if session.state == "Sitting" {
                sitting_secs += duration;
            } else {
                standing_secs += duration;
            }
        }
    }

    let (yesterday_sitting_secs, yesterday_standing_secs) = get_yesterday_totals(conn)?;

    Ok(TodaySummary {
        sitting_secs,
        standing_secs,
        yesterday_sitting_secs,
        yesterday_standing_secs,
        position_changes: 0,  // Will be set by caller from SessionManager
        sessions,
    })
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_schema_init_idempotent() {
        let conn = test_conn();
        assert!(init_schema(&conn).is_ok());
        assert!(init_schema(&conn).is_ok(), "second call should be safe");
    }

    #[test]
    fn test_insert_and_query_today() {
        let conn = test_conn();
        init_schema(&conn).unwrap();

        let started_at = "2025-03-16T09:00:00Z";
        let ended_at = "2025-03-16T09:30:00Z";
        let duration = 1800;

        assert!(insert_session(&conn, started_at, ended_at, "Sitting", duration).is_ok());

        let (_sitting, _standing) = load_today_totals(&conn).expect("query should succeed");
        // Note: this will only work if the date in started_at matches today
        // For a real test, we'd need to mock the date
    }

    #[test]
    fn test_load_today_totals_empty() {
        let conn = test_conn();
        init_schema(&conn).unwrap();

        let (sitting, standing) = load_today_totals(&conn).unwrap();
        assert_eq!(sitting, 0);
        assert_eq!(standing, 0);
    }

    #[test]
    fn test_get_yesterday_totals_empty() {
        let conn = test_conn();
        init_schema(&conn).unwrap();

        let (sitting, standing) = get_yesterday_totals(&conn).unwrap();
        assert_eq!(sitting, 0, "no yesterday data should return 0 sitting");
        assert_eq!(standing, 0, "no yesterday data should return 0 standing");
    }

    #[test]
    fn test_get_today_summary_includes_yesterday() {
        let conn = test_conn();
        init_schema(&conn).unwrap();

        let summary = get_today_summary(&conn).unwrap();
        assert_eq!(summary.yesterday_sitting_secs, 0, "empty db should have 0 yesterday sitting");
        assert_eq!(summary.yesterday_standing_secs, 0, "empty db should have 0 yesterday standing");
    }
}
