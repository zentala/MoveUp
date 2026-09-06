# Backend architecture review — MoveUp (desk app) Rust/Tauri layer

## TLDR

The backend (108 files, ~18.5k lines) is a Tauri 2 desktop app with a real
serial→session→UI pipeline. Tests are green: **492 passed, 0 failed** (`cargo
test --lib`, 0.15s). Clippy could not run (`cargo-clippy` component not
installed on this toolchain — see GAPS). The session/break-credit engine was
out of scope (reviewed elsewhere); everything *around* it is generally
well-factored into small files, but three structural claims in `CLAUDE.md`
and `.arch/ARCHITECTURE.md` don't match the code: (1) `TrayController` is not
"only executing signals" — it computes `PolicyInput`, standing-lap math,
tooltip text and remote broadcasting itself; the pure executor is
`tray_signal_exec.rs`. (2) There is no domain core free of Tauri: nearly
every module (including `serial_periodic.rs`, `commands*.rs`,
`profile_reload.rs`) takes `AppHandle`/`tauri::State` directly, so the
session/policy domain logic is reachable only through Tauri plumbing. (3)
Persistence is **four separate mechanisms** (SQLite, tauri-plugin-store, JSON
snapshot files, plain-text event log) with real semantic overlap between the
sessions table and the snapshot/event logs, and no single source of truth
document describing which one wins when they disagree. The IPC surface is 34
commands across 10 files, generally thin and consistently
`Result<T, String>`, but error handling is inconsistent: 244 `.unwrap()`
matches across 32 files, and `commands.rs` (a hot, load-bearing file) uses
plain `.unwrap()` on every lock instead of the poison-recovering
`unwrap_or_else(|e| e.into_inner())` pattern used correctly elsewhere
(`tray_controller.rs`, `tray_signal_exec.rs`, `remote_server.rs`). Two files
exceed the repo's 250-line cap (`google_fit.rs` 473, `google_fit_service.rs`
328) with no split-sibling pattern applied, unlike every other oversized
cluster in the codebase which *does* follow the `<module>_tests.rs` /
`<module>_helpers.rs` split convention consistently and well.

## Module map

