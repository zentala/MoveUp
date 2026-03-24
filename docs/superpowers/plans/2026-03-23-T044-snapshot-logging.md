# T044: Minute-by-minute Snapshot Logging + Event Log — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two logging layers — JSON minute snapshots and append-only event log — so developer/agent can diagnose app state at any point in the day.

**Architecture:** Two new structs (`SnapshotLogger`, `EventLogger`) passed as `Arc<T>` through the serial thread call chain. They write to `{app_data_dir}/logs/YYYY-MM-DD/` — snapshots as `HH-MM.json`, events as `events.log`. All I/O errors are warned+skipped, never panic. `lib.rs` is refactored first by extracting `setup_helpers.rs` to stay under 250 lines.

**Tech Stack:** Rust, serde_json, chrono, std::fs, Tauri 2 (AppHandle for app_data_dir)

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `src-tauri/src/setup_helpers.rs` | **Create** ~60 lines | Window positioning + device notification listeners (extracted from lib.rs) |
| `src-tauri/src/snapshot_logger.rs` | **Create** ~100 lines | `SnapshotLogger` struct: write JSON snapshots, `ensure_day_dir`, cleanup old logs |
| `src-tauri/src/event_logger.rs` | **Create** ~50 lines | `EventLogger` struct: append one-line events to `events.log` |
| `src-tauri/src/session_types.rs` | **Modify** L146-150 | Add `break_credit` field to `ReadingResult` |
| `src-tauri/src/session_reading.rs` | **Modify** L54-58, L95-109 | Set `break_credit: None` in early returns, propagate from `handle_state_exit` |
| `src-tauri/src/session_breaks.rs` | **Modify** L11-97 | Return `(Option<CompletedSession>, Option<(BreakCredit, i64)>)` from `handle_state_exit` |
| `src-tauri/src/commands.rs` | **Modify** L27-37 | Add `snapshot_logger` + `event_logger` to `AppState` |
| `src-tauri/src/lib.rs` | **Modify** | Add 3 `mod` declarations, init loggers, call cleanup, use setup_helpers, log START |
| `src-tauri/src/serial.rs` | **Modify** L65-71, L136-141 | Add logger params to `reader_loop`/`scan_and_connect`, log DEVICE events |
| `src-tauri/src/serial_periodic.rs` | **Modify** L16-19, L69-74 | Add logger params, call snapshot every 60s, log STATE/CREDIT/NOTIF/RESET/ALERT events |

---

### Task 1: Extract `setup_helpers.rs` from `lib.rs`

**Files:**
- Create: `src-tauri/src/setup_helpers.rs`
- Modify: `src-tauri/src/lib.rs`

This task frees ~40 lines in lib.rs to make room for logger initialization.

- [ ] **Step 1: Create `setup_helpers.rs` with window positioning function**

```rust
//! setup_helpers.rs — Extracted setup logic from lib.rs.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Listener, Manager};
use tauri_plugin_notification::NotificationExt;

/// Positions the main window in the bottom-right corner of the primary monitor.
pub fn position_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window_vibrancy::apply_acrylic(&window, Some((18, 18, 18, 200)));

        if let Ok(Some(monitor)) = window.current_monitor() {
            let mon = monitor.size();
            let win = window.outer_size().unwrap_or_default();
            let x = mon.width as i32 - win.width as i32 - 16;
            let y = mon.height as i32 - win.height as i32 - 56;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
}

/// Sets up throttled notifications for missing/lost sensor events.
pub fn setup_device_notifications(app: &AppHandle, cooldown_secs: u64) {
    let last_notif = Arc::new(Mutex::new(
        Instant::now() - Duration::from_secs(cooldown_secs),
    ));

    {
        let handle = app.clone();
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-missing", move |_| {
            let mut guard = last.lock().unwrap();
            if guard.elapsed().as_secs() >= cooldown_secs {
                let _ = handle
                    .notification()
                    .builder()
                    .title("zntlDesk")
                    .body("Sensor not connected. Plug in desk sensor.")
                    .show();
                *guard = Instant::now();
            }
        });
    }

    {
        let handle = app.clone();
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-lost", move |_| {
            let mut guard = last.lock().unwrap();
            if guard.elapsed().as_secs() >= cooldown_secs {
                let _ = handle
                    .notification()
                    .builder()
                    .title("zntlDesk")
                    .body("Sensor disconnected. Check USB cable.")
                    .show();
                *guard = Instant::now();
            }
        });
    }
}
```

