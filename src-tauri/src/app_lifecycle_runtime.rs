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
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() { lifecycle.request_quit(); }
}

static SHUTDOWN: Mutex<FlushProgress> = Mutex::new(FlushProgress::new());

#[derive(Default)]
pub(crate) struct FlushProgress {
    database_row: Option<ShutdownRow>,
    sealed: Option<String>,
    user_quit_pending: bool,
    cancellation_pending: bool,
}
struct ShutdownRow { id: i64, started: String, ended: String, state: String, duration: i64 }
impl FlushProgress {
    pub const fn new() -> Self {
        Self { database_row: None, sealed: None, user_quit_pending: false,
            cancellation_pending: false }
    }
    pub fn complete(&self) -> bool { self.sealed.is_some() }
    /// Only the row inserted by this pending shutdown is withdrawn after a
    /// rejected stop; the still-running engine owns its eventual completed row.
    pub fn cancel(&mut self, database: &Mutex<Option<rusqlite::Connection>>) -> Result<(), String> {
        self.sealed = None;
        self.user_quit_pending = false;
        self.cancellation_pending = true;
        if let Some(row) = &self.database_row {
            let db = database.try_lock().map_err(|_| "shutdown cancellation database locked")?;
            let conn = db.as_ref().ok_or("shutdown cancellation database missing")?;
            let removed = conn.execute("DELETE FROM sessions WHERE id = ?1 AND started_at = ?2 AND ended_at = ?3 AND state = ?4 AND duration_seconds = ?5",
                rusqlite::params![row.id, row.started, row.ended, row.state, row.duration])
                .map_err(|_| "shutdown cancellation failed")?;
            if removed != 1 { return Err("shutdown row identity changed; cancellation not confirmed".into()); }
        }
        self.database_row = None;
        self.sealed = None;
        self.user_quit_pending = false;
        self.cancellation_pending = false;
        Ok(())
    }
}

/// Explicit Store errors/missing Store block graceful exit; they are never saved.
/// Only a completed attempt is reused by CloseRequested/Exit.
pub fn prepare_shutdown(app: &AppHandle, user_quit: bool) -> Result<String, String> {
    let mut progress = SHUTDOWN.try_lock().map_err(|_| "shutdown flush already in progress".to_string())?;
    let report = flush(app, &mut progress);
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() {
        lifecycle.event(&format!("shutdown_flush {report}"));
    }
    if !progress.complete() { return Err(report); }
    progress.user_quit_pending |= user_quit;
    Ok(report)
}

pub fn cancel_shutdown(app: &AppHandle) -> Result<(), String> {
    let mut progress = SHUTDOWN.try_lock().map_err(|_| "shutdown cancellation busy".to_string())?;
    let state = app.state::<AppState>();
    progress.cancel(&state.db)?;
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() { lifecycle.event("shutdown_cancelled"); }
    Ok(())
}

pub fn shutdown(app: &AppHandle) {
    let report = prepare_shutdown(app, false).unwrap_or_else(|error| error);
    // The user requested Quit only on the explicit tray path. Exit 0 by itself
    // never creates this intent, and a refused stop can cancel it before exit.
    if SHUTDOWN.try_lock().is_ok_and(|progress| progress.user_quit_pending) { request_quit(app); }
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() {
        lifecycle.remote_listening.store(false, Ordering::Release);
        lifecycle.tray_initialized.store(false, Ordering::Release);
        lifecycle.finish_exit(&report);
    }
}

/// Same save points and session insertion as before, now with honest outcomes.
fn flush(app: &AppHandle, progress: &mut FlushProgress) -> String {
    let state = app.state::<AppState>();
    flush_state_progress(&state.session, &state.db, progress, |session| {
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
    flush_state_progress(session, database, &mut FlushProgress::new(), save_store)
}

pub(crate) fn flush_state_progress(session: &Mutex<crate::session::SessionManager>,
    database: &Mutex<Option<rusqlite::Connection>>, progress: &mut FlushProgress,
    save_store: impl FnOnce(&crate::session::SessionManager) -> &'static str) -> String {
    if progress.cancellation_pending && progress.cancel(database).is_err() {
        return "store=not_attempted database=cancellation_pending".into();
    }
    if let Some(report) = &progress.sealed { return report.clone(); }
    let (completed, state_label) = {
        let Ok(session) = session.try_lock() else {
            return "store=skipped_session_locked database=skipped_session_locked".into();
        };
        // An unfinished/cancelled attempt always overwrites Store again:
        // persisted manager flags and reset metadata can change independently
        // of SessionState. Only a fully completed flush is deduplicated.
        let store_result = save_store(&session);
        if store_result != "saved" {
            // No history row is committed for a failed partial attempt. The
            // running engine may still change and later complete this session.
            return format!("store={store_result} database=not_attempted");
        }
        (session.flush_current_session(), format!("{:?}", session.current_state()))
    };
    let db_result = match completed {
        None => "not_needed",
        Some(completed) => match database.try_lock() {
            Err(_) => "skipped_db_locked",
            Ok(db) => match db.as_ref() {
                None => "skipped_db_missing",
                Some(conn) => match crate::db_sessions::insert_session(conn,
                    &completed.started_at, &completed.ended_at, &state_label, completed.duration_secs) {
                    Ok(()) => {
                        progress.database_row = Some(ShutdownRow { id: conn.last_insert_rowid(),
                            started: completed.started_at, ended: completed.ended_at,
                            state: state_label, duration: completed.duration_secs });
                        "saved"
                    },
                    Err(_) => "failed",
                }
            }
        }
    };
    let report = format!("store=saved database={db_result}");
    if matches!(db_result, "saved" | "not_needed") { progress.sealed = Some(report.clone()); }
    report
}
