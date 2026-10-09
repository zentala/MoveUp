//! startup_diagnostics.rs — Capture early failures and panics.
//!
//! Provides a minimal diagnostic sink for failures that occur during setup
//! before normal logging and persistence are available. Must not panic,
//! poison application state, or lock up initialization.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Minimal diagnostic recorder for setup-phase failures.
///
/// Writes to a temporary startup diagnostic log that persists even if
/// normal logging fails. Designed to be lightweight and never panic.
pub struct StartupDiagnostics {
    /// Path to the diagnostic log file.
    path: PathBuf,
    /// Thread-safe writer (writes are synchronous and unbuffered).
    writer: Mutex<std::fs::File>,
}

impl StartupDiagnostics {
    /// Create a new diagnostics recorder at the given path.
    ///
    /// Returns `None` if the file cannot be created, which is acceptable
    /// for startup diagnostics — the absence of a log is itself a diagnostic.
    pub fn new(path: PathBuf) -> Option<Self> {
        std::fs::File::create(&path)
            .ok()
            .map(|f| Self {
                path,
                writer: Mutex::new(f),
            })
    }

    /// Record a diagnostic message (timestamp not included; let the caller choose).
    pub fn record(&self, message: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{}", message);
            let _ = w.flush();
        }
    }

    /// Record a panic backtrace (called from a panic hook).
    ///
    /// `message` is the panic message, `backtrace` is the string output
    /// of `std::backtrace::Backtrace::capture()`.
    pub fn record_panic(&self, message: &str, backtrace: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "PANIC: {}", message);
            let _ = writeln!(w, "Backtrace:\n{}", backtrace);
            let _ = w.flush();
        }
    }

    /// Consume the diagnostics and return the path for inspection.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Parse a panic message from a panic hook payload.
///
/// Extracts the string representation of a panic, supporting both
/// `String` and `&str` panic values.
pub fn extract_panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "<unknown panic type>".to_string()
    }
}

#[cfg(test)]
mod startup_diagnostics_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_diagnostics_file_creation() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join(format!("test_startup_diag_{}.log", std::process::id()));

        let _diag = StartupDiagnostics::new(path.clone()).expect("should create file");
        assert!(path.exists());

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_message() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join(format!("test_startup_msg_{}.log", std::process::id()));

        {
            let diag = StartupDiagnostics::new(path.clone()).expect("should create file");
            diag.record("Test message");
            diag.record("Another message");
        }

        let content = fs::read_to_string(&path).expect("should read file");
        assert!(content.contains("Test message"));
        assert!(content.contains("Another message"));

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_panic() {
        let tmp_dir = std::env::temp_dir();
        let path = tmp_dir.join(format!("test_startup_panic_{}.log", std::process::id()));

        {
            let diag = StartupDiagnostics::new(path.clone()).expect("should create file");
            diag.record_panic("out of memory", "backtrace line 1\nbacktrace line 2");
        }

        let content = fs::read_to_string(&path).expect("should read file");
        assert!(content.contains("PANIC: out of memory"));
        assert!(content.contains("backtrace line 1"));

        // Clean up
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_extract_panic_message_from_string() {
        let payload: Box<dyn std::any::Any + Send> =
            Box::new("panic message".to_string());
        let msg = extract_panic_message(&*payload);
        assert_eq!(msg, "panic message");
    }

    #[test]
    fn test_extract_panic_message_from_str() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("static panic");
        let msg = extract_panic_message(&*payload);
        assert_eq!(msg, "static panic");
    }

    #[test]
    fn test_extract_unknown_panic() {
        let payload: Box<dyn std::any::Any + Send> = Box::new(42i32);
        let msg = extract_panic_message(&*payload);
        assert_eq!(msg, "<unknown panic type>");
    }

    #[test]
    fn test_failed_file_creation() {
        // Try to create in a path that cannot exist (invalid characters on Windows).
        let path = PathBuf::from("\\??\\\0");
        let diag = StartupDiagnostics::new(path);
        assert!(diag.is_none());
    }
}
