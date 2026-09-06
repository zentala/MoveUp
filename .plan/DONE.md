# Completed Tasks

## 2026-05-18

- **Google Fit walking-steps integration** — Backend (4 Rust modules: client, service with broadcast-based dedup + DST-correct day window, models with classified ErrorKind, IPC commands), React `StepsWidget` slotted into `KpiStrip` via new `children` slot, zero-deps Node OAuth helper script (rundll32 browser open). Auto-discovery of step data source with `GOOGLE_FIT_STEPS_SOURCE` env override and 1h failure-cache. Error classification (`auth_revoked` → reconnect CTA, `transient` → exponential backoff 1m → 30m cap). Wiremock-tested HTTP path; chrono-tz host-independent DST tests. 38 Rust tests + 207 frontend tests. ADR-012 documents alternatives + Health Connect migration risk. Commits: `8702964` `ad59494` `a8560dd` plus polish-pass-2.

## E000 — Maintenance
- [x] [E000-T049 — Pre-dev process guard](epics/E000-maintenance/tasks/E000-T049-pre-dev-process-guard.md) — E000, 2026-03-23

## E001 — App Foundation (2026-03-15)
- [x] [E001-T01 — Config store + rusqlite](epics/E001-2026-03-15-app-foundation/tasks/E001-T01-config-store-rusqlite.md) — E001, 2026-03-15
- [x] [E001-T02 — Daily reset](epics/E001-2026-03-15-app-foundation/tasks/E001-T02-daily-reset.md) — E001, 2026-03-15
- [x] [E001-T03 — Settings panel UI](epics/E001-2026-03-15-app-foundation/tasks/E001-T03-settings-panel.md) — E001, 2026-03-15
- [x] [E001-T04 — Notification toggles](epics/E001-2026-03-15-app-foundation/tasks/E001-T04-notification-prefs.md) — E001, 2026-03-15
- [x] [E001-T05 — Stand reminder limit](epics/E001-2026-03-15-app-foundation/tasks/E001-T05-stand-limit.md) — E001, 2026-03-15
- [x] [E001-T06 — Position changes counter](epics/E001-2026-03-15-app-foundation/tasks/E001-T06-position-changes.md) — E001, 2026-03-15
- [x] [E001-T07 — Fix standing_secs](epics/E001-2026-03-15-app-foundation/tasks/E001-T07-standing-secs.md) — E001, 2026-03-15
- [x] [E001-T08 — Test framework](epics/E001-2026-03-15-app-foundation/tasks/E001-T08-test-framework.md) — E001, 2026-03-15
- [x] [E001-T09 — position_changes DB schema](epics/E001-2026-03-15-app-foundation/tasks/E001-T09-changes-db.md) — E001, 2026-03-23
- [x] [E001-T11 — Rail pulse animation](epics/E001-2026-03-15-app-foundation/tasks/E001-T11-rail-pulse.md) — E001, 2026-03-21
- [x] [E001-T12 — Yesterday delta arrow](epics/E001-2026-03-15-app-foundation/tasks/E001-T12-yesterday-delta.md) — E001, 2026-03-21

## E002 — Overlay Progress Bar (2026-03-16)
- [x] [E002-T01 — Dev mode visible bar](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T01-dev-mode-visible-bar.md) — E002, 2026-03-16
- [x] [E002-T02 — Device disconnected notification](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T02-device-notification.md) — E002, 2026-03-16
- [x] [E002-T03 — Compare OPAQUE vs LAYERED](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T03-compare-opaque-layered.md) — E002, 2026-03-16
- [x] [E002-T04 — Bar variants solid/gradient/pulsing](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T04-bar-variants.md) — E002, 2026-03-16
- [x] [E002-T05 — Adjustable bar height](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T05-adjustable-bar-height.md) — E002, 2026-03-16
- [x] [E002-T06 — Fix auto-test.sh](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T06-fix-auto-test.md) — E002, 2026-03-16
- [x] [E002-T07 — Rust unit tests](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T07-rust-unit-tests.md) — E002, 2026-03-16
- [x] [E002-T08 — DataSource refactor](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T08-datasource-refactor.md) — E002, 2026-03-16
- [x] [E002-T10 — Split overlay_renderer.rs](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T10-split-overlay-renderer.md) — E002, 2026-03-16
- [x] [E002-T12 — Precommit line count hook](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T12-precommit-line-count.md) — E002, 2026-03-16

