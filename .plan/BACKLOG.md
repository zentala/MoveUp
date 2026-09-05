# Backlog — Desk App

## Planned Epics

- [ ] **[E014 — Supervised release rollback](epics/E014-2026-09-05-supervised-release-rollback/PLAN.md)** — keep several installed builds with a `last-known-good` marker so a bad release can be rolled back, and let PM3 fill the gap when nothing holds the app. Waves 1-2 are unblocked; waves 3-4 wait on two PM3 backlog items (`int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md`, section "2026-09-05 — Nadzór z powrotem do poprzedniego builda").
- [ ] **[E013 — Signed Tauri release and PM3 deployment](epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md)** — build signed Windows release, deploy the installed SmartDesk executable under PM3, and expose the remote display at `moveup.internal`. Prepare implementation tasks and execute in a new session.
- [ ] **[E012 — Analyst Dashboard](epics/E012-2026-05-16-analyst-dashboard/PLAN.md)** — separate Tauri window with Data Catalog (8 sources, schema + samples) and Explorer (5 charts over last 7 days). Mockup-first per ux-design-flow rule. Research: [reports/2026-05-16-data-sources.md](epics/E012-2026-05-16-analyst-dashboard/reports/2026-05-16-data-sources.md). Version bump to 0.5.0 in epic setup.

---

## Bugs — Fix Now

- [x] **CRITICAL: Seeding ignores daily reset** — fixed in commit `c1d286f`: `load_today_totals()` now filters by `daily_reset_after` timestamp persisted to tauri-plugin-store. Sessions started before the last daily reset are excluded from DB seeding.
- [x] **hourly_break_tracker not persisted** — fixed: `HourlyBreakTracker` fields (`hours_with_break`, `hours_active`, `current_away_secs`) now persisted in `PersistedSessionState` via tauri-plugin-store. Survives restarts within same day. Backward compatible via `#[serde(default)]`.
- [x] **Duplicate notifications on sit limit** — fixed: removed `notify: "popup"` from second escalation step in all profiles. Now only one toast fires at limit, visual-only escalation (blink+pulse) at +5 min.
- [x] **Notification spam when ignored** — fixed: escalating cooldown (0→5m→15m→30m→silence), configurable per profile via `snooze.notify_cooldowns_secs`. Max 4 reminders, then silence until position change.
- [x] **3 conflicting autostart registry entries** — removed `zntlDesk`, `Smart Desk`, `SmartDesk` from HKCU Run.

---

## Autostart — findings 2026-09-05

- [x] **Autostart dead since the registry cleanup** — the earlier entry "3 conflicting
  autostart registry entries — removed `zntlDesk`, `Smart Desk`, `SmartDesk` from HKCU Run"
  deleted all three and re-registered none. Nothing could restore them: `ensure_autostart`
  is a deliberate no-op in debug builds ([src-tauri/src/setup_helpers.rs:92](../src-tauri/src/setup_helpers.rs)),
  and no release build existed on this machine. Fixed 2026-09-05 by building and installing
  release 0.5.0; `HKCU\...\Run\MoveUp` now points at
  `%LOCALAPPDATA%\MoveUp\desk.exe --minimized`. (Importance High, 3 points)
- [x] **Self-heal read the wrong registry key** — `read_autostart_registry_path` hardcoded
  `SmartDesk` while the plugin registers under `productName` (`MoveUp`), so the E011-T01
  stale-path check compared nothing. The value also carries `--minimized`, which never
  equals a bare exe path. Fixed in commit `7305214`,
  [src-tauri/src/setup_helpers.rs:115](../src-tauri/src/setup_helpers.rs). (Importance High, 2 points)
- [x] **Release build was blocked twice** — npm/Cargo Tauri version mismatch (commit `86b2f4d`)
  and a parallel-test env race in [src-tauri/src/google_fit.rs:375](../src-tauri/src/google_fit.rs)
  (commit `9559dea`). (Importance Medium, 3 points)
