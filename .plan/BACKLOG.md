# Backlog — Desk App

## Open Tasks from Previous Epics

### From E001 — App Foundation
- [ ] E001-T09 — [position_changes DB schema](epics/E001-2026-03-15-app-foundation/tasks/E001-T09-changes-db.md)
- [ ] E001-T11 — [Rail pulse animation](epics/E001-2026-03-15-app-foundation/tasks/E001-T11-rail-pulse.md)
- [ ] E001-T12 — [Yesterday delta arrow](epics/E001-2026-03-15-app-foundation/tasks/E001-T12-yesterday-delta.md)

### From E002 — Overlay Progress Bar
- [ ] E002-T09 — [Choose production render mode](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T09-choose-production-mode.md) (OPAQUE vs LAYERED)
- [ ] E002-T11 — [Verify debug overlay info](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T11-verify-debug-overlay.md)

### From E003 — Installer & Distribution
- [ ] E003-T07 — [GitHub Releases CI/CD](epics/E003-2026-03-16-installer-distribution/tasks/E003-T07-github-releases-automation.md) (requires code signing certificate)

### From E004 — Session Alerts & Snooze
- [ ] E004-T04 — [Integration test: full alert flow](epics/E004-2026-03-20-session-alerts/tasks/E004-T04-integration-test-alert-flow.md)
- [ ] E004-T05 — [Dismiss without sensor](epics/E004-2026-03-20-session-alerts/tasks/E004-T05-dismiss-without-sensor.md)
- [ ] T017 — Stages 3-5 implementation (no task file)
- [ ] T018 — Notification A/B testing (no task file)
- [ ] T019 — Success notifications + gamification (no task file)

### From E006 — Session Bugs & Polish
- [ ] E006-T14 — Connection status UI cleanup
- [ ] E006-T15 — Tooltip + UI integration tests

### From E007 — KPI Dashboard + Timer UX
- [ ] T047 — Visual verification (demo smoke test)
- [ ] T048 — Fix issues found during verification

---

## Tauri Plugins

- **`window-vibrancy`** (crate) — Windows Acrylic/Mica blur effect on floating window
  - requires `"transparent": true` in `tauri.conf.json` + `background: transparent` in CSS
- **`tauri-plugin-autostart`** — start app at system login

---

## App Icon — True Vector SVG

- `src-tauri/icons/icon-source.svg` is NOT a real vector — it's a PNG rasterized image wrapped in an SVG container (base64-encoded `data:image/png` inside `<image href=...>`). Colors cannot be changed via CSS/attributes.
- **To do:** Replace with a proper SVG drawn with `<path>` / `<rect>` elements so it can be recolored and scaled infinitely.
- Search terms: Noun Project `standing desk`, SVG Repo `adjustable desk`, Flaticon `sit stand`
- Or: design from scratch in Inkscape/Figma — a simple desk silhouette (horizontal top + two legs at different heights) works at 16px.
- Once real SVG exists: use `pnpm tauri icon source.png` to auto-generate all required sizes.

---

## Alert System — Future

- **AlertManager message strings in settings panel** — `AlertConfig` holds default neutral/positive message arrays. Expose as editable arrays so zentala can tune tone/wording without code changes. Depends on T015.
- **Notification strategy as pluggable system** — like widgets but for notifications. Different backends (toast, custom popup, both), different escalation patterns. Deferred until widget system proves the pattern.
- **Redesign alert popups — Tauri WebviewWindow instead of raw WinAPI** — current popups (`alert_popup_window.rs`) use raw WinAPI GDI, which looks like a 2003 Win32 dialog. Migrate to a Tauri WebviewWindow so we can style alerts with HTML/CSS using the instrument panel design system (--panel-*, --beam-*, --ink-*). This enables: dark themed popups matching the main window, animated transitions, rich content (progress bars, coach messages in popup), and the acrylic blur effect. Implementation: create a `popup.html` route, spawn via `WebviewWindowBuilder` (like welcome popup), communicate via Tauri events.

---

## Dokumentacja

- **Overlay Progress Bar Architecture Document** — full description of the bar system at top of screen

---

## Sensor & Readings

- **Height stabilization algorithms** — moving average, 1cm rounding, trend locking (scheduled: T037)
- **Sensor diagnostics panel** — show raw vs smoothed readings, debounce state, threshold visualization
  - Helps debug standing detection issues like T036