- [ ] **Step 2: Update `lib.rs` — add mod declaration and use helpers**

In `lib.rs`, add `mod setup_helpers;` after line 38 (after `mod serial_periodic;`).

Replace lines 159-236 (window positioning + device notification listeners) with:

```rust
// Apply Acrylic blur and position to bottom-right corner.
setup_helpers::position_main_window(app.handle());

// Throttled notifications for missing/lost sensor.
setup_helpers::setup_device_notifications(
    app.handle(),
    DEVICE_NOTIFICATION_COOLDOWN_SECS,
);
```

- [ ] **Step 3: Verify compilation**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors.

- [ ] **Step 4: Run existing tests**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test 2>&1 | tail -10`
Expected: all existing tests pass (101+ tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/setup_helpers.rs src-tauri/src/lib.rs
git commit -m "refactor(desk): extract setup_helpers.rs from lib.rs"
```

---

### Task 2: Extend `ReadingResult` with `break_credit` field

**Files:**
- Modify: `src-tauri/src/session_types.rs:146-150`
- Modify: `src-tauri/src/session_reading.rs:54-58,95-109`
- Modify: `src-tauri/src/session_breaks.rs:11-97`

- [ ] **Step 1: Add `break_credit` field to `ReadingResult`**

In `session_types.rs`, change `ReadingResult` (lines 146-150):

```rust
/// Result of processing a sensor reading.
#[derive(Debug, Clone)]
pub struct ReadingResult {
    pub state_change: Option<StateChangedPayload>,
    pub completed_session: Option<CompletedSession>,
    /// Break credit applied this reading (type + standing duration in secs).
    pub break_credit: Option<(BreakCredit, i64)>,
}
```

- [ ] **Step 2: Update `handle_state_exit` to return break credit info**

In `session_breaks.rs`, change the return type of `handle_state_exit` (line 11) to:
```rust
pub(crate) fn handle_state_exit(
    &mut self,
    candidate: &DeskState,
    now: DateTime<Utc>,
) -> (Option<CompletedSession>, Option<(BreakCredit, i64)>)
```

At the top of the function, add: `let mut break_credit_info: Option<(BreakCredit, i64)> = None;`

After each `self.apply_break_credit(break_dur)` call (lines 52 and 73), add:
```rust
break_credit_info = Some((self.state.last_break_credit.clone(), break_dur));
```

Change the return from `completed_session` to `(completed_session, break_credit_info)`.

- [ ] **Step 3: Update `on_reading` in `session_reading.rs` to destructure and propagate**

Early returns (lines 54-58, 65-69) get `break_credit: None`:
```rust
return ReadingResult {
    state_change: None,
    completed_session: None,
    break_credit: None,
};
```

Line 72 changes from:
```rust
let completed_session = self.handle_state_exit(&candidate, now);
```
to:
```rust
let (completed_session, break_credit) = self.handle_state_exit(&candidate, now);
```

Final `ReadingResult` (lines 95-109) adds `break_credit`:
```rust
ReadingResult {
    state_change: Some(StateChangedPayload { ... }),
    completed_session,
    break_credit,
}
```

- [ ] **Step 4: Verify compilation and tests**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test 2>&1 | tail -10`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/session_types.rs src-tauri/src/session_reading.rs src-tauri/src/session_breaks.rs
git commit -m "feat(desk): add break_credit field to ReadingResult"
```

---

### Task 3: Create `snapshot_logger.rs` with tests

**Files:**
- Create: `src-tauri/src/snapshot_logger.rs`

- [ ] **Step 1: Write tests first**

