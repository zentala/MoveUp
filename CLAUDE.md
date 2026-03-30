# Desk App — CLAUDE.md

## Naming
- **Product name**: SmartDesk (exe: `SmartDesk.exe`, camelCase, no spaces)
- **Display name**: "Smart Desk" (with space — used in notifications, window titles, share text)
- **Legacy name**: zntlDesk — same app, old name. Treat as synonym. Docs still use it in places.
- **Internal IDs**: `io.zntl.desk` (bundle identifier), `zntl-desk` (npm package) — do NOT change

## Purpose
Ergonomics tracker for a sit/stand desk. Detects sitting/standing via laser sensor, tracks session durations, and nudges user to take breaks via visual cues and notifications.

**Core philosophy:** This is an **experimentation platform for self-motivation**. The developer (zentala) uses this app to test different approaches to motivating himself to take breaks — different visual cues, notification styles, gamification ideas. Architecture must stay **modular and configurable** so it's easy to swap components, try new approaches, and A/B test motivation strategies without rewriting core logic.

## Target User
Single user (zentala) working at a motorized sit/stand desk.

## Logging & Debugging
- **Debug tab**: Settings → Debug — live session state dump (all raw values)
- **Minute snapshots**: JSON files at `{app_data_dir}/logs/YYYY-MM-DD/HH-MM.json` (T044, planned)
- **Event log**: `{app_data_dir}/logs/YYYY-MM-DD/events.log` — state transitions, notifications, alerts (T044, planned)
- **Full reference**: [`.claude/rules/logging.md`](.claude/rules/logging.md)

## Architecture Principles
- **Modular components** — visual cues (overlay bar, tray icon, notifications, popups) are independent modules. Each can be enabled/disabled/swapped.
- **Configurable behavior** — session limits, notification thresholds, dismiss/snooze logic, visual styles — all configurable, not hardcoded.
- **Experimentation-first** — build for easy prototyping. Mock data sources, dev mode, pnpm scripts per mode. Developer must see results fast.
- **Independent axes** — data source, render mode, visual style, notification type are orthogonal. Any combination works.

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
1. **System tray** — `↕ 72 cm` tooltip + color dot (green/yellow/red by progress)
2. **Floating window** — current height, today's totals, session history
3. **Top-of-screen progress bar** — green→red over 40min session (overlay_renderer.rs)
4. **Alert popup** — progressive escalation when limit reached (alert_manager.rs, planned)

## Communication Architecture & Profiles

**CommunicationPolicy** (`communication_policy.rs`) is the single source of truth for all UI signals. It evaluates `(state, elapsed_secs) → Signals` and tells tray, overlay, popup, and notifications what to show. TrayController only executes signals — no decision logic.

**Two profile types** (JSON, hot-reloadable, in `{app_data_dir}/profiles/`):
- **Ergonomic Profile** (`ergonomic/*.json`) — limits, scoring, KPI thresholds, break credit
- **Communication Profile** (`communication/*.json`) — escalation timing, channels, blink patterns, messages

**Built-in profiles**: default, aggressive, gentle, silent, demo (communication) + standard, strict, relaxed, demo (ergonomic).

**Break credit**: proportional — each second of break cancels `break_credit_multiplier` (default 2.0) seconds of sitting. Configurable in ergonomic profile. See [ADR 008](.arch/ADR/008-proportional-break-credit.md).

