---
created: 2026-03-16
status: completed
title: Overlay Progress Bar
---
# E002 — Overlay Progress Bar

## What
Implement a native WinAPI progress bar overlay at the top of the screen that shows sitting session progress. The bar is 4px tall, full screen width, always-on-top, with no decorations.

## Why
Tauri `WebviewWindow` always has system decorations (rounded corners, shadows, minimum height). Five approaches using Tauri windows all failed. A raw WinAPI window was the only viable path for a clean 4px overlay.

## Key Architectural Decisions

1. **Raw WinAPI window (not Tauri WebviewWindow)** — uses the `windows` crate already in `Cargo.toml`. `CreateWindowExW()` with `WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`.

2. **Two render backends: OPAQUE vs LAYERED** — OPAQUE uses GDI `BeginPaint`/`FillRect` (black background, reliable). LAYERED uses `UpdateLayeredWindow` with 32-bit ARGB DIBSection (transparent background, experimental).

3. **DataSource enum** — `Demo` (cycling animation for dev), `Live` (real sensor data), `Mock` (simulated 40-min session). Selected by `OVERLAY_DATA` env var.

4. **Three independent config axes (env vars)**:
   - `OVERLAY_DATA`: demo/live/mock (default: demo in debug, live in release)
   - `OVERLAY_MODE`: opaque/layered (default: opaque)
   - `OVERLAY_VARIANT`: 0 solid, 1 gradient, 2 pulsing (default: 0)
   - `OVERLAY_HEIGHT`: 1-20 px (default: 4)

5. **Shared state via `Arc<Mutex<OverlayState>>`** — written from Tauri thread, read from WinAPI thread. Dirty flag (`needs_redraw`) prevents unnecessary redraws.

## Data Flow (Live mode)
```
serial.rs --> desk:distance --> tray_controller.rs --> overlay.update(progress, color)
serial.rs --> desk:state-changed --> tray_controller.rs --> overlay.show() / hide()
```

## Scope
- WinAPI overlay window creation and rendering
- Demo mode (always visible, cycling progress)
- Live mode (wired to sensor data)
- Mock mode (simulated sit/stand cycle)
- OPAQUE and LAYERED render backends
- Bar variants: solid, gradient, pulsing
- Configurable bar height
- Device disconnected notification
- Rust unit tests for overlay logic
- File split (overlay_renderer.rs was 1132 lines)
- Precommit line count hook

## Out of Scope
- macOS/Linux support (Windows-only)
- Multi-monitor spanning (V1: primary monitor only)
- Choose production render mode (T-OVR-009 deferred)
- Verify debug overlay info (T-OVR-011 deferred)

## Acceptance Criteria
- [x] 4px bar at top of primary monitor, no decorations
- [x] Always-on-top
- [x] Colors: green (0-60%) -> amber (60-85%) -> red (85%+)
- [x] Shows when Sitting, hides otherwise
- [x] Demo mode: cycling animation visible in dev
- [x] Device disconnected notification fires
- [x] 101 Rust unit tests pass
- [x] All overlay files under 250 lines