- **Extended height stabilization** — when desk is stationary for extended period (no significant movement), increase smoothing window 2x to eliminate residual jitter (e.g. 92→91→92→91 flickering). Current HeightStabilizer uses 10-sample window; for stationary desk, double to 20 samples or use exponential moving average with lower alpha. Detect "stationary" = all readings within ±3mm for last N seconds.

---

## Persistent KPI Strip

Always-visible KPI strip on every view — fixed position, same place. Shows 4 daily ergonomics metrics with color coding.

### KPIs (all RELATIVE, not raw counts):

| KPI | Format | Green | Yellow | Red |
|-----|--------|-------|--------|-----|
| **Standing %** | `12%` | ≥15% | 10-15% | <10% |
| **Position changes/h** | `1.2/h` | ≥1.0 | 0.5-1.0 | <0.5 |
| **Hourly breaks** | `5/7h` | all hours covered | 1-2 missed | ≥3 missed |
| **Longest session** | `47m` | <45m | 45-75m | >75m |

- Position change count (raw number) = secondary/small, shown alongside changes/h
- No raw counts without context — everything relative to time worked

### Design rules:

- Visible on EVERY view, fixed position (e.g. bottom bar, header strip)
- Compact — 4 numbers + colors, zero descriptive text
- Color communicates state (green/yellow/red) — no need to read

### Position change definition:

- 5+ minutes standing OR 5+ minutes away = 1 position change
- Standing and away are interchangeable as "break"
- Goal: ~1 change/hour

### Hourly break definition:

- Binary check per hour: "Was there ≥5 min break from screen?"
- KPI = how many hours had a break / how many hours we worked
- One 30-min break in one hour = 1/1 for that hour, does NOT make up for others
- Away = rest for eyes (even standing, eyes are working)

---

## Coach Messages → Progress Bar

Replace text coach messages ("Half limit used", "On track", etc.) in OneBarCoach with visual elements. Existing 4px progress bar in OneBarTimer works well — coach message below it is unnecessary/unreadable.

- Progress bar already exists and is dynamic (OneBarTimer.tsx) — reuse/extend
- Remove or simplify coach text messages — bar itself communicates state
- Alternatively: coach message as tooltip, not permanent text

---

## KPI "Session" Label — Confusing, Rename Required

- KPI badge labeled "Session" actually means "longest continuous screen time without 5+ min away"
- User interprets "Session" as sitting session or app uptime — both wrong
- **Rename to**: "Screen" or "Screen time" or "At desk" — something that communicates "continuous time at computer"
- **Possible bug**: Away detection doesn't work (Away state unreachable) → screen time counter NEVER resets → always red after 75 min. Fix Away state first (see "State Machine Redesign" below), then verify this KPI resets properly.
- **Not covered by tests**: no test verifying that 5+ min away resets the longest_session counter in practice (only unit tests on counter logic, but Away is never triggered by sensor)

---

## Max Continuous Computer Time Alert

Alert: max continuous work at computer. Standing ≠ break from screen.

- Separate timer "time at computer" (independent from sitting session timer)
- Thresholds: <45m green, 45-75m yellow, >75m red
- Alert on limit exceeded (default 75m, configurable)
- Reset: ≥5 min away from computer
- Shown in KPI strip as "Screen time" (not "Session" — confusing)

---

## Activity Tracking

- **Activity status in UI** — show keyboard/mouse activity status (active/idle) in floating window
  - `activity.rs` already detects idle ≥60s via `GetLastInputInfo`, but UI doesn't expose this
  - Show: "Active" / "Idle 2m" in widget footer or status bar
  - Useful for debugging Walking/Away state transitions

- **Cross-platform activity detection** — current implementation is Windows-only (`GetLastInputInfo`). Before release, need Linux/macOS support. Options: `rdev` crate (cross-platform input hooks), X11/Wayland idle APIs, macOS `CGEventSource`. Non-Windows currently returns `idle=0` (always active) — Away state will never trigger on Linux/macOS.

---

## Notification Centralization (pre-requisite for alert escalation)

> Before implementing full alert escalation flow (T017 Stages 3-5), centralize all notification sources.

