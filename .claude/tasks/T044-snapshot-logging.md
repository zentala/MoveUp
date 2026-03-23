---
id: T044
status: todo
created: 2026-03-23
reviewed: 2026-03-23
review: CEO + Eng review completed, all decisions locked
---
# T044: Minute-by-minute snapshot logging + event log

## Goal

Two logging layers so the developer (and Claude agent) can diagnose
what happened at any point in the day — what data the app had and
what decisions it made.

**Motivation:** Recent git history shows 6+ debugging sessions (standing timer zero,
connected flag wrong, serde case mismatch) that would have been trivial to diagnose
with persistent state snapshots.

## Architecture Overview

```
                    ┌──────────────┐
                    │   lib.rs     │
                    │  (setup)     │
                    └──┬───┬───┬──┘
                       │   │   │
            init       │   │   │  init
         ┌─────────────┘   │   └──────────────┐
         ▼                 │                   ▼
  ┌──────────────┐         │          ┌──────────────┐
  │SnapshotLogger│         │          │ EventLogger  │
  │  (new file)  │         │          │  (new file)  │
  └──────┬───────┘         │          └──────┬───────┘
         │                 │                 │
         │  called from    │   called from   │
         ▼                 ▼                 ▼
  ┌─────────────────────────────────────────────┐
  │          serial_periodic.rs                 │
  │  check_periodic() → snapshot every 60s      │
  │  handle_reading() → event on state change   │
  └─────────────────────────────────────────────┘
         │                                │
  ┌──────┴──────┐                  ┌──────┴──────┐
  │ serial.rs   │                  │ session.rs  │
  │ device evt  │                  │  snapshot() │
  └─────────────┘                  └─────────────┘
```

**Key decision:** Loggers are passed as `Arc<SnapshotLogger>` and `Arc<EventLogger>`
through the call chain: `scan_and_connect` → `reader_loop` → `check_periodic`/`handle_reading`.
They do NOT go through `AppState` — they follow the existing pattern of cloning individual
`Arc`s into the serial thread.

**Snapshots only when sensor active.** When sensor is disconnected, no snapshots are written.
Absence of snapshot files = sensor was disconnected. Implicit signal.

## Layer 1: Minute snapshots (JSON files)

Every 60 seconds, dump `SessionManager::snapshot()` + wrapper to a JSON file.

### Directory structure

```
{app_data_dir}/logs/
  2026-03-23/
    14-30.json
    14-31.json
    ...
  2026-03-24/
    ...
```

- One folder per day: `YYYY-MM-DD/`
- One file per minute: `HH-MM.json`
- Max files per day: 1440 (24 x 60)
- Each file ~500 bytes → ~720 KB/day → ~5 MB/week (7-day retention)

### Snapshot content — DTO wrapper approach

**Decision (CEO review):** Serialize `SessionStateDto` directly via serde, wrapped in a
struct that adds metadata fields. NO custom snapshot struct — prevents field drift
(which caused bugs in T040, standing timer serde mismatch, etc).

```rust
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
```

Example output:
```json
{
  "ts": "2026-03-23T14:30:00Z",
  "state": "Standing",
  "desk_height_cm": 84.0,
  "sitting_seconds": 1200,
  "standing_seconds": 450,
  "break_seconds": 120,
  "current_session_secs": 0,
  "session_limit_secs": 2700,
  "stand_limit_secs": 1800,
  "limit_used_secs": 1200,
  "position_changes": 3,
  "daily_score": 12.5,
  "standing_session_secs": 450,
  "connected": true,
  "port": "COM3",
  "version": "0.1.0"
}
```

Note: `SessionStateDto` fields are flattened into the root. When DTO gains new fields,
snapshots automatically include them — zero maintenance.

### Implementation

