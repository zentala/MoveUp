//! serial_periodic.rs — Periodic checks and reading processing for serial loop.
//!
//! Extracted from the reader loop to keep serial.rs focused on I/O.
//! Notification firing is delegated to [`NotificationService`] for central routing.

use std::sync::{Arc, Mutex};

use log::{error, info};
use tauri::{AppHandle, Emitter, Manager};

use crate::activity::is_active;
use crate::commands::AppState;
use crate::event_logger::EventLogger;
use crate::metrics::MetricEngine;
use crate::notification_service::NotificationService;
use crate::session::{DeskState, SessionManager};
use crate::snapshot_logger::SnapshotLogger;

/// Checks daily reset and notification conditions, firing appropriate events.
/// Called every ~60 seconds from the serial reader loop.
pub fn check_periodic(
    app: &AppHandle,
    session: &Arc<Mutex<SessionManager>>,
    config: &crate::config::AppConfig,
    snapshot_logger: &Arc<SnapshotLogger>,
    event_logger: &Arc<EventLogger>,
    port_name: &str,
    alert_popup: &Arc<Mutex<crate::alert_popup::AlertPopup>>,
) {
    let state: tauri::State<'_, AppState> = app.state();
    let policy = state.comm_policy.lock().unwrap();
    let ergo = policy.ergo_profile().clone();
    let comm = policy.comm_profile().clone();
    drop(policy);

    // Send telemetry BEFORE daily reset so we capture the full day's data.
    let daily_reset_occurred = {
        let mut sess = session.lock().unwrap();
        let will_reset = sess.needs_daily_reset();
        if will_reset {
            crate::telemetry::send_telemetry_if_enabled(config, &sess);
        }
        sess.check_daily_reset()
    };

    if daily_reset_occurred {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit("desk:daily-reset", ());
        event_logger.log("RESET daily");
        // Clear stale persisted flags so they don't leak into tomorrow.
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            crate::session_persistence::clear(store.inner());
        }
    }

    // Write per-minute snapshot with computed metrics.
    {
        let sess = session.lock().unwrap();
        let now = chrono::Utc::now();
        let snapshot = sess.snapshot();
        let mut raw = sess.state.clone();
        raw.sitting_seconds_total = sess.get_live_sitting_seconds_total(now);
        raw.standing_seconds = sess.get_live_standing_seconds(now);
        let metrics = MetricEngine::with_defaults().compute_all(&raw, &ergo);
        snapshot_logger.log_snapshot(
            &snapshot,
            true,
            Some(port_name.to_string()),
            crate::APP_VERSION,
            metrics,
        );
    }

    // Raw daily totals: the PostureBalance message compares sitting against
    // standing, so both sides must be uncredited (E015-T03).
    let (notification_events, sitting_secs_total, standing_secs) = {
        let mut sess = session.lock().unwrap();
        let events = sess.check_notification_conditions(&comm);
        let snap = sess.snapshot();
        (events, snap.sitting_seconds_total, snap.standing_seconds)
    };

    let intents = NotificationService::build_intents(
        &notification_events,
        sitting_secs_total,
        standing_secs,
    );
    NotificationService::dispatch(
        &intents,
        &comm.notification_backend,
        event_logger,
        alert_popup,
    );

    // Persist flags if any notifications were fired in this periodic check.
    if !notification_events.is_empty() {
        save_session_state(app, session);
    }

    // Hot-reload communication and ergonomic profiles if files changed on disk.
    reload_profiles_if_changed(app);
}