- **NotificationService** — single Rust module that routes ALL notifications through one backend
  - Currently two parallel systems: native toasts (`tauri-plugin-notification`) scattered across `serial_periodic.rs` + custom WinAPI popup (`alert_popup_window.rs`)
  - 6 toast conditions spread across `serial_periodic.rs` and `session_breaks.rs` with no central coordination
  - `notification_backend` field exists in `config.rs` (`"toast" | "popup" | "both"`) but is **dead code** — never read
  - **Goal:** one `NotificationService` that:
    1. Collects all notification intents (inactivity, posture balance, praise, alerts, escalation)
    2. Routes through selected backend: native toast OR custom popup OR both
    3. Manages once-per-day / once-per-session gates centrally (not scattered flags)
    4. Makes T018 (A/B testing) trivial — just flip `notification_backend` in config
  - **Blocks:** T017 (Stages 3-5), T018 (A/B testing)
  - **Custom popup redesign** — migrate from raw WinAPI GDI (Win32 2003 look) to Tauri WebviewWindow (HTML/CSS, dark theme, acrylic blur). Already in backlog above under "Alert System — Future".

---

## State Machine Redesign: Away Detection

> Current state machine is fundamentally broken for Away detection.

- **Problem 1: "Walking" is a misnomer** — we don't know if user walks. We know: desk HIGH + no keyboard/mouse. Should be `Away` or `Inactive`.
- **Problem 2: Desk LOW + inactive = still "Sitting"** — if user leaves with desk down, app counts sitting time, fires alerts. Wrong.
- **Problem 3: `Away` enum variant exists but is unreachable** — only used as initial state. `away_bout_secs` counter is dead code. Timeline never shows Away periods.
- **Proposed fix:**
  - `active == false` (≥60s no input) → `Away`, regardless of desk height
  - Remove or rename `Walking` → merge into `Away`
  - `away_bout_secs` logic becomes reachable, `continuous_computer_secs` reset works
  - Away sessions saved to DB → visible in timeline (gray blocks)
  - Away time NOT counted as sitting or standing
- **Impact:** touches `session_reading.rs`, `session_types.rs`, all widgets, timeline, tray tooltip, tests
- **Priority:** HIGH — without this, sitting timer is wrong every time user walks away with desk down

---

## Logging & Observability

- **SQLite time-series storage** — replace file-per-minute snapshots with queryable SQLite table. Enables: search across days, trend analysis (sitting % over weeks), anomaly detection (daily score dropping), dashboard visualization. Natural phase 2 after T044 file-based logging proves useful. Effort: M, Priority: P3.

---

## Timeline Full Window — Expandable History View

- Click on timeline in popup → opens a dedicated window with full-day/multi-day timeline
- Scroll through days, zoom in/out, click sessions for details
- Shows: all states with durations, breaks, position changes, daily score
- Currently popup is ephemeral (opens/closes) — history needs a persistent view
- Implementation: Tauri WebviewWindow (like welcome popup), separate route
- Priority: P3 — needs SQLite history infrastructure first

---

## KPI Time Range Selector

- Switch to view stats for: Today / 7 days / 30 days
- Requires: SQLite time-series storage (see Logging & Observability below)
- Currently all KPIs are today-only (in-memory, reset at midnight)
- For 7d/30d: need historical daily summaries in DB
- UI: small toggle/tabs above KPI strip ("Today | 7d | 30d")
- Priority: P3 — needs DB history infrastructure first

---

## Future Features

- **Notification A/B testing** — two backends simultaneously with feature flag (T018)
- **Success notifications + gamification** — streak tracking, milestone celebrations (T019)
- **Notification strategy plugins** — like widget system but for how/when to nudge
- **Phone-as-hub** — old phone + BLE sensor, works without desktop app
- **Smartwatch integration** — proximity detection, walking state, HRV
- Data export to CSV
- Height threshold calibration via UI
- App icon design: desk silhouette SVG → PNG/ICO assets

---

## Vision / Architecture (future)

- **T-ARCH-001** — Extended body positions: kneeling / leaning / sitting / standing
  - Same sensor, richer height-to-position mapping
  - `DeskPosition` enum extensible, per-position session stats + points
  - Max standing session: 90 min (configurable, option: 60 min) — symmetric to sit limit

---

## Removed (already implemented or superseded)

- ~~`tauri-plugin-notification`~~ — already integrated
- ~~`tauri-plugin-store`~~ — already integrated
- ~~Dynamic tray icon~~ — superseded by T016 (color dot approach)
- ~~Standing Mode Theme~~ — superseded by T032 (widget temperature system)
- ~~Bar flashing at limit~~ — implemented in T013 (alert Stage 1, variant 2 pulsing)
- ~~Red popup at limit~~ — implemented in T013+T014 (AlertPopup)
- ~~Snooze logic~~ — implemented in T015 (deescalating snooze)
- ~~Color dot on tray icon~~ — implemented as T016
