//! Tests for the `voice_notes` table (E021-T06).
//!
//! Every test runs against a real SQLite connection created by the real
//! [`crate::db::init_schema`] — never a stub. The point is to prove the app's
//! own migration path creates a table the queries in `db_voice_notes` can
//! actually read, which a hand-written `CREATE TABLE` in the test would hide.

#![cfg(test)]

use rusqlite::Connection;

use crate::db_voice_notes::{
    date_local_of, get_voice_note, insert_voice_note, list_voice_notes, set_reply, NewVoiceNote,
};

/// In-memory database migrated by the production schema function.
fn db() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory db opens");
    crate::db::init_schema(&conn).expect("schema applies");
    conn
}

fn note<'a>(captured_at_ms: i64, transcript: &'a str, intent: &'a str) -> NewVoiceNote<'a> {
    NewVoiceNote {
        captured_at_ms,
        transcript,
        lang: Some("pl-PL"),
        intent,
    }
}

/// Noon local time on a fixed day, so `date_local` bucketing is unambiguous in
/// any timezone the test machine runs in.
fn noon_local_ms(year: i32, month: u32, day: u32) -> i64 {
    use chrono::TimeZone;
    chrono::Local
        .with_ymd_and_hms(year, month, day, 12, 0, 0)
        .single()
        .expect("noon exists")
        .timestamp_millis()
}

#[test]
fn e021_t06_db_init_schema_creates_the_voice_notes_table() {
    let conn = db();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='voice_notes'",
            [],
            |row| row.get(0),
        )
        .expect("query runs");
    assert_eq!(count, 1, "db::init_schema must create voice_notes");
}

#[test]
fn e021_t06_init_schema_is_idempotent() {
    let conn = db();
    crate::db::init_schema(&conn).expect("second run");
    crate::db_voice_notes::init_schema(&conn).expect("third run");
    let ms = noon_local_ms(2026, 9, 6);
    insert_voice_note(&conn, &note(ms, "drzemka 5", "snooze")).expect("insert");
    assert_eq!(list_voice_notes(&conn, &date_local_of(ms)).unwrap().len(), 1);
}

#[test]
fn e021_t06_insert_then_list_round_trips_every_field() {
    let conn = db();
    let ms = noon_local_ms(2026, 9, 6);
    let id = insert_voice_note(&conn, &note(ms, "idę na spacer", "walk_start")).expect("insert");
    assert!(id > 0, "insert must return a real row id");

    let rows = list_voice_notes(&conn, &date_local_of(ms)).expect("list");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.id, id);
    assert_eq!(row.captured_at_ms, ms);
    assert_eq!(row.transcript, "idę na spacer");
    assert_eq!(row.lang.as_deref(), Some("pl-PL"));
    assert_eq!(row.intent, "walk_start");
    assert_eq!(row.reply, None, "a fresh note has no reply yet");
    assert!(
        chrono::DateTime::parse_from_rfc3339(&row.recorded_at).is_ok(),
        "recorded_at must be RFC3339, got {}",
        row.recorded_at
    );
}

#[test]
fn e021_t06_a_missing_lang_stays_null_rather_than_becoming_empty() {
    let conn = db();
    let ms = noon_local_ms(2026, 9, 6);
    let id = insert_voice_note(
        &conn,
        &NewVoiceNote {
            captured_at_ms: ms,
            transcript: "no language reported",
            lang: None,
            intent: "note",
        },
    )
    .expect("insert");

    let row = get_voice_note(&conn, id).expect("query").expect("row");
    assert_eq!(
        row.lang, None,
        "an absent language must read back as unknown, not as an empty string"
    );
}

#[test]
fn e021_t06_set_reply_attaches_a_reply_to_an_existing_note() {
    let conn = db();
    let ms = noon_local_ms(2026, 9, 6);
    let id = insert_voice_note(&conn, &note(ms, "drzemka 5", "snooze")).expect("insert");

    assert_eq!(set_reply(&conn, id, "Rozumiem, wracam za 5 minut.").unwrap(), 1);
    let row = get_voice_note(&conn, id).expect("query").expect("row");
    assert_eq!(row.reply.as_deref(), Some("Rozumiem, wracam za 5 minut."));
}

#[test]
fn e021_t06_set_reply_on_an_unknown_id_updates_nothing() {
    let conn = db();
    assert_eq!(
        set_reply(&conn, 4242, "orphan").unwrap(),
        0,
        "a stale id must report zero rows, not look like a successful write"
    );
}

#[test]
fn e021_t06_notes_are_bucketed_by_local_day_and_ordered_oldest_first() {
    let conn = db();
    let today = noon_local_ms(2026, 9, 6);
    let yesterday = noon_local_ms(2026, 9, 5);

    insert_voice_note(&conn, &note(today + 60_000, "second", "note")).expect("insert");
    insert_voice_note(&conn, &note(today, "first", "note")).expect("insert");
    insert_voice_note(&conn, &note(yesterday, "older", "note")).expect("insert");

    let rows = list_voice_notes(&conn, &date_local_of(today)).expect("list");
    let transcripts: Vec<&str> = rows.iter().map(|r| r.transcript.as_str()).collect();
    assert_eq!(
        transcripts,
        vec!["first", "second"],
        "only today's notes, oldest first"
    );

    let older = list_voice_notes(&conn, &date_local_of(yesterday)).expect("list");
    assert_eq!(older.len(), 1);
    assert_eq!(older[0].transcript, "older");
}

#[test]
fn e021_t06_a_day_with_no_notes_lists_empty_rather_than_failing() {
    let conn = db();
    let rows = list_voice_notes(&conn, "1999-01-01").expect("query must succeed");
    assert!(rows.is_empty());
}

#[test]
fn e021_t06_a_malformed_day_string_is_an_empty_result_not_an_error() {
    let conn = db();
    assert!(list_voice_notes(&conn, "").expect("query runs").is_empty());
    assert!(list_voice_notes(&conn, "not-a-day")
        .expect("query runs")
        .is_empty());
}

#[test]
fn e021_t06_date_local_uses_the_local_zone_and_survives_a_nonsense_timestamp() {
    let ms = noon_local_ms(2026, 9, 6);
    assert_eq!(date_local_of(ms), "2026-09-06");

    // Out of chrono's representable range: must fall back to today, not panic.
    let fallback = date_local_of(i64::MAX);
    assert_eq!(fallback, chrono::Local::now().format("%Y-%m-%d").to_string());
}

#[test]
fn e021_t06_get_voice_note_reports_a_missing_row_as_none() {
    let conn = db();
    assert!(get_voice_note(&conn, 1).expect("query runs").is_none());
}
