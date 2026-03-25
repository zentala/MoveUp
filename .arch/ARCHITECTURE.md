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
│    └── session_breaks.rs   (break credit rules)              │
│       │                                                      │
│       ├──→ db.rs / db_sessions.rs / db_queries.rs (SQLite)   │
│       │                                                      │
│       ▼                                                      │
│  tray_controller.rs ──wires events──→                        │
│    ├── tray.rs + tray_icon.rs   (system tray)                │
│    ├── overlay_renderer.rs      (WinAPI progress bar)        │
│    │    ├── overlay_opaque.rs   (GDI render backend)         │
│    │    ├── overlay_layered.rs  (experimental transparent)   │
│    │    ├── overlay_variants.rs (solid/gradient/pulsing)     │
│    │    └── overlay_standing.rs (gold bar + lap indicators)  │
│    ├── alert_manager.rs         (escalation state machine)   │
│    │    ├── alert_actions.rs    (AlertAction enum)           │
│    │    ├── alert_config.rs     (snooze durations)           │
│    │    └── alert_popup*.rs     (WinAPI popup window)        │
│    └── notification_service.rs  (desktop notifications)      │
│                                                              │
│  activity.rs  (Windows idle detection via GetLastInputInfo)  │
│  config.rs    (AppConfig, tauri-plugin-store persistence)    │
│  colors.rs    (progress-to-color mapping)                    │
│  commands.rs  (IPC: get_session_state, inject_reading, etc.) │
│  commands_config.rs (IPC: settings, calibration, overlay)    │
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

## Data Flow

```
VL53L1X sensor
    → ESP32-C3 (serial USB, 115200 baud)
    → serial.rs (auto-detect via PING/PONG handshake)
    → serial_parser.rs (parse distance readings)
    → SessionManager.on_reading() (state transitions, debounce)
    → tray_controller.rs (routes to all UI)
        → overlay_renderer.rs (WinAPI 4px bar, top of screen)
        → tray.rs (icon + tooltip update)
        → alert_manager.rs (escalation check)
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

## Key Architectural Decisions

| Decision | Rationale | Reference |
|----------|-----------|-----------|
| **WinAPI overlay** (not Tauri WebviewWindow) | Tauri windows always have decorations, min height, event delivery bugs | `.arch/overlay/decisions/` |
| **rusqlite** (not tauri-plugin-sql) | Rust owns the DB — direct access from session manager without IPC overhead | E001 |
| **MetricEngine trait** | Stateless KPI computation — each metric is a pure function of SessionState snapshot | E007 |
| **Widget architecture** | WidgetProps interface + registry — swap UI layouts without touching logic | E005 |
| **Arc<Mutex<>> shared state** | Thread-safe state between Tauri thread and WinAPI overlay thread | E002 |
| **DataSource enum** | Demo/Live/Mock modes — develop overlay without hardware | E002 |

## Native UI Elements (WinAPI, outside Tauri)

- **Overlay bar**: 4px top-of-screen, green→red (sitting) or gold (standing)
- **Alert popup**: WinAPI popup window, progressive escalation with snooze

Both run in dedicated background threads with their own Windows message loops.

## Persistence

- **SQLite** (via rusqlite): session history, daily summaries
- **tauri-plugin-store**: user config (session limits, calibration, notification prefs, widget selection)

## Stack

- **Frontend**: React + TypeScript (Vite, port 1443)
- **Backend**: Rust (Tauri 2)
- **DB**: SQLite via rusqlite
- **Serial**: `serialport` crate, background thread
- **Activity**: Windows `GetLastInputInfo` API
- **Overlay/Popups**: `windows` crate (raw WinAPI, GDI rendering)
