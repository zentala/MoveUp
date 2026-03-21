//! serial_periodic.rs — Periodic checks and reading processing for serial loop.
//!
//! Extracted from the reader loop to keep serial.rs focused on I/O.

use std::sync::{Arc, Mutex};

use log::{error, info};
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::activity::is_active;
use crate::session::{DeskState, NotificationEvent, SessionManager};

/// Checks daily reset and notification conditions, firing appropriate events.
/// Called every ~60 seconds from the serial reader loop.
pub fn check_periodic(
    app: &AppHandle,
    session: &Arc<Mutex<SessionManager>>,
    config: &crate::config::AppConfig,
) {
    let daily_reset_occurred = {
        let mut sess = session.lock().unwrap();
        sess.check_daily_reset()
    };

    if daily_reset_occurred {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit("desk:daily-reset", ());
    }

    let notification_events = {
        let mut sess = session.lock().unwrap();
        sess.check_notification_conditions(config)
    };

    for event in notification_events {
        match event {
            NotificationEvent::Inactivity => {
                let _ = app.notification().builder()
                    .title("No position change in 60 minutes")
                    .body("Time to move.")
                    .show();
            }
            NotificationEvent::PostureBalance => {
                let _ = app.notification().builder()
                    .title("You've been sitting most of today")
                    .body("Consider standing for a while.")
                    .show();
            }
            NotificationEvent::Praise => {
                let _ = app.notification().builder()
                    .title("Halfway through your standing goal!")
                    .body("Keep it up.")
                    .show();
            }
            NotificationEvent::StandLimitReached => {}
            NotificationEvent::StandingTargetReached => {
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
) {
    let active = is_active();
    let (state_before, result) = {
        let mut sess = session.lock().unwrap();
        let before = sess.current_state();
        let res = sess.on_reading(mm, active);
        sess.accumulate_score_tick(config);
        (before, res)
    };

    if let Some(payload) = result.state_change {
        let _ = app.emit("desk:state-changed", &payload);

        {
            let db_lock = db.lock().unwrap();
            if let Some(ref conn) = *db_lock {
                if let Err(e) = crate::db::save_session_state(conn, &payload) {
                    error!("Failed to save session state to database: {}", e);
                }
            }
        }

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

    if let Some(completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
    }

    let alert = { session.lock().unwrap().should_alert() };
    if alert {
        let _ = app.notification().builder()
            .title("Time to stand up!")
            .body("You've been sitting for 40 minutes. Take a break.")
            .show();
    }

    let stand_alert = { session.lock().unwrap().should_stand_alert() };
    if stand_alert {
        let _ = app.notification().builder()
            .title("You've been standing a while")
            .body("Ready to sit down for a bit?")
            .show();
    }
}
