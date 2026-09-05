# Desk App — CLAUDE.md

## Naming
- **Product name**: MoveUp — `productName` in `src-tauri/tauri.conf.json`. This is the
  name Windows uses for the autostart registry value under `HKCU\...\Run`, and the
  installer uses it for `%LOCALAPPDATA%\MoveUp\`. Never hardcode it; read it from
  `app.package_info().name`.
- **Binary name**: `desk.exe` — from the Cargo package name, NOT from `productName`.
- **Legacy names**: SmartDesk, "Smart Desk", zntlDesk — same app, older names. Treat as
  synonyms in prose. They must not appear in code that looks anything up by name.
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
- **USB identity**: `VID_303A&PID_1001`, serial `64:E8:33:84:05:BC`. Enumerates as a
  composite device: `MI_00` → `usbser` → COM3, `MI_02` → WinUSB (JTAG/serial debug).
- **[CRITICAL] Cable sensitivity — most USB-C cables do NOT work with this board.**
  The XIAO ESP32-C3 uses *native* USB (no CH340/CP2102 bridge), so it is far pickier
  than a classic Arduino. Three overlapping causes, all observed 2026-09-06:
  1. Charge-only cables (VBUS+GND, no D+/D-) — board lights up, host sees nothing,
     and Windows enumerates **zero** COM ports.
  2. C-to-C links depend on the board's 5.1k CC resistors; flipping the plug 180°
     or using an A-to-C cable often fixes a link that refuses to come up.
  3. Voltage drop on thin (28 AWG) or long cables — the ESP32-C3 plus VL53L1X
     browns out mid-enumeration, producing a `DEVICE connected` / `DEVICE lost`
     loop within the same second, audible as repeated Windows plug/unplug chimes.
  Diagnosis order when the app reports no sensor: check for a COM port at all
  (`HKLM\HARDWARE\DEVICEMAP\SERIALCOMM`; empty = cable or power, not software),
  then check `events.log` for connect/lost churn. Prefer a short (<=1 m) A-to-C
  cable straight into the motherboard, bypassing USB hubs.

## User States
- `SITTING` — desk low, user at keyboard
- `STANDING` — desk high (counts as break)
- `WALKING` — desk high, user away (also a break)
- `AWAY` — no keyboard/mouse activity

## Session Logic
- Default session limit: **40 minutes** of sitting
- **Session Break Credit** (ADR 008): proportional — each second of break cancels `break_credit_multiplier` (default 2.0) seconds of sitting. Breaks < `break_min_secs` (default 60s) get no credit.
- **Day Break Credit** (ADR 009): breaks ≥ `day_break_min_secs` (default 6h) reset notification flags and daily_score for a fresh motivational start. Does NOT reset daily KPI counters.
- **PostureBalance notification**: only fires when `sitting_seconds_total >= 6h` AND `sitting_seconds > standing_seconds * 2`. Prevents false "sitting most of today" after short sessions.
- Debounce state changes: require 5s stable reading
- **Session State Persistence**: notification flags, credit-reduced `sitting_seconds`, and `daily_score` are persisted to `tauri-plugin-store` (key: `persisted_session_state`). Survives app restarts within the same day. Date-guarded: stale data from a previous day is discarded on load. Cleared on daily reset. Save points: every state transition, alert fire, periodic notification, and shutdown.
- **Computer time tracking** (ADR 011): `continuous_computer_secs` tracks total time at computer (Sitting + Standing). After `computer_break_reset_secs` (default 5 min) of Away, resets to 0. When standing and computer time exceeds `max_continuous_computer_secs` (default 60 min), one gentle toast nudge fires encouraging a screen break. Configurable in ergonomic profile. Toggle: Settings → More → "Screen time tracking".

## UI Components
1. **System tray** — `↕ 72 cm` tooltip + color dot (green/yellow/red by progress)
2. **Floating window** — current height, today's totals, session history
3. **Top-of-screen progress bar** — green→red over 40min session (overlay_renderer.rs)
4. **Alert popup** — progressive escalation when limit reached (alert_manager.rs, planned)
5. **Activity status** — "Active" / "Idle Xm Ys" in StateIndicator, shown when keyboard/mouse idle ≥30s. Toggle: Settings → More → "Show activity status".
6. **Analyst window** — separate 1280×800 Tauri window opened from tray ("Open Analyst"). Two tabs: Catalog (all data sources the app produces) and Explorer. Explorer leads with `DateNavigator` (range + 14 day tabs with grouped mini bars) and a hero `TimelineDetail` (one continuous proportional strip with smooth-scroll between days), followed by KpiTrend / BreakCreditHistogram / DeskHeightTimeline and the full-width DailyScoreTrajectory. Default range 14 days. Route `/#/analyst` live, `/#/mockup/analyst` with fake fixtures. See [ADR 012](.arch/ADR/012-analyst-dashboard-separate-window.md) for window-vs-route, [ADR 013](.arch/ADR/013-date-navigator-merge.md) for the DateNavigator merge.

## Communication Architecture & Profiles

**CommunicationPolicy** (`communication_policy.rs`) is the single source of truth for all UI signals. It evaluates `(state, elapsed_secs) → Signals` and tells tray, overlay, popup, and notifications what to show. TrayController only executes signals — no decision logic.

**Two profile types** (JSON, hot-reloadable, in `{app_data_dir}/profiles/`):
- **Ergonomic Profile** (`ergonomic/*.json`) — limits, scoring, KPI thresholds, break credit
- **Communication Profile** (`communication/*.json`) — escalation timing, channels, blink patterns, messages

**Built-in profiles**: default, aggressive, gentle, silent, demo (communication) + standard, strict, relaxed, demo (ergonomic).