/// Processes a single sensor reading through the session state machine.
/// Emits state changes, persists to DB, and checks alert conditions.
pub fn handle_reading(
    app: &AppHandle,
    mm: i32,
    session: &Arc<Mutex<SessionManager>>,
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    _config: &crate::config::AppConfig,
    event_logger: &Arc<EventLogger>,
    alert_popup: &Arc<Mutex<crate::alert_popup::AlertPopup>>,
) {
    let state: tauri::State<'_, AppState> = app.state();
    let policy = state.comm_policy.lock().unwrap();
    let ergo = policy.ergo_profile().clone();
    let comm = policy.comm_profile().clone();
    drop(policy);

    let active = is_active();
    let idle_secs = crate::activity::get_idle_seconds();
    let (state_before, result) = {
        let mut sess = session.lock().unwrap();
        let before = sess.current_state();
        let res = sess.on_reading(mm, active);
        if sess.last_accumulate_ran {
            sess.accumulate_score_tick(&ergo);
        }
        (before, res)
    };

    // Diagnostic: log idle time every ~30s when Standing to catch is_active() issues
    if state_before == DeskState::Standing && idle_secs > 30 {
        log::debug!(
            "Standing idle diagnostic: idle={}s active={} mm={}",
            idle_secs, active, mm
        );
    }

    if let Some(ref payload) = result.state_change {
        let height_cm = payload.desk_height_cm;
        event_logger.log(&format!(
            "STATE {:?}\u{2192}{:?} h={:.0}cm idle={}s",
            state_before, payload.state, height_cm, idle_secs
        ));
        let _ = app.emit("desk:state-changed", payload);

        // Persist notification flags and credit-reduced sitting_seconds on state change.
        save_session_state(app, session);

        if state_before == DeskState::Sitting && payload.state == DeskState::Standing {
            let praise = {
                let mut sess = session.lock().unwrap();
                sess.should_send_praise_halfway(&comm)
            };
            if praise {
                let intent = NotificationService::praise_halfway_intent();
                NotificationService::dispatch(
                    &[intent],
                    &comm.notification_backend,
                    event_logger,
                    alert_popup,
                );
            }
        }
    }

    if let Some((ref credit, dur)) = result.break_credit {
        let mut sess = session.lock().unwrap();
        let sitting = sess.snapshot().sitting_seconds;
        event_logger.log(&format!(
            "CREDIT {:?} dur={}s sitting={}",
            credit, dur, sitting
        ));
        if sess.day_break_applied {
            event_logger.log(&format!("CREDIT day_break dur={}s", dur));
            sess.day_break_applied = false;
        }
        drop(sess);
    }

    if let Some(ref completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
        // A break span that ended this tick carries the credit it earned; a
        // sitting span carries none, and the column stays NULL for it.
        let credit = result.break_credit.as_ref().map(|(c, _)| c);
        let db_lock = db.lock().unwrap();
        if let Some(ref conn) = *db_lock {
            if let Err(e) = crate::db_sessions::insert_session_with_credit(
                conn,
                &completed.started_at,
                &completed.ended_at,
                &format!("{:?}", state_before),
                completed.duration_secs,
                credit,
            ) {
                error!("Failed to save completed session: {}", e);
            }
        }
    }

    let alert = { session.lock().unwrap().should_alert() };
    if alert {
        let sitting = session.lock().unwrap().snapshot().sitting_seconds;
        event_logger.log(&format!("ALERT sit_limit sitting={}s", sitting));
        let intent = NotificationService::sit_limit_intent();
        NotificationService::dispatch(
            &[intent],
            &comm.notification_backend,
            event_logger,
            alert_popup,
        );
        save_session_state(app, session);
    }

    let stand_alert = { session.lock().unwrap().should_stand_alert() };
    if stand_alert {
        let standing = session.lock().unwrap().snapshot().standing_session_secs;
        event_logger.log(&format!("ALERT stand_limit standing={}s", standing));
        let intent = NotificationService::stand_limit_intent();
        NotificationService::dispatch(
            &[intent],
            &comm.notification_backend,
            event_logger,
            alert_popup,
        );
        save_session_state(app, session);
    }
}

/// Persists notification flags and credit-reduced sitting_seconds to the store.
fn save_session_state(app: &AppHandle, session: &Arc<Mutex<SessionManager>>) {
    let sess = session.lock().unwrap();
    crate::session_persistence::save_via_app(app, &sess);
}

// Profile hot-reload logic extracted to profile_reload.rs
use crate::profile_reload::reload_profiles_if_changed;