## E003 — Installer & Distribution (2026-03-16)
- [x] [E003-T01 — NSIS installer config + signing scaffold](epics/E003-2026-03-16-installer-distribution/tasks/E003-T01-tauri-installer-config.md) — E003, 2026-03-16
- [x] [E003-T02 — Build size monitoring](epics/E003-2026-03-16-installer-distribution/tasks/E003-T02-build-size-monitoring.md) — E003, 2026-03-16
- [x] [E003-T03 — Memory profiling test](epics/E003-2026-03-16-installer-distribution/tasks/E003-T03-memory-profiling.md) — E003, 2026-03-16
- [x] [E003-T04 — User documentation](epics/E003-2026-03-16-installer-distribution/tasks/E003-T04-user-documentation.md) — E003, 2026-03-16
- [x] [E003-T05 — Update CLAUDE.md](epics/E003-2026-03-16-installer-distribution/tasks/E003-T05-claude-md-update.md) — E003, 2026-03-16
- [x] [E003-T06 — Optimization guide](epics/E003-2026-03-16-installer-distribution/tasks/E003-T06-optimization-guide.md) — E003, 2026-03-16

## E004 — Session Alerts & Snooze (2026-03-20)
- [x] [E004-T02 — AlertManager + Stage1 + Stage2](epics/E004-2026-03-20-session-alerts/tasks/E004-T02-alert-manager-stage1-stage2.md) — E004, 2026-03-20
- [x] [E004-T03 — Snooze with deescalating frequency](epics/E004-2026-03-20-session-alerts/tasks/E004-T03-snooze-logic.md) — E004, 2026-03-20
- [x] [E004-T05 — Dismiss without sensor](epics/E004-2026-03-20-session-alerts/tasks/E004-T05-dismiss-without-sensor.md) — E004, 2026-03-20

## E005 — UX Communication + Widgets (2026-03-21)
- [x] [E005-T01 — Split session.rs](epics/E005-2026-03-21-ux-widgets/tasks/E005-T01-split-session-rs.md) — E005, 2026-03-21
- [x] [E005-T02 — Split db/serial/commands](epics/E005-2026-03-21-ux-widgets/tasks/E005-T02-split-oversized-files.md) — E005, 2026-03-21
- [x] [E005-T03 — Fix tooltip while standing](epics/E005-2026-03-21-ux-widgets/tasks/E005-T03-fix-tooltip-standing.md) — E005, 2026-03-21
- [x] [E005-T04 — Notifications debug + fix](epics/E005-2026-03-21-ux-widgets/tasks/E005-T04-notifications-debug-fix.md) — E005, 2026-03-21
- [x] [E005-T05 — Floating window spec + tests](epics/E005-2026-03-21-ux-widgets/tasks/E005-T05-floating-window-spec.md) — E005, 2026-03-21
- [x] [E005-T06 — Standing progress bar gold](epics/E005-2026-03-21-ux-widgets/tasks/E005-T06-standing-progress-bar.md) — E005, 2026-03-21
- [x] [E005-T07 — Points system](epics/E005-2026-03-21-ux-widgets/tasks/E005-T07-points-system.md) — E005, 2026-03-21
- [x] [E005-T08 — Welcome popup](epics/E005-2026-03-21-ux-widgets/tasks/E005-T08-welcome-popup.md) — E005, 2026-03-21
- [x] [E005-T09 — Floating window fixes](epics/E005-2026-03-21-ux-widgets/tasks/E005-T09-floating-window-fix.md) — E005, 2026-03-21
- [x] [E005-T10 — Tray icon redesign](epics/E005-2026-03-21-ux-widgets/tasks/E005-T10-tray-icon-redesign.md) — E005, 2026-03-21
- [x] [E005-T11 — Widget architecture](epics/E005-2026-03-21-ux-widgets/tasks/E005-T11-widget-architecture.md) — E005, 2026-03-21
- [x] [E005-T12 — One Bar widget](epics/E005-2026-03-21-ux-widgets/tasks/E005-T12-widget-one-bar.md) — E005, 2026-03-21
- [x] [E005-T13 — Timeline Zen widget](epics/E005-2026-03-21-ux-widgets/tasks/E005-T13-widget-timeline-zen.md) — E005, 2026-03-21

