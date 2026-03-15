# Desk App — CLAUDE.md

## Purpose
Ergonomics tracker for a sit/stand desk. Detects whether user is sitting or standing via VL53L1X laser sensor, tracks session durations, and nudges the user to take breaks.

## Target User
Single user (zentala) working at a motorized sit/stand desk.

## Hardware
- **MCU**: Seeed XIAO ESP32-C3 on COM3 (Windows)
- **Sensor**: Grove VL53L1X v2 — mounted **under the desk, pointing down to the floor**
- **Height formula**: `desk_height = sensor_reading_mm - desk_thickness_mm`
- **Auto-detection**: firmware sends `DEVICE: zntl-desk-sensor v1` on connect

## User States
- `SITTING` — desk low, user at keyboard
- `STANDING` — desk high (counts as break)
- `WALKING` — desk high, user away (also a break)
- `AWAY` — no keyboard/mouse activity

## Session Logic
- Default session limit: **40 minutes** of sitting
- Break rules:
  - < 5 min standing → no effect
  - 5–9 min standing → subtract 20 min from session
  - ≥ 10 min standing → reset session to 0
- Debounce state changes: require 5s stable reading

## UI Components
1. **System tray text** — `↕ 72 cm` or session warning
2. **Floating window** — current height, today's totals, session history
3. **Top-of-screen progress bar** — green→red over 40min session, popup at limit
4. **Popup notification** — "Sitting 40 min, take a break"

## Stack
- **Frontend**: React + TypeScript (Vite, port 1443)
- **Backend**: Rust (Tauri 2)
- **DB**: SQLite via `tauri-plugin-sql`
- **Serial**: `serialport` crate, background thread
- **Activity tracking**: Rust (keyboard/mouse hooks via `rdev` or Windows API)

## Tauri Plugins
- `tauri-plugin-notification` — native desktop notifications ("time to stand")
- `tauri-plugin-autostart` — start app at system login
- `tauri-plugin-single-instance` — prevent multiple instances
- `tauri-plugin-store` — persist user config (session limit, height thresholds)
- `tauri-plugin-log` — logging
- `window-vibrancy` crate — Windows Acrylic/Mica blur effect on floating window
  - requires `"transparent": true` in tauri.conf.json + `background: transparent` in CSS
- Tray icon: built-in Tauri 2 (`tray-icon` feature) — `set_icon()` for dynamic tray icon changes

## Testing
**Full coverage required from day one.**

- **Unit tests** (Rust `#[test]`): session state machine, break rules, height parsing, device auto-detection
- **Integration tests** (Rust): serial reader with mock port, SQLite read/write
- **Hardware integration tests**: real device on COM3 — test auto-detection, real readings
  - Tag with `#[cfg(feature = "hardware-tests")]` so they don't run in CI without device
- **Frontend component tests** (Vitest + Testing Library): ProgressBar, session display
- **E2E tests** (Playwright + Tauri): full app flow — connect device, session progression, notifications
- Minimum coverage: **80%** (enforced by pre-commit hook)

## Key Files
- `firmware/` — Arduino sketch for XIAO ESP32-C3 (in `C:/code/desk-seduino/`)
- `src-tauri/src/serial.rs` — serial reader, auto-detect
- `src-tauri/src/session.rs` — session state machine
- `src-tauri/src/activity.rs` — keyboard/mouse idle detection
- `src-tauri/src/db.rs` — SQLite persistence
- `src/App.tsx` — floating window UI
- `src/components/ProgressBar.tsx` — top-of-screen overlay

## Vision Doc
See `.agent/vision/2026-03-15-desk-app-vision.md` for full spec.
