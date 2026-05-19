# History — Desk App

## 2026-05-18 — E000: Google Fit walking-steps integration

- Surfaced today's step count in the OneBar popup via a new KPI badge — Google Fit REST API + OAuth2 offline access. Opt-in by `.env` credentials; absence yields a calm "connect google fit" UI.
- Architecture: 4 Rust modules in `src-tauri/src/` (`google_fit.rs` client, `google_fit_models.rs` wire types incl. `ErrorKind`, `google_fit_service.rs` cache with broadcast-based dedup + DST-correct local-day window, `commands_google_fit.rs` IPC). React `StepsWidget` slotted into `KpiStrip` via new `children` prop. Zero-deps Node OAuth helper script.
- Key choices (rationale in [ADR-012](./ADR/012-google-fit-integration.md)): REST API over Health Connect (no cloud read), per-call token refresh (no cache), compact KPI badge UI (no new layout surface), env-var-driven config, auto-discovery of step source with override + 1h failure-cache, classified errors (`auth_revoked` → reconnect CTA, `transient` → exponential backoff), `useExponentialPoll` extracted as reusable hook.
- Tests: Rust 457 → 495 (+38 — wiremock-backed HTTP integration, host-independent DST via `chrono-tz`, service-layer dedup); TS 194 → 207 (+13 — widget states, hook math).
- Deprecation risk acknowledged: Google has signaled Fit API retirement in favor of Health Connect. Integration is encapsulated behind `GoogleFitClient` + `StepsView` contract — backend swap is local to ~3 files. Mitigation tracked in ADR-012 Consequences.

## 2026-05-16 — E012: Analyst Dashboard (v0.5.0)
- Separate Tauri window (1280x800) + tray "Open Analyst" entry exposing two tabs over the last 7 days
- **Catalog tab**: live `get_data_catalog()` Rust command describing 8 data sources (sensor, sqlite_sessions, snapshots, events_log, profiles_*, store, remote_ws) with fields, retention, sample rows
- **Explorer tab**: live range queries (`get_snapshots_range`, `get_events_range`, `get_sessions_range`), 5 SVG charts: DeskHeightTimeline (downsampled to 1000 points), StateGantt, DailyScoreTrajectory, BreakCreditHistogram, KpiTrend
- Architecture: 4 new Tauri commands; `commands_analyst.rs`, `commands_catalog{,_sources,_tests}.rs`; React `analyst/` module with hooks-and-fixtures seam (mockup `/#/mockup/analyst` for iteration; live `/#/analyst`)
- Tests: Rust 434 → 457 (+23); TS 168 → 194 (+26)
- Deviations parked in epic IMPROVEMENTS.md: apps/tray typecheck blocking commits; StateGantt sessions wiring; break_credit persistence; KPI daily rollup; visual smoke pending

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
