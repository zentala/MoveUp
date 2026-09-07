//! db_voice_notes.rs — persistence for dictated voice notes (E021-T06).
//!
//! Voice is an *inlet*, not a second source of truth about the session: a note
//! is stored here and, at most, nudges [`CommunicationPolicy`](crate::communication_policy)'s
//! snooze. Nothing in `session_*.rs` reads this table (ADR 021 / E021-D3), so
//! a corrupt or missing `voice_notes` table can never change a counter.
//!
//! ## Day bucketing
//! `date_local` is derived from `captured_at_ms` in the **local** zone, the
//! same way `sessions.date_local` is. Deriving it from the UTC instant would
//! file a note dictated at 23:30 under tomorrow.
//!
//! ## The reply is written twice on purpose
//! A note is inserted before the optional AI reply exists, so the reply lands
//! later through [`set_reply`]. That ordering is deliberate: the note must
//! survive an AI call that times out.

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

/// A note about to be stored. Separate from [`VoiceNoteRow`] because the row
/// id and the local-day bucket are assigned by the insert, not by the caller.
#[derive(Debug, Clone)]
pub struct NewVoiceNote<'a> {
    pub captured_at_ms: i64,
    pub transcript: &'a str,
    pub lang: Option<&'a str>,
    pub intent: &'a str,
}

/// One row of the `voice_notes` table, as returned to the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct VoiceNoteRow {
    pub id: i64,
    /// When the phone captured the dictation (unix ms).
    pub captured_at_ms: i64,
    /// RFC3339 UTC timestamp of the insert.
    pub recorded_at: String,
    /// Local `YYYY-MM-DD` bucket derived from `captured_at_ms`.
    pub date_local: String,
    pub transcript: String,
    /// BCP-47 tag the phone reported, when it reported one.
    pub lang: Option<String>,
    /// [`Intent::label`](crate::voice_intent::Intent::label) — `snooze`,
    /// `note`, `walk_start`, `walk_end`.
    pub intent: String,
    /// The AI coaching reply, when one was produced. `None` means no reply was
    /// generated — never "the reply was empty".
    pub reply: Option<String>,
}

/// Creates the `voice_notes` table and its day index. Idempotent.
///
/// Called from [`crate::db::init_schema`] so every connection that opens the
/// app database gets it, including the ones tests build in a temp directory.
pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS voice_notes (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            captured_at_ms INTEGER NOT NULL,
            recorded_at    TEXT    NOT NULL,
            date_local     TEXT    NOT NULL,
            transcript     TEXT    NOT NULL,
            lang           TEXT,
            intent         TEXT    NOT NULL,
            reply          TEXT
        )
        "#,
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_voice_notes_date_local ON voice_notes(date_local)",
        [],
    )?;
    Ok(())
}

/// Local `YYYY-MM-DD` bucket for a unix-millisecond instant.
///
/// A timestamp outside the representable range falls back to today rather than
/// erroring — a nonsense `captured_at_ms` must not lose the note.
pub fn date_local_of(captured_at_ms: i64) -> String {
    use chrono::TimeZone;
    match chrono::Utc.timestamp_millis_opt(captured_at_ms).single() {
        Some(utc) => utc
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string(),
        None => chrono::Local::now().format("%Y-%m-%d").to_string(),
    }
}

/// Inserts a note and returns its row id.
pub fn insert_voice_note(conn: &Connection, note: &NewVoiceNote) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO voice_notes \
         (captured_at_ms, recorded_at, date_local, transcript, lang, intent, reply) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
        rusqlite::params![
            note.captured_at_ms,
            chrono::Utc::now().to_rfc3339(),
            date_local_of(note.captured_at_ms),
            note.transcript,
            note.lang,
            note.intent,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Attaches an AI reply to an already-stored note.
///
/// Returns the number of rows updated, so a caller that passes a stale id sees
/// `0` rather than a silent success.
pub fn set_reply(conn: &Connection, id: i64, reply: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE voice_notes SET reply = ?1 WHERE id = ?2",
        rusqlite::params![reply, id],
    )
}

/// Every note bucketed to `day` (`YYYY-MM-DD`, local), oldest first.
///
/// An unknown day yields an empty vector — the caller must render that as "no
/// notes", which is a different statement from a query that failed.
pub fn list_voice_notes(conn: &Connection, day: &str) -> Result<Vec<VoiceNoteRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, captured_at_ms, recorded_at, date_local, transcript, lang, intent, reply \
         FROM voice_notes WHERE date_local = ?1 ORDER BY captured_at_ms ASC, id ASC",
    )?;
    let rows = stmt.query_map([day], |row| {
        Ok(VoiceNoteRow {
            id: row.get(0)?,
            captured_at_ms: row.get(1)?,
            recorded_at: row.get(2)?,
            date_local: row.get(3)?,
            transcript: row.get(4)?,
            lang: row.get(5)?,
            intent: row.get(6)?,
            reply: row.get(7)?,
        })
    })?;
    rows.collect()
}

/// One note by id. `None` means no such row.
#[cfg_attr(not(test), allow(dead_code))]
pub fn get_voice_note(conn: &Connection, id: i64) -> Result<Option<VoiceNoteRow>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, captured_at_ms, recorded_at, date_local, transcript, lang, intent, reply \
         FROM voice_notes WHERE id = ?1",
        [id],
        |row| {
            Ok(VoiceNoteRow {
                id: row.get(0)?,
                captured_at_ms: row.get(1)?,
                recorded_at: row.get(2)?,
                date_local: row.get(3)?,
                transcript: row.get(4)?,
                lang: row.get(5)?,
                intent: row.get(6)?,
                reply: row.get(7)?,
            })
        },
    )
    .optional()
}

// Tests live in db_voice_notes_tests.rs.
