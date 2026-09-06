//! db.rs — SQLite persistence layer: schema, types, and re-exports.
//!
//! Sub-modules:
//! - `db_sessions` — session CRUD (insert, load, save state)
//! - `db_queries` — summary and aggregate queries

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::session_types::BreakCredit;

// Re-export sub-module functions for backwards compatibility.
pub use crate::db_queries::get_today_summary;
pub use crate::db_sessions::load_today_totals;

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
            session_limit_secs    INTEGER DEFAULT 2700,
            break_credit          TEXT
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
        // date_local: stores the LOCAL date when a session was recorded.
        // Queries use this column instead of parsing UTC timestamps,
        // so timezone changes (e.g. travel) don't break daily aggregation.
        if !cols.contains(&"date_local".to_string()) {
            let _ = conn.execute("ALTER TABLE sessions ADD COLUMN date_local TEXT", []);
            // Backfill existing rows: extract date from UTC started_at.
            // Not perfectly timezone-correct for historical data, but best effort.
            let _ = conn.execute_batch(
                "UPDATE sessions SET date_local = SUBSTR(started_at, 1, 10) WHERE date_local IS NULL",
            );
        }
        // break_credit: the credit actually applied when this session ended
        // ("none" | "partial" | "full", ADR 008). Deliberately left NULL for
        // pre-migration rows — the credit was never recorded for them, and
        // NULL says "unknown" where 'none' would claim a break earned nothing.
        if !cols.contains(&"break_credit".to_string()) {
            let _ = conn.execute("ALTER TABLE sessions ADD COLUMN break_credit TEXT", []);
        }
    }

    Ok(())
}

// ─── Break credit column ─────────────────────────────────────────────────────

/// Serialises a [`BreakCredit`] for the `sessions.break_credit` column.
pub fn break_credit_to_db_str(credit: &BreakCredit) -> &'static str {
    match credit {
        BreakCredit::None => "none",
        BreakCredit::Partial => "partial",
        BreakCredit::Full => "full",
    }
}

/// Parses the `sessions.break_credit` column. Returns `None` for NULL and for
/// any value this build does not know — an unreadable value is unknown, never
/// "no credit".
pub fn break_credit_from_db_str(raw: &str) -> Option<BreakCredit> {
    match raw {
        "none" => Some(BreakCredit::None),
        "partial" => Some(BreakCredit::Partial),
        "full" => Some(BreakCredit::Full),
        _ => None,
    }
}

// ─── Row types ───────────────────────────────────────────────────────────────

/// A single row from the `sessions` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct SessionRow {
    #[serde(skip_serializing)]
    // Never on the wire, so it is absent from the generated TypeScript too.
    #[cfg_attr(test, ts(skip))]
    #[allow(dead_code)] // read from DB but not accessed directly
    pub id: i64,
    #[serde(rename = "start")]
    pub started_at: String,
    #[serde(rename = "end")]
    pub ended_at: Option<String>,
    pub state: String,
    #[serde(rename = "duration_secs")]
    pub duration_seconds: Option<i64>,
    /// Break credit applied when this session ended (ADR 008).
    ///
    /// `None` means the row predates the `break_credit` column — the value is
    /// unknown, not zero. Consumers must render it as unknown rather than
    /// silently treating it as "no credit".
    pub break_credit: Option<BreakCredit>,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct TodaySummary {
    pub sitting_secs: i64,
    pub standing_secs: i64,
    pub yesterday_sitting_secs: i64,
    pub yesterday_standing_secs: i64,
    pub position_changes: u32,
    pub sessions: Vec<SessionRow>,
}
