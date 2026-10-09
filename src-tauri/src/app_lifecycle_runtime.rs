//! Tauri adapters; all engine persistence stays in its existing owners.
use std::sync::{Arc, Mutex, TryLockError, atomic::Ordering};
use tauri::{AppHandle, Manager};
use crate::{app_lifecycle::AppLifecycle, commands::AppState,
    runtime_readiness::{ReadinessReader, ReadinessStatus},
    startup_diagnostics::{self, Diagnostic}};

pub fn bootstrap() -> Arc<AppLifecycle> {
    let path = std::env::temp_dir().join(format!("moveup-startup-{}-{}.log",
        std::process::id(), uuid::Uuid::new_v4()));
    let lifecycle = Arc::new(AppLifecycle::new(crate::APP_VERSION, path));
    startup_diagnostics::install(lifecycle.clone());
    lifecycle.diagnostics.record(&lifecycle.identity.run_id, Diagnostic::Started);
    lifecycle
}

pub fn readiness_reader(app: &AppHandle) -> ReadinessReader {
    let lifecycle = app.state::<Arc<AppLifecycle>>().inner().clone();
    let state = app.state::<AppState>();
    let db = state.db.clone();
    let conn = state.conn.clone();
    let store_available = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>().is_some();
    Arc::new(move || {
        local_readiness(&lifecycle, &db, &conn.connected_port, store_available)
    })
}

pub(crate) fn local_readiness(lifecycle: &AppLifecycle,
    db: &Mutex<Option<rusqlite::Connection>>, port: &Mutex<Option<String>>,
    store_available: bool) -> ReadinessStatus {
    let mut status = ReadinessStatus::snapshot(lifecycle);
    // Never wait on session/DB/COM mutexes or invoke a health provider.
    match db.try_lock() {
        Ok(db) => status.persistence_initialized &= db.is_some(),
        // Ordinary writes/queries do not invalidate known initialization.
        Err(TryLockError::WouldBlock) => {},
        Err(TryLockError::Poisoned(_)) => status.persistence_initialized = false,
    }
    status.sensor_present = port.try_lock().is_ok_and(|port| port.is_some());
    status.session_store_available = store_available;
    status.compute_state();
    status
}

pub fn request_quit(app: &AppHandle) {
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() {
        lifecycle.request_quit();
    }
}

pub fn shutdown(app: &AppHandle) {
    let report = flush(app);
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() {
        lifecycle.remote_listening.store(false, Ordering::Release);
        lifecycle.tray_initialized.store(false, Ordering::Release);
        lifecycle.finish_exit(&report);
    }
}

/// Same save points and session insertion as before, now with honest outcomes.
fn flush(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    flush_state(&state.session, &state.db, |session| {
        match app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            Some(store) => {
                let reset = crate::session_persistence::load_reset_after(store.inner());
                match crate::session_persistence::PersistedEngineState::from_session(session, reset)
                    .save(store.inner()) {
                    Ok(()) => "saved",
                    Err(_) => "failed",
                }
            }
            None => "skipped_store_missing",
        }
    })
}

pub(crate) fn flush_state(session: &Mutex<crate::session::SessionManager>,
    database: &Mutex<Option<rusqlite::Connection>>,
    save_store: impl FnOnce(&crate::session::SessionManager) -> &'static str) -> String {
    let (completed, state_label, store_result) = {
        let Ok(session) = session.try_lock() else {
            return "store=skipped_session_locked database=skipped_session_locked".into();
        };
        let store_result = save_store(&session);
        (session.flush_current_session(), format!("{:?}", session.current_state()), store_result)
    };
    let db_result = match completed {
        None => "not_needed",
        Some(completed) => match database.try_lock() {
            Err(_) => "skipped_db_locked",
            Ok(db) => match db.as_ref() {
                None => "skipped_db_missing",
                Some(conn) => match crate::db_sessions::insert_session(conn,
                    &completed.started_at, &completed.ended_at, &state_label, completed.duration_secs) {
                    Ok(()) => "saved",
                    Err(_) => "failed",
                }
            }
        }
    };
    format!("store={store_result} database={db_result}")
}
