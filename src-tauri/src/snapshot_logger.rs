//! snapshot_logger.rs — Minute-by-minute JSON snapshot writer.
//!
//! Writes `SessionStateDto` snapshots to `{base_dir}/YYYY-MM-DD/HH-MM.json`.
//! Cleans up log directories older than a configurable retention period.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate, Utc};
use serde::Serialize;

use crate::metrics::MetricSnapshot;
use crate::session::SessionStateDto;

/// Thin wrapper around SessionStateDto for JSON snapshot files.
#[derive(Serialize)]
struct SnapshotWrapper {
    /// ISO 8601 timestamp of when snapshot was taken.
    ts: String,
    /// All session state fields — serialized directly from DTO.
    #[serde(flatten)]
    session: SessionStateDto,
    /// Whether sensor is currently connected.
    connected: bool,
    /// Connected serial port name (e.g. "COM3"), or null.
    port: Option<String>,
    /// App version for format compatibility tracking.
    version: String,
    /// KPI metric snapshots (added by E001-T07).
    metrics: Vec<MetricSnapshot>,
}

/// Writes per-minute JSON snapshots of session state.
pub struct SnapshotLogger {
    base_dir: PathBuf,
}

impl SnapshotLogger {
    /// Creates a new logger writing to `base_dir` (e.g. `{app_data}/logs`).
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Writes a snapshot JSON file for the current minute.
    pub fn log_snapshot(
        &self,
        snapshot: &SessionStateDto,
        connected: bool,
        port: Option<String>,
        version: &str,
        metrics: Vec<MetricSnapshot>,
    ) {
        let now = Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let file_name = now.format("%H-%M.json").to_string();

        let day_dir = match ensure_day_dir(&self.base_dir, &date_str) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("snapshot: cannot create day dir: {}", e);
                return;
            }
        };

        let wrapper = SnapshotWrapper {
            ts: Utc::now().to_rfc3339(),
            session: snapshot.clone(),
            connected,
            port,
            version: version.to_string(),
            metrics,
        };

        let json = match serde_json::to_string_pretty(&wrapper) {
            Ok(j) => j,
            Err(e) => {
                log::warn!("snapshot: serialize failed: {}", e);
                return;
            }
        };

        if let Err(e) = fs::write(day_dir.join(&file_name), json) {
            log::warn!("snapshot: write failed: {}", e);
        }
    }

    /// Deletes log directories older than `retention_days`.
    pub fn cleanup_old_logs(&self, retention_days: u64) {
        let entries = match fs::read_dir(&self.base_dir) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("snapshot cleanup: cannot read logs dir: {}", e);
                return;
            }
        };

        let today = Local::now().date_naive();

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            let date = match NaiveDate::parse_from_str(&name_str, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    log::warn!("snapshot cleanup: skipping non-date folder: {}", name_str);
                    continue;
                }
            };

            let age_days = (today - date).num_days();
            if age_days > retention_days as i64 {
                if let Err(e) = fs::remove_dir_all(entry.path()) {
                    log::warn!("snapshot cleanup: failed to remove {}: {}", name_str, e);
                }
            }
        }
    }
}

/// Creates `{base}/YYYY-MM-DD/` directory if it doesn't exist.
pub(crate) fn ensure_day_dir(base: &Path, date: &str) -> io::Result<PathBuf> {
    let day_path = base.join(date);
    fs::create_dir_all(&day_path)?;
    Ok(day_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::SessionManager;
    use tempfile::TempDir;

    fn make_snapshot() -> SessionStateDto {
        SessionManager::new().snapshot()
    }

    #[test]
    fn snapshot_creates_correct_path() {
        let tmp = TempDir::new().unwrap();
        let logger = SnapshotLogger::new(tmp.path().to_path_buf());
        logger.log_snapshot(&make_snapshot(), true, Some("COM3".into()), "0.1.0", Vec::new());

        let today = Local::now().format("%Y-%m-%d").to_string();
        let day_dir = tmp.path().join(&today);
        assert!(day_dir.exists(), "day dir should exist");

        let minute_file = Local::now().format("%H-%M.json").to_string();
        assert!(day_dir.join(&minute_file).exists(), "minute file should exist");
    }

    #[test]
    fn snapshot_writes_valid_json() {
        let tmp = TempDir::new().unwrap();
        let logger = SnapshotLogger::new(tmp.path().to_path_buf());
        logger.log_snapshot(&make_snapshot(), true, Some("COM3".into()), "0.1.0", Vec::new());

        let today = Local::now().format("%Y-%m-%d").to_string();
        let minute = Local::now().format("%H-%M.json").to_string();
        let content = fs::read_to_string(tmp.path().join(&today).join(&minute)).unwrap();
        let val: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(val.get("ts").is_some());
        assert!(val.get("connected").is_some());
        assert!(val.get("version").is_some());
        assert!(val.get("state").is_some());
        assert!(val.get("sitting_seconds").is_some());
    }

    #[test]
    fn cleanup_deletes_old_keeps_recent() {
        let tmp = TempDir::new().unwrap();
        let logger = SnapshotLogger::new(tmp.path().to_path_buf());

        let today = Local::now().date_naive();
        let old_date = (today - chrono::Duration::days(10)).format("%Y-%m-%d").to_string();
        let recent_date = (today - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();

        fs::create_dir_all(tmp.path().join(&old_date)).unwrap();
        fs::create_dir_all(tmp.path().join(&recent_date)).unwrap();

        logger.cleanup_old_logs(7);

        assert!(!tmp.path().join(&old_date).exists(), "old dir should be deleted");
        assert!(tmp.path().join(&recent_date).exists(), "recent dir should be kept");
    }

    #[test]
    fn snapshot_io_error_no_panic() {
        let logger = SnapshotLogger::new(PathBuf::from("/nonexistent/path/that/should/fail"));
        logger.log_snapshot(&make_snapshot(), false, None, "0.1.0", Vec::new());
        // Should not panic — just warns.
    }

    #[test]
    fn cleanup_skips_non_date_folders() {
        let tmp = TempDir::new().unwrap();
        let logger = SnapshotLogger::new(tmp.path().to_path_buf());

        fs::create_dir_all(tmp.path().join("random-folder")).unwrap();
        logger.cleanup_old_logs(7);

        assert!(tmp.path().join("random-folder").exists(), "non-date folder should be kept");
    }

    #[test]
    fn ensure_day_dir_creates_nested() {
        let tmp = TempDir::new().unwrap();
        let result = ensure_day_dir(tmp.path(), "2026-03-23");
        assert!(result.is_ok());
        assert!(tmp.path().join("2026-03-23").exists());
    }
}
