//! Tauri adapters; all engine persistence stays in its existing owners.
use std::sync::{Arc, Mutex, TryLockError, atomic::Ordering};
use rusqlite::OptionalExtension;
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
    let store_available = app.try_state::<crate::app_store::AppStore<tauri::Wry>>().is_some();
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
    database_rows: Vec<ShutdownRow>,
    sealed: Option<String>,
    user_quit_pending: bool,
    cancellation_pending: bool,
}
#[derive(Clone)]
struct ShutdownRow { id: i64, pending: crate::session_types::PendingHistory }
impl FlushProgress {
    pub const fn new() -> Self {
        Self { database_rows: Vec::new(), sealed: None, user_quit_pending: false,
            cancellation_pending: false }
    }
    pub fn complete(&self) -> bool { self.sealed.is_some() }
    /// Roll back only rows inserted by this shutdown and return them to the
    /// running engine's retry queue. Deduplicated and unrelated rows are never owned.
    pub fn cancel(
        &mut self,
        session: &Mutex<crate::session::SessionManager>,
        database: &Mutex<Option<rusqlite::Connection>>,
        mut save_store: impl FnMut(&crate::session::SessionManager) -> &'static str,
    ) -> Result<(), String> {
        self.sealed = None;
        self.user_quit_pending = false;
        self.cancellation_pending = true;
        {
            let mut manager = session.lock().unwrap_or_else(|error| error.into_inner());
            for row in &self.database_rows {
                manager.queue_history(
                    row.pending.state.clone(),
                    row.pending.span.clone(),
                    row.pending.credit.clone(),
                );
            }
            // Make the retry envelope durable before removing any database row.
            // If this save fails, every previously acknowledged row remains in SQLite.
            if save_store(&manager) != "saved" {
                return Err("shutdown cancellation Store save failed".into());
            }
        }
        while !self.database_rows.is_empty() {
            let row = self.database_rows[0].clone();
            let db = database.try_lock().map_err(|_| "shutdown cancellation database locked")?;
            let conn = db.as_ref().ok_or("shutdown cancellation database missing")?;
            let identity = conn.query_row(
                "SELECT started_at, ended_at, state, duration_seconds FROM sessions WHERE id = ?1",
                [row.id],
                |record| Ok((record.get::<_, String>(0)?, record.get::<_, Option<String>>(1)?,
                    record.get::<_, String>(2)?, record.get::<_, Option<i64>>(3)?)),
            ).optional().map_err(|_| "shutdown cancellation inspection failed")?;
            match identity {
                Some((started, Some(ended), state, Some(duration)))
                    if started == row.pending.span.started_at
                        && ended == row.pending.span.ended_at
                        && state == format!("{:?}", row.pending.state)
                        && duration == row.pending.span.duration_secs => {
                    let removed = conn.execute(
                        "DELETE FROM sessions WHERE id = ?1 AND started_at = ?2 AND ended_at = ?3 AND state = ?4 AND duration_seconds = ?5",
                        rusqlite::params![row.id, started, ended, state, duration],
                    ).map_err(|_| "shutdown cancellation failed")?;
                    if removed != 1 {
                        return Err("shutdown row identity changed; cancellation not confirmed".into());
                    }
                    drop(db);
                    self.database_rows.remove(0);
                }
                None => {
                    drop(db);
                    self.database_rows.remove(0);
                }
                Some(_) => return Err("shutdown row identity changed; cancellation not confirmed".into()),
            }
        }
        self.sealed = None;
        self.user_quit_pending = false;
        self.cancellation_pending = false;
        Ok(())
    }
}

/// Explicit Store errors/missing Store block graceful exit; they are never saved.
/// Only a completed attempt is reused by CloseRequested/Exit.
pub fn prepare_shutdown(app: &AppHandle, user_quit: bool) -> Result<String, String> {
    prepare_shutdown_inner(app, user_quit, true)
}

fn prepare_shutdown_inner(
    app: &AppHandle,
    user_quit: bool,
    restart_on_failure: bool,
) -> Result<String, String> {
    let mut progress = SHUTDOWN.try_lock().map_err(|_| "shutdown flush already in progress".to_string())?;
    if let Err(error) = crate::persistence_checkpoint::stop(app) {
        if restart_on_failure {
            crate::persistence_checkpoint::start(app);
        }
        return Err(format!("Could not stop the periodic persistence checkpoint: {error}"));
    }
    let report = flush(app, &mut progress);
    if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() {
        lifecycle.event(&format!("shutdown_flush {report}"));
    }
    if !progress.complete() {
        if restart_on_failure {
            crate::persistence_checkpoint::start(app);
        }
        return Err(report);
    }
    progress.user_quit_pending |= user_quit;
    Ok(report)
}

