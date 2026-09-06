# Architecture — Desk App (zntlDesk)

## System Overview

Tauri 2 desktop application for Windows. Rust backend handles hardware communication, session logic, and native UI elements. React+TypeScript frontend renders the floating popup window.

**Core purpose:** Ergonomics tracker for a motorized sit/stand desk. Detects sitting/standing via VL53L1X ToF laser sensor, tracks session durations, nudges user to take breaks via visual cues and notifications.

## Component Map

```
┌─────────────────────────────────────────────────────────────┐
│                    HARDWARE LAYER                            │
│  VL53L1X sensor → XIAO ESP32-C3 (COM3, 115200 baud)        │
└──────────────────────┬──────────────────────────────────────┘
                       │ serial USB
┌──────────────────────▼──────────────────────────────────────┐
│                    RUST BACKEND                              │
│                                                              │
│  serial.rs ──parse──→ serial_parser.rs                       │
│       │                                                      │
│       ▼                                                      │
│  session.rs (SessionManager)                                 │
│    ├── session_types.rs    (DeskState, SessionState, DTOs)   │
│    ├── session_manager.rs  (state machine, score)            │
│    ├── session_reading.rs  (reading processing)              │
│    ├── session_breaks.rs   (break credit rules)              │
│    ├── session_daily.rs    (local-day reset, score tick)     │
│    └── session_persistence.rs (versioned engine snapshot)    │
│       │                                                      │
│       ├──→ db.rs / db_sessions.rs / db_queries.rs (SQLite)   │
│       │                                                      │
│       ▼                                                      │
│  tray_controller.rs ──wires events──→                        │
│    ├── tray.rs + tray_icon.rs   (system tray icon)           │
│    ├── tray_signal_exec.rs      (executes Signals → UI)      │
│    ├── tray_blink.rs            (blink engine, 50ms thread)  │
│    ├── overlay_renderer.rs      (WinAPI progress bar)        │
│    │    ├── overlay_opaque.rs   (GDI render backend)         │
│    │    ├── overlay_variants.rs (solid/gradient/pulsing)     │
│    │    └── overlay_standing.rs (neutral bar + lap tracking) │
│    ├── communication_policy.rs  (central signal decisions)   │
│    │    ├── communication_types.rs  (Signal enums)           │
│    │    ├── communication_profile.rs (JSON profile struct)   │
│    │    └── ergonomic_profile.rs (limits/scoring/KPI)        │
│    ├── profile_loader.rs        (JSON load/validate/watch)   │
│    ├── profile_reload.rs        (hot-reload active profile)  │
│    ├── alert_popup*.rs          (WinAPI popup window)        │
│    └── notification_service.rs  (desktop notifications)      │
│                                                              │
│  activity.rs  (Windows idle detection via GetLastInputInfo)  │
│  config.rs    (AppConfig — hardware/UI/telemetry only)       │
│  colors.rs    (4-color dictionary: yellow/red/gray/neutral)  │
│  commands.rs  (IPC: get_session_state, inject_reading, etc.) │
│  commands_config.rs (IPC: settings, calibration, overlay)    │
│  commands_profiles.rs (IPC: profile list/switch/duplicate)   │
│  commands_share.rs  (IPC: get_share_text for viral sharing)  │
│  telemetry.rs       (opt-in daily aggregate telemetry)       │
│  remote_server.rs   (HTTP+WS server on :3390)               │
│  ws_broadcaster.rs  (broadcasts state to WS clients)         │
│  lib.rs       (app entry, plugins, AppState)                 │
└──────────────────────┬──────────────────────────────────────┘
                       │ IPC commands + events + HTTP/WS
┌──────────────────────▼──────────────────────────────────────┐
│                  REACT FRONTEND                              │
│                                                              │
│  App.tsx ──→ Widget system (WidgetProps interface)            │
│    ├── widgets/OneBar/       (primary widget)                │
│    ├── widgets/TimelineZen/  (timeline view)                 │
│    └── widgets/Placeholder/  (fallback)                      │
│                                                              │
│  hooks/useDesk.ts        (IPC bridge, polls session state)   │
│  hooks/useRemoteDesk.ts  (WS bridge for remote display)     │
│  hooks/useDeskAuto.ts    (auto-selects IPC or WS)           │
│  components/ConnectionOverlay.tsx (remote mode status)       │
│  components/         (ProgressBar, KpiStrip, Settings, etc.) │
│  types.ts            (shared TS types matching Rust DTOs)    │
└─────────────────────────────────────────────────────────────┘
```

