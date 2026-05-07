---
id: E011-T01
epic: E011
status: open
created: 2026-05-07
branch: feat/E011-T01-autostart-self-heal
worktree: .claude/worktrees/E011-T01-autostart-self-heal
---

# E011-T01 — Autostart self-heal + dev guard + log to events

## Goal

Replace `setup_helpers.rs::ensure_autostart()` so that it:
- **A**: Skips registration entirely in debug builds
- **B**: Self-heals when the registered Run-key path differs from `current_exe()`
- **C**: Always emits an `AUTOSTART …` line to `events.log` so we can verify state

## Files

- `apps/desk/src-tauri/src/setup_helpers.rs` — rewrite `ensure_autostart()`, change call site signature
- `apps/desk/src-tauri/Cargo.toml` — add `winreg = "0.52"` under `[target.'cfg(windows)'.dependencies]`
- `apps/desk/src-tauri/src/setup_helpers_tests.rs` — NEW (path-equality unit tests)

## Implementation

See plan section "E011-T01" in `~/.claude/plans/wszystko-jiggly-wall.md` for full code skeleton. Key points:

1. New helper `read_autostart_registry_path() -> Option<String>` using `winreg` on Windows, returns `None` elsewhere
2. New helper `paths_equal(a, b)` — trim quotes, normalize slashes, lowercase compare
3. `ensure_autostart` now takes `&Arc<EventLogger>`; emit one of: `AUTOSTART skipped reason=debug_build` / `AUTOSTART stale_path old=… new=…` / `AUTOSTART enabled path=…` / `AUTOSTART verified path=…` / `AUTOSTART error msg=…`
4. Call site update in `perform_app_setup` (line ~38): pass `&event_logger`

## Tests

Unit (in new `setup_helpers_tests.rs`, follow bare `#[test]` pattern from `tray_controller_tests.rs`):
- `paths_equal_case_insensitive` — `c:\Foo` == `C:\foo`
- `paths_equal_trims_quotes` — `"C:\bar.exe"` == `C:\bar.exe`
- `paths_equal_normalizes_slashes` — `C:/foo/bar` == `C:\foo\bar`
- `paths_equal_rejects_different_paths` — `C:\a` != `C:\b`

`ensure_autostart` itself can't be unit-tested without a Tauri AppHandle — covered by smoke.

## Smoke (manual, document outcome in JOURNAL)

1. `Set-ItemProperty 'HKCU:\...\Run' -Name SmartDesk -Value 'C:\fake\desk.exe'`
2. `Start-Process 'C:\code\zntl-tray\target\release\desk.exe'`
3. After 5s: `(Get-ItemProperty 'HKCU:\...\Run').SmartDesk` should be the real release path
4. `Get-Content "$env:APPDATA\io.zntl.desk\logs\<today>\events.log"` should contain `AUTOSTART stale_path old=… new=…` followed by `AUTOSTART enabled path=…`

## Done when

- [ ] `cargo test --lib` passes (existing 420+ + new path tests)
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] Smoke 1-4 above pass
- [ ] Committed as `feat(desk): autostart self-heal + dev guard + events log`
