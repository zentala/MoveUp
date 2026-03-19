# Desk App — CLAUDE.md

## Purpose
Ergonomics tracker for a sit/stand desk. Detects whether user is sitting or standing via VL53L1X laser sensor, tracks session durations, and nudges the user to take breaks.

## Target User
Single user (zentala) working at a motorized sit/stand desk.

## Project Map
See [PROJECT.xml](./PROJECT.xml) for a full structured map of the codebase, architecture, IPC events, and test strategy.

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

## Overlay Progress Bar — Development

**Project folder:** `.claude/overlay/`

| File | Purpose |
|------|---------|
| `.claude/overlay/DEVELOPER-GUIDE.md` | **START HERE** — env vars, modes, styles, testing |
| `.claude/overlay/KNOWLEDGE-BASE.md` | Architecture, root causes, rules |
| `.claude/overlay/MODE-COMPARISON.md` | OPAQUE vs LAYERED technical comparison |
| `.claude/overlay/TASKS.md` | Development tasks (T-OVR-001 through T-OVR-009) |
| `.claude/overlay/ORCHESTRATOR.md` | Wave-based parallel execution plan |
| `.claude/overlay/v1-opaque-debugging/` | Historical debugging session (archived) |
| `.claude/overlay/test-infrastructure/` | auto-test.sh, screenshot tests |

**Key rules (from debugging session 2026-03-19):**
- Progress bar is INVISIBLE without desk sensor (`visible=false` by default)
- To see bar in development: use dev mode (`OVERLAY_DEV_MODE=true`)
- Two render backends: OPAQUE (GDI, black bg) and LAYERED (UpdateLayeredWindow, transparent)
- Test the SAME mode the user sees — don't test LAYERED if user runs default OPAQUE
- Don't remove working code without proven replacement
- Document every iteration in `.claude/overlay/v1-*/OVERLAY-REPORT.md`

**Dev mode commands:**
```bash
# See bar immediately (OPAQUE, dev mode)
OVERLAY_DEV_MODE=true pnpm tauri:dev

# See bar (LAYERED, transparent background)
OVERLAY_MODE=layered OVERLAY_DEV_MODE=true pnpm tauri:dev
```

