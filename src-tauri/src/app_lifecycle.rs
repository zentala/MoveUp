//! app_lifecycle.rs — Application lifecycle tracking and diagnostics.
//!
//! Records lifecycle events (started, ready, quit, failures) with per-run identity
//! (PID/version/build mode) to diagnose startup and runtime issues. Persists run
//! outcome after singleton ownership is established, so a second-instance activation
//! cannot overwrite a running instance's record.

use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Application build modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BuildMode {
    Debug,
    Release,
}

impl std::fmt::Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Debug => write!(f, "debug"),
            BuildMode::Release => write!(f, "release"),
        }
    }
}

/// Per-run identifier for this application instance.
///
/// Includes PID, version, and build mode to correlate log entries across
/// multiple runs and diagnose failed startup sequences.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunId {
    /// Process ID at startup.
    pub pid: u32,
    /// Application version (e.g., "0.6.0").
    pub version: String,
    /// Build mode (debug or release).
    pub build_mode: BuildMode,
    /// Unix timestamp (milliseconds) of run start.
    pub started_at_ms: u64,
}

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}:{}",
            self.pid, self.version, self.build_mode, self.started_at_ms
        )
    }
}

impl RunId {
    /// Create a new run ID with the current process ID and timestamp.
    pub fn new(version: &str, build_mode: BuildMode) -> Self {
        let started_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            pid: process::id(),
            version: version.to_string(),
            build_mode,
            started_at_ms,
        }
    }
}

/// Lifecycle stage names for readiness and diagnostic purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LifecycleStage {
    /// App has started, entering Tauri setup.
    Starting,
    /// Singleton lock acquired, beginning initialization.
    Initializing,
    /// Tray initialized and visible.
    TrayReady,
    /// Database opened and migrated.
    DatabaseReady,
    /// Serial device scan completed.
    SerialReady,
    /// Remote server listening.
    RemoteReady,
    /// All subsystems ready, application fully operational.
    Ready,
    /// User explicitly quit via tray menu.
    QuitRequested,
    /// Normal application exit.
    CleanExit,
    /// Failure prevented normal progression.
    Failed,
}

impl std::fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Initializing => write!(f, "initializing"),
            Self::TrayReady => write!(f, "tray_ready"),
            Self::DatabaseReady => write!(f, "database_ready"),
            Self::SerialReady => write!(f, "serial_ready"),
            Self::RemoteReady => write!(f, "remote_ready"),
            Self::Ready => write!(f, "ready"),
            Self::QuitRequested => write!(f, "quit_requested"),
            Self::CleanExit => write!(f, "clean_exit"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Tracks application lifecycle state independent of the session engine.
///
/// Used by startup diagnostics and readiness probes to report what stage
/// of initialization the app has reached, and why it failed if it did.
pub struct AppLifecycle {
    run_id: RunId,
    current_stage: Arc<Mutex<LifecycleStage>>,
    quit_requested: Arc<AtomicBool>,
    /// Optional failure reason, recorded only after a failure stage.
    failure_reason: Arc<Mutex<Option<String>>>,
}

impl AppLifecycle {
    /// Create a new lifecycle tracker.
    pub fn new(version: &str, build_mode: BuildMode) -> Self {
        let run_id = RunId::new(version, build_mode);
        Self {
            run_id,
            current_stage: Arc::new(Mutex::new(LifecycleStage::Starting)),
            quit_requested: Arc::new(AtomicBool::new(false)),
            failure_reason: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns the run ID for this instance.
    pub fn run_id(&self) -> &RunId {
        &self.run_id
    }

    /// Advance to the next lifecycle stage.
    pub fn advance_stage(&self, stage: LifecycleStage) {
        if let Ok(mut current) = self.current_stage.lock() {
            *current = stage;
        }
    }

    /// Record an explicit quit request.
    pub fn mark_quit_requested(&self) {
        self.quit_requested.store(true, Ordering::Relaxed);
        self.advance_stage(LifecycleStage::QuitRequested);
    }

    /// Record a failure with an optional reason.
    pub fn record_failure(&self, reason: Option<&str>) {
        self.advance_stage(LifecycleStage::Failed);
        if let Ok(mut fr) = self.failure_reason.lock() {
            *fr = reason.map(|s| s.to_string());
        }
    }

    /// Mark normal exit after full initialization.
    pub fn mark_clean_exit(&self) {
        self.advance_stage(LifecycleStage::CleanExit);
    }

    /// Get the current stage.
    pub fn current_stage(&self) -> LifecycleStage {
        self.current_stage
            .lock()
            .ok()
            .map(|s| *s)
            .unwrap_or(LifecycleStage::Starting)
    }

    /// Check if quit was explicitly requested (not a crash).
    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested.load(Ordering::Relaxed)
    }

    /// Get the failure reason if any.
    pub fn failure_reason(&self) -> Option<String> {
        self.failure_reason.lock().ok().and_then(|fr| fr.clone())
    }
}

#[cfg(test)]
mod app_lifecycle_tests {
    use super::*;

    #[test]
    fn test_run_id_creation() {
        let run_id = RunId::new("0.6.0", BuildMode::Release);
        assert_eq!(run_id.version, "0.6.0");
        assert_eq!(run_id.build_mode, BuildMode::Release);
        assert!(run_id.started_at_ms > 0);
        assert_eq!(run_id.pid, process::id());
    }

    #[test]
    fn test_run_id_display() {
        let run_id = RunId::new("0.6.0", BuildMode::Debug);
        let s = run_id.to_string();
        assert!(s.contains("0.6.0"));
        assert!(s.contains("debug"));
    }

    #[test]
    fn test_lifecycle_stage_progression() {
        let lc = AppLifecycle::new("0.6.0", BuildMode::Release);
        assert_eq!(lc.current_stage(), LifecycleStage::Starting);

        lc.advance_stage(LifecycleStage::Initializing);
        assert_eq!(lc.current_stage(), LifecycleStage::Initializing);

        lc.advance_stage(LifecycleStage::TrayReady);
        assert_eq!(lc.current_stage(), LifecycleStage::TrayReady);
    }

    #[test]
    fn test_quit_requested() {
        let lc = AppLifecycle::new("0.6.0", BuildMode::Release);
        assert!(!lc.is_quit_requested());

        lc.mark_quit_requested();
        assert!(lc.is_quit_requested());
        assert_eq!(lc.current_stage(), LifecycleStage::QuitRequested);
    }

    #[test]
    fn test_failure_recording() {
        let lc = AppLifecycle::new("0.6.0", BuildMode::Release);
        lc.record_failure(Some("serial port not found"));

        assert_eq!(lc.current_stage(), LifecycleStage::Failed);
        assert_eq!(lc.failure_reason(), Some("serial port not found".to_string()));
    }

    #[test]
    fn test_clean_exit() {
        let lc = AppLifecycle::new("0.6.0", BuildMode::Release);
        lc.advance_stage(LifecycleStage::Ready);
        lc.mark_clean_exit();

        assert_eq!(lc.current_stage(), LifecycleStage::CleanExit);
    }

    #[test]
    fn test_lifecycle_stage_display() {
        assert_eq!(LifecycleStage::Starting.to_string(), "starting");
        assert_eq!(LifecycleStage::Ready.to_string(), "ready");
        assert_eq!(LifecycleStage::Failed.to_string(), "failed");
    }
}