At the bottom of `snapshot_logger.rs`, write the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_types::*;
    use crate::session::SessionStateDto;
    use std::fs;
    use tempfile::TempDir; // NO — don't add dependency. Use std::env::temp_dir + unique subdir

    fn sample_dto() -> SessionStateDto {
        SessionStateDto {
            state: DeskState::Sitting,
            sitting_seconds: 1200,
            standing_seconds: 450,
            break_seconds: 0,
            session_limit_secs: 2700,
            stand_limit_secs: 1800,
            desk_height_cm: 72.5,
            position_changes: 3,
            limit_used_secs: 1200,
            daily_score: 12.5,
            standing_session_secs: 0,
            current_session_secs: 600,
        }
    }

    fn temp_base() -> (PathBuf, String) {
        let id = format!("desk-test-{}", std::process::id());
        let base = std::env::temp_dir().join(id);
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        (base.clone(), base.to_string_lossy().to_string())
    }

    #[test]
    fn snapshot_creates_correct_path() {
        let (base, _) = temp_base();
        let logger = SnapshotLogger::new(base.clone());
        logger.log_snapshot(&sample_dto(), true, Some("COM3".into()), "0.1.0");
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let minute = chrono::Local::now().format("%H-%M").to_string();
        let path = base.join(&today).join(format!("{}.json", minute));
        assert!(path.exists(), "Expected snapshot at {:?}", path);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn snapshot_writes_valid_json() {
        let (base, _) = temp_base();
        let logger = SnapshotLogger::new(base.clone());
        logger.log_snapshot(&sample_dto(), true, Some("COM3".into()), "0.1.0");
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let minute = chrono::Local::now().format("%H-%M").to_string();
        let path = base.join(&today).join(format!("{}.json", minute));
        let content = fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["sitting_seconds"], 1200);
        assert_eq!(v["connected"], true);
        assert_eq!(v["port"], "COM3");
        assert!(v["ts"].as_str().unwrap().len() > 10);
        assert_eq!(v["version"], "0.1.0");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cleanup_deletes_old_keeps_recent() {
        let (base, _) = temp_base();
        let old_date = (chrono::Local::now() - chrono::Duration::days(10))
            .format("%Y-%m-%d").to_string();
        let recent_date = (chrono::Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d").to_string();
        fs::create_dir_all(base.join(&old_date)).unwrap();
        fs::create_dir_all(base.join(&recent_date)).unwrap();
        fs::write(base.join(&old_date).join("test.json"), "{}").unwrap();
        fs::write(base.join(&recent_date).join("test.json"), "{}").unwrap();

        let logger = SnapshotLogger::new(base.clone());
        logger.cleanup_old_logs(7);

        assert!(!base.join(&old_date).exists(), "Old folder should be deleted");
        assert!(base.join(&recent_date).exists(), "Recent folder should remain");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn snapshot_io_error_no_panic() {
        // Write to a path that can't exist
        let logger = SnapshotLogger::new(PathBuf::from("/nonexistent/path/that/cant/work"));
        // Should not panic — just warn
        logger.log_snapshot(&sample_dto(), false, None, "0.1.0");
    }

    #[test]
    fn cleanup_skips_non_date_folders() {
        let (base, _) = temp_base();
        fs::create_dir_all(base.join("random-folder")).unwrap();
        fs::create_dir_all(base.join("not-a-date")).unwrap();
        let logger = SnapshotLogger::new(base.clone());
        logger.cleanup_old_logs(7);
        assert!(base.join("random-folder").exists(), "Non-date folder should be skipped");
        assert!(base.join("not-a-date").exists(), "Non-date folder should be skipped");
        let _ = fs::remove_dir_all(&base);
    }
}
```

- [ ] **Step 2: Implement `SnapshotLogger`**

```rust
//! snapshot_logger.rs — Minute-by-minute JSON snapshot writer.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use log::warn;
use serde::Serialize;

use crate::session::SessionStateDto;

/// Writes periodic JSON snapshots of session state to disk.
pub struct SnapshotLogger {
    base_dir: PathBuf,
}

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
}

impl SnapshotLogger {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Writes a JSON snapshot to `logs/YYYY-MM-DD/HH-MM.json`.
    /// Overwrites if same minute (idempotent). Never panics.
    pub fn log_snapshot(
        &self,
        snapshot: &SessionStateDto,
        connected: bool,
        port: Option<String>,
        version: &str,
    ) {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H-%M").to_string();

        let day_dir = match ensure_day_dir(&self.base_dir, &date_str) {
            Ok(d) => d,
            Err(e) => {
                warn!("snapshot: cannot create day dir: {}", e);
                return;
            }
        };

        let wrapper = SnapshotWrapper {
            ts: chrono::Utc::now().to_rfc3339(),
            session: snapshot.clone(),
            connected,
            port,
            version: version.to_string(),
        };

        let path = day_dir.join(format!("{}.json", time_str));
        match serde_json::to_string_pretty(&wrapper) {
            Ok(json) => {
                if let Err(e) = fs::write(&path, json) {
                    warn!("snapshot write failed: {}", e);
                }
            }
            Err(e) => warn!("snapshot serialize failed: {}", e),
        }
    }

    /// Deletes log folders older than `retention_days`.
    pub fn cleanup_old_logs(&self, retention_days: u64) {
        let entries = match fs::read_dir(&self.base_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("cleanup: cannot read logs dir: {}", e);
                return;
            }
        };

        let today = chrono::Local::now().date_naive();

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let date = match NaiveDate::parse_from_str(&name, "%Y-%m-%d") {
                Ok(d) => d,
                Err(_) => {
                    warn!("cleanup: skipping non-date folder: {}", name);
                    continue;
                }
            };
            let age = (today - date).num_days();
            if age > retention_days as i64 {
                if let Err(e) = fs::remove_dir_all(entry.path()) {
                    warn!("cleanup: failed to remove {}: {}", name, e);
                }
            }
        }
    }
}