1. **New file: `src-tauri/src/snapshot_logger.rs`** (~80 lines)
   - `pub struct SnapshotLogger { base_dir: PathBuf }`
   - `pub fn new(base_dir: PathBuf) -> Self`
   - `pub fn log_snapshot(&self, snapshot: &SessionStateDto, connected: bool, port: Option<String>, version: &str)`
     - Calls `ensure_day_dir()` → serialize `SnapshotWrapper` → `fs::write(HH-MM.json)`
     - Overwrites if same minute (idempotent)
   - `pub(crate) fn ensure_day_dir(base: &Path, date: &str) -> io::Result<PathBuf>`
     - Shared helper, also used by `EventLogger`
     - `create_dir_all(base.join(date))` → returns the day path
   - `pub fn cleanup_old_logs(&self, retention_days: u64)`
     - `read_dir(base_dir)` → parse folder name as `NaiveDate`
     - Age > retention_days → `remove_dir_all`
     - Non-date folder → `log::warn`, skip
     - Any `io::Error` → `log::warn`, skip (never panic)

2. **Call site: `serial_periodic.rs::check_periodic()`**
   - Add `snapshot_logger: &SnapshotLogger` param
   - After daily reset check, call `snapshot_logger.log_snapshot()`
   - Pass `session.lock().unwrap().snapshot()` + connection info

3. **Loggers in `AppState` (for initialization only)**
   - Add `snapshot_logger: Arc<SnapshotLogger>` and `event_logger: Arc<EventLogger>` to `AppState`
   - Initialize in `lib.rs` setup with `app_data_dir.join("logs")`
   - Clone `Arc`s into `scan_and_connect` call chain

### Cleanup

- **Runs once on startup** in `lib.rs` setup (NOT every 60s in `check_periodic`)
- 7-day retention: `cleanup_old_logs(7)`
- Handles edge cases:
  - Missing `logs/` dir → `read_dir` returns error → warn, skip
  - Non-date folder names → skip with warning
  - `remove_dir_all` fails (folder in use) → warn, skip

```
  App startup
      │
      ▼
  read_dir(logs/)
      │
      ├──▶ entry is valid date folder
      │        ├── age ≤ 7 days → keep
      │        └── age > 7 days → remove_dir_all → warn on error
      │
      └──▶ NOT a date folder → skip (warn)
```

## Layer 2: Event log (append-only text file)

One log file per day, append-only, human-readable lines.

### File path

```
{app_data_dir}/logs/2026-03-23/events.log
```

### What gets logged

| Event | Format | Call site |
|-------|--------|-----------|
| State transition | `14:30:01 STATE Sitting→Standing h=110cm` | `handle_reading()` |
| Break credit | `14:40:15 CREDIT partial dur=305s sitting=1200→0` | `handle_reading()` |
| Notification fired | `14:30:01 NOTIF posture_balance ratio=2.3` | `check_periodic()` |
| Device connected | `14:00:05 DEVICE connected COM3` | `serial.rs` |
| Device lost | `15:20:00 DEVICE lost` | `serial.rs` |
| Daily reset | `00:00:03 RESET daily` | `check_periodic()` |
| Alert fired | `14:30:01 ALERT sit_limit sitting=2700s` | `handle_reading()` |
| App start | `08:30:00 START v0.1.0` | `lib.rs` setup |

### Implementation

1. **New file: `src-tauri/src/event_logger.rs`** (~50 lines)
   - `pub struct EventLogger { base_dir: PathBuf }`
   - `pub fn new(base_dir: PathBuf) -> Self`
   - `pub fn log(&self, event: &str)`
     - Imports `ensure_day_dir` from `snapshot_logger`
     - `OpenOptions::new().append(true).create(true).open(events.log)`
     - `writeln!(file, "{} {}", timestamp_hms, event)`
     - Any `io::Error` → `log::warn`, skip (never panic)

2. **Call sites (one-liner calls):**
   - `serial_periodic.rs::handle_reading()` — state transitions, alerts, break credits
   - `serial_periodic.rs::check_periodic()` — notifications, daily reset
   - `serial.rs` — device connected/lost (in `scan_and_connect` closure)
   - `lib.rs` setup — `START v{version}` on app launch

## Error handling — CRITICAL RULE

**Logging must NEVER crash the app or block the sensor loop.**