- [x] **Decide the supervision model for the installed app** — decided 2026-09-05.
  The Windows `Run` key stays as the login path, because it starts the app even when
  the PM3 daemon is down and needs no infrastructure. PM3 does not replace it; PM3
  owns only gap-filling and rollback. Supervision splits in two: PM3 gets the generic
  loop (60-second poll, act only after two consecutive misses, stand down when another
  process holds the `io.zntl.desk` singleton, demote to an older candidate), MoveUp
  gets the app-specific half (a store of several builds, the `last-known-good` marker,
  the definition of a healthy build, retention). Written up as
  [E014](epics/E014-2026-09-05-supervised-release-rollback/PLAN.md). **MoveUp is waiting
  on two PM3 items** filed in `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` under
  "2026-09-05 — Nadzór z powrotem do poprzedniego builda": candidate lists with demotion
  (8 points) and singleton stand-down (5 points). E014 tasks T05, T06 and T07 cannot
  start until those land. (Importance Medium, 5 points)

## UX Issues — High Priority

- [x] **Timeline readability** — fixed: implemented timeline skin system with 3 switchable themes (Semantic, Amber, Clinical). Each skin defines distinct `--tl-*` CSS vars. Dropdown in Settings → More → Timeline Theme. Default: Semantic (burgundy/green/blue/gray).

---

## Open Tasks from Previous Epics

### From E002 — Overlay Progress Bar
- [ ] E002-T09 — [Choose production render mode](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T09-choose-production-mode.md) (OPAQUE vs LAYERED) — decision made (OPAQUE=default), needs ADR
- [ ] E002-T11 — [Verify debug overlay info](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T11-verify-debug-overlay.md) — display works, needs manual QA sign-off

### From E003 — Installer & Distribution
- [ ] E003-T07 — [GitHub Releases CI/CD](epics/E003-2026-03-16-installer-distribution/tasks/E003-T07-github-releases-automation.md) (requires code signing certificate)

### From E004 — Session Alerts & Snooze
- [ ] E004-T04 — [Integration test: full alert flow](epics/E004-2026-03-20-session-alerts/tasks/E004-T04-integration-test-alert-flow.md) — basic tests exist, time simulation missing
- [ ] T017 — Stages 3-5 implementation (no task file)
- [ ] T018 — Notification A/B testing (no task file) — profiles now enable this; see Profile System section
- [ ] T019 — Success notifications + gamification (no task file)

### From E006 — Session Bugs & Polish
- [ ] E006-T14 — Connection status UI cleanup — functional in debug section only
- [ ] E006-T15 — Tooltip + UI integration tests — tooltips done, e2e coverage basic

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

- **Redesign alert popups — Tauri WebviewWindow instead of raw WinAPI** — current popups (`alert_popup_window.rs`) use raw WinAPI GDI, which looks like a 2003 Win32 dialog. Migrate to a Tauri WebviewWindow so we can style alerts with HTML/CSS using the instrument panel design system (--panel-*, --beam-*, --ink-*). This enables: dark themed popups matching the main window, animated transitions, rich content (progress bars, coach messages in popup), and the acrylic blur effect.

---

## Sensor & Readings

- **Extended height stabilization** — when desk is stationary for extended period (no significant movement), increase smoothing window 2x to eliminate residual jitter (e.g. 92→91→92→91 flickering). Current HeightStabilizer uses 10-sample window; for stationary desk, double to 20 samples or use exponential moving average with lower alpha.
- **Sensor diagnostics panel** — show raw vs smoothed readings, debounce state, threshold visualization

---

## Logging & Observability

- **SQLite time-series storage** — replace file-per-minute snapshots with queryable SQLite table. Enables: search across days, trend analysis (sitting % over weeks), anomaly detection (daily score dropping), dashboard visualization. Natural phase 2 after file-based logging proves useful. Effort: M, Priority: P3.
- Note: file-per-minute snapshots (`snapshot_logger.rs`) and event log (`event_logger.rs`) are already implemented. `height_readings` table exists in DB schema but is never populated — could be used for time-series.

---

## Max Continuous Computer Time Alert

Alert: max continuous work at computer. Standing ≠ break from screen.

- Separate timer "time at computer" (independent from sitting session timer)
- Thresholds: <45m green, 45-75m yellow, >75m red
- Alert on limit exceeded (default 75m, configurable)
- Reset: ≥5 min away from computer
- Note: `longest_computer_session_secs` and `continuous_computer_secs` fields already exist and work. This is about adding an alert when limit exceeded — the KPI already shows the value.

---

## Activity Tracking

- [x] **Activity status in UI** — done: StateIndicator shows "Active" / "Idle Xm Ys". Toggle: Settings → More → "Show activity status".

