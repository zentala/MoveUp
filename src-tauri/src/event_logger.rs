//! event_logger.rs — Append-only event log writer.
//!
//! Writes one line per event to `{base_dir}/YYYY-MM-DD/events.log`.
//! Format: `HH:MM:SS TYPE details`

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;

use crate::snapshot_logger::ensure_day_dir;

/// Appends timestamped event lines to a daily log file.
pub struct EventLogger {
    base_dir: PathBuf,
}

impl EventLogger {
    /// Creates a new logger writing to `base_dir` (e.g. `{app_data}/logs`).
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Appends a single event line to today's `events.log`.
    pub fn log(&self, event: &str) {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M:%S").to_string();

        let day_dir = match ensure_day_dir(&self.base_dir, &date_str) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("event log: cannot create day dir: {}", e);
                return;
            }
        };

        let path = day_dir.join("events.log");
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path);

        match file {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{} {}", time_str, event) {
                    log::warn!("event log: write failed: {}", e);
                }
            }
            Err(e) => {
                log::warn!("event log: open failed: {}", e);
            }
        }
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
        let content = std::fs::read_to_string(tmp.path().join(&today).join("events.log")).unwrap();
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
        let content = std::fs::read_to_string(tmp.path().join(&today).join("events.log")).unwrap();
        let line = content.lines().next().unwrap();
        // Format: HH:MM:SS TYPE details
        assert!(line.contains("DEVICE connected COM3"));
        // Verify time prefix pattern (HH:MM:SS)
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        assert_eq!(parts[0].len(), 8); // HH:MM:SS
    }

    #[test]
    fn event_log_io_error_no_panic() {
        let logger = EventLogger::new(PathBuf::from("/nonexistent/path/that/should/fail"));
        logger.log("START v0.1.0");
        // Should not panic — just warns.
    }
}