Every I/O operation in both loggers:
- Catches all `io::Error` and `serde_json::Error`
- Logs via `log::warn("snapshot write failed: {}", e)` (Rust stderr logging)
- Continues execution — no `unwrap()`, no `expect()`, no `?` propagation
- Uses `if let Err(e) = ...` pattern

Windows-specific: `OpenOptions::append` can fail with sharing violation if
antivirus is scanning the file. Same pattern: warn + skip.

## lib.rs refactor — extract setup_helpers.rs

**Decision (CEO review):** `lib.rs` is at 250 lines (project max). Adding logger init
would exceed this. Extract window positioning + notification listener setup into
`setup_helpers.rs` (~40 lines freed).

**New file: `src-tauri/src/setup_helpers.rs`** (~60 lines)
Move from `lib.rs`:
- Window positioning code (lines 160-171): `pub fn position_main_window(app: &AppHandle)`
- Device notification listeners (lines 203-236): `pub fn setup_device_notifications(app: &AppHandle)`

## Threading — how loggers reach the serial thread

Current call chain:
```
lib.rs setup
  → scan_and_connect(app, conn, session, db, config)  // serial.rs:136
    → thread::spawn
      → reader_loop(app, port, stop, session, db, config)  // serial.rs:65
        → handle_reading(app, mm, session, db, config)      // serial_periodic.rs:69
        → check_periodic(app, session, config)               // serial_periodic.rs:16
```

**After T044** — add 2 params to each function in the chain:
```
lib.rs setup
  → scan_and_connect(app, conn, session, db, config, snapshot_logger, event_logger)
    → thread::spawn (clone Arcs)
      → reader_loop(app, port, stop, session, db, config, snapshot_logger, event_logger)
        → handle_reading(app, mm, session, db, config, event_logger)
        → check_periodic(app, session, config, snapshot_logger, event_logger)
```

Functions that only need EventLogger don't get SnapshotLogger (minimal params).

## ReadingResult extension for CREDIT logging

**Issue found in 3rd review:** The CREDIT event requires break credit data, but `ReadingResult`
(session_types.rs:147) only has `state_change` and `completed_session` — no break credit info.

**Fix:** Add `break_credit: Option<(BreakCredit, i64)>` to `ReadingResult`.
- `session_reading.rs` sets it when `apply_break_credit()` is called (already in `on_reading()`)
- `handle_reading()` checks it and logs `CREDIT {type} dur={secs}s`

```rust
// session_types.rs — add to ReadingResult
pub struct ReadingResult {
    pub state_change: Option<StateChangedPayload>,
    pub completed_session: Option<CompletedSession>,
    /// Break credit applied this reading (type + standing duration in secs).
    pub break_credit: Option<(BreakCredit, i64)>,  // NEW
}
```

This adds ~3 lines to `session_reading.rs` (set the field) and ~5 lines to
`serial_periodic.rs::handle_reading()` (check + log).

## Files to create/modify

| File | Action | Lines est. |
|------|--------|-----------|
| `src-tauri/src/snapshot_logger.rs` | **Create** — snapshot JSON writer + `ensure_day_dir` + cleanup | ~100 |
| `src-tauri/src/event_logger.rs` | **Create** — append-only event log | ~50 |
| `src-tauri/src/setup_helpers.rs` | **Create** — extracted from lib.rs (window pos + notif listeners) | ~60 |
| `src-tauri/src/lib.rs` | Add 3 `mod` declarations, init loggers, call cleanup, log START, use setup_helpers | ~-20 net |
| `src-tauri/src/commands.rs` | Add `snapshot_logger` + `event_logger` to `AppState` struct | ~+5 |
| `src-tauri/src/serial.rs` | Add logger params to `scan_and_connect`/`reader_loop`, log DEVICE events | ~+15 |
| `src-tauri/src/serial_periodic.rs` | Add logger params, call snapshot + event logger, log CREDIT | ~+25 |
| `src-tauri/src/session_types.rs` | Add `break_credit` field to `ReadingResult` | ~+3 |
| `src-tauri/src/session_reading.rs` | Set `break_credit` in `on_reading()` return | ~+3 |

