//! serial_periodic.rs — Periodic checks and reading processing for serial loop.
//!
//! Extracted from the reader loop to keep serial.rs focused on I/O.
//! Notification firing is delegated to [`NotificationService`] for central routing.
//!
//! The two public functions here are thin orchestrators: they own the
//! [`AppHandle`] (profiles, event emission, store persistence) and call into
//! [`crate::serial_periodic_reset`] for every step that does not need one.

use std::sync::{Arc, Mutex};

use chrono::Offset;
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
    let (ergo, _comm) = active_profiles(app);
    session
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .set_limits(&ergo.limits);

    if steps::run_daily_reset(session, config) {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit(DESK_DAILY_RESET, ());
        event_logger.log("RESET daily");
        // Save the reset snapshot through the shared AppStore, retaining any
        // unacknowledged spans while clearing the manager's daily flags.
        if let Some(store) = app.try_state::<crate::app_store::AppStore<tauri::Wry>>() {
            let manager = session.lock().unwrap_or_else(|error| error.into_inner());
            if !crate::session_persistence::save_via_app_with_reset(
                app, &manager, store.inner(), Some(chrono::Utc::now().to_rfc3339()),
            ) {
                log::error!("Failed to save daily reset and pending history");
            }
        }
    }

    steps::log_minute_snapshot(session, snapshot_logger, port_name, &ergo);

    let now = chrono::Utc::now();
    let intents = {
        let mut sess = session.lock().unwrap_or_else(|error| error.into_inner());
        let facts = sess.notification_facts(now);
        let intents = app.state::<AppState>().comm_policy
            .lock().unwrap_or_else(|error| error.into_inner())
            .resolve_session_notifications(&facts, true);
        let events: Vec<_> = intents.iter().map(|intent| intent.event.clone()).collect();
        sess.record_notification_events(&events);
        intents
    };
    NotificationService::dispatch(&intents, event_logger, alert_popup);
    // This is the existing sensor-driven minute checkpoint. It also retains
    // policy silence/repeat state when no notification fired this minute.
    save_session_state(app, session);

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
    let (ergo, _comm) = active_profiles(app);

    // Input inactivity is a heuristic, not a measurement of physical absence.
    let active = is_active();
    let idle_secs = crate::activity::get_idle_seconds();
    // One clock read per tick: the state machine and the score accumulator must
    // agree on the same instant, or a slow lock makes them disagree by a second.
    let now = chrono::Utc::now();
    let today_local = crate::session_daily::local_date_of(now);
    if let Some(day_boundary) = crate::session_daily::local_day_start_utc(&chrono::Local, today_local) {
        if steps::run_daily_reset_before_reading(
            session, _config, now, today_local, day_boundary,
        ) {
            info!("Daily reset occurred before sensor transition — in-memory counters cleared");
            let _ = app.emit(DESK_DAILY_RESET, ());
            event_logger.log("RESET daily");
            if let Some(store) = app.try_state::<crate::app_store::AppStore<tauri::Wry>>() {
                let sess = session.lock().unwrap_or_else(|error| error.into_inner());
                if !crate::session_persistence::save_via_app_with_reset(
                    app, &sess, store.inner(), Some(now.to_rfc3339()),
                ) {
                    log::error!("Failed to save daily reset and pending history");
                }
            }
        }
    } else {
        log::warn!("daily reset deferred: local date {today_local} has no valid instant");
    }
    let (state_before, result) = {
        let mut sess = session.lock().unwrap();
        // Config for this tick comes from the profile just loaded above, so an
        // edited ergonomic profile applies without an app restart.
        sess.set_limits(&ergo.limits);
        let before = sess.current_state();
        let res = sess.on_reading_at_with_local_hour(mm, active, now, |at| {
            let local = at.with_timezone(&chrono::Local);
            crate::hourly_break_tracker::LocalHourInterval::from_offset(
                at, local.offset().fix().local_minus_utc(),
            )
        });
        if sess.last_accumulate_ran {
            sess.accumulate_score_tick_at(&ergo, now);
        }
        sess.queue_interrupted_history();
        (before, res)
    };

    let has_pending_history = session.lock().unwrap_or_else(|error| error.into_inner())
        .pending_history.len() > 0;
    let store_saved = if has_pending_history || result.state_change.is_some() {
        save_session_state(app, session)
    } else {
        true
    };
    if has_pending_history {
        if !store_saved {
            log::error!("History remains pending because its shared Store snapshot was not saved");
        } else {
            let persisted = {
                let database = db.lock().unwrap_or_else(|error| error.into_inner());
                match database.as_ref() {
                    None => Err("session database is unavailable".to_string()),
                    Some(conn) => {
                        let mut manager = session.lock().unwrap_or_else(|error| error.into_inner());
                        steps::persist_pending_history(&mut manager, conn, |_, _| {})
                            .map_err(|error| error.to_string())
                    }
                }
            };
            match persisted {
                Ok(()) => {
                    if !save_session_state(app, session) {
                        log::error!("History rows were committed but Store acknowledgment save failed; replay remains idempotent");
                    }
                }
                Err(error) => log::error!("History retry pending: {}", error),
            }
        }
    }

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

    }

    if let Some((ref credit, dur)) = result.break_credit {
        steps::log_break_credit(session, event_logger, credit, dur);
    }

    if let Some(ref completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
        // The engine queued this individual span above; the Store-before-DB
        // acknowledgment sequence owns persistence.
    }

    let now = chrono::Utc::now();
    let intents = {
        let mut sess = session.lock().unwrap_or_else(|error| error.into_inner());
        let facts = sess.notification_facts(now);
        let intents = app.state::<AppState>().comm_policy
            .lock().unwrap_or_else(|error| error.into_inner())
            .resolve_session_notifications(&facts, false);
        let events: Vec<_> = intents.iter().map(|intent| intent.event.clone()).collect();
        sess.record_notification_events(&events);
        intents
    };
    NotificationService::dispatch(&intents, event_logger, alert_popup);
    if !intents.is_empty() {
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
fn save_session_state(app: &AppHandle, session: &Arc<Mutex<SessionManager>>) -> bool {
    let sess = session.lock().unwrap();
    crate::session_persistence::save_via_app(app, &sess)
}

// Profile hot-reload logic extracted to profile_reload.rs
use crate::profile_reload::reload_profiles_if_changed;
