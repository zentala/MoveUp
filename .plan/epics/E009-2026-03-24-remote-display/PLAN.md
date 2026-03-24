# E009: Remote Display — Phone as Desk Dashboard (Web Kiosk)

**Created**: 2026-03-24
**Status**: Planned
**Version**: 0.3.0 (bump before first task)

---

## Goal

Turn an old Android phone into a dedicated desk dashboard. The phone displays
the same UI as the desktop popup — real-time ergonomic state, progress bars,
KPIs, timeline — fullscreen, landscape, always-on.

## Why

The developer works at a sit/stand desk tracked by a Tauri desktop app.
Currently, the only way to see desk state is via:
- System tray tooltip (tiny, requires hover)
- Popup window (ephemeral, covers other windows)
- Overlay bar (4px, shows progress only)

A dedicated phone display provides at-a-glance visibility without
interrupting workflow. Old phone sits on the desk as a permanent dashboard.

## Approach: Web Kiosk

The simplest viable solution — zero mobile code:

1. **Rust backend** gets an embedded HTTP + WebSocket server (axum)
2. Server serves the **same built React frontend** as static files
3. WebSocket pushes real-time events (same payloads as Tauri IPC)
4. **Phone** runs a kiosk browser app (e.g. Fully Kiosk Browser on Android)
5. Phone opens `http://<PC-IP>:3390/display` → fullscreen, landscape

The React frontend detects whether it runs inside Tauri (`window.__TAURI_INTERNALS__`)
or in a browser. If browser → uses `useRemoteDesk()` hook (WebSocket) instead of
`useDesk()` (Tauri IPC). Same components, same rendering, different data source.

## Architecture

```
┌────────────────── PC (Windows) ──────────────────┐
│                                                    │
│  Tauri Process                                     │
│  ├── SessionManager (existing, unchanged)          │
│  ├── Serial reader (existing, unchanged)           │
│  ├── tray_controller.rs (existing, unchanged)      │
│  │                                                 │
│  └── NEW: Remote Display Server                    │
│       ├── axum HTTP server on :3390                │
│       │   ├── GET /display      → React SPA        │
│       │   └── GET /display/api  → JSON snapshot    │
│       │                                            │
│       └── WebSocket /display/ws                    │
│           ├── On connect → send full state snapshot│
│           ├── Forward desk:* events (real-time)    │
│           └── Heartbeat every 5s                   │
│                                                    │
│  Event wiring:                                     │
│  app.emit("desk:*") ──→ tray_controller (existing) │
│                     └──→ ws_broadcaster (NEW)       │
│                          broadcasts to all WS       │
│                          clients via tokio channel   │
└────────────────────────────────────────────────────┘
            │ WiFi LAN (port 3390)
            ▼
┌────────── Phone (Android) ─────────┐
│  Fully Kiosk Browser               │
│  → http://192.168.X.X:3390/display │
│  → fullscreen, landscape locked    │
│  → auto-reconnect on WS drop       │
└─────────────────────────────────────┘
```

## Scope

### In scope
- Embedded HTTP+WS server in Rust backend (axum)
- WebSocket protocol for real-time state push
- `useRemoteDesk()` React hook (WebSocket client, same interface as `useDesk()`)
- `useDeskAuto()` wrapper that auto-selects Tauri IPC or WebSocket
- Responsive CSS for landscape phone display
- Reconnection logic with status indicator
- REST fallback endpoint for debugging (`/display/api`)
- Documentation for phone setup (Fully Kiosk Browser configuration)

### Out of scope
- Native Android app (Phase 2)
- Standalone mode / sensor-to-phone (Phase 3)
- Auto-discovery (mDNS/Bonjour)
- Authentication / PIN
- USB tethering support
- Dedicated mobile-optimized layout (responsive is enough for Phase 1)
- Settings modification from phone (read-only display)

## Acceptance Criteria

1. `pnpm tauri:dev` starts both Tauri window AND HTTP+WS server on `:3390`
2. Opening `http://<PC-IP>:3390/display` in any browser shows the desk dashboard
3. State changes (sit→stand, height changes, score updates) appear on phone within 1s
4. Disconnecting WiFi → phone shows "Reconnecting..." → auto-reconnects when WiFi returns
5. Phone in landscape shows full UI without horizontal scrolling
6. Server handles multiple simultaneous clients (phone + browser debugging)
7. No performance impact on existing Tauri app (WS broadcast is fire-and-forget)
8. All existing tests still pass (no breaking changes to existing code)

## Technical Decisions

### Why axum?
- Already using tokio (Tauri 2 runtime)
- Lightweight, no extra runtime overhead
- tower-http has static file serving built in
- Well-maintained, widely used in Rust ecosystem

### Why port 3390?
- Not conflicting with common ports (3000=React, 1443=Vite, 5432=Postgres)
- Easy to remember
- Configurable via `DESK_REMOTE_PORT` env var

### Why broadcast channel (not direct WS writes)?
- Decouples event production from WS client management
- `tray_controller.rs` just sends to channel — doesn't know about WS clients
- New clients subscribe to channel — automatic fan-out
- Dropped clients don't block event flow

### Why same React build (not separate app)?
- Zero duplication — one codebase, one build
- Changes to widgets immediately available on phone
- Test scenarios work identically in both modes
- Hook abstraction (`useDeskAuto`) is the only branching point

## Constraints

- Phone and PC must be on the same WiFi network
- Phone needs a kiosk browser app installed (free: Fully Kiosk Browser)
- User manually enters PC's IP address (no auto-discovery)
- Read-only — phone cannot change settings or calibrate sensor
- Desktop app must be running for phone to work (PC-dependent)

## Risks

| Risk | Mitigation |
|------|------------|
| Firewall blocks port 3390 | Document firewall rule setup in user docs |
| WiFi disconnects frequently | Robust reconnect with exponential backoff |
| axum adds binary size | Expected ~200KB — acceptable |
| Performance impact on main app | Broadcast channel is non-blocking, fire-and-forget |
| Phone browser doesn't support WS | REST polling fallback as degraded mode |

## References

- Existing frontend hooks: `src/hooks/useDesk.ts`, `src/hooks/useWidgetData.ts`
- Existing types: `src/types.ts` (all payloads), `src-tauri/src/session_types.rs`
- Widget system: `src/types.ts` → `WidgetProps` interface
- Vision doc: `.plan/vision/2026-03-15-desk-app-vision.md` → "Remote Display" section
- Current app state: `src-tauri/src/commands.rs` → `AppState` struct
