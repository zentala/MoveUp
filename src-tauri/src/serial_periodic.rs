//! serial_periodic.rs — Periodic checks and reading processing for serial loop.
//!
//! Extracted from the reader loop to keep serial.rs focused on I/O.

use std::sync::{Arc, Mutex};

use log::{error, info};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::activity::is_active;
use crate::event_logger::EventLogger;
use crate::session::{DeskState, NotificationEvent, SessionManager};
use crate::snapshot_logger::SnapshotLogger;

/// Checks daily reset and notification conditions, firing appropriate events.
/// Called every ~60 seconds from the serial reader loop.
pub fn check_periodic(
    app: &AppHandle,
    session: &Arc<Mutex<SessionManager>>,
    config: &crate::config::AppConfig,
    snapshot_logger: &Arc<SnapshotLogger>,
    event_logger: &Arc<EventLogger>,
) {
    let daily_reset_occurred = {
        let mut sess = session.lock().unwrap();
        sess.check_daily_reset()
    };

    if daily_reset_occurred {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit("desk:daily-reset", ());
        event_logger.log("RESET daily");
    }

    // Write per-minute snapshot.
    {
        let sess = session.lock().unwrap();
        let snapshot = sess.snapshot();
        let connected = true; // called from reader_loop = sensor is connected
        let port = None::<String>; // port name not available here
        let version = env!("CARGO_PKG_VERSION");
        snapshot_logger.log_snapshot(&snapshot, connected, port, version);
    }

    let notification_events = {
        let mut sess = session.lock().unwrap();
        sess.check_notification_conditions(config)
    };

    for event in &notification_events {
        match event {
            NotificationEvent::Inactivity => {
                event_logger.log("NOTIF inactivity");
                let _ = app.notification().builder()
                    .title("No position change in 60 minutes")
                    .body("Time to move.")
                    .show();
            }
            NotificationEvent::PostureBalance => {
                let sess = session.lock().unwrap();
                let snap = sess.snapshot();
                let ratio = if snap.standing_seconds > 0 {
                    snap.sitting_seconds as f32 / snap.standing_seconds as f32
                } else {
                    f32::INFINITY
                };
                event_logger.log(&format!("NOTIF posture_balance ratio={:.1}", ratio));
                let _ = app.notification().builder()
                    .title("You've been sitting most of today")
                    .body("Consider standing for a while.")
                    .show();
            }
            NotificationEvent::Praise => {
                event_logger.log("NOTIF praise");
                let _ = app.notification().builder()
                    .title("Halfway through your standing goal!")
                    .body("Keep it up.")
                    .show();
            }
            NotificationEvent::StandLimitReached => {}
            NotificationEvent::StandingTargetReached => {
                event_logger.log("NOTIF standing_target_reached");
                let _ = app.notification().builder()
                    .title("Standing target reached!")
                    .body("Great break! You stood for the full target duration.")
                    .show();
            }
        }
    }
}

/// Processes a single sensor reading through the session state machine.
/// Emits state changes, persists to DB, and checks alert conditions.
pub fn handle_reading(
    app: &AppHandle,
    mm: i32,
    session: &Arc<Mutex<SessionManager>>,
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    config: &crate::config::AppConfig,
    event_logger: &Arc<EventLogger>,
) {
    let active = is_active();
    let (state_before, result) = {
        let mut sess = session.lock().unwrap();
        let before = sess.current_state();
        let res = sess.on_reading(mm, active);
        sess.accumulate_score_tick(config);
        (before, res)
    };

    if let Some(ref payload) = result.state_change {
        let height_cm = payload.desk_height_cm;
        event_logger.log(&format!(
            "STATE {:?}\u{2192}{:?} h={:.0}cm",
            state_before, payload.state, height_cm
        ));
        let _ = app.emit("desk:state-changed", payload);

        if state_before == DeskState::Sitting && payload.state == DeskState::Standing {
            let praise = {
                let mut sess = session.lock().unwrap();
                sess.should_send_praise_halfway(config)
            };
            if praise {
                let _ = app.notification().builder()
                    .title("Halfway through your standing goal!")
                    .body("Keep it up.")
                    .show();
            }
        }
    }

    if let Some((ref credit, dur)) = result.break_credit {
        let sitting = session.lock().unwrap().snapshot().sitting_seconds;
        event_logger.log(&format!(
            "CREDIT {:?} dur={}s sitting={}",
            credit, dur, sitting
        ));
    }

    if let Some(ref completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
        let db_lock = db.lock().unwrap();
        if let Some(ref conn) = *db_lock {
            if let Err(e) = crate::db_sessions::insert_session(
                conn,
                &completed.started_at,
                &completed.ended_at,
                &format!("{:?}", state_before),
                completed.duration_secs,
            ) {
                error!("Failed to save completed session: {}", e);
            }
        }
    }

    let alert = { session.lock().unwrap().should_alert() };
    if alert {
        let sitting = session.lock().unwrap().snapshot().sitting_seconds;
        event_logger.log(&format!("ALERT sit_limit sitting={}s", sitting));
        let _ = app.notification().builder()
            .title("Time to stand up!")
            .body("You've been sitting for 40 minutes. Take a break.")
            .show();
    }

    let stand_alert = { session.lock().unwrap().should_stand_alert() };
    if stand_alert {
        let standing = session.lock().unwrap().snapshot().standing_session_secs;
        event_logger.log(&format!("ALERT stand_limit standing={}s", standing));
        let _ = app.notification().builder()
            .title("You've been standing a while")
            .body("Ready to sit down for a bit?")
            .show();
    }
}