**Break credit**: proportional — each second of break cancels `break_credit_multiplier` (default 2.0) seconds of sitting. Configurable in ergonomic profile. See [ADR 008](.arch/ADR/008-proportional-break-credit.md).

**Notification philosophy**: fewer, well-timed nudges > bombardment. Escalation is visual-first (yellow→red→blink+pulse), with a single toast at the limit. No popup notifications by default. Escalating silence: after each reminder, cooldown grows (0→5m→15m→30m→silence). Configurable per profile via `snooze.notify_cooldowns_secs`. After all reminders exhausted, system stays silent until position changes. See [ADR 010](.arch/ADR/010-notification-escalating-silence.md).

**Color dictionary**: Yellow (#ffc107) = warning, Red (#f44336) = action needed, Gray (#808080) = sensor issue, None = all OK. Green removed from system.

**Timeline skins**: 3 switchable color themes for timeline segments (`--tl-*` CSS vars in `globals.css`):
- **Semantic** (default) — burgundy/green/blue/gray, health-app convention
- **Amber** — copper/amber/gold/shadow, instrument panel aesthetic
- **Clinical** — purple/teal/sage/steel, data-viz feel

Setting: Settings → More → Timeline Theme. Persisted in `AppConfig.timeline_skin`. Hook: `useTimelineSkin()`. CSS: `.tl-skin-{id}` class on widget container.

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

## Conventions

### Test file location
- **Rust:** sibling test file `<module>_tests.rs` (registered as `#[cfg(test)] mod <module>_tests;` in `lib.rs`). Examples: `commands_analyst.rs` ↔ `commands_analyst_tests.rs`, `commands_catalog.rs` ↔ `commands_catalog_tests.rs`. Reason: keeps each `.rs` file under the 250-line cap and makes tests trivially `cargo test --lib -- <module>` runnable.
- **TS:** co-located `<module>.test.ts(x)` next to source. Run via `pnpm test:unit`.

### React hook locations
- Cross-cutting hooks (used by multiple features): `apps/desk/src/hooks/`.
- Feature-scoped hooks: `apps/desk/src/<feature>/hooks/`. Example: analyst-only `useDataCatalog`, `useRangeQuery`, `useSnapshotsRange` live in `src/analyst/hooks/`. Don't promote until a second feature consumes them.

## Key Files

### Application Code
- `firmware/` — Arduino sketch for XIAO ESP32-C3
- `src-tauri/src/serial.rs` — serial reader, auto-detect
- `src-tauri/src/session.rs` — session state machine
- `src-tauri/src/activity.rs` — keyboard/mouse idle detection
- `src-tauri/src/screen_break_nudge.rs` — random nudge message picker for screen breaks
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

## Business Context (REQUIRED reading for planning)

**File: [`.plan/BUSINESS_CONTEXT.md`](.plan/BUSINESS_CONTEXT.md)**

Every agent planning features, epics, or strategy MUST read this file first. It contains links to all vision docs, research reports, ADRs, design specs, and pricing config. This is the accumulated business knowledge that informs all product decisions.

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
| [011](.arch/ADR/011-unified-sit-stand-walk-cycle.md) | Unified sit-stand-walk cycle (no separate screen timer) |

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

## Google Fit Integration

Walking-steps integration uses Google OAuth2 with the
`https://www.googleapis.com/auth/fitness.activity.read` scope.

### Required `.env` keys (in `apps/desk/.env`)

```
GOOGLE_CLIENT_ID=...        # from Google Cloud Console
GOOGLE_CLIENT_SECRET=...    # from Google Cloud Console
GOOGLE_REFRESH_TOKEN=...    # obtained via the auth helper script below
```

When any of these is missing, the StepsWidget renders a "connect google fit"
hint and the backend service no-ops (no errors).

### Optional — pin a specific steps data source

```
GOOGLE_FIT_STEPS_SOURCE=derived:com.google.step_count.delta:com.google.android.gms:merge_step_deltas
```

By default the backend auto-discovers step sources via
`users/me/dataSources?dataTypeName=com.google.step_count.delta` and picks
the highest-priority one (`merge_step_deltas` > `estimated_steps` > other
derived > raw). Set this env var to skip discovery and pin a specific
source — useful when the account has data only in a non-default source
(e.g. Samsung Health sensors, Mi Band raw streams).

### Obtaining `GOOGLE_REFRESH_TOKEN` — run when needed

Whenever the refresh token is revoked, missing, or you switch Google
accounts, regenerate it with:

```
node apps/desk/scripts/google-fit-auth.cjs
```

Prerequisites (one-time):
1. In [Google Cloud Console → Credentials](https://console.cloud.google.com/apis/credentials),
   open your OAuth Client and add **`http://localhost:8765/callback`** to
   *Authorized redirect URIs*.
2. Enable the *Fitness API* in the same project.

The script opens your browser, walks through Google's consent screen,
and prints `GOOGLE_REFRESH_TOKEN=...` to the terminal. Paste that line
into `apps/desk/.env` and restart `pnpm tauri:dev`.

### Source map
- `src-tauri/src/google_fit.rs` — OAuth + Fitness API client (endpoints injectable for tests)
- `src-tauri/src/google_fit_models.rs` — wire types + `StepsView` with `error_kind`
- `src-tauri/src/google_fit_service.rs` — cache, dedup, DST-correct day window
- `src-tauri/src/google_fit_http_tests.rs` — wiremock-backed integration tests
- `src-tauri/src/commands_google_fit.rs` — `get_steps_today`, `refresh_steps_now` (both return `StepsView`)
- `src/components/StepsWidget.tsx` — KPI-style badge in `OneBarWidget`, with reconnect CTA + stale detection + exponential backoff
- `.arch/ADR/012-google-fit-integration.md` — decision record