| Cluster | Purpose | Lines | Verdict |
|---|---|---|---|
| `lib.rs` | Tauri builder wiring, `AppState`, plugin registration, invoke_handler list | 250 | OK — but duplicated invoke_handler list for debug/release (see Findings) |
| `setup_helpers.rs` | App setup orchestration: autostart, window position, remote server, device notifications, shutdown flush | 442 (incl. tests) | Borderline "god function" — `perform_app_setup` does 10 distinct things sequentially (see Findings) |
| `commands.rs` + `commands_*.rs` (10 files) | IPC surface: 34 `#[tauri::command]` functions | ~1,900 total | Thin, consistent `Result<_, String>` shape; inconsistent lock-poison handling |
| `tray.rs`, `tray_controller.rs`, `tray_signal_exec.rs`, `tray_blink.rs`, `tray_icon.rs`, `tray_helpers.rs` | Tray icon, event wiring, signal execution, blink engine | 1,257 | Well split, but `tray_controller.rs` does more than "execute signals" — see Findings |
| `communication_policy*.rs`, `communication_profile*.rs`, `communication_types.rs`, `ergonomic_profile.rs` | Central signal-decision engine + two JSON profile schemas | ~1,600 | Coherent design, cleanly separated stateful policy vs. stateless helpers |
| `profile_loader.rs`, `profile_reload.rs` | JSON profile load/validate/hot-reload | 323 | Clean, small, single-purpose |
| `overlay_*.rs` (renderer/opaque/layered/variants/standing + tests) | WinAPI GDI overlay bar | ~1,600 | Isolated behind `OverlayRenderer` API; Windows-only by necessity (documented) |
| `alert_popup*.rs` | WinAPI popup window | 371 | Thread-per-popup model, isolated |
| `notification_service.rs`, `notify.rs` | Notification intent building + toast dispatch | 249 | Small, clear |
| `serial.rs`, `serial_parser.rs`, `serial_periodic.rs` | Hardware I/O, background reader thread, periodic housekeeping | 576 | `serial_periodic.rs::check_periodic`/`handle_reading` do too much for their names (DB writes + notifications + profile reload + snapshot logging all inline) |
| `db.rs`, `db_sessions.rs`, `db_queries.rs`, `db_backup.rs` | SQLite schema, CRUD, backups | 701 (+ tests) | Reasonable split; in-place `ALTER TABLE` migration with no version table (see Findings) |
| `remote_server.rs`, `ws_broadcaster.rs` | HTTP+WS server for phone/browser display | 374 | Clean; duplicates `broadcast_remote_state`/`build_remote_display_state` logic with `tray_controller.rs::broadcast_remote_state` (see Findings) |
| `google_fit.rs`, `google_fit_models.rs`, `google_fit_service.rs`, `commands_google_fit.rs` | Google Fit OAuth2 + steps integration | 1,157 (+ tests) | Best-documented module in the codebase; two files breach the 250-line cap |
| `config.rs` | `AppConfig` (calibration/UI/telemetry prefs) via tauri-plugin-store | 118 | Clean |
| `telemetry.rs` | Opt-in daily telemetry payload builder | 122 | Dead-ended: builds and logs a payload, never sends it — stale comment tells future readers not to add `reqwest`, which is already a dependency |
| `event_logger.rs`, `snapshot_logger.rs` | Plain-text event log + per-minute JSON snapshots | 368 | Both solid, both panic-free except `EventLogger::new` which **panics** on dir-create failure |
| `metrics/` | `MetricEngine` trait + 4 stateless KPI metrics | 484 (incl. tests) | Textbook — matches ADR "MetricEngine trait" claim exactly |
| `height_stabilizer/` | Rolling-window height smoothing | 351 (incl. tests) | Clean, well-tested pure logic |
| `activity.rs` | Windows idle detection (`GetLastInputInfo`) | not read in full (small, referenced) | Windows-only, expected |
| `session*.rs` (23 files) | Session state machine, break credit, KPI | ~4,000+ | **Out of scope** — reviewed by another agent |

## Findings table