- [ ] **Split position_changes into two metrics: desk changes + posture changes** — currently `position_changes` counts both desk height changes (Sitting↔Standing) and Away resets (user left computer for 5+ min). These are different things:
  - **Desk changes** (`desk_position_changes`) — physical desk movement (sit↔stand)
  - **Posture changes** (`posture_changes`) — any change including leaving computer (desk changes + away returns)
  - Both matter for health: desk changes = ergonomic variety, posture changes = not being sedentary
  - KPI "Changes/h" should show posture_changes (broader metric)
  - New KPI badge or detail view could show desk_position_changes separately
  - Requires: split `position_changes` field into two, update KPI metric engine, update frontend
  - Priority: P3 — works fine as-is, split when adding detailed analytics

- **Cross-platform activity detection** — current implementation is Windows-only (`GetLastInputInfo`). Before release, need Linux/macOS support. Options: `rdev` crate (cross-platform input hooks), X11/Wayland idle APIs, macOS `CGEventSource`. Non-Windows currently returns `idle=0` (always active) — Away state will never trigger on Linux/macOS.

---

## Visualization & Charts

- **[E012-T09 — Analyst layout flip](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md)** — top-centred date header, bottom day-navigator. Sticky bottom nav.
- **[E012-T10 — Recharts donut KPIs](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md)** — replace KpiTrend + BreakCreditHistogram bars with donut/gauge cards. Numeric centres. New ADR 014.
- **[E012-T11 — Pulse on selectedDay change](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T11-selected-day-pulse.md)** — CSS-only highlight animation on KPI cards when day changes.
- **[Spike — time-viz library evaluation](reports/2026-05-21-time-viz-libraries-spike.md)** — research charting + timeline + time-data libs (Recharts / ECharts / Visx / vis-timeline / crossfilter / Temporal / etc.). Output: recommendation matrix + migration cost for replacing custom SVG timeline.
- **[Spike — timeline visual upgrades](reports/2026-05-21-timeline-visual-upgrades-spike.md)** — HTML demo gallery with 5-8 visual treatments of our SVG strip (gradients, cardiogram, stream graph, mini-multiples, typographic header, hover cursor, heatmap calendar). User picks 1-2 to productise.

---

## Timeline Full Window — Expandable History View

- Click on timeline in popup → opens a dedicated window with full-day/multi-day timeline
- Scroll through days, zoom in/out, click sessions for details
- Shows: all states with durations, breaks, position changes, daily score
- Implementation: Tauri WebviewWindow (like welcome popup), separate route
- Priority: P3 — needs SQLite history infrastructure first

---

## KPI Time Range Selector

- Switch to view stats for: Today / 7 days / 30 days
- Requires: SQLite time-series storage
- Currently all KPIs are today-only (in-memory, reset at midnight)
- UI: small toggle/tabs above KPI strip ("Today | 7d | 30d")
- Priority: P3 — needs DB history infrastructure first

---

## PostureBalance Ratio Refinement

- [ ] **PostureBalance ratio uses mixed counters** — condition is `sitting_seconds_total >= 6h AND sitting_seconds > standing_seconds * 2`. But `sitting_seconds` is reduced by break credit while `sitting_seconds_total` is not. After a day of 8h sitting with breaks, `sitting_seconds_total = 28800` but `sitting_seconds` may be 0 — ratio never triggers despite genuinely sedentary day. Consider using `sitting_seconds_total` for both checks, or a separate daily-ratio metric. Track in real usage first. See [ADR 009](.arch/ADR/009-day-break-credit.md).

---

## Gamification & Scoring

- [x] **Notification flag persistence** — done: flags + credit-reduced `sitting_seconds` + `daily_score` persisted to tauri-plugin-store, survive restarts. Date-guarded (discards stale data after midnight).
- **Gamification techniques research** — deep research report listing 100+ gamification techniques (loss aversion, streaks, milestones, social proof, progression systems, comeback mechanics, etc.). Output: `.plan/reports/gamification-techniques.md` with categorized list, brief descriptions, and applicability to desk app. Reference material for future epic planning.
- **Scoring & metrics redesign** — umbrella task for iterating on daily_score formula, KPI thresholds, comeback mechanics (show user HOW to recover negative score), penalty/reward balance. Collect ideas here, plan as epic when ready.
- **Notification flag persistence** — `alert_fired`, `standing_target_reached_fired`, `notify_inactivity_fired` etc. reset on app restart causing notification spam. Persist in DB or derive from today's event log on startup. Priority: P2 (affects UX on every dev restart).

