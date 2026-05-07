---
id: E011-T05
epic: E011
status: open
created: 2026-05-07
branch: feat/E011-T05-minimized-startup
worktree: .claude/worktrees/E011-T05-minimized-startup
depends_on: E011-T01
---

# E011-T05 — Autostart `--minimized` + popup hidden on autostart

## Goal

When the OS autostarts the app at login, it should appear silently in the tray (with overlay bar) instead of flashing the popup window onscreen. User-initiated launches still show the popup as expected.

## Files

- `apps/desk/src-tauri/src/lib.rs` (line ~123) — add `--minimized` to autostart args
- `apps/desk/src-tauri/src/setup_helpers.rs::position_main_window()` — hide window if started with `--minimized`

## Implementation

`lib.rs`:
```rust
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    Some(vec!["--minimized"])
))
```

`setup_helpers.rs::position_main_window()` — at the end:
```rust
let started_minimized = std::env::args().any(|a| a == "--minimized");
if started_minimized {
    let _ = window.hide();
    log::info!("autostart: started with --minimized, popup hidden");
}
```

## Tauri config

Verify `tauri.conf.json` window `"visible": false` — already true. If popup still appears, search for a `show()` call in setup that fires unconditionally (`Grep "window.show()" src-tauri/src/`).

## Single-instance compatibility

The single-instance callback in `lib.rs` calls `window.show()` + `set_focus()`. That's correct: when user re-launches the exe, they want the popup.

## Tests

Unit (in `setup_helpers_tests.rs` from T01 or new `args_test.rs`):
- `is_minimized_arg_present_at_start` — `["exe", "--minimized"]` → true
- `is_minimized_arg_absent` — `["exe"]` → false
- `is_minimized_arg_in_middle` — `["exe", "--foo", "--minimized", "--bar"]` → true

## Smoke

1. `Start-Process 'C:\code\zntl-tray\target\release\desk.exe' -ArgumentList '--minimized'`
2. Observe: tray icon appears, overlay bar at top of screen, **no popup window**
3. Click tray icon → popup opens (single-instance / tray-click handler)

End-to-end: reboot system, log in, observe no popup flash but tray + overlay present.

## Done when

- [ ] Unit tests pass
- [ ] Smoke 1-3 pass
- [ ] Reboot test: no popup flash on login
- [ ] Committed as `feat(desk): minimized autostart launch`
