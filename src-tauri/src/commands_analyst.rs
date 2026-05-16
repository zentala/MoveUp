//! commands_analyst.rs — Tauri IPC commands for the Analyst Explorer.
//!
//! Exposes three range-scoped read commands that pull raw historical data
//! from disk (minute snapshots, `events.log`) and SQLite (`sessions`):
//!
//! - [`get_snapshots_range`] — flattened minute snapshots, sorted by `ts`.
//! - [`get_events_range`] — parsed event log lines.
//! - [`get_sessions_range`] — completed session rows.

use std::fs;
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::commands::{ensure_initialized, AppState};
use crate::db::SessionRow;

/// One row per minute snapshot, flattened for charting in the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotRow {
    /// ISO 8601 timestamp when the snapshot was written (UTC).
    pub ts: String,
    /// Desk state at snapshot time (lower-snake e.g. `"sitting"`).
    pub state: String,
    pub sitting_seconds: i64,
    pub standing_seconds: i64,
    pub break_seconds: i64,
    pub desk_height_cm: f32,
    pub idle_secs: i64,
    pub continuous_computer_secs: i64,
    pub position_changes: u32,
    pub daily_score: Option<f32>,
}

/// One parsed line from `events.log`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EventRow {
    /// `"YYYY-MM-DD HH:MM:SS"` — date stitched from the folder, time from the line.
    pub ts: String,
    /// Event kind: `STATE | DEVICE | ALERT | RESET | CREDIT | START | NOTIF | AUTOSTART` (or similar).
    pub kind: String,
    /// Raw rest-of-line after the kind (may be empty).
    pub detail: String,
}

/// Returns flattened snapshot rows across a `[from, to]` inclusive date range.
#[tauri::command]
pub async fn get_snapshots_range(
    app: AppHandle,
    from: String,
    to: String,
) -> Result<Vec<SnapshotRow>, String> {
    let dates = parse_range(&from, &to)?;
    let logs_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("logs");
    Ok(collect_snapshots(&logs_dir, &dates))
}

/// Returns parsed event-log rows across a `[from, to]` inclusive date range.
#[tauri::command]
pub async fn get_events_range(
    app: AppHandle,
    from: String,
    to: String,
) -> Result<Vec<EventRow>, String> {
    let dates = parse_range(&from, &to)?;
    let logs_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("logs");
    Ok(collect_events(&logs_dir, &dates))
}

/// Returns completed session rows across a `[from, to]` inclusive date range.
#[tauri::command]
pub async fn get_sessions_range(
    app: AppHandle,
    state: State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Vec<SessionRow>, String> {
    // Validate inputs early — same parse path as the other range commands.
    let _ = parse_range(&from, &to)?;
    ensure_initialized(&app, &state)?;

    let db_guard = state
        .db
        .lock()
        .map_err(|e| format!("DB mutex poisoned: {}", e))?;
    let conn = db_guard
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    crate::db_queries::get_sessions_range(conn, &from, &to)
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Parses `from`/`to` (`YYYY-MM-DD`) and yields each inclusive date as a `String`.
/// Returns an empty `Vec` when `to < from`.
fn parse_range(from: &str, to: &str) -> Result<Vec<String>, String> {
    let start = NaiveDate::parse_from_str(from, "%Y-%m-%d")
        .map_err(|e| format!("Invalid `from` date '{}': {}", from, e))?;
    let end = NaiveDate::parse_from_str(to, "%Y-%m-%d")
        .map_err(|e| format!("Invalid `to` date '{}': {}", to, e))?;
    if end < start {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let mut cur = start;
    while cur <= end {
        out.push(cur.format("%Y-%m-%d").to_string());
        cur = match cur.succ_opt() {
            Some(d) => d,
            None => break,
        };
    }
    Ok(out)
}

/// Walks `logs_dir/<date>/*.json`, deserializing each into a `SnapshotRow`.
/// Missing days and malformed files are silently skipped (warned via `log`).
fn collect_snapshots(logs_dir: &Path, dates: &[String]) -> Vec<SnapshotRow> {
    let mut out: Vec<SnapshotRow> = Vec::new();
    for date in dates {
        let day_dir = logs_dir.join(date);
        let entries = match fs::read_dir(&day_dir) {
            Ok(e) => e,
            Err(_) => continue, // missing day = nothing to read.
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("analyst: cannot read {}: {}", path.display(), e);
                    continue;
                }
            };
            match serde_json::from_str::<SnapshotRow>(&content) {
                Ok(row) => out.push(row),
                Err(e) => {
                    log::warn!("analyst: skipping malformed {}: {}", path.display(), e);
                }
            }
        }
    }
    out.sort_by(|a, b| a.ts.cmp(&b.ts));
    out
}

/// Walks `logs_dir/<date>/events.log` and parses each line.
/// Malformed lines are skipped silently.
fn collect_events(logs_dir: &Path, dates: &[String]) -> Vec<EventRow> {
    let mut out: Vec<EventRow> = Vec::new();
    for date in dates {
        let path = logs_dir.join(date).join("events.log");
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            if let Some(row) = parse_event_line(date, line) {
                out.push(row);
            }
        }
    }
    out
}

/// Parses one `events.log` line of the form `HH:MM:SS TYPE [details...]`.
/// Returns `None` for malformed lines (missing time or kind).
pub fn parse_event_line(date: &str, line: &str) -> Option<EventRow> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.splitn(3, ' ');
    let time = parts.next()?;
    let kind = parts.next()?;
    // Minimal validation: HH:MM:SS has two colons and 8 chars.
    if time.len() != 8 || time.matches(':').count() != 2 {
        return None;
    }
    if kind.is_empty() {
        return None;
    }
    let detail = parts.next().unwrap_or("").to_string();
    Some(EventRow {
        ts: format!("{} {}", date, time),
        kind: kind.to_string(),
        detail,
    })
}

// Tests live in `commands_analyst_tests.rs` to keep this file ≤250 lines.
// Helpers needed by the sibling test module are exposed via `pub(crate)`.
#[cfg(test)]
pub(crate) use self::testing::*;

#[cfg(test)]
mod testing {
    use super::*;
    pub(crate) fn t_parse_range(from: &str, to: &str) -> Result<Vec<String>, String> {
        parse_range(from, to)
    }
    pub(crate) fn t_collect_snapshots(d: &Path, dates: &[String]) -> Vec<SnapshotRow> {
        collect_snapshots(d, dates)
    }
    pub(crate) fn t_collect_events(d: &Path, dates: &[String]) -> Vec<EventRow> {
        collect_events(d, dates)
    }
}
