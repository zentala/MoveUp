//! db.rs — SQLite persistence layer: schema, types, and re-exports.
//!
//! Sub-modules:
//! - `db_sessions` — session CRUD (insert, load, save state)
//! - `db_queries` — summary and aggregate queries

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// Re-export sub-module functions for backwards compatibility.
pub use crate::db_queries::get_today_summary;
pub use crate::db_sessions::{load_today_totals, save_session_state};

// ─── Schema ──────────────────────────────────────────────────────────────────

/// Initializes the database schema on first startup.
/// Idempotent — safe to call multiple times.
pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id                    INTEGER PRIMARY KEY AUTOINCREMENT,
            started_at            TEXT    NOT NULL,
            ended_at              TEXT,
            state                 TEXT    NOT NULL,
            duration_seconds      INTEGER,
            sitting_seconds       INTEGER DEFAULT 0,
            standing_seconds      INTEGER DEFAULT 0,
            position_changes      INTEGER DEFAULT 0,
            session_limit_secs    INTEGER DEFAULT 2700
        )
        "#,
        [],
    )?;

    // Migrate existing sessions table if it has old schema
    let old_schema = conn
        .prepare("PRAGMA table_info(sessions)")
        .and_then(|mut stmt| {
            let columns: Vec<String> = stmt
                .query_map([], |row| row.get(1))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(columns)
        });

    if let Ok(cols) = old_schema {
        if !cols.contains(&"sitting_seconds".to_string()) {
            let _ = conn.execute_batch(
                "
                ALTER TABLE sessions ADD COLUMN sitting_seconds INTEGER DEFAULT 0;
                ALTER TABLE sessions ADD COLUMN standing_seconds INTEGER DEFAULT 0;
                ALTER TABLE sessions ADD COLUMN position_changes INTEGER DEFAULT 0;
                ALTER TABLE sessions ADD COLUMN session_limit_secs INTEGER DEFAULT 2700;
                ",
            );
        }
    }

    conn.execute_batch(
        r#"
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
