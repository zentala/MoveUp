---
id: E011-T02
epic: E011
status: pending
created: 2026-05-07
branch: feat/E011-T02-event-logger-fix
worktree: .claude/worktrees/E011-T02-event-logger-fix
title: E011-T02 — EventLogger investigation + write reliability
---

# E011-T02 — EventLogger investigation + write reliability

## Goal

Find out why `events.log` doesn't get written for today (2026-05-07) despite the release exe running successfully, then fix it and add a test that would have caught the regression.

## Files

- `apps/desk/.plan/investigations/2026-05-07-events-log-not-written.md` — NEW investigation file
- `apps/desk/src-tauri/src/event_logger.rs` — escalate `warn!` → `error!` with full context, add 1-retry on write fail, validate `base_dir` is writable in `EventLogger::new()`
- `apps/desk/src-tauri/tests/event_logger_integration.rs` — NEW Rust integration test (`#[test]` in `tests/` dir = integration)

## Investigation step (do FIRST, before changing code)

1. Add a temporary `eprintln!` before each `log::warn!` in `EventLogger::log()` printing the resolved path and the actual error
2. Run release exe with stderr captured:
   ```powershell
   Start-Process 'C:\code\zntl-tray\target\release\desk.exe' -RedirectStandardError "$env:TEMP\desk-stderr.log" -NoNewWindow
   Start-Sleep 5
   Get-Content "$env:TEMP\desk-stderr.log"
   ```
3. Document hypothesis → action → result in the investigation file
4. Once root cause is known, **revert the eprintln!** — replace with proper `log::error!()` with structured context

## Hypotheses to falsify

- H1: `app.path().app_data_dir()` returns a path that doesn't exist and `create_dir_all` fails silently elsewhere
- H2: `ensure_day_dir` errors because parent `logs/` is missing
- H3: `OpenOptions::open` errors due to permissions or AV interference
- H4: A panic earlier in setup short-circuits before `event_logger.log("START …")` runs (would conflict with the fact that overlay log appears later)

## Fix (after diagnosis)

1. `log::warn!` → `log::error!` with `path={display}` and `error_kind={:?}`
2. On write fail: 1 retry after `std::thread::sleep(Duration::from_millis(100))`
3. In `EventLogger::new(base_dir)`: try `create_dir_all(&base_dir)`. If it fails, panic with a clear message — logging is critical infrastructure, silent failure is unacceptable.

## Tests

Integration (`tests/event_logger_integration.rs`):
```rust
#[test]
fn writes_to_real_path() {
    let tmp = tempfile::tempdir().unwrap();
    let logger = desk_lib::event_logger::EventLogger::new(tmp.path().join("logs"));
    logger.log("TEST_MESSAGE");
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = tmp.path().join("logs").join(&today).join("events.log");
    assert!(path.exists(), "events.log not created at {:?}", path);
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("TEST_MESSAGE"), "missing message in: {body}");
}
```

Existing unit tests in `event_logger.rs` already cover happy path with TempDir — keep them.

## Smoke

1. Delete `%APPDATA%\io.zntl.desk\logs\<today>\` if exists
2. `Start-Process 'C:\code\zntl-tray\target\release\desk.exe'`
3. After 5s: `Test-Path "$env:APPDATA\io.zntl.desk\logs\<today>\events.log"` → `True`
4. File contains `START v0.3.X`

## Done when

- [ ] Investigation file lists hypothesis, action, result, learned per attempt
- [ ] `cargo test` (lib + integration) passes
- [ ] Smoke 1-4 pass
- [ ] Committed as `fix(desk): event_logger write reliability + diagnostics`
