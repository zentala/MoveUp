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

## Testing — 3-Layer Strategy

**Coverage requirement: ≥80% (enforced by vite.config.ts coverage gate)**

### Layer 1 — Unit Tests

**Rust** (`src-tauri/src/*.rs` — `#[test]` modules):
- Session state machine: sitting/standing/walking transitions, debounce
- Break credit rules: <5min, 5-9min, ≥10min scenarios
- Daily reset: state cleanup, throttling
- Alert firing: once-per-session, suppression
- Height calculation: calibration formulas
- Run: `cargo test` from `src-tauri/`

**TypeScript** (`src/**/*.test.tsx` — Vitest + jsdom):
- Components: ProgressBar, SessionProgress, SettingsPanel, HeightRail
- Hooks: useDesk (mocked @tauri-apps/api)
- Utils: formatDuration, formatDurationShort
- Run: `pnpm test:unit`

### Layer 2 — Integration Tests

**Rust + TypeScript** (`tests/integration/*.test.ts` — Vitest + node environment):
- Use `inject_reading` command to simulate sensor readings
- Verify SessionManager state via `get_session_state` IPC
- Test 10 scenarios: sitting session, standing break, walking, debounce, alerts, daily reset, device reconnect
- App must be running: `pnpm tauri:dev` in another terminal
- Run: `pnpm test:integration`

**Device Emulator** (`tests/emulator/`):
- `DeskDeviceEmulator.ts` — formats device responses (PONG, distance, error)
- `scenarios.ts` — helpers for multi-reading sequences (debounce, position changes)
- Constants: `SITTING_DISTANCE_MM=750`, `STANDING_DISTANCE_MM=1080`, `DEBOUNCE_COUNT=5`

### Layer 3 — E2E Tests

**Playwright** (`tests/e2e/*.test.ts` — WebDriver mode):
- App startup: main window loads, UI elements visible
- Session progress: state display, progress bar updates, tray tooltip
- User interactions: settings panel, calibration, overlay toggle
- Event emissions: desk:state-changed, desk:device-connected/lost
- Setup: `tauri-driver --compatibility-mode` in one terminal, then `pnpm test:e2e`
- Run: `pnpm test:e2e` or `pnpm test:e2e:ui`

### Pre-Build Hook

Tests **must pass** before every build:
- `pnpm build` → runs `test:all` (unit + rust) → vite build
- `pnpm tauri:build` → runs `test:all` → tauri build

If tests fail, the build is halted. Commit is NOT blocked (tests don't run on pre-commit), only builds.

## Key Files
- `firmware/` — Arduino sketch for XIAO ESP32-C3 (in `C:/code/desk-seduino/`)
- `src-tauri/src/serial.rs` — serial reader, auto-detect
- `src-tauri/src/session.rs` — session state machine
- `src-tauri/src/activity.rs` — keyboard/mouse idle detection
- `src-tauri/src/db.rs` — SQLite persistence
- `src/App.tsx` — floating window UI
- `src/components/ProgressBar.tsx` — top-of-screen overlay

## Icon Requirements

Icons must exist in `src-tauri/icons/`:
- **32x32.png** — Tray icon, system tray display
- **128x128.png** — App window icon
- **icon.ico** — Installer icon, Windows display

If icons are missing, `pnpm tauri:build` fails with:
```
error: Icon file not found: icons/icon.ico
```

To generate icons from a PNG:
[Link to Tauri icon guide](https://tauri.app/en/develop/guides/assets/#icons)

## Code Signing (Scaffolded)

Code signing ensures Windows trusts the installer.

### Setup (One-time)

1. Acquire Windows code-signing certificate (.pfx file):
   - Vendor: DigiCert, GlobalSign, etc.
   - Cost: ~$200-500/year
   - File: `mycert.pfx` + password

2. Add to GitHub Secrets (Settings → Secrets and variables):
   - `SIGN_CERT_PATH`: `/home/runner/mycert.pfx` (upload as action secret)
   - `SIGN_PASSWORD`: certificate password (sensitive)

3. Reference in CI/CD (future `release.yml`):
   ```yaml
   env:
     SIGN_CERT_PATH: ${{ secrets.SIGN_CERT_PATH }}
     SIGN_PASSWORD: ${{ secrets.SIGN_PASSWORD }}
   ```

### Script

Location: `scripts/sign-installer.sh`

Currently a template. When certificate acquired, uncomment `signtool` command.

### Implementation Timeline

- Now: Scaffold placeholder
- Upon first release: Acquire cert + activate signing
- CI/CD integration: Part of T007 (release.yml)

## Auto-Update Mechanism (Scaffolded)

Users can auto-update when new versions ship to GitHub Releases.

### How It Works

1. Tauri checks `https://releases.example.com/latest.json` periodically
2. If new version available, downloads + verifies signature
3. Prompts user: "Update available: click to install?"
4. Applies update in background

### Setup (Future)

- Acquire code-signing certificate (see ## Code Signing above)
- Configure `updater` in tauri.conf.json with manifest URL
- Create GitHub Releases workflow (T007)
- Generate update manifest with signatures

### Timeline

- Now: Placeholder in tauri.conf.json
- Upon code signing: Implement updater config
- With T007: Auto-generate update manifests in CI/CD

## Memory Profiling (On-Demand)

Before tagging a release, run memory profiling:

```bash
pnpm test:perf
```

This launches the full app, measures memory for 10 seconds, and compares against baseline.

**Targets:**
- Peak memory: <250 MB (alert: >300 MB)
- Stable memory: <200 MB
- Growth vs baseline: <20 MB

**Baseline:** `.perf-baseline.json`

If memory exceeds targets, review optimization strategies in `docs/OPTIMIZATION_GUIDE.md`.

## Vision Doc
See `.agent/vision/2026-03-15-desk-app-vision.md` for full spec.