## IPC Events (Rust → Frontend)

| Event | Payload | Source |
|-------|---------|--------|
| `desk:distance` | `{ distance_mm, height_cm }` | serial.rs (~1/s) |
| `desk:state-changed` | `StateChangedPayload` | session_manager.rs |
| `desk:device-connected` | `{ port }` | serial.rs |
| `desk:device-lost` | `{}` | serial.rs |

## IPC Commands (Frontend → Rust)

Key commands: `get_session_state`, `get_today_summary`, `inject_reading`, `list_ports`, `start_auto_connect`, `stop_reading`, `trigger_test_notification`, `set_session_limit`, `set_stand_limit`, `calibrate`, `get_settings`, `save_settings`, `get_overlay_state`, `dismiss_welcome`, `show_welcome`.

Rust owns the shape of everything these send. `ts-rs` writes the TypeScript
mirror into `src/generated/`, which `src/types.ts` and
`components/settings/SettingsTypes.ts` re-export — a renamed Rust field fails
`pnpm typecheck` instead of reading `undefined` at runtime. Regenerate with
`cargo test --manifest-path src-tauri/Cargo.toml --lib -- ts_export`; see
[ADR 017](ADR/017-ts-rs-for-rust-ts-codegen.md).

## Data Flow

```
VL53L1X sensor
    → ESP32-C3 (serial USB, 115200 baud)
    → serial.rs (auto-detect via PING/PONG handshake)
    → serial_parser.rs (parse distance readings)
    → SessionManager.on_reading() (state transitions, debounce)
    → tray_controller.rs (routes to all UI)
        → CommunicationPolicy.evaluate() (central signal decisions)
        → tray_signal_exec.rs (executes returned Signals)
            → overlay_renderer.rs (WinAPI 4px bar, top of screen)
            → tray.rs + tray_blink.rs (icon + blinking)
            → notification_service.rs (toast/popup)
        → IPC emit (desk:state-changed → frontend)
    → useDesk hook (frontend polls via get_session_state)
    → Widget renders (OneBar, TimelineZen, etc.)

Remote display path (parallel to IPC):
    tray_controller.rs
        → ws_broadcaster.rs (snapshot + events to all WS clients)
        → remote_server.rs (:3390 — serves React UI + /display/ws + /display/api)
        → useRemoteDesk.ts (WS client in browser, auto-reconnect)
        → Widget renders (same components, phone viewport)
```

## Session Counters (one credited value)

`SessionManager` keeps two kinds of counter, and mixing them is the bug class
E015 closed:

| Counter | Kind | Who reads it |
|---------|------|--------------|
| `sitting_seconds` → DTO `limit_used_secs` (`limitUsedSecs`) | credited — reduced by break credit (ADR 008), never reset on a return to sitting | **everything the user reads as progress**: popup timer, overlay fill, colour band, sit-limit alert |
| `sitting_seconds_total`, `standing_seconds` | raw daily totals | KPI, Analyst, PostureBalance — compared only against other raw counters |
| `secs_since_last_break` | raw, restarts on every position change | Debug tab only |

There is no separate "current session" counter. The field that used to serve
that role (`current_session_secs`) is deleted from the state, the DTO and the
`desk:state-changed` payload — a timer reading it dropped to zero on every
return to sitting, contradicting the credit the engine had just applied. The
rule is enforced by the type system: the wrong field no longer exists.

Source: `session_types.rs`, `session_breaks.rs`,
[ADR 008](ADR/008-proportional-break-credit.md) revision 2026-09-06,
[E015](../.plan/epics/E015-2026-09-06-engine-single-truth/PLAN.md).

## Session Engine (pure core)

`SessionManager` and the `session_*.rs` modules are a pure core: they hold
what happened, and take everything else as an argument. Four rules, recorded
in [ADR 015](ADR/015-pure-ergo-engine.md):