## E006 — Session Bugs & Polish (2026-03-22)
- [x] E006-T01 — Fix insert_session never called — E006, 2026-03-22
- [x] E006-T02 — Sensor race condition fix — E006, 2026-03-22
- [x] E006-T03 — Fix "CONNECTING" label + default Away — E006, 2026-03-22
- [x] E006-T04 — Fix standing timer showing sitting value — E006, 2026-03-22
- [x] E006-T05 — UX-FLOW.md creation — E006, 2026-03-22
- [x] [E006-T06 — Visual layout polish](epics/E006-2026-03-22-session-bugs-polish/tasks/E006-T06-visual-layout-polish.md) — E006, 2026-03-22
- [x] E006-T07 — Window positioning + tray click behavior — E006, 2026-03-22
- [x] E006-T08 — Fix memory leak in tray icon — E006, 2026-03-22
- [x] [E006-T09 — Settings tabbed layout](epics/E006-2026-03-22-session-bugs-polish/tasks/E006-T09-settings-tabbed-layout.md) — E006, 2026-03-22
- [x] E006-T10 — HeightStabilizer module — E006, 2026-03-23
- [x] E006-T11 — Popup closes on blur — E006, 2026-03-23
- [x] [E006-T12 — "No sessions yet" serde fix](epics/E006-2026-03-22-session-bugs-polish/tasks/E006-T12-no-sessions-yet-fix.md) — E006, 2026-03-23
- [x] E006-T13 — Session timer unit tests — E006, 2026-03-23

## E007 — KPI Dashboard + Timer UX (2026-03-23)
- [x] E007-T01 — New ProgressBar + Animation Components — E007, 2026-03-23
- [x] E007-T02 — WidgetProps + useWidgetData Extensions — E007, 2026-03-23
- [x] E007-T03 — Timer UX Integration + Cleanup — E007, 2026-03-23
- [x] E007-T04 — SessionManager Extensions — E007, 2026-03-23
- [x] E007-T05 — HourlyBreakTracker + GapHandler — E007, 2026-03-23
- [x] E007-T06 — MetricEngine + 4 Metrics — E007, 2026-03-23
- [x] E007-T07 — Rust IPC + Persistence Wiring — E007, 2026-03-23
- [x] E007-T08 — KpiStrip Component — E007, 2026-03-23
- [x] E007-T09 — Timeline Hour Markers — E007, 2026-03-23
- [x] E007-T10 — Debug Tab Metrics — E007, 2026-03-23
- [x] E007-T11 — Final Integration + Coach Removal — E007, 2026-03-23

## E008 — UX Integrity (2026-03-24)
- [x] E008-T01 — Update UX-FLOW.md — E008, 2026-03-24
- [x] E008-T02 — Away state fix — E008, 2026-03-24
- [x] E008-T03 — Away sessions to DB + live timeline — E008, 2026-03-24
- [x] E008-T04 — Away flow tests — E008, 2026-03-24
- [x] E008-T05 — KPI rename + icons — E008, 2026-03-24
- [x] E008-T06 — Overlay/popup color unification — E008, 2026-03-24
- [x] E008-T07 — Standing lap 2-layer visual — E008, 2026-03-24
- [x] E008-T08 — Popup layout redesign — E008, 2026-03-24
- [x] E008-T09 — NotificationService — E008, 2026-03-24
- [x] E008-T10 — NotificationService tests — E008, 2026-03-24
- [x] E008-T11 — Process guard restore — E008, 2026-03-24
- [x] E008-T12 — Regression tests for popup standing timer — E008, 2026-03-24
- [x] E008-T13 — Final UX-FLOW.md verification — E008, 2026-03-24

## E009 — Remote Display / Web Kiosk (2026-03-25)
- [x] [E009-T01 — Version bump](epics/E009-2026-03-24-remote-display/tasks/E009-T01-version-bump.md) — E009, 2026-03-25
- [x] [E009-T02 — WS broadcaster](epics/E009-2026-03-24-remote-display/tasks/E009-T02-ws-broadcaster.md) — E009, 2026-03-25
- [x] [E009-T03 — HTTP+WS server](epics/E009-2026-03-24-remote-display/tasks/E009-T03-http-server.md) — E009, 2026-03-25
- [x] [E009-T04 — useRemoteDesk hook](epics/E009-2026-03-24-remote-display/tasks/E009-T04-use-remote-desk-hook.md) — E009, 2026-03-25
- [x] [E009-T05 — Responsive display](epics/E009-2026-03-24-remote-display/tasks/E009-T05-responsive-display.md) — E009, 2026-03-25
- [x] [E009-T06 — Connection overlay](epics/E009-2026-03-24-remote-display/tasks/E009-T06-reconnect-ui.md) — E009, 2026-03-25
- [x] [E009-T07 — E2E tests + docs](epics/E009-2026-03-24-remote-display/tasks/E009-T07-e2e-test.md) — E009, 2026-03-25

