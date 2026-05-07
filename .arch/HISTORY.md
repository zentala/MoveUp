# History — Desk App

## 2026-05-07 — E011: Autostart Hardening (v0.4.0)
- Autostart self-heal: reads registry via `winreg`, re-registers if path differs from `current_exe()`
- Dev guard: `#[cfg(debug_assertions)]` skips autostart registration in debug builds
- EventLogger reliability: `create_dir_all` on init, `log::error!` with full context, 1-retry on write fail
- Precommit gate: `tsc --noEmit` for `apps/desk/` added to `.husky/pre-commit`
- Minimized autostart: `--minimized` arg passed via autostart plugin; `position_main_window()` hides popup when arg present
- Root cause of events.log not written: `EventLogger::new()` never created `base_dir` — fixed with eager `create_dir_all` + panic on fail
- Tests: 427 Rust lib + 3 integration (was 420)

## 2026-03-15 — E001: App Foundation
- Built core: SessionManager, rusqlite DB, config persistence, daily reset, settings panel
- Decisions: rusqlite over tauri-plugin-sql (Rust owns DB), ReadingResult struct

## 2026-03-16–19 — E002: Overlay Progress Bar
- WinAPI native window (4px, top of screen), 3 DataSource modes (Demo/Live/Mock)
- 3 bar variants (solid/gradient/pulsing), configurable height
- Decisions: WinAPI over Tauri WebviewWindow (transparency issues)

## 2026-03-16 — E003: Installer & Distribution
- NSIS installer, build-report.js, memory profiling, user docs (4 files)
- Code signing + auto-update scaffolded for future

## 2026-03-20 — E004: Session Alerts & Snooze
- AlertManager state machine (Idle→Stage1→Stage2→Snoozed)
- WinAPI popup, deescalating snooze [5,15,30,60]min, tone shift at dismiss #3

## 2026-03-21–22 — E005: UX Communication + Widget System
- Refactored all oversized Rust files (session.rs 1348→4 files)
- Widget system (WidgetProps, registry), One Bar + Timeline Zen
- Standing progress bar (gold, lap flash), points system, welcome popup

## 2026-03-22–23 — E006: Session Bugs & Polish
- Fixed: insert_session, sensor race, "No sessions yet" serde, standing timer
- HeightStabilizer module (10-sample moving avg), settings tabbed layout
- Design review: 7 CSS findings, window positioning, tray click behavior

## 2026-03-23 — E007: KPI Dashboard + Timer UX (was E001)
- MetricEngine trait + 4 metrics (StandingPct, PositionChangeRate, HourlyBreakCoverage, LongestSession)
- KPI strip component, timer elapsed/total, unified ProgressBar
- 11 tasks, 429→508 tests

## 2026-03-24 — E008: UX Integrity (was E002)
- Away state = inactive regardless of desk height (fixed fundamental gap)
- NotificationService centralized 6 toasts, wired notification_backend config
- Popup redesign: timer-first, Unicode KPI icons, color unification
- 13 tasks, 508 tests total

## 2026-03-25 — E009: Remote Display — Web Kiosk (v0.3.0)
- Embedded HTTP+WS server (axum on :3390) serves React UI to phones via LAN
- ws_broadcaster.rs: tokio broadcast channel, RemoteDisplayState, zero SQLite in hot path
- remote_server.rs: WS with max 10 clients, heartbeat, graceful port binding
- useRemoteDesk.ts: WS-based hook, auto-reconnect with exponential backoff
- useDeskAuto.ts: auto-selects Tauri IPC or WebSocket based on runtime
- ConnectionOverlay: reconnecting overlay + sensor-disconnected banner
- Responsive CSS: landscape enforcement, portrait prompt, wake lock
- ADRs: [001-remote-display-web-kiosk](./../.arch/ADR/001-remote-display-web-kiosk.md)
- 7 tasks, 528 tests total (+32 new)
