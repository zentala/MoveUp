//! event_logger.rs — Append-only event log writer.
//!
//! Writes one line per event to `{base_dir}/YYYY-MM-DD/events.log`.
//! Format: `HH:MM:SS TYPE details`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;

use crate::snapshot_logger::ensure_day_dir;

/// Appends timestamped event lines to a daily log file.
pub struct EventLogger {
    base_dir: PathBuf,
}

impl EventLogger {
    /// Creates a new logger writing to `base_dir` (e.g. `{app_data}/logs`).
    ///
    /// Panics if `base_dir` cannot be created — event logging is critical
    /// infrastructure and silent failure masks data-loss bugs.
    pub fn new(base_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&base_dir).unwrap_or_else(|e| {
            panic!(
                "event_logger: cannot create base log dir {:?}: {} (kind={:?})",
                base_dir,
                e,
                e.kind()
            )
        });
        Self { base_dir }
    }

    /// Appends a single event line to today's `events.log`.
    ///
    /// On write failure, retries once after 100 ms. Errors are logged at
    /// `error!` level with the full path and error kind for debuggability.
    pub fn log(&self, event: &str) {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M:%S").to_string();

        let day_dir = match ensure_day_dir(&self.base_dir, &date_str) {
            Ok(d) => d,
            Err(e) => {
                log::error!(
                    "event_logger: cannot create day dir {:?}/{}: {} (kind={:?})",
                    self.base_dir,
                    date_str,
                    e,
                    e.kind()
                );
                return;
            }
        };

        let path = day_dir.join("events.log");
        let line = format!("{} {}\n", time_str, event);

        if self.try_write(&path, &line).is_err() {
            std::thread::sleep(Duration::from_millis(100));
            if let Err(e) = self.try_write(&path, &line) {
                log::error!(
                    "event_logger: write failed after retry at {:?}: {} (kind={:?})",
                    path,
                    e,
                    e.kind()
                );
            }
        }
    }

    /// Opens `path` in append+create mode and writes `line`.
    fn try_write(&self, path: &std::path::Path, line: &str) -> std::io::Result<()> {
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| {
                log::error!(
                    "event_logger: open failed at {:?}: {} (kind={:?})",
                    path,
                    e,
                    e.kind()
                );
                e
            })?;
        f.write_all(line.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn event_log_appends_lines() {
        let tmp = TempDir::new().unwrap();
        let logger = EventLogger::new(tmp.path().to_path_buf());

        logger.log("STATE Sitting→Standing h=110cm");
        logger.log("ALERT sit_limit sitting=2700s");
        logger.log("RESET daily");

        let today = Local::now().format("%Y-%m-%d").to_string();
        let content =
            std::fs::read_to_string(tmp.path().join(&today).join("events.log")).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn event_log_creates_dirs() {
        let tmp = TempDir::new().unwrap();
        let logger = EventLogger::new(tmp.path().to_path_buf());

        logger.log("START v0.1.0");

        let today = Local::now().format("%Y-%m-%d").to_string();
        assert!(tmp.path().join(&today).join("events.log").exists());
    }

    #[test]
    fn event_log_format() {
        let tmp = TempDir::new().unwrap();
        let logger = EventLogger::new(tmp.path().to_path_buf());

        logger.log("DEVICE connected COM3");

        let today = Local::now().format("%Y-%m-%d").to_string();
        let content =
            std::fs::read_to_string(tmp.path().join(&today).join("events.log")).unwrap();
        let line = content.lines().next().unwrap();
        // Format: HH:MM:SS TYPE details
        assert!(line.contains("DEVICE connected COM3"));
        // Verify time prefix pattern (HH:MM:SS)
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        assert_eq!(parts[0].len(), 8); // HH:MM:SS
    }

    #[test]
    fn new_creates_base_dir() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("a").join("b").join("logs");
        assert!(!nested.exists(), "dir should not exist yet");
        let _logger = EventLogger::new(nested.clone());
        assert!(nested.exists(), "EventLogger::new must create base_dir");
    }
}