/// Creates the day directory (`base/YYYY-MM-DD/`) if it doesn't exist.
pub(crate) fn ensure_day_dir(base: &Path, date: &str) -> io::Result<PathBuf> {
    let day_path = base.join(date);
    fs::create_dir_all(&day_path)?;
    Ok(day_path)
}
```

- [ ] **Step 3: Add `mod snapshot_logger;` to `lib.rs`** (after `mod serial_periodic;`)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test snapshot 2>&1 | tail -15`
Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/snapshot_logger.rs src-tauri/src/lib.rs
git commit -m "feat(desk): add SnapshotLogger with tests"
```

---

### Task 4: Create `event_logger.rs` with tests

**Files:**
- Create: `src-tauri/src/event_logger.rs`

- [ ] **Step 1: Write tests first**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_base() -> PathBuf {
        let id = format!("desk-evttest-{}", std::process::id());
        let base = std::env::temp_dir().join(id);
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn event_log_appends_lines() {
        let base = temp_base();
        let logger = EventLogger::new(base.clone());
        logger.log("STATE Sitting→Standing h=110cm");
        logger.log("CREDIT full dur=600s sitting=1200→0");
        logger.log("NOTIF posture_balance ratio=2.3");

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let content = fs::read_to_string(base.join(&today).join("events.log")).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("STATE"));
        assert!(lines[1].contains("CREDIT"));
        assert!(lines[2].contains("NOTIF"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn event_log_creates_dirs() {
        let base = temp_base();
        let sub = base.join("subdir");
        let logger = EventLogger::new(sub.clone());
        logger.log("START v0.1.0");

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(sub.join(&today).join("events.log").exists());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn event_log_format() {
        let base = temp_base();
        let logger = EventLogger::new(base.clone());
        logger.log("DEVICE connected COM3");

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let content = fs::read_to_string(base.join(&today).join("events.log")).unwrap();
        let line = content.trim();
        // Format: HH:MM:SS DETAILS
        let re = regex::Regex::new(r"^\d{2}:\d{2}:\d{2} .+$").unwrap();
        // No regex dep — just check structure manually
        assert!(line.len() > 8, "Line too short: {}", line);
        assert_eq!(&line[2..3], ":");
        assert_eq!(&line[5..6], ":");
        assert_eq!(&line[8..9], " ");
        assert!(line.contains("DEVICE connected COM3"));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn event_log_io_error_no_panic() {
        let logger = EventLogger::new(PathBuf::from("/nonexistent/path/that/cant/work"));
        logger.log("TEST should not panic");
    }
}
```

- [ ] **Step 2: Implement `EventLogger`**

```rust
//! event_logger.rs — Append-only event log writer.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use log::warn;

use crate::snapshot_logger::ensure_day_dir;

/// Appends timestamped event lines to `logs/YYYY-MM-DD/events.log`.
pub struct EventLogger {
    base_dir: PathBuf,
}

impl EventLogger {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Appends one event line: `HH:MM:SS {event}\n`. Never panics.
    pub fn log(&self, event: &str) {
        let now = chrono::Local::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let time_str = now.format("%H:%M:%S").to_string();

        let day_dir = match ensure_day_dir(&self.base_dir, &date_str) {
            Ok(d) => d,
            Err(e) => {
                warn!("event_log: cannot create day dir: {}", e);
                return;
            }
        };

        let path = day_dir.join("events.log");
        let line = format!("{} {}\n", time_str, event);

        let result = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .and_then(|mut f| f.write_all(line.as_bytes()));

        if let Err(e) = result {
            warn!("event_log write failed: {}", e);
        }
    }
}
```