**Already committed** (5e9e1c0, no changes needed):
- `apps/desk/CLAUDE.md` — Logging & Debugging section
- `apps/desk/.claude/rules/logging.md` — logging reference doc

## Tests

### Unit tests (~6 tests in `snapshot_logger.rs` `#[cfg(test)]` module)

**Happy path:**
1. `snapshot_creates_correct_path` — verify `logs/YYYY-MM-DD/HH-MM.json` path
2. `snapshot_writes_valid_json` — write snapshot, read back, verify all DTO fields present + wrapper fields (ts, connected, port, version)
3. `cleanup_deletes_old_keeps_recent` — create folders dated 1-day and 10-day ago, run cleanup, verify old deleted and recent kept

**Error paths:**
4. `snapshot_io_error_no_panic` — write to read-only directory, verify no panic (just returns)
5. `cleanup_skips_non_date_folders` — put a "random-folder" in logs/, verify it's not deleted and no panic
6. `event_log_io_error_no_panic` — append to read-only path, verify no panic

### Unit tests (~3 tests in `event_logger.rs` `#[cfg(test)]` module)

7. `event_log_appends_lines` — log 3 events, read file, verify 3 lines with correct format
8. `event_log_creates_dirs` — log to non-existent day dir, verify dir + file created
9. `event_log_format` — verify `HH:MM:SS TYPE details` format

**No integration tests needed** — pure file I/O, no IPC, no sensor.

## Acceptance criteria

- [ ] JSON snapshot every 60s in `logs/YYYY-MM-DD/HH-MM.json`
- [ ] Snapshot uses `SessionStateDto` directly (serde flatten) + wrapper (ts, connected, port, version)
- [ ] Event log appends to `logs/YYYY-MM-DD/events.log`
- [ ] All 8 event types logged (STATE, CREDIT, NOTIF, DEVICE, ALERT, RESET, START, DEVICE lost)
- [ ] Old logs cleaned up after 7 days (runs once on startup)
- [ ] All I/O errors → `log::warn` + skip (never panic, never block sensor)
- [ ] `setup_helpers.rs` extracted from lib.rs (lib.rs stays ≤250 lines)
- [ ] 9 unit tests pass (6 snapshot/cleanup + 3 event logger)
- [ ] `cargo test` passes
- [ ] CLAUDE.md + logging.md already committed ✓

## Layer 3: IPC for agent/debug access (DEFERRED)

NOT in scope for T044. Can be added later if debug panel needs to read logs.

```rust
#[tauri::command]
fn get_latest_snapshot(state: State<AppState>) -> Option<serde_json::Value>

#[tauri::command]
fn get_today_events(state: State<AppState>) -> Result<String, String>
```

## Implementation order (for orchestrator)

Recommended execution as a single task, ~2-3 hours:

1. **Create `setup_helpers.rs`** — extract from lib.rs first (unblocks space for logger init)
2. **Extend `ReadingResult`** — add `break_credit: Option<(BreakCredit, i64)>` to session_types.rs, set it in session_reading.rs
3. **Create `snapshot_logger.rs`** — struct + `log_snapshot` + `ensure_day_dir` + `cleanup_old_logs` + tests
4. **Create `event_logger.rs`** — struct + `log` + tests (imports `ensure_day_dir`)
5. **Wire `lib.rs`** — add mods, init loggers in AppState, call cleanup on startup, log START event
6. **Wire `commands.rs`** — add logger fields to AppState
7. **Wire `serial.rs`** — add logger params to `scan_and_connect`/`reader_loop`, log DEVICE events
8. **Wire `serial_periodic.rs`** — add logger params, snapshot in `check_periodic`, events in `handle_reading` (STATE, ALERT, CREDIT, NOTIF, RESET)
9. **Run `cargo test`** — verify all 9+ new tests + existing tests pass
10. **Manual verify** — `pnpm tauri:dev`, wait 60s, check `AppData/Roaming/com.zentala.desk/logs/`
