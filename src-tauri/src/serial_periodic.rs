//! serial_periodic.rs — Periodic checks and reading processing for serial loop.
//!
//! Extracted from the reader loop to keep serial.rs focused on I/O.
//! Notification firing is delegated to [`NotificationService`] for central routing.
//!
//! The two public functions here are thin orchestrators: they own the
//! [`AppHandle`] (profiles, event emission, store persistence) and call into
//! [`crate::serial_periodic_reset`] for every step that does not need one.

use std::sync::{Arc, Mutex};

use log::info;
use tauri::{AppHandle, Emitter, Manager};

use crate::activity::is_active;
use crate::commands::AppState;
use crate::desk_events::{DESK_DAILY_RESET, DESK_STATE_CHANGED};
use crate::event_logger::EventLogger;
use crate::notification_service::NotificationService;
use crate::serial_periodic_reset as steps;
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
    let (ergo, comm) = active_profiles(app);

    if steps::run_daily_reset(session, config) {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit(DESK_DAILY_RESET, ());
        event_logger.log("RESET daily");
        // Clear stale persisted flags so they don't leak into tomorrow.
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            crate::session_persistence::clear(store.inner());
        }
    }

    steps::log_minute_snapshot(session, snapshot_logger, port_name, &ergo);

    let tick = steps::collect_notification_intents(session, &comm);
    NotificationService::dispatch(
        &tick.intents,
        &comm.notification_backend,
        event_logger,
        alert_popup,
    );
    // Persist flags if any notifications were fired in this periodic check.
    if tick.events_fired {
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
    let (ergo, comm) = active_profiles(app);

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
        event_logger.log(&steps::format_state_change_line(
            &state_before,
            &payload.state,
            payload.desk_height_cm,
            idle_secs,
        ));
        let _ = app.emit(DESK_STATE_CHANGED, payload);

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
        steps::log_break_credit(session, event_logger, credit, dur);
    }

    if let Some(ref completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
        // A break span that ended this tick carries the credit it earned; a
        // sitting span carries none, and the column stays NULL for it.
        let credit = result.break_credit.as_ref().map(|(c, _)| c);
        steps::persist_completed_session(db, completed, &state_before, credit);
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

/// Clones the currently active ergonomic and communication profiles.
///
/// Cloning releases the policy lock before any of the work below runs.
fn active_profiles(
    app: &AppHandle,
) -> (
    crate::ergonomic_profile::ErgonomicProfile,
    crate::communication_profile::CommunicationProfile,
) {
    let state: tauri::State<'_, AppState> = app.state();
    let policy = state.comm_policy.lock().unwrap();
    (policy.ergo_profile().clone(), policy.comm_profile().clone())
}

/// Persists notification flags and credit-reduced sitting_seconds to the store.
fn save_session_state(app: &AppHandle, session: &Arc<Mutex<SessionManager>>) {
    let sess = session.lock().unwrap();
    crate::session_persistence::save_via_app(app, &sess);
}

// Profile hot-reload logic extracted to profile_reload.rs
use crate::profile_reload::reload_profiles_if_changed;