- [ ] **Step 3: Add `mod event_logger;` to `lib.rs`** (after `mod snapshot_logger;`)

- [ ] **Step 4: Run tests**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test event_log 2>&1 | tail -15`
Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/event_logger.rs src-tauri/src/lib.rs
git commit -m "feat(desk): add EventLogger with tests"
```

---

### Task 5: Wire loggers into AppState, lib.rs, serial.rs, and serial_periodic.rs

**Design decision:** Loggers live in `AppState` as `Arc<SnapshotLogger>` / `Arc<EventLogger>`.
They're created before `.manage()` using `%APPDATA%` env var to compute the logs path
(matches Tauri's `app_data_dir` on Windows: `%APPDATA%/com.zentala.desk/logs`).
Cleanup and START event run in `.setup()`. `port_name` is threaded through to
`check_periodic` so snapshots include the connected port.

**Files:**
- Modify: `src-tauri/src/commands.rs:1-37` — add logger fields to AppState + imports
- Modify: `src-tauri/src/lib.rs` — compute logs_dir, init loggers, cleanup, START event, pass to scan_and_connect
- Modify: `src-tauri/src/serial.rs:65-71,136-192` — add logger + port_name params to reader_loop/scan_and_connect, log DEVICE events
- Modify: `src-tauri/src/serial_periodic.rs` — add logger + port params, write snapshot, log events

- [ ] **Step 1: Add logger fields to `AppState` in `commands.rs`**

Add imports at top of `commands.rs`:
```rust
use crate::snapshot_logger::SnapshotLogger;
use crate::event_logger::EventLogger;
```

Add two fields to `AppState` struct (after `alert_popup`):
```rust
/// Writes JSON snapshots every 60s to logs/ dir.
pub snapshot_logger: Arc<SnapshotLogger>,
/// Appends event lines to logs/ dir.
pub event_logger: Arc<EventLogger>,
```

Update `start_auto_connect` to pass loggers:
```rust
#[tauri::command]
pub fn start_auto_connect(app: tauri::AppHandle, state: State<'_, AppState>) {
    scan_and_connect(
        app,
        state.conn.clone(),
        state.session.clone(),
        state.db.clone(),
        state.config.clone(),
        state.snapshot_logger.clone(),
        state.event_logger.clone(),
    );
}
```

- [ ] **Step 2: Initialize loggers in `lib.rs`**

Add imports to `lib.rs`:
```rust
use std::path::PathBuf;
use snapshot_logger::SnapshotLogger;
use event_logger::EventLogger;
```

Before the `.manage(AppState { ... })` call, compute `logs_dir` and create loggers:
```rust
let logs_dir = std::env::var("APPDATA")
    .map(|d| PathBuf::from(d).join("com.zentala.desk").join("logs"))
    .unwrap_or_else(|_| PathBuf::from("logs"));
let snapshot_logger = Arc::new(SnapshotLogger::new(logs_dir.clone()));
let event_logger = Arc::new(EventLogger::new(logs_dir));
```

Add to `.manage(AppState { ... })`:
```rust
snapshot_logger: snapshot_logger.clone(),
event_logger: event_logger.clone(),
```

In `.setup()`, after `app_data_dir` creation, add cleanup + START event:
```rust
{
    let state: tauri::State<'_, AppState> = app.state();
    state.snapshot_logger.cleanup_old_logs(7);
    state.event_logger.log(&format!("START v{}", env!("CARGO_PKG_VERSION")));
}
```

Update `scan_and_connect` call in `.setup()`:
```rust
serial::scan_and_connect(
    app.handle().clone(),
    state.conn.clone(),
    state.session.clone(),
    state.db.clone(),
    state.config.clone(),
    state.snapshot_logger.clone(),
    state.event_logger.clone(),
);
```

- [ ] **Step 3: Update `serial.rs` — add logger params + DEVICE events**

Add imports to `serial.rs`:
```rust
use crate::snapshot_logger::SnapshotLogger;
use crate::event_logger::EventLogger;
```

Update `reader_loop` signature to accept loggers + `port_name` is already a param:
```rust
fn reader_loop(
    app: &AppHandle,
    port_name: &str,
    stop: &Arc<AtomicBool>,
    session: &Arc<Mutex<SessionManager>>,
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    config: &crate::config::AppConfig,
    snapshot_logger: &SnapshotLogger,
    event_logger: &EventLogger,
) {
```

Inside `reader_loop`, after `let _ = app.emit("desk:device-lost", ());` (line 97), add:
```rust
event_logger.log("DEVICE lost");
```

Update `handle_reading` and `check_periodic` calls to pass loggers + port:
```rust
handle_reading(app, mm, session, db, config, event_logger);

if last_periodic.elapsed() >= Duration::from_secs(60) {
    check_periodic(app, session, config, snapshot_logger, event_logger, port_name);
    last_periodic = std::time::Instant::now();
}
```

Update `scan_and_connect` signature:
```rust
pub fn scan_and_connect(
    app: AppHandle,
    conn: Arc<ConnectionState>,
    session: Arc<Mutex<SessionManager>>,
    db: Arc<Mutex<Option<rusqlite::Connection>>>,
    config: Arc<Mutex<Option<crate::config::AppConfig>>>,
    snapshot_logger: Arc<SnapshotLogger>,
    event_logger: Arc<EventLogger>,
) {
```

Inside the spawned thread, clone Arcs before the loop:
```rust
let snap = snapshot_logger;
let evt = event_logger;
```

After `"Desk sensor found on {port_name}"` log and device-connected emit, add:
```rust
evt.log(&format!("DEVICE connected {}", port_name));
```

Pass loggers to `reader_loop`:
```rust
reader_loop(&app, &port_name, &stop, &session, &db, &cfg, &snap, &evt);
```

- [ ] **Step 4: Update `serial_periodic.rs` — full rewrite with loggers**

Add imports:
```rust
use crate::snapshot_logger::SnapshotLogger;
use crate::event_logger::EventLogger;
```

Update `check_periodic` to accept loggers + port_name, write snapshot:
```rust
pub fn check_periodic(
    app: &AppHandle,
    session: &Arc<Mutex<SessionManager>>,
    config: &crate::config::AppConfig,
    snapshot_logger: &SnapshotLogger,
    event_logger: &EventLogger,
    port_name: &str,
) {
    let daily_reset_occurred = {
        let mut sess = session.lock().unwrap();
        sess.check_daily_reset()
    };

    if daily_reset_occurred {
        info!("Daily reset occurred — in-memory counters cleared");
        let _ = app.emit("desk:daily-reset", ());
        event_logger.log("RESET daily");
    }

    // Write minute snapshot — sensor IS connected (we're in reader_loop)
    {
        let sess = session.lock().unwrap();
        let dto = sess.snapshot();
        snapshot_logger.log_snapshot(
            &dto,
            true,
            Some(port_name.to_string()),
            env!("CARGO_PKG_VERSION"),
        );
    }

    let notification_events = {
        let mut sess = session.lock().unwrap();
        sess.check_notification_conditions(config)
    };

    for event in &notification_events {
        match event {
            NotificationEvent::Inactivity => {
                event_logger.log("NOTIF inactivity");
                let _ = app.notification().builder()
                    .title("No position change in 60 minutes")
                    .body("Time to move.")
                    .show();
            }
            NotificationEvent::PostureBalance => {
                let sess = session.lock().unwrap();
                let ratio = if sess.state.standing_seconds > 0 {
                    sess.state.sitting_seconds as f32 / sess.state.standing_seconds as f32
                } else {
                    f32::INFINITY
                };
                event_logger.log(&format!("NOTIF posture_balance ratio={:.1}", ratio));
                let _ = app.notification().builder()
                    .title("You've been sitting most of today")
                    .body("Consider standing for a while.")
                    .show();
            }
            NotificationEvent::Praise => {
                event_logger.log("NOTIF praise_halfway");
                let _ = app.notification().builder()
                    .title("Halfway through your standing goal!")
                    .body("Keep it up.")
                    .show();
            }
            NotificationEvent::StandLimitReached => {}
            NotificationEvent::StandingTargetReached => {
                event_logger.log("NOTIF standing_target_reached");
                let _ = app.notification().builder()
                    .title("Standing target reached!")
                    .body("Great break! You stood for the full target duration.")
                    .show();
            }
        }
    }
}
```

Update `handle_reading` to accept `EventLogger`:
```rust
pub fn handle_reading(
    app: &AppHandle,
    mm: i32,
    session: &Arc<Mutex<SessionManager>>,
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    config: &crate::config::AppConfig,
    event_logger: &EventLogger,
) {
    let active = is_active();
    let (state_before, result) = {
        let mut sess = session.lock().unwrap();
        let before = sess.current_state();
        let res = sess.on_reading(mm, active);
        sess.accumulate_score_tick(config);
        (before, res)
    };

    if let Some(ref payload) = result.state_change {
        event_logger.log(&format!(
            "STATE {:?}→{:?} h={:.0}cm",
            state_before, payload.state, payload.desk_height_cm
        ));
        let _ = app.emit("desk:state-changed", payload);

        if state_before == DeskState::Sitting && payload.state == DeskState::Standing {
            let praise = {
                let mut sess = session.lock().unwrap();
                sess.should_send_praise_halfway(config)
            };
            if praise {
                let _ = app.notification().builder()
                    .title("Halfway through your standing goal!")
                    .body("Keep it up.")
                    .show();
            }
        }
    }

    if let Some((ref credit, dur)) = result.break_credit {
        let sitting_after = session.lock().unwrap().state.sitting_seconds;
        event_logger.log(&format!(
            "CREDIT {:?} dur={}s sitting→{}",
            credit, dur, sitting_after
        ));
    }

    if let Some(ref completed) = result.completed_session {
        info!("Completed session: {:?}", completed);
        let db_lock = db.lock().unwrap();
        if let Some(ref conn) = *db_lock {
            if let Err(e) = crate::db_sessions::insert_session(
                conn,
                &completed.started_at,
                &completed.ended_at,
                &format!("{:?}", state_before),
                completed.duration_secs,
            ) {
                error!("Failed to save completed session: {}", e);
            }
        }
    }

    let alert = { session.lock().unwrap().should_alert() };
    if alert {
        let sitting = session.lock().unwrap().state.sitting_seconds;
        event_logger.log(&format!("ALERT sit_limit sitting={}s", sitting));
        let _ = app.notification().builder()
            .title("Time to stand up!")
            .body("You've been sitting for 40 minutes. Take a break.")
            .show();
    }

    let stand_alert = { session.lock().unwrap().should_stand_alert() };
    if stand_alert {
        event_logger.log("ALERT stand_limit");
        let _ = app.notification().builder()
            .title("You've been standing a while")
            .body("Ready to sit down for a bit?")
            .show();
    }
}
```

- [ ] **Step 5: Verify full compilation**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo check 2>&1 | tail -10`
Expected: `Finished` with no errors.

- [ ] **Step 6: Run all tests**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test 2>&1 | tail -15`
Expected: all tests pass (existing 101+ tests + 9 new tests).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/serial.rs src-tauri/src/serial_periodic.rs
git commit -m "feat(desk): wire snapshot + event loggers through serial chain"
```

---

### Task 6: Final verification and cleanup

**Files:** none new — verification only

- [ ] **Step 1: Run full test suite**

Run: `cd /c/code/zntl-tray/apps/desk/src-tauri && cargo test 2>&1`
Expected: all tests pass. Note exact count (should be ~110+).

- [ ] **Step 2: Check lib.rs line count**

Run: `wc -l /c/code/zntl-tray/apps/desk/src-tauri/src/lib.rs`
Expected: ≤250 lines.

- [ ] **Step 3: Check new file line counts**

Run: `wc -l /c/code/zntl-tray/apps/desk/src-tauri/src/{snapshot_logger,event_logger,setup_helpers}.rs`
Expected: snapshot_logger ~100-130, event_logger ~50-70, setup_helpers ~60.

- [ ] **Step 4: Update task status**

Edit `.claude/tasks/T044-snapshot-logging.md`: change `status: todo` to `status: done`.

- [ ] **Step 5: Final commit**

```bash
git add .claude/tasks/T044-snapshot-logging.md
git commit -m "chore(desk): mark T044 snapshot logging as done"
```