| # | Finding | Path:line | Importance | Points |
|---|---|---|---|---|
| 1 | `TrayController` is not "only executing signals" as `CLAUDE.md` claims — it computes `PolicyInput` (elapsed_secs, standing lap math), builds tooltip text, decides overlay show/hide for Sitting, and broadcasts to WS clients. The real signal *executor* is `tray_signal_exec.rs`. Either the doc claim or the module boundary should change. | `src-tauri/src/tray_controller.rs:53-186`, `src-tauri/src/tray_signal_exec.rs:96-247` | Med | 3 |
| 2 | No domain core free of Tauri/WinAPI. `.arch/ARCHITECTURE.md`/`DDD.md` imply a layered design, but `serial_periodic.rs`, `profile_reload.rs`, most `commands_*.rs` take `AppHandle`/`tauri::State` directly and reach into `tauri_plugin_store::Store` inline — there is no pure "session+policy" crate/module boundary a test or a future non-Tauri frontend could depend on without pulling in Tauri. | `src-tauri/src/serial_periodic.rs:21-100`, `src-tauri/src/profile_reload.rs:36-108` | Med | 8 |
| 3 | Four overlapping persistence mechanisms with no documented precedence: SQLite `sessions` table, `tauri-plugin-store` (config + `persisted_session_state` + active profile IDs), per-minute JSON snapshot files, and plain-text `events.log`. Session totals are reconstructed from DB on `ensure_initialized`, then patched from store's `persisted_session_state`, and independently re-derived from snapshot files for the Analyst view — three different code paths compute "today's totals" (`db_sessions::load_today_totals`, `session_persistence::PersistedSessionState::load`, `commands_analyst::collect_snapshots`). A discrepancy between these is currently undetectable at runtime. | `src-tauri/src/commands.rs:60-82`, `src-tauri/src/session_persistence.rs` (not in scope but referenced), `src-tauri/src/commands_analyst.rs:129-159` | High | 8 |
| 4 | `commands.rs` — the hottest, most load-bearing IPC file — uses plain `.unwrap()` on every mutex lock (14 occurrences), while newer files (`tray_controller.rs`, `tray_signal_exec.rs`, `remote_server.rs`) consistently use `unwrap_or_else(\|e\| e.into_inner())` to survive a poisoned lock. A panic anywhere while a lock is held (e.g. in `session.on_reading`) will poison the mutex and then crash every subsequent IPC call that touches `commands.rs`, defeating the poison-recovery pattern established elsewhere. Violates `rules/rust.md` ("No `unwrap()` in production code"). | `src-tauri/src/commands.rs:44,74,75,80,86,89,95,97,102,155,166,182,193,199,216` | High | 3 |
| 5 | `google_fit.rs` (473 lines) and `google_fit_service.rs` (328 lines) both breach the repo's 250-line file cap (`CLAUDE.md`/`rules/rust.md`/`rules/code-style.md`), unlike every other oversized cluster in the codebase, which is split via a consistent `<module>_tests.rs` / `<module>_helpers.rs` / `<module>_defaults.rs` pattern (see `communication_profile*.rs`, `tray_signal_exec.rs` extracted from `tray_controller.rs`, `commands_catalog_sources.rs` extracted from `commands_catalog.rs`). Google Fit is otherwise the best-documented module in the tree — the size violation looks like an oversight, not a deliberate exception. | `src-tauri/src/google_fit.rs` (473 lines), `src-tauri/src/google_fit_service.rs` (328 lines) | Med | 3 |
| 6 | `EventLogger::new` panics (`unwrap_or_else(\|e\| panic!(...))`) if the log directory cannot be created, and this runs during `perform_app_setup` before the tray/serial scan are set up — a permissions issue on the log directory would crash the whole app at startup instead of degrading gracefully (the sibling `SnapshotLogger` only warns on the same failure class). Inconsistent failure philosophy for two structurally identical loggers. | `src-tauri/src/event_logger.rs:25-35`, contrast `src-tauri/src/snapshot_logger.rs:39-43` | Med | 2 |
| 7 | `lib.rs` maintains two near-identical, hand-duplicated `tauri::generate_handler![...]` lists (debug/test vs release) that differ by exactly 3 commands (`inject_reading`, `get_overlay_state`, plus the test cfg gate). Any new command must be remembered in both lists or it silently only exists in one build profile — no compile-time or test-time check enforces parity. | `src-tauri/src/lib.rs:155-232` | Med | 2 |
| 8 | `setup_helpers::perform_app_setup` is a 60-line, 10-step orchestration function (logging, profiles, DB backup, snapshot/event loggers, autostart, tray, tray_controller, blink thread, window position, session restore from store, welcome popup, remote display, serial scan, device notifications) with no phase separation or error aggregation — a failure partway through (e.g. `app.path().app_data_dir()?`) aborts everything after it with a single `?`, and there is no logged manifest of "what actually started". Borderline god-function; not yet in violation of the 50-line rule only because it delegates to helper functions, but the *sequencing knowledge* (must autostart before tray, must load config before serial scan) lives only in this one function's order. | `src-tauri/src/setup_helpers.rs:21-82` | Med | 3 |
| 9 | Duplicate remote-broadcast logic: `tray_controller::broadcast_remote_state` (called every ~1s tick) and `remote_server::build_remote_display_state` (called on WS client connect / REST poll) independently re-derive the same `RemoteDisplayState { session, metrics, today }` shape from `AppState`/`RemoteState`, with slightly different locking order and no shared helper. A metrics-computation change must be made in two places. | `src-tauri/src/tray_controller.rs:167-186`, `src-tauri/src/remote_server.rs:159-190` | Med | 2 |
| 10 | `telemetry.rs::send_telemetry_if_enabled` is dead-ended: it builds a payload, logs it via `debug!`, and never sends an HTTP request. Its own comment says "Do NOT add reqwest as a dependency yet — wait until the telemetry backend is deployed", but `reqwest` **is already a dependency** (`Cargo.toml:35`, used by `google_fit.rs`). The comment is stale and mildly misleading about the actual blocker (there's no deployed telemetry endpoint, not a missing crate). | `src-tauri/src/telemetry.rs:75-84`, `src-tauri/Cargo.toml:35` | Low | 1 |
| 11 | DB schema migration is ad-hoc `ALTER TABLE ... ADD COLUMN` gated by `PRAGMA table_info` string checks, with no schema-version table. Two migration generations already exist (`sitting_seconds`/`standing_seconds`/`position_changes`/`session_limit_secs`, then `date_local`) chained through nested `if let Ok(cols)` — a third migration will nest again. Works today (tests cover it: `test_schema_migration_adds_new_columns`), but doesn't scale past 2-3 more schema changes. | `src-tauri/src/db.rs:36-68` | Low | 3 |
| 12 | `commands_share.rs` hardcodes the legacy domain `desk.zentala.io` in the user-facing share text, and `commands_welcome.rs`/`alert_popup` UI strings say "Smart Desk" — both are documented legacy names per `CLAUDE.md` ("must not appear in code that looks anything up by name"), but this is plain UI-string legacy naming, not lookup logic, so it's cosmetic drift rather than a functional bug. Flagging per the "znalazłeś błąd → napraw albo zapisz" rule. | `src-tauri/src/commands_share.rs:60`, `src-tauri/src/commands_welcome.rs:13`, `src-tauri/src/notify.rs:19` | Low | 1 |
| 13 | `serial_periodic::check_periodic` and `handle_reading` each independently re-fetch `comm_policy` (ergo + comm clones), lock `session` multiple times in sequence rather than once, and inline DB writes, notification dispatch, snapshot logging, and profile-reload triggering — a 100+-line function doing five unrelated jobs per the file's own doc comment promising "Extracted... to keep serial.rs focused on I/O", which it exceeds. | `src-tauri/src/serial_periodic.rs:21-100`, `104-224` | Low | 3 |
| 14 | 244 `.unwrap()` call sites across 32 non-test-only files (test-only files dominate the count, but `commands.rs`, `serial.rs`, `serial_periodic.rs`, `db_backup.rs`, `event_logger.rs`, `google_fit_service.rs`, `snapshot_logger.rs`, `tray_helpers.rs`, `setup_helpers.rs` all carry production `.unwrap()`s outside tests) — no single policy decides which subset is "acceptable because state is invariant" vs. "should recover". Worth a follow-up pass to classify and convert the mutex-lock ones to the poison-safe pattern uniformly. | grep result, see Grep command in this review | Low | 5 |

## Persistence mechanisms inventory

| Mechanism | What it stores | Written by | Read by |
|---|---|---|---|
| SQLite (`{app_data_dir}/desk.db`, `sessions` table) | Completed session rows (start/end/state/duration/position_changes/date_local) | `db_sessions::insert_session` (on state transition + graceful shutdown) | `commands::get_today_summary`, `commands_analyst::get_sessions_range`, `db_queries::get_today_summary` |
| `tauri-plugin-store` (JSON key-value file) | `app_config` (calibration/UI/telemetry), `persisted_session_state` (flags + credit-reduced sitting seconds), `active_communication_profile`, `active_ergonomic_profile`, `reset_after` | `commands_config::save_settings`, `session_persistence::save_via_app`, `commands_profiles::save_active_id` | `commands::ensure_initialized`, `setup_helpers::perform_app_setup`, `profile_reload.rs` |
| JSON minute snapshots (`{app_data_dir}/logs/YYYY-MM-DD/HH-MM.json`) | Full `SessionStateDto` + connection + metrics, one file per minute, 7-day retention | `snapshot_logger::SnapshotLogger::log_snapshot` (called from `serial_periodic::check_periodic`, ~1/min) | `commands_analyst::get_snapshots_range` (Analyst Explorer only) |
| Plain-text event log (`{app_data_dir}/logs/YYYY-MM-DD/events.log`) | Line-per-event: STATE, CREDIT, NOTIF, DEVICE, ALERT, RESET, START, AUTOSTART | `event_logger::EventLogger::log` (many call sites) | `commands_analyst::get_events_range`, `commands_analyst::parse_event_line` |
| DB backups (`{app_data_dir}/backups/desk-*.db`) | Timestamped full copies of `desk.db`, max 30 kept | `db_backup::backup_database` (on every startup) | `commands_backup::list_db_backups` / `restore_db_backup` |
| In-memory only | `AppState` (session, comm_policy, alert_popup, today_cache), WS broadcast channel (capacity 64, dropped on disconnect) | — | tray/overlay/remote clients |

No single document states precedence when these disagree (Finding #3).

## Config sources inventory

| Source | Scope | Examples |
|---|---|---|
| `AppConfig` (via `tauri-plugin-store`) | Hardware calibration + UI prefs + telemetry opt-in | `sitting_mm`, `standing_mm`, `desk_thickness_mm`, `active_widget`, `timeline_skin`, `show_welcome_on_startup`, `show_activity_status`, `telemetry_enabled`, `telemetry_device_id` |
| `ErgonomicProfile` (JSON file, hot-reloadable) | Limits, scoring, KPI thresholds, break credit | `{app_data_dir}/profiles/ergonomic/*.json` — built-ins: default/standard, strict, relaxed, demo |
| `CommunicationProfile` (JSON file, hot-reloadable) | Escalation timing, channels, blink/overlay patterns, messages, snooze | `{app_data_dir}/profiles/communication/*.json` — built-ins: default, aggressive, gentle, silent, demo |
| Env vars — overlay | Overlay dev-mode axes, read once at process start | `OVERLAY_DATA` (demo/live/mock), `OVERLAY_MODE` (opaque/layered), `OVERLAY_VARIANT` (0/1/2), `OVERLAY_HEIGHT` (1-20px) |
| Env vars — remote display | Server port / dist path | `DESK_REMOTE_PORT` (default 3390), `DESK_REMOTE_DIST` (release-only static dir) |
| Env vars — Google Fit | OAuth2 credentials, source override | `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REFRESH_TOKEN`, `GOOGLE_FIT_STEPS_SOURCE` |
| `.env` / `.env.local` (via `dotenv` crate) | Loads the above Google Fit env vars in dev; best-effort, absence tolerated | `src-tauri/src/lib.rs:129-130` |
| `Cargo.toml` build-time | `[[bench]]` `session_state_machine`, `[lib]` crate-type triple (staticlib/cdylib/rlib — the cdylib+rlib mix is unusual for a Tauri app; rlib is used by the bench, staticlib/cdylib by Tauri's mobile/desktop bundlers) | — |

Config sprawl: **5 distinct places** a behavior-affecting value can live
(store-backed `AppConfig`, 2 profile-JSON kinds, env vars split three ways,
`.env`). None conflict today, but there is no single "effective config" dump
command an operator could call to see the merged view (the closest is
`get_overlay_state`, debug-only and overlay-scoped only).

## Event/IPC inventory

### Tauri events (`app.emit` / `app.listen`), 8 distinct `desk:*` names

| Event | Emitted by | Listened by | Payload |
|---|---|---|---|
| `desk:distance` | `serial.rs:115` | frontend only (no Rust listener) | `DistanceReading { mm, cm, timestamp }` |
| `desk:state-changed` | `commands.rs:220` (test/debug `inject_reading`), `serial_periodic.rs:145` | `tray_controller.rs:22` | `StateChangedPayload` |
| `desk:device-connected` | `serial.rs:173-176` | `setup_helpers.rs:233` (WS broadcast), `tray_controller.rs:44` (clears disconnect flag) | `DeviceConnected { port }` |
| `desk:device-lost` | `serial.rs:102` | `setup_helpers.rs:196,244`, `tray_controller.rs:34` | `()` |
| `desk:device-missing` | `serial.rs:201` | `setup_helpers.rs:182`, `tray_controller.rs:39` | `()` |
| `desk:sensor-error` | `serial.rs:218` (`emit_error`) | no Rust listener found (frontend-only, unverified) | `SensorError { message, timestamp }` |
| `desk:daily-reset` | `serial_periodic.rs:48` | `setup_helpers.rs:251` (WS broadcast + cache clear) | `()` |
| `desk:show-widget`, `desk:show-settings`, `desk:popup-theme` | `tray.rs:175,185`, `tray_signal_exec.rs:234` | frontend only | `()` / `()` / theme string |

Events are **stringly typed on both emit and listen sides** — no shared
Rust constant module for event names (each site retypes the literal
`"desk:state-changed"` etc.), and no generated/shared contract with the TS
side (`src/types.ts`, out of scope for this review but worth cross-checking).
A typo in one of the ~15 call sites would fail silently (Tauri's `listen`
just never fires). This is the "stringly scattered" pattern the review
prompt asked about — confirmed: **8 distinct event names, 0 centralized
constants module**, scattered across `serial.rs`, `serial_periodic.rs`,
`commands.rs`, `tray.rs`, `tray_signal_exec.rs`, `setup_helpers.rs`,
`tray_controller.rs`, `ws_broadcaster.rs` (which re-encodes 5 of the 8 as a
separate `DisplayEvent` enum for the WS channel, serialized via `serde(tag =
"event", content = "payload")` — a *second*, better-typed representation of
the same event universe, not shared with the Tauri-event side of the house).

### IPC commands — 34 total across 10 files

| File | Commands | Count |
|---|---|---|
| `commands.rs` | `list_ports`, `start_auto_connect`, `stop_reading`, `get_session_state`, `get_dashboard_state`, `get_connected_port`, `get_today_summary`, `inject_reading` (test/debug only), `trigger_test_notification` | 9 (8 in release) |
| `commands_config.rs` | `get_settings`, `save_settings`, `calibrate`, `set_session_limit`, `set_stand_limit`, `get_overlay_state` (debug only) | 6 (5 in release) |
| `commands_welcome.rs` | `dismiss_welcome`, `show_welcome` | 2 |
| `commands_share.rs` | `get_share_text` | 1 |
| `commands_backup.rs` | `list_db_backups`, `restore_db_backup` | 2 |
| `commands_profiles.rs` | `list_communication_profiles`, `list_ergonomic_profiles`, `get_active_profiles`, `switch_communication_profile`, `switch_ergonomic_profile`, `open_profile_in_editor`, `duplicate_profile` | 7 |
| `commands_analyst.rs` | `get_snapshots_range`, `get_events_range`, `get_sessions_range` | 3 |
| `commands_catalog.rs` | `get_data_catalog` | 1 |
| `commands_google_fit.rs` | `get_steps_today`, `refresh_steps_now` | 2 |
| `tray.rs` | `open_analyst_window` | 1 |

Naming is consistent (`snake_case`, verb-first: `get_`/`set_`/`list_`/
`switch_`/`save_`). Error handling is consistently `Result<T, String>` (no
custom error enum crossing the IPC boundary — acceptable for a single-user
desktop app, but every error becomes an opaque string on the TS side with no
machine-readable `kind`, unlike `google_fit`'s `StepsView.error_kind` pattern
which is strictly better and not reused elsewhere). Commands are thin: almost
all delegate to a `crate::db_*`/`crate::session*`/`crate::profile_loader`
function after `ensure_initialized`; the one outlier is
`commands_profiles::duplicate_profile`, which inlines file I/O + JSON
patching directly in the command body (42 lines) rather than delegating to
`profile_loader.rs`.

## Recommended target structure

Given this is a single-user desktop app (not a service with multiple
consumers), a full crate split is over-engineering. Two proportionate moves:

1. **Keep the flat `src/` layout** — the existing `<module>` /
   `<module>_tests.rs` / `<module>_helpers.rs` convention already gives
   progressive disclosure without directory ceremony, and it is applied
   consistently everywhere except Google Fit (Finding #5). Do not introduce
   a `domain/`, `infra/`, `adapters/` directory split — it would fight the
   250-line-file convention that is already doing that job at file
   granularity.
2. **Extract two things that would pay for themselves**:
   - A `desk_events.rs` module with `pub const` event-name strings (or a
     `DeskEvent` enum with `as_str()`), so `emit`/`listen` pairs stop
     retyping literals — closes the "stringly scattered" gap without
     inventing a second serialization layer (Finding, Event/IPC inventory).
   - Split `google_fit.rs`/`google_fit_service.rs` following the existing
     pattern already used for `communication_profile*.rs` (e.g.
     `google_fit_client.rs` for the raw HTTP calls, keep `google_fit.rs` for
     credentials + orchestration).
3. **Rewrite, not keep**: `serial_periodic::check_periodic`/`handle_reading`
   — split by responsibility (daily-reset, snapshot-logging,
   notification-dispatch, profile-reload-trigger) the same way
   `tray_controller.rs`/`tray_signal_exec.rs` were split. This is the
   clearest "should be two-plus files" case in the non-session code.
4. **Do not** attempt the "domain core free of Tauri" ideal (Finding #2) as
   a standalone refactor — the session/policy logic already tests cleanly
   without a running Tauri app (492 tests, many construct `SessionManager`/
   `CommunicationPolicy` directly), so the *practical* value of extracting a
   Tauri-free crate is low for a single-binary app; the ADR/DDD docs'
   language should be corrected instead of the code being forced to match
   it.

## GAPS

- **Clippy did not run.** `cargo-clippy` is not installed for this
  toolchain (`stable-x86_64-pc-windows-msvc`) — `rustup component add
  clippy` was not run because it would modify the user's toolchain outside
  this review's remit. Warning count in the prompt's ask ("run clippy, grep
  -c warning") is **unknown**, not zero — do not read the earlier `0` as a
  clean bill of clippy health.
- **Frontend TS side of the event contract was not cross-checked.** The
  Event/IPC inventory lists Rust-side `emit`/`listen` pairs only; whether
  `src/types.ts` and `useDesk.ts` actually listen for `desk:sensor-error`
  (no Rust listener was found for it) was not verified — this review did
  not open the `src/` (frontend) tree per the assigned scope.
  `desk:show-widget` was found in `tray.rs` but I did not confirm the
  frontend still consumes it.
  the frontend still consumes it.
- **`activity.rs` and `colors.rs` were read only via grep/test-name context,
  not in full** — line counts and a doc comment were seen, but full-file
  review was skipped to stay within scope/time; both are small (their test
  names look complete and specific) and low risk given they're pure/Windows
  API wrappers with passing tests.
- **`overlay_layered.rs`/`overlay_opaque.rs`/`overlay_variants.rs` internals
  (GDI drawing code) were not read line-by-line** — reviewed via file list,
  line counts, and their test files only, per the prompt's framing that
  overlay is in-scope structurally but the session engine (not overlay) was
  the excluded-in-depth cluster. If Windows-GDI-specific bugs exist, they
  are not covered by this review.
- **No attempt was made to run the app or exercise IPC end-to-end** — this
  is a static/read-only architecture review per the task; "tests pass"
  (492/492) is the only executed evidence gathered, consistent with
  `rules/evidence.md`'s distinction between a proxy and a live check.
- **Session/break-credit engine internals** (`session*.rs`, 23 files,
  `hourly_break_tracker.rs`, `ergonomic_profile.rs`) were deliberately
  **not** reviewed in depth per the task's explicit exclusion — only their
  external call shape (as seen from `commands.rs`, `serial_periodic.rs`,
  `tray_controller.rs`) was considered.
