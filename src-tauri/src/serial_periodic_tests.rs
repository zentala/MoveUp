//! serial_periodic_tests.rs — Tests for the responsibility units extracted
//! from `serial_periodic` into `serial_periodic_reset` (E019-T06).
//!
//! Everything here runs without a Tauri `AppHandle` — that is the whole point
//! of the split. Before it, each of these steps was reachable only through
//! `check_periodic`/`handle_reading`, which need a running app.

use std::sync::{Arc, Mutex};

use chrono::{Local, Utc};
use rusqlite::Connection;
use tempfile::TempDir;

use crate::communication_profile::CommunicationProfile;
use crate::config::AppConfig;
use crate::db::init_schema;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::event_logger::EventLogger;
use crate::serial_periodic_reset as steps;
use crate::session::{BreakCredit, CompletedSession, DeskState, SessionManager};
use crate::snapshot_logger::SnapshotLogger;

fn shared_session() -> Arc<Mutex<SessionManager>> {
    Arc::new(Mutex::new(SessionManager::new()))
}

fn read_events(dir: &TempDir) -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();
    std::fs::read_to_string(dir.path().join(today).join("events.log")).unwrap()
}

#[test]
fn e019_t06_serial_periodic_daily_reset_is_quiet_on_the_same_day() {
    let session = shared_session();
    let config = AppConfig::default();

    // A session created moments ago has not crossed a day boundary.
    assert!(
        !steps::run_daily_reset(&session, &config),
        "no rollover should be reported within the same day"
    );
}

#[test]
fn e019_t06_serial_periodic_daily_reset_sends_telemetry_when_opted_in() {
    let session = shared_session();
    let config = AppConfig {
        telemetry_enabled: true,
        telemetry_device_id: "test-uuid".to_string(),
        ..AppConfig::default()
    };

    // Opting in must not change the rollover verdict, and must not panic on
    // the telemetry path.
    assert!(!steps::run_daily_reset(&session, &config));
}

#[test]
fn e019_t06_serial_periodic_snapshot_writes_the_minute_file() {
    let tmp = TempDir::new().unwrap();
    let logger = SnapshotLogger::new(tmp.path().to_path_buf());
    let session = shared_session();

    steps::log_minute_snapshot(&session, &logger, "COM3", &ErgonomicProfile::default());

    let day = Local::now().format("%Y-%m-%d").to_string();
    let minute = Local::now().format("%H-%M.json").to_string();
    let raw = std::fs::read_to_string(tmp.path().join(day).join(minute))
        .expect("periodic tick must write a snapshot for the current minute");
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(json["port"], "COM3");
    assert_eq!(json["connected"], true);
    assert!(
        json["metrics"].as_array().is_some_and(|m| !m.is_empty()),
        "snapshot must carry computed KPI metrics, got {}",
        json["metrics"]
    );
}

#[test]
fn e019_t06_serial_periodic_snapshot_survives_an_unwritable_dir() {
    let tmp = TempDir::new().unwrap();
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, b"not a directory").unwrap();
    let logger = SnapshotLogger::new(blocker.join("logs"));
    let session = shared_session();

    // A dead log directory must not take the serial loop down with it.
    steps::log_minute_snapshot(&session, &logger, "COM3", &ErgonomicProfile::default());
}

#[test]
fn e019_t06_serial_periodic_fresh_session_raises_no_notifications() {
    let session = shared_session();
    let tick = steps::collect_notification_intents(&session, &CommunicationProfile::default());

    assert!(!tick.events_fired);
    assert!(tick.intents.is_empty());
}

#[test]
fn e019_t06_serial_periodic_persists_a_completed_span_with_credit() {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    let db = Arc::new(Mutex::new(Some(conn)));

    let now = Utc::now();
    let completed = CompletedSession {
        started_at: (now - chrono::Duration::seconds(600)).to_rfc3339(),
        ended_at: now.to_rfc3339(),
        duration_secs: 600,
    };
    steps::persist_completed_session(&db, &completed, &DeskState::Standing, Some(&BreakCredit::Full));

    let guard = db.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let (state, duration, credit): (String, i64, Option<String>) = conn
        .query_row(
            "SELECT state, duration_seconds, break_credit FROM sessions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("the completed span must reach the sessions table");

    assert_eq!(state, "Standing");
    assert_eq!(duration, 600);
    assert!(credit.is_some(), "a break span must carry its credit");
}

#[test]
fn e019_t06_serial_periodic_sitting_span_stores_no_credit() {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    let db = Arc::new(Mutex::new(Some(conn)));

    let now = Utc::now();
    let completed = CompletedSession {
        started_at: (now - chrono::Duration::seconds(300)).to_rfc3339(),
        ended_at: now.to_rfc3339(),
        duration_secs: 300,
    };
    steps::persist_completed_session(&db, &completed, &DeskState::Sitting, None);

    let guard = db.lock().unwrap();
    let credit: Option<String> = guard
        .as_ref()
        .unwrap()
        .query_row("SELECT break_credit FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert!(credit.is_none(), "a sitting span must leave the column NULL");
}

#[test]
fn e019_t06_serial_periodic_persist_without_a_connection_is_a_noop() {
    let db: Arc<Mutex<Option<Connection>>> = Arc::new(Mutex::new(None));
    let completed = CompletedSession {
        started_at: Utc::now().to_rfc3339(),
        ended_at: Utc::now().to_rfc3339(),
        duration_secs: 1,
    };
    // Sensor readings arrive before the DB is opened; that must not panic.
    steps::persist_completed_session(&db, &completed, &DeskState::Sitting, None);
}

#[test]
fn e019_t06_serial_periodic_state_change_line_format() {
    let line = steps::format_state_change_line(&DeskState::Sitting, &DeskState::Standing, 110.4, 42);
    assert_eq!(line, "STATE Sitting\u{2192}Standing h=110cm idle=42s");
}

#[test]
fn e019_t06_serial_periodic_credit_line_format() {
    let line = steps::format_credit_line(&BreakCredit::Partial, 300, 1200);
    assert_eq!(line, "CREDIT Partial dur=300s sitting=1200");
}

#[test]
fn e019_t06_serial_periodic_credit_logs_day_break_once() {
    let tmp = TempDir::new().unwrap();
    let logger = EventLogger::new(tmp.path().to_path_buf());
    let session = shared_session();
    session.lock().unwrap().day_break_applied = true;

    steps::log_break_credit(&session, &logger, &BreakCredit::Full, 30_000);
    steps::log_break_credit(&session, &logger, &BreakCredit::Full, 30_000);

    let content = read_events(&tmp);
    assert_eq!(
        content.matches("CREDIT day_break").count(),
        1,
        "the day-break flag is one-shot; got:\n{content}"
    );
    assert_eq!(content.matches("CREDIT Full").count(), 2);
    assert!(
        !session.lock().unwrap().day_break_applied,
        "the flag must be drained after logging"
    );
}