| Rule | In code |
|------|---------|
| **The caller owns the clock** | every stepping function has an `_at` variant taking `now: DateTime<Utc>` — `on_reading_at`, `check_daily_reset_at`, `needs_daily_reset_at`, `check_notification_conditions_at`, `accumulate_ongoing`, `handle_state_exit`, `policy_input`. The zero-argument names are one-line wrappers that read `Utc::now()` once and delegate; they and the adapters (`serial_periodic.rs`, `commands.rs::inject_reading`) are the only impure boundary. Tests call the `_at` form, so a midnight or DST instant is constructed, not waited for |
| **A day is a local day** | the daily reset takes the local calendar date as an explicit argument, matching `session_persistence.rs` and `db_sessions.rs`. It used to compare UTC days, so the in-memory reset fired hours away from the persisted "today" |
| **Config is borrowed, not copied into state** | `SessionState` holds no profile fields. `SessionManager.limits: Limits` is the one copy in force, refreshed each tick from the profile the serial loop already reloads (`set_limits`). A profile edit applies on the next tick; before this it silently needed a restart |
| **The engine computes every derived value a UI reads** | `policy_input(now, sensor_connected)` fills the whole `PolicyInput`, standing-lap trio and `elapsed_secs` included. `tray_controller.rs` recomputes nothing — it passes the input to `CommunicationPolicy` and hands the returned `Signals` to `tray_signal_exec.rs` |

Persistence follows from the same rule: one versioned
`PersistedEngineState { schema_version, .. }` wraps `SessionState` plus the
manager-level flags and `HourlyBreakTracker`, migrating a v0 (unversioned)
file on load, instead of a second struct kept in sync by hand.

"Break" names three unrelated things, kept apart and labelled — Session Break
Credit (`[break:credit]`, [ADR 008](ADR/008-proportional-break-credit.md)),
Day Break Credit (`[break:day]`, [ADR 009](ADR/009-day-break-credit.md)), and
hourly break coverage (`[break:hourly]`, a KPI in `hourly_break_tracker.rs`).
The table in `session_breaks.rs`'s module doc comment is the map.

Source: `session_manager.rs`, `session_reading.rs`, `session_daily.rs`,
`session_breaks.rs`, `session_persistence.rs`,
[E020](../.plan/epics/E020-2026-09-06-engine-pure-core/PLAN.md).

## Key Architectural Decisions

| Decision | Rationale | Reference |
|----------|-----------|-----------|
| **WinAPI overlay** (not Tauri WebviewWindow) | Tauri windows always have decorations, min height, event delivery bugs | `.arch/overlay/decisions/` |
| **rusqlite** (not tauri-plugin-sql) | Rust owns the DB — direct access from session manager without IPC overhead | E001 |
| **MetricEngine trait** | Stateless KPI computation — each metric is a pure function of SessionState snapshot | E007 |
| **Widget architecture** | WidgetProps interface + registry — swap UI layouts without touching logic | E005 |
| **Arc<Mutex<>> shared state** | Thread-safe state between Tauri thread and WinAPI overlay thread | E002 |
| **DataSource enum** | Demo/Live/Mock modes — develop overlay without hardware | E002 |
| **One credited session counter** | A second, uncredited counter let timer and colour disagree; deleting it makes the compiler enforce the rule | E015, ADR 008 |
| **One composition point for today's totals** | Three stores each hold part of today; composing them per call site let the startup cache seed `position_changes: 0` | E019, ADR 014 |
| **Rust generates the TypeScript DTOs** | Hand-typed mirrors of the wire format drifted silently; ts-rs turns a renamed Rust field into a `pnpm typecheck` failure | E018, ADR 017 |
| **Pure engine, impure adapters** | The engine reading its own clock made midnight and DST untestable; config copied into state made profile hot-reload a no-op; tray re-derivation let two consumers disagree | E020, ADR 015 |

## Native UI Elements (WinAPI, outside Tauri)

- **Overlay bar**: 4px top-of-screen, green→red (sitting) or gold (standing)
- **Alert popup**: WinAPI popup window, progressive escalation with snooze

Both run in dedicated background threads with their own Windows message loops.

## Persistence

- **SQLite** (via rusqlite): session history, daily summaries
- **tauri-plugin-store**: user config (session limits, calibration, notification prefs, widget selection)
- **Per-minute JSON snapshots** (`logs/YYYY-MM-DD/HH-MM.json`): the Analyst time series

The three overlap, so precedence is fixed rather than decided per call site:
SQLite owns today's durations and session list, the in-memory `SessionManager`
owns `position_changes` (the DB count excludes the running span), and
`tauri-plugin-store` owns notification flags, credited `sitting_seconds` and
`daily_score`. `today_totals::load_today_summary` is the one place that
composes them — see
[ADR 014](ADR/014-persistence-precedence.md).

## Stack

- **Frontend**: React + TypeScript (Vite, port 1443)
- **Backend**: Rust (Tauri 2)
- **DB**: SQLite via rusqlite
- **Serial**: `serialport` crate, background thread
- **Activity**: Windows `GetLastInputInfo` API
- **Overlay/Popups**: `windows` crate (raw WinAPI, GDI rendering)
