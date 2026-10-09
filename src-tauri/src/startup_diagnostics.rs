//! Append-only, best-effort diagnostics before Tauri. No application locks.
use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Arc};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct StartupDiagnostics { path: PathBuf }
impl StartupDiagnostics {
    pub fn new(path: PathBuf) -> Self { Self { path } }
    pub fn record(&self, run_id: &str, code: Diagnostic) {
        // Only internally generated UUIDs and bounded codes, never panic payloads.
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.path) {
            let _ = writeln!(file, "LIFECYCLE run={} pid={} version={} build_mode={} event={}", run_id,
                std::process::id(), env!("CARGO_PKG_VERSION"),
                if cfg!(debug_assertions) { "debug" } else { "release" }, code.as_str());
        }
    }
    pub fn path(&self) -> &std::path::Path { &self.path }
}
#[derive(Clone, Copy)]
pub enum Diagnostic { Started, Panic, SetupFailed, BuildFailed, StateWriteFailed, StateLockFailed }
impl Diagnostic {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started", Self::Panic => "panic",
            Self::SetupFailed => "setup_failed", Self::BuildFailed => "build_failed",
            Self::StateWriteFailed => "state_write_failed",
            Self::StateLockFailed => "state_lock_failed",
        }
    }
}
/// Preserve the previous hook and normal unwind/abort behavior, never catch-and-continue.
pub fn install(lifecycle: Arc<crate::app_lifecycle::AppLifecycle>) {
    let previous = std::panic::take_hook();
    let entered = AtomicBool::new(false);
    std::panic::set_hook(Box::new(move |info| {
        if !entered.swap(true, Ordering::SeqCst) {
            lifecycle.panic_observed();
            previous(info);
            entered.store(false, Ordering::SeqCst);
        }
    }));
}
