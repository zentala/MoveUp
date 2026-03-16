//! db.rs — SQLite persistence layer via tauri-plugin-sql.
//!
//! Defines the schema DDL and serialisable row types used by Tauri commands.
//! The actual database connection is managed by tauri-plugin-sql; commands
//! receive the connection via the plugin's `Database` type.

use serde::{Deserialize, Serialize};

// ─── Schema ──────────────────────────────────────────────────────────────────

/// DDL executed on startup to ensure the schema exists.
/// Exposed via the `get_schema_sql` Tauri command for the frontend to execute.
pub const SCHEMA_SQL: &str = r#"
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
"#;

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
// TODO: used when height readings persistence is implemented
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
    pub sessions: Vec<SessionRow>,
}