## E011 — Autostart Hardening (2026-05-07)
- **[E011-T01](epics/E011-2026-05-07-autostart-hardening/tasks/E011-T01-autostart-self-heal.md)** — Autostart self-heal + dev guard + events log. winreg reads registry path, re-registers if stale; #[cfg(debug_assertions)] skips in dev builds; all paths emit AUTOSTART … to events.log. Commits: 2c95911.
- **[E011-T02](epics/E011-2026-05-07-autostart-hardening/tasks/E011-T02-event-logger-fix.md)** — EventLogger write reliability. Root cause: 
ew() never created ase_dir. Fix: eager create_dir_all + panic on fail, log::error! with full context, 1-retry. 3 integration tests added. Commits: c159b04.
- **[E011-T03](epics/E011-2026-05-07-autostart-hardening/tasks/E011-T03-commit-test-fix.md)** — OneBarTimeline.test.tsx missing WidgetProps fields. Commits: 95305b7.
- **[E011-T04](epics/E011-2026-05-07-autostart-hardening/tasks/E011-T04-precommit-tsc-gate.md)** — Precommit tsc gate for apps/desk. Blocks commits with TS type errors. Commits: 2dcf49c.
- **[E011-T05](epics/E011-2026-05-07-autostart-hardening/tasks/E011-T05-minimized-startup.md)** — Minimized autostart launch. --minimized arg via autostart plugin; popup hidden on autostart, single-instance show() preserved. Commits: bd7df6.

## E012 — Analyst Dashboard (2026-05-16, v0.5.0)
- **[E012-T01](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T01-range-queries.md)** — Rust range query commands: get_snapshots_range, get_events_range, get_sessions_range. commands_analyst.rs + sibling test file; events.log parser; date-range bounds; +14 tests. Commit 8122a29.
- **[E012-T02](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T02-catalog-meta.md)** — Rust get_data_catalog command describing 8 data sources (sensor, sqlite_sessions, snapshots, events_log, profiles_*, store, remote_ws). Split into commands_catalog{,_sources,_tests}.rs for 250-line cap. +7 tests. Commits 0accbab -> f42c8e4.
- **[E012-T03](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T03-mockup.md)** — React mockup at /#/mockup/analyst: AnalystWindow shell, CatalogTab (sortable 8-source table), ExplorerTab with 5 SVG charts (DeskHeightTimeline, StateGantt, DailyScoreTrajectory, BreakCreditHistogram, KpiTrend), DateRangePicker, fixtures across 4 files. +13 tests. Commit 9d00663.
- **[E012-T04](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T04-window.md)** — Analyst Tauri window (1280x800, decorated) + tray "Open Analyst" entry; capabilities registered; 9 icon tests relocated to tray_tests.rs. +11 tests. Commit 292952e.
- **[E012-T05](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T05-catalog-live.md)** — useDataCatalog() hook + CatalogTab live wiring; mockup keeps fixtures via explicit data prop. +6 tests. Commit fb77066.
- **[E012-T06](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T06-explorer-release.md)** — useRangeQuery + 3 wrappers (snapshots/events/sessions), Explorer live, /#/analyst route + window URL flip, UX-FLOW section 11, PROJECT.xml updated. +7 tests. Commit aab3e7c.

## E016 — Plan Hygiene and AO Readiness (2026-09-06)
Run through AO (`E016-20260906-0348`), promoted at `0995f42`. All 5 tasks merged and verified.
- **[E016-T01](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/tasks/E016-T01-fix-state-md.md)** — Fixed `.plan/STATE.md` frontmatter/body self-contradiction; reconciled E010's 3-vs-10 human-task count discrepancy.
- **[E016-T02](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/tasks/E016-T02-consolidate-backlog.md)** — Merged root `BACKLOG.md`+`TASKS.md` verbatim into `.plan/BACKLOG.md`; deleted the three root files; fixed README/CONTRIBUTING links.
- **[E016-T03](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/tasks/E016-T03-close-e011-ceremony.md)** — Closed E011's ceremony (`.plan/HISTORY.md` entry, IMPRO triage); marked E003-T07 and E013 superseded.
- **[E016-T04](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/tasks/E016-T04-untrack-build-artifacts.md)** — `git rm -r --cached coverage test-performance-report` + `.gitignore` entries.
- **[E016-T05](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/tasks/E016-T05-ao-readiness.md)** — Created root `.giter.yaml` and `justfile` for AO readiness.

