//! Application lifecycle outside the ergonomic engine.
use std::{path::PathBuf, sync::{Arc, Mutex, OnceLock}};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::{app_lifecycle_store::{RunStore, RunRecord, PreviousRun},
    event_logger::EventLogger, startup_diagnostics::{Diagnostic, StartupDiagnostics}};

#[derive(Default, Clone)]
pub struct CheckpointStatus {
    pub last_success_utc: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RunId {
    pub run_id: String,
    pub pid: u32,
    pub version: String,
    pub build_mode: String,
    pub supervisor_run_id: Option<String>,
}
impl RunId {
    pub fn new(version: &str) -> Self {
        Self {
            run_id: uuid::Uuid::new_v4().to_string(), pid: std::process::id(),
            version: version.into(),
            build_mode: if cfg!(debug_assertions) { "debug" } else { "release" }.into(),
            supervisor_run_id: std::env::var("PM3_RUN_ID").ok()
                .filter(|s| !s.is_empty() && s.len() <= 128 && s.chars().all(|c|
                    c.is_ascii_alphanumeric() || "-_.:".contains(c))),
        }
    }
}
pub struct AppLifecycle {
    pub identity: RunId,
    pub diagnostics: StartupDiagnostics,
    pub failed: AtomicBool,
    pub tray_initialized: AtomicBool,
    pub persistence_initialized: AtomicBool,
    pub setup_complete: AtomicBool,
    pub remote_listening: AtomicBool,
    pub quit_requested: AtomicBool,
    pub checkpoint_worker: crate::persistence_checkpoint::CheckpointWorker,
    pub checkpoint_status: Mutex<CheckpointStatus>,
    ready_logged: AtomicBool,
    exited: AtomicBool,
    store: Mutex<Option<RunStore>>,
    logger: OnceLock<Arc<EventLogger>>,
}
impl AppLifecycle {
    pub fn new(version: &str, diagnostic_path: PathBuf) -> Self {
        Self {
            identity: RunId::new(version), diagnostics: StartupDiagnostics::new(diagnostic_path),
            failed: AtomicBool::new(false), tray_initialized: AtomicBool::new(false),
            persistence_initialized: AtomicBool::new(false), setup_complete: AtomicBool::new(false),
            remote_listening: AtomicBool::new(false), quit_requested: AtomicBool::new(false),
            checkpoint_worker: crate::persistence_checkpoint::CheckpointWorker::default(),
            checkpoint_status: Mutex::new(CheckpointStatus::default()),
            ready_logged: AtomicBool::new(false), exited: AtomicBool::new(false),
            store: Mutex::new(None), logger: OnceLock::new(),
        }
    }
    /// Only called after singleton plugin setup. An OS lock also guards activation races.
    pub fn claim_primary(&self, directory: &std::path::Path) -> std::io::Result<PreviousRun> {
        let mut slot = self.store.lock().map_err(|_| std::io::Error::other("state lock poisoned"))?;
        if slot.is_some() { return Err(std::io::ErrorKind::AlreadyExists.into()); }
        let (store, previous) = RunStore::claim(directory)?;
        *slot = Some(store);
        drop(slot);
        self.persist("started", "not_attempted");
        Ok(previous)
    }
    pub fn attach_logger(&self, logger: Arc<EventLogger>, previous: PreviousRun) {
        let _ = self.logger.set(logger);
        self.event("started");
        self.event(previous.as_str());
    }
    pub fn event(&self, stage: &str) {
        if let Some(logger) = self.logger.get() {
            logger.log(&format!("LIFECYCLE run={} pid={} version={} build_mode={} stage={}",
                self.identity.run_id, self.identity.pid, self.identity.version,
                self.identity.build_mode, stage));
        }
    }
    pub fn essential_ready(&self) -> bool {
        !self.failed.load(Ordering::Acquire) && !self.quit_requested.load(Ordering::Acquire)
            && !self.exited.load(Ordering::Acquire)
            && self.tray_initialized.load(Ordering::Acquire)
            && self.persistence_initialized.load(Ordering::Acquire)
            && self.setup_complete.load(Ordering::Acquire)
    }
    pub fn setup_finished(&self) {
        self.setup_complete.store(true, Ordering::Release);
        if self.essential_ready() && !self.ready_logged.swap(true, Ordering::AcqRel) {
            self.event("ready"); self.persist("ready", "not_attempted");
        }
    }
    pub fn fail(&self, code: Diagnostic) {
        self.failed.store(true, Ordering::Release);
        self.diagnostics.record(&self.identity.run_id, code);
        self.event(code.as_str()); self.persist("failed", "not_attempted");
    }
    /// Panic path never acquires logger, state, session or persistence locks.
    pub fn panic_observed(&self) {
        self.failed.store(true, Ordering::Release);
        self.diagnostics.record(&self.identity.run_id, Diagnostic::Panic);
    }
    pub fn request_quit(&self) {
        self.quit_requested.store(true, Ordering::Release);
        self.event("quit_requested"); self.persist("quit_requested", "not_attempted");
    }
    pub fn finish_exit(&self, flush: &str) {
        self.exited.store(true, Ordering::Release);
        let outcome = if self.failed.load(Ordering::Acquire) { "failed" } else { "clean_exit" };
        self.event(&format!("{outcome} flush={flush}")); self.persist(outcome, flush);
    }
    fn persist(&self, outcome: &str, flush: &str) {
        let Ok(slot) = self.store.lock() else { return };
        let Some(store) = slot.as_ref() else { return };
        let outcome = if self.failed.load(Ordering::Acquire) { "failed" } else { outcome };
        let record = RunRecord { identity: self.identity.clone(), outcome: outcome.into(),
            quit_requested: self.quit_requested.load(Ordering::Acquire), flush: flush.into() };
        if store.write(&record).is_err() {
            self.diagnostics.record(&self.identity.run_id, Diagnostic::StateWriteFailed);
            self.event("state_write_failed");
        }
    }
}