pub fn cancel_shutdown(app: &AppHandle) -> Result<(), String> {
    let result = (|| {
        let mut progress = SHUTDOWN.try_lock().map_err(|_| "shutdown cancellation busy".to_string())?;
        let state = app.state::<AppState>();
        progress.cancel(&state.session, &state.db, |session| {
            if crate::session_persistence::save_via_app(app, session) { "saved" } else { "failed" }
        })?;
        if let Some(lifecycle) = app.try_state::<Arc<AppLifecycle>>() { lifecycle.event("shutdown_cancelled"); }
        Ok(())
    })();
    crate::persistence_checkpoint::start(app);
    result
}

pub fn shutdown(app: &AppHandle) {
    let report = prepare_shutdown_inner(app, false, false).unwrap_or_else(|error| error);
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
        match app.try_state::<crate::app_store::AppStore<tauri::Wry>>() {
            Some(store) => {
                let reset = crate::session_persistence::load_reset_after(store.inner());
                let wall_now = chrono::Utc::now();
                let monotonic_now = std::time::Instant::now();
                let policy = state.comm_policy.lock().unwrap_or_else(|error| error.into_inner());
                match crate::session_persistence::PersistedEngineState::from_session_with_policy(
                    session, reset, &policy, wall_now, monotonic_now,
                ).save(store.inner()) {
                    Ok(()) => {
                        crate::persistence_checkpoint::record_result(app, &Ok(()), wall_now);
                        "saved"
                    }
                    Err(error) => {
                        crate::persistence_checkpoint::record_result(app, &Err(error), wall_now);
                        "failed"
                    }
                }
            }
            None => "skipped_store_missing",
        }
    })
}

pub(crate) fn flush_state(session: &Mutex<crate::session::SessionManager>,
    database: &Mutex<Option<rusqlite::Connection>>,
    save_store: impl FnMut(&crate::session::SessionManager) -> &'static str) -> String {
    flush_state_progress(session, database, &mut FlushProgress::new(), save_store)
}

pub(crate) fn flush_state_progress(session: &Mutex<crate::session::SessionManager>,
    database: &Mutex<Option<rusqlite::Connection>>, progress: &mut FlushProgress,
    save_store: impl FnMut(&crate::session::SessionManager) -> &'static str) -> String {
    flush_state_progress_at(session, database, progress, chrono::Utc::now(), save_store)
}

pub(crate) fn flush_state_progress_at(
    session: &Mutex<crate::session::SessionManager>,
    database: &Mutex<Option<rusqlite::Connection>>,
    progress: &mut FlushProgress,
    now: chrono::DateTime<chrono::Utc>,
    mut save_store: impl FnMut(&crate::session::SessionManager) -> &'static str,
) -> String {
    if progress.cancellation_pending
        && progress.cancel(session, database, &mut save_store).is_err()
    {
        return "store=not_attempted database=cancellation_pending".into();
    }
    if let Some(report) = &progress.sealed { return report.clone(); }
    let has_pending = {
        let Ok(mut session) = session.try_lock() else {
            return "store=skipped_session_locked database=skipped_session_locked".into();
        };
        session.queue_shutdown_history(now);
        // An unfinished/cancelled attempt always overwrites Store again:
        // persisted manager flags and reset metadata can change independently
        // of SessionState. Persist the retry queue before attempting DB ack.
        let store_result = save_store(&session);
        if store_result != "saved" {
            return format!("store={store_result} database=not_attempted");
        }
        !session.pending_history.is_empty()
    };
    let db_result = if !has_pending {
        "not_needed"
    } else {
        match database.try_lock() {
            Err(_) => "skipped_db_locked",
            Ok(db) => match db.as_ref() {
                None => "skipped_db_missing",
                Some(conn) => {
                    let Ok(mut manager) = session.try_lock() else {
                        return "store=saved database=skipped_session_locked".into();
                    };
                    match crate::serial_periodic_reset::persist_pending_history(
                        &mut manager, conn, |pending, id| {
                            progress.database_rows.push(ShutdownRow {
                                id,
                                pending: pending.clone(),
                            });
                        },
                    ) {
                        Ok(()) => "saved",
                        Err(_) => "failed",
                    }
                }
            },
        }
    };
    let store_result = if db_result == "saved" {
        let manager = session.lock().unwrap_or_else(|error| error.into_inner());
        save_store(&manager)
    } else {
        "saved"
    };
    let report = format!("store={store_result} database={db_result}");
    if store_result == "saved" && matches!(db_result, "saved" | "not_needed") {
        progress.sealed = Some(report.clone());
    }
    report
}