## E015 — One Truth for the Sitting Counter (2026-09-06, v0.6.0)
Run through AO (`E015-20260906-0449`, 3rd attempt), promoted at `7a8bbf3`. Tagged `v0.6.0`. 504 Rust + 255 TS tests green, typecheck clean.
- **[E015-T01](epics/E015-2026-09-06-engine-single-truth/tasks/E015-T01-rust-delete-second-counter.md)** — Deleted `current_session_secs` from `SessionState`/DTO/payload across every consumer found (several beyond the original write_set — see JOURNAL.md); inverted the reset test; added the real-path scenario test (sit 30 min, stand 2 min, sit → DTO shows `1800 − 120·m`).
- **[E015-T02](epics/E015-2026-09-06-engine-single-truth/tasks/E015-T02-ts-consumers-and-drift-test.md)** — TS consumers (hooks, `OneBarTimer`/`OneBarTimeline`, `DebugSection`) read the credited `limitUsedSecs` field; added a DTO drift test against a Rust-emitted JSON fixture.
- **[E015-T03](epics/E015-2026-09-06-engine-single-truth/tasks/E015-T03-posture-balance-and-break-credit-row.md)** — PostureBalance now compares raw `sitting_seconds_total` vs `standing_seconds` (was comparing credited vs raw, making it unreachable for anyone who takes breaks); added a `break_credit` column on `sessions` with migration, written by `serial_periodic.rs`, read by `commands_analyst.rs`.
- **[E015-T04](epics/E015-2026-09-06-engine-single-truth/tasks/E015-T04-docs-adr-uxflow-decisions.md)** — ADR 008 revised (multiplier 3.0 in `standard`), `.arch/UX-FLOW.md`, `.arch/ARCHITECTURE.md`, `CLAUDE.md` Session Logic, `.plan/decisions.jsonl` (D1/D2/D4).
- **[E015-T05](epics/E015-2026-09-06-engine-single-truth/tasks/E015-T05-verify-and-browser-pass.md)** — NOT completed. `cargo test`/`pnpm test:unit` full-suite check done (green), but the browser/visual pass on the popup could not run: dev-mode gaps unrelated to E015 (no Vite proxy for `/display`, mock mode doesn't drive the real session engine) blocked it. See JOURNAL.md and `.plan/BACKLOG.md` "Dev-mode remote display" section.

## E017 — Release Readiness (2026-09-06, v0.6.0 tagged)
Run through AO (`E017-20260906-0622`), promoted at `91b3572` after one operator
intervention (a stale/transient `merge_conflict` report on T01 — a dry-run
merge proved the branch was actually clean; `ao resume` completed it). All 8
tasks merged and independently re-verified on `main`. See JOURNAL.md.
- **[E017-T01](epics/E017-2026-09-06-release-readiness/tasks/E017-T01-docs-naming-rewrite.md)** — Rewrote `README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `REMOTE_DISPLAY.md`, `OPTIMIZATION_GUIDE.md` to MoveUp naming/paths/repo; added the cable-sensitivity troubleshooting section.
- **[E017-T02](epics/E017-2026-09-06-release-readiness/tasks/E017-T02-user-updates-manual-path.md)** — Rewrote `USER_UPDATES.md` for the real manual update path; filed the real-updater follow-on to `.plan/BACKLOG.md`.
- **[E017-T03](epics/E017-2026-09-06-release-readiness/tasks/E017-T03-privacy-google-fit.md)** — Rewrote `PRIVACY.md`: naming sweep + Google Fit OAuth2/`fitness.activity.read` disclosure, dropped the blanket "no data leaves this machine" claim.
- **[E017-T04](epics/E017-2026-09-06-release-readiness/tasks/E017-T04-license-and-metadata.md)** — Added MIT `LICENSE`, `license` field in `package.json`/`Cargo.toml`, fixed the `Cargo.toml` description to name MoveUp.
- **[E017-T05](epics/E017-2026-09-06-release-readiness/tasks/E017-T05-fix-test-workflow.md)** — Fixed `.github/workflows/test.yml` to run from the repo root (was pointed at a nonexistent `apps/desk`); left `release-baseline.yml` untouched.
- **[E017-T06](epics/E017-2026-09-06-release-readiness/tasks/E017-T06-firmware-readme.md)** — Wrote `firmware/README.md`: board, sensor, flashing steps, connect confirmation string, cable-sensitivity warning.
- **[E017-T07](epics/E017-2026-09-06-release-readiness/tasks/E017-T07-perf-baseline-refresh.md)** — Refreshed `.perf-baseline.json` via `pnpm test:perf` (was stale since 2026-03-16).
- **[E017-T08](epics/E017-2026-09-06-release-readiness/tasks/E017-T08-verify-and-decisions.md)** — Aggregate check across T01-T07; recorded decision `E017-D3` (licence = MIT) in `.plan/decisions.jsonl`.
- **Outside AO — build gate**: full `pnpm tauri:build` from repo root, real MSI+NSIS bundles produced (`MoveUp_0.6.0_x64_en-US.msi` 6.2 MB, `MoveUp_0.6.0_x64-setup.exe` 4.3 MB). Corrected the stale 60-70 MB target in `.claude/rules/installer.md` against this measured output.
- **Outside AO — first-run browser pass**: PARTIAL. Welcome screen visually confirmed; calibration is structurally unreachable from the remote-display path (CSS hides Settings under `.remote-display` by design — desktop-only); no-sensor state unreachable because a real sensor was connected. Found and fixed 3 real bugs surfaced by the pass: mangled Polish diacritics in the welcome popup, a CWD-relative static-file path in `remote_server.rs` that could 404 depending on launch context, and unguarded Tauri `listen()`/`invoke()` calls throwing in remote mode. All three fixed same-session (506 Rust + 259 TS tests green after the fix), `.plan/BACKLOG.md` entries closed in place.
- **Outside AO — code-signing decision**: deliberately deferred by Paweł (2026-09-06); recorded as an open `.plan/BACKLOG.md` item, does not block this epic.
- **Outside AO — tag**: `v0.6.0` (already tagged during E015; confirmed still current after this epic's build gate passed).

## E019 — Backend Hardening (2026-09-06)
Run through AO (`E019-20260906-0822`), promoted at `4649dc5` after two operator
interventions — a Claude session-limit hit on the first wave, and a Windows
Smart App Control block on freshly-compiled Rust build DLLs that surfaced only
during T04's verification. Neither was a code defect. All 8 tasks merged and
independently re-verified on `main` (533 Rust + 3 integration + 261 TS tests).
See JOURNAL.md.
- **[E019-T01](epics/E019-2026-09-06-backend-hardening/tasks/E019-T01-commands-mutex-poison-safe.md)** — Poison-safe mutex pattern across 14 sites in `commands.rs`; regression test for a poisoned lock.
- **[E019-T02](epics/E019-2026-09-06-backend-hardening/tasks/E019-T02-today-totals-single-path.md)** — New `today_totals.rs` consolidating the duplicate "today's totals" computation; `.arch/ADR/014-persistence-precedence.md`.
- **[E019-T03](epics/E019-2026-09-06-backend-hardening/tasks/E019-T03-desk-events-constants.md)** — New `desk_events.rs`/`src/events.ts` — named constants for all 8 IPC event strings, used at every emit/listen site.
- **[E019-T04](epics/E019-2026-09-06-backend-hardening/tasks/E019-T04-logger-warn-and-handler-parity.md)** — `EventLogger::new` warns instead of panicking on an unusable base dir; debug/release `generate_handler!` parity enforced by a test.
- **[E019-T05](epics/E019-2026-09-06-backend-hardening/tasks/E019-T05-split-google-fit.md)** — Split `google_fit.rs`/`google_fit_service.rs` into ≤250-line siblings.
- **[E019-T06](epics/E019-2026-09-06-backend-hardening/tasks/E019-T06-split-serial-periodic.md)** — Split `serial_periodic::check_periodic`/`handle_reading` by responsibility.
- **[E019-T07](epics/E019-2026-09-06-backend-hardening/tasks/E019-T07-dedupe-remote-display-state.md)** — Deduped `tray_controller`/`remote_server` remote-display-state derivation into `remote_display_state.rs`.
- **[E019-T08](epics/E019-2026-09-06-backend-hardening/tasks/E019-T08-fix-claude-md-tray-claim.md)** — Corrected a stale `CLAUDE.md` claim about `TrayController`.
- **Outside AO**: none — all 8 tasks were backend/non-UI, verified by `cargo test`/`vitest` alone per HANDOFF.md.
