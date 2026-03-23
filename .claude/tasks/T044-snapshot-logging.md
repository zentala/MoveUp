---
id: T044
status: todo
created: 2026-03-23
---
# T044: Minute-by-minute snapshot logging + event log

## Goal

Two logging layers so the developer (and Claude agent) can diagnose
what happened at any point in the day — what data the app had and
what decisions it made.

## Layer 1: Minute snapshots (JSON files)

Every 60 seconds, dump `SessionManager::snapshot()` + extras to a JSON file.

### Directory structure

```
{app_data_dir}/logs/
  2026-03-23/
    14-30.json
    14-31.json
    14-32.json
    ...
  2026-03-24/
    ...
```

- One folder per day: `YYYY-MM-DD/`
- One file per minute: `HH-MM.json`
- Max files per day: 1440 (24 × 60)
- Each file ~500 bytes → ~720 KB/day → ~22 MB/month

### Snapshot content

```json
{
  "ts": "2026-03-23T14:30:00Z",
  "state": "Standing",
  "desk_height_cm": 84,
  "sitting_seconds": 1200,
  "standing_seconds": 450,
  "break_seconds": 120,
  "current_session_secs": 0,
  "limit_used_secs": 1200,
  "limit_remaining_secs": 1500,
  "session_limit_secs": 2700,
  "position_changes": 3,
  "daily_score": 12.5,
  "connected": true,
  "port": "COM3"
}
```

### Implementation

1. New file: `src-tauri/src/snapshot_logger.rs`
   - `SnapshotLogger` struct holding `base_dir: PathBuf`
   - `pub fn log_snapshot(&self, snapshot: &SessionStateDto, port: Option<String>)`
   - Creates day folder if missing
   - Writes JSON file with serde_json
   - Overwrites if same minute (idempotent)

2. Call site: `serial_periodic.rs` `check_periodic()` (already runs every ~60s)
   - After daily reset check, call `snapshot_logger.log_snapshot()`
   - Pass `session.lock().unwrap().snapshot()` + port from `conn.connected_port`

3. Add `SnapshotLogger` to `AppState` in `commands.rs`
   - Initialize with `app_data_dir.join("logs")`

### Cleanup

- Delete folders older than 7 days on startup (`check_daily_reset`)
- Simple: `fs::read_dir` → filter by date → `fs::remove_dir_all`

## Layer 2: Event log (append-only text file)

One log file per day, append-only, human-readable lines.

### File path

```
{app_data_dir}/logs/2026-03-23/events.log
```

### What gets logged

| Event | Format |
|-------|--------|
| State transition | `14:30:01 STATE Sitting→Standing h=110cm` |
| Break credit | `14:40:15 CREDIT partial dur=305s sitting=1200→0` |
| Notification fired | `14:30:01 NOTIF posture_balance ratio=2.3` |
| Device connected | `14:00:05 DEVICE connected COM3` |
| Device lost | `15:20:00 DEVICE lost` |
| Daily reset | `00:00:03 RESET daily` |
| Alert fired | `14:30:01 ALERT sit_limit sitting=2700s` |
| App start | `08:30:00 START v0.1.0` |

### Implementation

1. New file: `src-tauri/src/event_logger.rs`
   - `EventLogger` struct holding `base_dir: PathBuf`
   - `pub fn log(&self, event: &str)` — appends timestamped line
   - Creates day folder + file if missing
   - Uses `OpenOptions::append(true).create(true)`

2. Call sites (add one-liner calls):
   - `serial_periodic.rs` `handle_reading()` — state transitions, alerts
   - `serial_periodic.rs` `check_periodic()` — notifications, daily reset
   - `serial.rs` — device connected/lost
   - `lib.rs` `run()` — app start

3. Add `EventLogger` to `AppState`

## Layer 3: IPC for agent/debug access

Two new commands so the debug panel (or Claude agent via inject) can read logs:

```rust
#[tauri::command]
fn get_latest_snapshot(state: State<AppState>) -> Option<serde_json::Value>

#[tauri::command]
fn get_today_events(state: State<AppState>) -> Result<String, String>
```

Optional — can be added later if needed.

## Files to create/modify

| File | Action |
|------|--------|
| `src-tauri/src/snapshot_logger.rs` | **Create** — snapshot JSON writer |
| `src-tauri/src/event_logger.rs` | **Create** — append-only event log |
| `src-tauri/src/lib.rs` | Add modules, init loggers in AppState |
| `src-tauri/src/commands.rs` | Add loggers to AppState struct |
| `src-tauri/src/serial_periodic.rs` | Call snapshot + event logger |
| `src-tauri/src/serial.rs` | Log device connect/lost events |
| `apps/desk/CLAUDE.md` | Add Logging & Debugging section |
| `apps/desk/.claude/rules/logging.md` | **Create** — logging reference doc |

## Tests

- Unit: `snapshot_logger` creates correct path, writes valid JSON
- Unit: `event_logger` appends lines, creates dirs
- Unit: cleanup deletes old folders, keeps recent
- No integration tests needed (file I/O only)

## Acceptance criteria

- [ ] JSON snapshot every 60s in `logs/YYYY-MM-DD/HH-MM.json`
- [ ] Event log appends to `logs/YYYY-MM-DD/events.log`
- [ ] Old logs cleaned up after 7 days
- [ ] CLAUDE.md documents log location + format
- [ ] `.claude/rules/logging.md` has full reference