**Color dictionary**: Yellow (#ffc107) = warning, Red (#f44336) = action needed, Gray (#808080) = sensor issue, None = all OK. Green removed from system.

**Spec**: `docs/superpowers/specs/2026-03-30-communication-architecture-design.md`

## Overlay Progress Bar

Native WinAPI overlay at top of screen. Full docs: [`.claude/rules/overlay.md`](.claude/rules/overlay.md)
- Dev: `pnpm tauri:dev` (live sensor, default) / `pnpm tauri:dev:demo` (animation) / `pnpm tauri:dev:mock` (sim)
- Key files: `overlay_renderer.rs`, `tray_controller.rs`, `colors.rs`, `serial.rs`, `session.rs`

## Stack
- **Frontend**: React + TypeScript (Vite, port 1443)
- **Backend**: Rust (Tauri 2)
- **DB**: SQLite via `tauri-plugin-sql`
- **Serial**: `serialport` crate, background thread
- **Activity tracking**: Rust (keyboard/mouse hooks via `rdev` or Windows API)

## Remote Display (Phone Dashboard)
- Embedded HTTP+WS server on `:3390` (configurable via `DESK_REMOTE_PORT`)
- Same React UI served to browsers; `useDeskAuto()` hook selects WS or Tauri IPC
- Auto-reconnects with exponential backoff; REST polling fallback when WS is down
- `ConnectionOverlay` component shows connection/sensor status in remote mode
- See `docs/REMOTE_DISPLAY.md` for phone setup instructions
- Key files: `remote_server.rs`, `ws_broadcaster.rs`, `useRemoteDesk.ts`, `ConnectionOverlay.tsx`

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

Full details: [`.claude/rules/installer.md`](.claude/rules/installer.md)
- Build: `pnpm tauri:build` (runs tests → builds frontend → compiles Rust → NSIS installer)
- Size targets: installer 60-70 MB, peak memory <250 MB
- Code signing & auto-update: scaffolded, not yet active

## Vision & Strategy Docs

| Document | What it covers |
|----------|---------------|
| [Product Vision](`.plan/vision/2026-03-15-desk-app-vision.md`) | Hardware, states, UI, session logic, remote display phases |
| [Business Vision & GTM](.plan/vision/2026-03-24-business-vision.md) | Dev kit → SaaS → consumer product; pricing; open core; certification CE; funding (Kickstarter + EU grants); competitive moat |
| [Validation & Marketing](.plan/vision/2026-03-25-validation-and-gtm.md) | Landing page design; pre-order (49 EUR) + waitlist; marketing posts (Reddit, HN); validation thresholds; fulfillment plan |
| [Premium Tier Definition](.plan/vision/2026-03-25-premium-tier-definition.md) | Free vs Pro features; Pro = cloud sync, history, AI coaching, smartwatch; implementation epics E010-E014 with tasks |
| [Distribution & Tiers](.plan/vision/2026-03-25-distribution-and-tiers.md) | 3 tiers (DIY free / Dev Kit 49 EUR / Founder's 99 EUR); EU-first shipping; future regional hubs (deferred); pre-order threshold model |
| [Marketing Launch Plan](.plan/vision/2026-03-25-marketing-launch-plan.md) | Two-site strategy; 5 Reddit/HN posts (full text); Google/FB ads (500 PLN/mo); KPI dashboard; content calendar; SEO; social proof; conversion optimization |
| [Story-Driven Launch](.plan/vision/2026-03-26-story-driven-launch.md) | Founder story landing page; 3 product tiers (Basic €49/Pro €79/Founder €149); mission/vision; viral video strategy; influencer outreach; privacy-first analytics; app telemetry opt-in; sedentary research report |

### Pricing (canonical source of truth)

**File: [`.plan/vision/config/pricing.json`](.plan/vision/config/pricing.json)**
All landing pages, docs, and tasks MUST read pricing from this file.
Do NOT hardcode prices in markdown — they may change or be A/B tested.

### Key Business Context (for all agents)

- **Positioning:** "Developer Platform + Reference Hardware" — software is the business, hardware is the entry point
- **Three tiers:** Basic Kit (49 EUR, 200 threshold), Pro Kit (79 EUR, 500 threshold), Founder's Edition (149 EUR, 5yr Pro + smartwatch)
- **Phase 1 (now):** DIY Kickstarter on own site, story-driven landing page, EU-only shipping from Poland
- **Analytics:** Privacy-first (Plausible/CF Analytics, NO Google Analytics). App telemetry opt-in only.
- **Software model:** Open core — app is open source (MIT/Apache), cloud/AI/smartwatch are closed/paid
- **Certification:** USB-only (no RED), ToF sensor Class 1 (no laser cert), CE via self-declaration, EMC ~5-15k PLN
- **Hardware design:** Off-the-shelf modules (VL53L1X + MCU), plexi/PCB carrier mount, no custom PCB
- **Revenue:** Hardware margin (thin) + SaaS subscriptions (4.99 EUR/month) — SaaS is the real revenue after validation

### Architecture Decision Records

| ADR | Decision |
|-----|----------|
| [001](.arch/ADR/001-remote-display-web-kiosk.md) | Remote display via embedded HTTP+WS server |
| [002](.arch/ADR/002-tof-sensor-over-laser.md) | VL53L1X ToF module — Class 1 eye-safe, no laser re-certification |
| [003](.arch/ADR/003-usb-only-no-radio-phase1.md) | USB-only in Phase 1 — avoids RED directive, saves 15-50k PLN |
| [004](.arch/ADR/004-dev-kit-before-consumer-product.md) | Dev Kit before consumer product — validate demand first |
| [005](.arch/ADR/005-open-core-software-model.md) | Open core — app open source, cloud/AI closed |
| [006](.arch/ADR/006-self-declaration-ce-not-notified-body.md) | CE self-declaration (not notified body) |
| [007](.arch/ADR/007-plexi-mount-dev-kit-enclosure.md) | Plexi/PCB carrier mount for dev kit |

### Hardware Design

See [`.arch/hardware/HARDWARE-OPTIONS.md`](.arch/hardware/HARDWARE-OPTIONS.md) for:
- MCU comparison (XIAO RP2040 vs ESP32-C3 vs alternatives)
- Carrier PCB design (replaces plexi mount, ~1 PLN/unit at 100 pcs)
- Optional components (vibration motor, piezo presence sensor)
- Updated BOM (~42-68 PLN per unit at production quantities)
- Production timeline (~6-8 weeks for 100 units)

## PROJECT.xml Maintenance

**Rule: Update PROJECT.xml before every commit** if any of the following changed:
- New or removed source files
- New or removed features / IPC events
- New or removed test files or layers
- Dependency changes (Cargo.toml / package.json)
