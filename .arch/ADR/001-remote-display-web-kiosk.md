# ADR 001: Remote Display via Embedded Web Server (Web Kiosk)

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: E009

## Context

We want to turn an old Android phone into a dedicated desk dashboard
showing real-time ergonomic state. The phone needs to display the same
UI as the desktop popup — fullscreen, landscape, always-on.

The question: how does the phone get the data and render the UI?

## Decision

**Phase 1: Embedded HTTP+WS server in Rust backend (web kiosk).**

The Tauri desktop app runs an axum HTTP server on port 3390 that:
- Serves the built React frontend as static files
- Exposes a WebSocket endpoint for real-time event streaming
- Uses a tokio broadcast channel to fan out events to all clients

The phone runs a kiosk browser app (Fully Kiosk Browser) pointing to
the PC's local IP. The React frontend detects browser vs Tauri mode
and uses WebSocket instead of Tauri IPC for data.

## Alternatives

### A: Tauri Mobile (native Android app)
- **Rejected for Phase 1** — Tauri Mobile is beta, requires Android SDK
  setup, APK building, more code. But planned as Phase 2 when the
  concept is validated and Tauri Mobile matures.

### B: Progressive Web App (PWA)
- **Rejected** — half-measures: no real kiosk mode, no auto-start on boot,
  no screen-always-on. Worse than both A and C.

### C: Standalone app (sensor → phone directly)
- **Rejected for now** — requires firmware changes (USB Serial → BLE/WiFi),
  USB OTG compatibility testing. Planned as Phase 3.

### D: Separate React app for phone
- **Rejected** — duplicates codebase. Same React build with hook abstraction
  (`useDesk` vs `useRemoteDesk`) achieves zero duplication.

## Consequences

- `axum` + `tower-http` added as dependencies (~200KB binary impact)
- Port 3390 must be opened in Windows Firewall
- Phone depends on PC being running (no standalone mode)
- Same React codebase serves desktop and phone (single build)
- Future phases (Tauri Mobile, standalone) build on the WS protocol established here
