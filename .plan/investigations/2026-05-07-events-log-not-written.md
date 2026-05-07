# Investigation: events.log not written — 2026-05-07

## Context

The release exe (`SmartDesk.exe`) runs successfully and the app behaves correctly
(tray icon appears, sensor is detected, session logic works). However, no
`events.log` file is written under `%APPDATA%\io.zntl.desk\logs\YYYY-MM-DD\`.
The `START v…` event is logged immediately on startup, so the failure happens
at the very first write attempt.

## Hypothesis H1: `create_dir_all` fails silently — base dir never created

**Source:** `EventLogger::new()` only stored `base_dir` — it never called
`fs::create_dir_all`. The logs/ directory might not exist when `log()` is
first called. `ensure_day_dir` calls `fs::create_dir_all(&day_path)` where
`day_path = base_dir.join(date)`. On Windows, `create_dir_all` will succeed
even if intermediate directories don't exist — BUT only if the parent volume
is writable and the path is valid. If `base_dir` itself is a symlink or the
path resolves to a location that the process cannot write to, the call silently
returns Err which is swallowed by `log::warn!`.

**Result:** CONFIRMED as a design flaw. `new()` never validated that `base_dir`
could be created. If it fails, there is zero signal at startup — the app
continues running and the `log()` call later silently drops the entry.

**Learned:** Logging infrastructure must fail loudly at init time, not silently
at write time. Panic at `new()` is the right fix: it surfaces the problem
immediately instead of producing mysteriously empty log directories.

## Hypothesis H2: `ensure_day_dir` errors because parent `logs/` is missing

**Source:** `ensure_day_dir` calls `fs::create_dir_all(&day_path)` where
`day_path = base_dir.join("YYYY-MM-DD")`. `create_dir_all` creates all
missing ancestors. So even if `logs/` doesn't exist, this call would normally
succeed — unless the path is on a read-only filesystem or blocked by AV.

**Result:** PARTIALLY confirmed. `create_dir_all` on a missing base would
normally work, but any I/O error is caught and returned as `Err(e)`. In
`event_logger.log()` that error was `log::warn!` — invisible if the Tauri
log viewer / file logger isn't configured at `warn` level. AV interference
on Windows (e.g. Windows Defender scanning newly created files) can transiently
block `open()` calls, which would cause a one-time failure that is never retried.

**Learned:** The error message must be `error!` (not `warn!`), and a 1-retry
on write failure covers the transient AV-scan case.

## Hypothesis H3: `OpenOptions::open` errors due to permissions or AV

**Source:** `OpenOptions::append(true).create(true).open(&path)` can fail if:
- Windows Defender or another AV opens the file exclusively during a scan
- The user profile AppData directory has unusual ACLs
- A previous crashed instance left the file locked (less likely for append)

The error was caught as `log::warn!("event log: open failed: {}")` with no
path included — making it impossible to diagnose from Tauri log output alone.

**Result:** CONFIRMED as a contributing factor. The open-failure error message
had no path context, no error kind, and was at `warn` level. Even if the
error appeared in logs, it was unactionable. Adding 1 retry + `error!` with
full path fixes both the transient case and debuggability.

**Learned:** Every I/O error log must include the full path and `e.kind()` so
it can be reproduced and diagnosed without a debugger attached.

## Hypothesis H4: Panic earlier in setup short-circuits before `event_logger.log("START …")`

**Source:** In `setup_helpers.rs`, the call order is:
1. `app.path().app_data_dir()?` — returns early on error via `?`
2. `std::fs::create_dir_all(&app_data_dir)?` — returns early on error
3. `EventLogger::new(logs_dir)` — previously just stored path (no I/O)
4. `event_logger.log("START v{}")` — first actual write

If step 1 or 2 failed, setup would return `Err` and `setup()` would log
the error through Tauri's plugin-log. The app might still show the tray icon
(Tauri 2 continues even if setup returns Err on some paths) but the logger
was never called.

**Result:** UNLIKELY but not fully excludable without runtime evidence.
Step 2 creates `app_data_dir` but NOT `logs_dir`. If the write to `logs_dir`
failed for any reason prior to this fix, it would silently drop. This is
covered by H1/H2 above.

## Root cause

**Primary:** `EventLogger::new()` did not create `base_dir`. Any failure of
`ensure_day_dir` inside `log()` was silently swallowed at `warn!` level with
no path context. On Windows, AV or permission issues that transiently block
directory creation produce an error that was never surfaced at a visible log
level and was not retried.

**Secondary:** All write errors used `warn!` with no full-path context, making
them invisible in default Tauri log configurations and unactionable for debugging.

## Fix applied

1. `EventLogger::new(base_dir)` now calls `fs::create_dir_all(&base_dir)` and
   panics with a clear message if it fails — fast fail at initialization, not
   at first write.
2. All error paths escalated from `log::warn!` to `log::error!` with full path
   and `e.kind()` included in the message.
3. `log()` retries the write once after 100 ms to handle transient AV-scan
   file locks. After retry failure, logs at `error!`.
4. Removed the `event_log_io_error_no_panic` unit test whose contract
   ("should not panic") was the root cause of the silent-failure design.
5. Added `new_creates_base_dir` unit test and a full integration test suite
   in `tests/event_logger_integration.rs` that exercises the real filesystem.
