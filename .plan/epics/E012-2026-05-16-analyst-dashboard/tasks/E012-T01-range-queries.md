---
id: E012-T01
epic: E012
status: pending
created: 2026-05-16
branch: feat/E012-T01-range-queries
title: E012-T01 — Backend: snapshot / event / session range query commands
---

# E012-T01 — Backend: snapshot / event / session range query commands

## Goal
Expose three Tauri commands that let the frontend pull date-range-scoped raw data from disk, returning typed shapes ready for charting.

## Why
Today the frontend has `get_today_summary` and `get_session_state` — both today-only. The Analyst Explorer needs a 7-day window. Snapshot JSONs and `events.log` are on disk but no command reads them. SQLite has `sessions` but no range query.

## Commands to add

```rust
// In a new file: apps/desk/src-tauri/src/commands_analyst.rs

#[tauri::command]
async fn get_snapshots_range(
    app: AppHandle,
    from: String,  // ISO 8601 date "YYYY-MM-DD"
    to: String,    // ISO 8601 date "YYYY-MM-DD" (inclusive)
) -> Result<Vec<SnapshotRow>, String>;

#[tauri::command]
async fn get_events_range(
    app: AppHandle,
    from: String,
    to: String,
) -> Result<Vec<EventRow>, String>;

#[tauri::command]
async fn get_sessions_range(
    state: State<AppState>,
    from: String,
    to: String,
) -> Result<Vec<SessionRow>, String>;
```

## Types

```rust
#[derive(Serialize)]
struct SnapshotRow {
    ts: String,             // ISO 8601
    state: String,
    sitting_seconds: i64,
    standing_seconds: i64,
    break_seconds: i64,
    desk_height_cm: f32,
    idle_secs: u64,
    continuous_computer_secs: i64,
    position_changes: u32,
    daily_score: Option<f32>,
}

#[derive(Serialize)]
struct EventRow {
    ts: String,             // "YYYY-MM-DD HH:MM:SS"
    kind: String,           // STATE | DEVICE | ALERT | RESET | CREDIT | START | NOTIF | AUTOSTART
    detail: String,         // raw rest-of-line
}
```
Reuse `SessionRow` from `db_queries.rs` (already shaped).

## Implementation notes

- `snapshot_logger.rs` already writes `HH-MM.json` per minute under `logs/YYYY-MM-DD/`. New command walks the date folders, deserializes, filters by range, returns flat `Vec`. Tolerate missing days. Skip (don't fail) malformed JSON files — log a warning to `events.log` with kind `ANALYST` once per file.
- For events.log: parse each line with a small `parse_event_line(date: &str, line: &str)` helper. Format is `HH:MM:SS TYPE details`. Combine `date + " " + HH:MM:SS` into the `ts` field.
- For sessions: extend `db_queries.rs` with `get_sessions_range(conn, from, to)` using `WHERE date_local BETWEEN ? AND ?`.

## Tests

**Unit (Rust):**
- `parse_event_line` — one assertion per event type from `.claude/rules/logging.md` (STATE, DEVICE, ALERT, RESET, CREDIT, START, NOTIF, AUTOSTART)
- `parse_event_line` — malformed line returns `Err`, caller skips
- `snapshot walker` — date range filter (tempdir with synthetic folders)
- `snapshot walker` — malformed JSON in one file doesn't abort the walk

**Integration (Rust):**
- `get_snapshots_range` with a tempdir `app_data_dir` fixture containing 3 days of snapshots → returns flat sorted vec

## Files touched
- `apps/desk/src-tauri/src/commands_analyst.rs` (new)
- `apps/desk/src-tauri/src/lib.rs` — register the three commands in `invoke_handler`
- `apps/desk/src-tauri/src/db_queries.rs` — add `get_sessions_range`
- `apps/desk/src-tauri/capabilities/default.json` — allowlist new commands

## DoD
- [ ] Three commands registered and callable from JS via `invoke('get_snapshots_range', { from, to })`
- [ ] All new unit + integration tests pass
- [ ] `cargo test` overall count grows; existing tests untouched
- [ ] File ≤ 250 lines (split if needed)
- [ ] No `unwrap()` in production paths