**Memory references:**
- Persistent learnings: `C:\Users\zentala\.claude\projects\C--code-zntl-tray\memory\`
- `memory/MEMORY.md` — index of all memory entries (auto-loaded by Claude Code)
- Memory is stored OUTSIDE the project dir (per Claude Code design) — not moveable

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

### Application Code
- `firmware/` — Arduino sketch for XIAO ESP32-C3 (in `C:/code/desk-seduino/`)
- `src-tauri/src/serial.rs` — serial reader, auto-detect
- `src-tauri/src/session.rs` — session state machine
- `src-tauri/src/activity.rs` — keyboard/mouse idle detection
- `src-tauri/src/db.rs` — SQLite persistence
- `src/App.tsx` — floating window UI
- `src/components/ProgressBar.tsx` — top-of-screen overlay

### Build & Distribution
- `scripts/build-report.js` — Generates `.build-sizes.json` (installer metrics)
- `scripts/sign-installer.sh` — Code signing scaffold (future)
- `src-tauri/tauri.conf.json` — Tauri config (icon paths, updater, permissions)
- `.build-sizes.json` — Latest installer size metrics
- `.perf-baseline.json` — Memory profiling baseline

### Documentation
- `docs/README.md` — User documentation index
- `docs/USER_INSTALL.md` — Installation guide for users
- `docs/USER_UPDATES.md` — Auto-update behavior guide
- `docs/USER_SUPPORT.md` — Troubleshooting and common issues
- `docs/PRIVACY.md` — Privacy policy (data stays local)
- `docs/OPTIMIZATION_GUIDE.md` — Performance baselines and optimization strategies

## Installer & Distribution

### Build Command

```bash
pnpm tauri:build
```

This command:
1. Runs `pnpm test:all` (unit + Rust tests must pass)
2. Builds React frontend (optimized, code-split)
3. Compiles Rust backend with LTO enabled
4. Generates Windows installer (NSIS format)
5. Outputs to `src-tauri/target/release/bundle/`

### Output Files

- `zntlDesk_*.exe` — Installer executable
- `.exe.sig` — Code signature (if signing enabled)
- `Latest.json` — Auto-update manifest (generated by CI/CD)

### Size Targets & Baselines

**Installer size:** 60–70 MB (NSIS compressed)
**Peak memory:** < 250 MB (alert: > 300 MB)
**Stable memory:** < 200 MB
**Bundle size growth:** < 20 MB per release

To check current sizes:
```bash
pnpm build:report
```

Output: `.build-sizes.json` (installer bytes, timestamps)

### Memory Profiling

Before tagging a release, run:
```bash
pnpm test:perf
```

This launches the full app, samples memory every 500ms for 10 seconds, and compares against baseline.

**Baseline file:** `.perf-baseline.json`
- Peak memory (MB)
- Stable memory (MB)
- Trend (stable/growing/shrinking)

If memory exceeds targets, review strategies in `docs/OPTIMIZATION_GUIDE.md`.

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

Code signing ensures Windows trusts the installer and prevents "Unknown Publisher" warnings.

### Why Code Signing?

When users download zntlDesk:
- **Without signing:** "Unknown Publisher" warning, SmartScreen blocks
- **With signing:** Trusted publisher, installs without warnings

### Setup (One-time)

1. **Acquire Windows code-signing certificate** (.pfx file):
   - Vendor: DigiCert, GlobalSign, Sectigo, etc.
   - Cost: ~$200–500/year
   - Requirements: Personal ID, proof of company/domain ownership
   - File: `mycert.pfx` + private key password

2. **Store certificate securely:**
   - Never commit `.pfx` to Git
   - Store on secure file storage or GitHub Secrets

3. **Add to GitHub Secrets** (Settings → Secrets and variables):
   - Secret name: `SIGN_CERT_PFX`
   - Value: Base64-encoded `.pfx` file content
   - Secret name: `SIGN_CERT_PASSWORD`
   - Value: certificate password

4. **Reference in CI/CD** (`.github/workflows/release.yml`):
   ```yaml
   env:
     SIGN_CERT_PATH: ./cert.pfx
     SIGN_PASSWORD: ${{ secrets.SIGN_CERT_PASSWORD }}
   ```

### Signing Script

Location: `scripts/sign-installer.sh`

**Current status:** Template (placeholder commands)
**Future:** Will use `signtool.exe` (Windows SDK) to sign `.exe`

### Implementation Timeline

- **Now:** Scaffolded (placeholder in scripts/)
- **Upon first release:** Acquire certificate + integrate into CI/CD
- **T007 (GitHub Releases):** Full CI/CD integration with signing

## Auto-Update Mechanism (Scaffolded)

Users can auto-update when new versions ship to GitHub Releases.

### How It Works (Future Implementation)

1. **Check phase:** Tauri checks GitHub Releases for new versions (every 24 hours)
2. **Download:** If new version found, downloads installer in background
3. **Verify:** Validates code signature (if signing enabled)
4. **Notify:** Prompts user: "Update available: Install now or later?"
5. **Apply:** Updates installed on next app launch or immediately if user chooses
6. **Preserve:** User data remains intact after update

### User Experience

See `docs/USER_UPDATES.md` for complete user guide.

**User can:**
- Disable auto-checks in Settings
- Manually trigger update check
- Choose to install now or on next launch
- Rollback to previous version if needed

### Configuration (scaffolded in `tauri.conf.json`)

```json
{
  "updater": {
    "active": false,
    "dialog": true,
    "pubkey": "dW1....",
    "endpoints": [
      "https://releases.githubusercontent.com/repos/zentala/zntl-tray/releases/latest"
    ]
  }
}
```

**Status:** Placeholder (disabled until signing is implemented)

### Setup Timeline

- **Now:** Placeholder in tauri.conf.json
- **Upon code signing:** Activate signing + configure updater
- **T007 (CI/CD):** Auto-generate update manifests and publish to GitHub Releases

## Pre-Release Checklist

Before tagging a release version (e.g., `v0.1.0`):

```bash
# 1. Run all tests
pnpm test:all

# 2. Run performance profiling
pnpm test:perf

# 3. Check memory baseline
cat .perf-baseline.json

# 4. Check build sizes
cat .build-sizes.json

# 5. Review CHANGELOG
# (if applicable)

# 6. Tag release
git tag v0.1.0

# 7. Push to GitHub
git push origin v0.1.0
```

**Automated checks:**
- All tests must pass (unit, integration, Rust)
- Memory peak < 300 MB
- No growth > 20 MB vs baseline
- Build size < 100 MB

**Manual review:**
- Verify release notes are accurate
- Check CHANGELOG for completeness
- Confirm code signing secrets are configured
- Test installer on clean Windows VM

**CI/CD (T007):**
- Release workflow auto-generates GitHub Release
- Signs installer (if certificate available)
- Generates update manifest
- Publishes to GitHub Releases

## Vision Doc
See `.agent/vision/2026-03-15-desk-app-vision.md` for full spec.

## PROJECT.xml Maintenance

**Rule: Update PROJECT.xml before every commit** if any of the following changed:
- New or removed source files
- New or removed features / IPC events
- New or removed test files or layers
- Dependency changes (Cargo.toml / package.json)