## Profile System

- [ ] **Profile editor UI** — visual form for editing profiles instead of raw JSON. Dropdown is done, but editing requires opening JSON in external editor. Build an in-app form with live preview.
- [ ] **Per-application profiles** — auto-switch profile when gaming (detect fullscreen app), on calls (detect audio), or in presentations. Needs activity/application detection.
- [ ] **Scheduled profiles** — time-based auto-switching (e.g., "aggressive" during work hours 9-17, "gentle" evenings). Cron-like schedule in profile config.
- [ ] **Profile analytics** — track which profile produces better ergonomic outcomes over time. Compare daily scores across profiles.

---

## Communication

- [ ] **Profile reload notification** — show a subtle toast/log when profile is hot-reloaded so user knows the edit was picked up. Currently silent.
- [ ] **Overlay neutral bar style** — the "neutral" overlay bar (sitting within limit) needs visual design. Currently uses dark panel color — may need a subtle gradient or pattern to be distinguishable from "no bar."
- [ ] **Per-profile cooldown tuning** — `snooze.notify_cooldowns_secs` is now configurable per profile, but built-in profiles all use the same default `[0, 300, 900, 1800]`. Tune: aggressive = shorter (e.g., `[0, 120, 300, 600]`), gentle = longer (e.g., `[0, 600, 1800, 3600]`), silent = empty `[]` (no notifications).

---

## Content / Articles

- [ ] **Article: Screen time & eye health** — blog post for landing page based on [screen time research report](reports/screen-time-eye-health-research.md). Topics: CVS statistics, 20-20-20 rule, break frequency research, how SmartDesk helps. Target: SEO for "screen break reminder" / "computer eye strain". Link to product page. Priority: P2 — write after computer time alert feature ships.

---

## Future Features

- **Notification A/B testing** — now enabled via communication profiles: switch between profiles to A/B test notification strategies (T018)
- **Success notifications + gamification** — streak tracking, milestone celebrations (T019)
- **Phone-as-hub** — old phone + BLE sensor, works without desktop app
- **Smartwatch integration** — proximity detection, walking state, HRV
- **Sleep API integration for Analyst timeline** — current "Away" overnight is honest but uninformed; pull from Google Fit / Health Connect / Apple Health / smartwatch to overlay actual sleep window on the Daily Timeline Strip. Replaces the cut "sleep band" heuristic (E012-T07 brainstorm 2026-05-18).
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
- ~~E001-T09 position_changes DB~~ — implemented (db.rs schema + migrations)
- ~~E001-T11 Rail pulse animation~~ — implemented (HeightRail.tsx + CSS keyframes)
- ~~E001-T12 Yesterday delta arrow~~ — implemented (TodayStats.tsx ↑/↓ delta)
- ~~E004-T05 Dismiss without sensor~~ — implemented (alert_manager.rs snooze)
- ~~KPI Strip~~ — implemented in E007 (4 badges: standing%, changes/h, breaks, screen)
- ~~KPI "Session" rename~~ — renamed to "Screen" in E008-T05
- ~~State Machine Redesign (Away)~~ — implemented in E008-T02 (Away triggers on 60s idle)
- ~~NotificationService~~ — implemented in E008-T09 (centralized routing)
- ~~Coach Messages removal~~ — removed in E007-T11
- ~~Height stabilization (basic)~~ — implemented in E006-T10 (HeightStabilizer module)
- ~~Persistent KPI Strip~~ — implemented in E007-T08 (KpiStrip component)
- ~~HourlyBreakTracker wiring~~ — wired into session loop, metric uses real data
- ~~Standing % counts Away time~~ — fixed: standing_bout_started tracks actual standing only
- ~~AlertManager message strings in settings panel~~ — superseded by CommunicationPolicy profiles (messages live in communication profile JSON, editable per profile)
- ~~Notification strategy as pluggable system~~ — superseded by CommunicationPolicy + profiles (different backends, escalation patterns, thresholds are all configurable per profile)
- ~~Color inconsistency (green/gold)~~ — fixed: unified color dictionary, green/gold removed from color system
