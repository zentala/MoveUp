# UX Flow Map — Desk App

Complete reference for what the user sees and experiences in every state of the desk application.

---

## 1. State Machine Overview

> **Known bug (pre-E002):** Away state is unreachable in current code. `session_reading.rs`
> returns Sitting when desk is low, even if user is inactive. Only desk-high + inactive
> triggers Walking. E002-T02 will fix this: `inactive ≥ 60s → Away` regardless of desk height.

### Current State Machine (v0.1.0)

```
                   desk height <= midpoint
                   + 5 consecutive readings
                ┌──────────────────────────────┐
                │                              │
                ▼                              │
 ┌──────────┐     height > midpoint      ┌─────┴────┐
 │ SITTING  │────────────────────────────►│ STANDING │
 │          │  + active=true             │          │
 │          │  + 5 consecutive readings  │          │
 └──────────┘                            └─────┬────┘
      ▲                                        │
      │  height <= midpoint                    │  active=false
      │  + 5 readings                          │  + 5 readings
      │                                        ▼
      │                                  ┌──────────┐
      └──────────────────────────────────│ WALKING  │
         height <= midpoint              │          │
         + 5 readings                    └──────────┘

 Away enum variant exists but is UNREACHABLE (only initial state).
```

### Target State Machine (E002-T02)

```
                   height <= midpoint
                   + 5 readings + active
                ┌──────────────────────────────┐
                │                              │
                ▼                              │
 ┌──────────┐     height > midpoint      ┌─────┴────┐
 │ SITTING  │────────────────────────────►│ STANDING │
 │          │  + active + 5 readings     │          │
 └──────────┘                            └──────────┘
      ▲            ▲                        │    │
      │            │                        │    │
      │   active   │     inactive ≥ 60s     │    │ inactive ≥ 60s
      │   + low    │                        │    │
      │            │                        ▼    ▼
      │            │                     ┌──────────┐
      │            └─────────────────────│  AWAY    │
      │              active + high       │          │
      └──────────────────────────────────└──────────┘
         active + low
```

Key change: **inactive → Away regardless of desk height**. Walking reserved for future (smartwatch).

### Transition Rules

- **Height threshold**: midpoint = `(sitting_height_cm + standing_height_cm) / 2`
  - Default sitting_height_cm = 72.0 cm (`sitting_mm=720 / 10`)
  - Default standing_height_cm = 105.0 cm (`standing_mm=1050 / 10`)
  - Default desk_thickness_cm = 3.0 cm (`desk_thickness_mm=30 / 10`)
  - **Midpoint = (72.0 + 105.0) / 2 = 88.5 cm**
- **desk_height_cm** = `sensor_reading_mm / 10 - desk_thickness_cm`
- **Debounce**: `DEBOUNCE_COUNT = 5` consecutive readings at the same candidate state before transitioning. Sensor fires ~1/s, so ~5 seconds of stable readings.
- **Active detection**: `is_active()` checks keyboard/mouse activity via `GetLastInputInfo` (Windows). Idle threshold = 60s.
- **Position change counter**: incremented on Sitting ↔ Standing transitions. Also incremented after 5 continuous minutes of Away (`away_bout_secs == 300`).

### Continuous Computer Time Tracking (E001)

- `continuous_computer_secs` — time at keyboard without 5+ min Away break
- `longest_computer_session_secs` — daily maximum of above (for "Screen time" KPI)
- `away_bout_secs` — current Away duration; at 300s triggers position change + resets `continuous_computer_secs`
- `first_reading_at` — timestamp of first sensor reading today (for hours_worked calculation)

Source: `session_reading.rs:12-101`, `session_types.rs:9`

---

## 2. UI Element x State Matrix

### 2.1 Tray Icon

| Property | Sitting | Standing | Walking | Away |
|----------|---------|----------|---------|------|
| **Icon PNG** | `tray-ok` / `tray-warn` / `tray-alert` | `tray-standing` | `tray-idle` | `tray-idle` |
| **Dot color (fallback)** | green `#00C864` (<60%) / amber `#C89600` (60-85%) / red `#C80000` (>85%) | gold `#DAA520` | gray `#808080` | gray `#808080` |
| **Tooltip** | `↕ 72.3 cm — Sitting (MM:SS) +score` | `↕ 105.0 cm — Standing (MM:SS) +score` | `↕ 105.0 cm — Walking (MM:SS) +score` | `↕ 0.0 cm — Away (00:00) +score` |

**Tooltip format**: `↕ {height} cm — {State} ({duration}) {score}`
- Sitting: duration = total sitting seconds today
- Standing: duration = current break seconds (time since standing up)
- Walking: duration = current break seconds
- Away: duration = 0

**Score format**: `+38` when positive, `-12` when negative.

Source: `tray.rs:119-158`, `tray_icon.rs:18-35`, `tray_controller.rs:187-210`

### 2.2 Overlay Progress Bar (WinAPI, top of screen)

| Property | Sitting | Standing | Walking | Away |
|----------|---------|----------|---------|------|
| **Visible** | Yes | Yes (gold bar) | No | No |
| **Color** | green `#4caf50` (<60%) / yellow `#ffc107` (60-85%) / red `#f44336` (>=85%) | goldenrod `#DAA520` -> bright gold `#FFD720` (interpolated by lap progress) | n/a | n/a |
| **Progress** | `limit_used_secs / session_limit_secs` (0.0-1.0, fills left to right) — credited counter, see §5 | lap progress within standing_target_mins (resets each lap) | n/a | n/a |
| **Variant** | 0=solid (normal), 2=pulsing (alert Stage1) | 0=solid, 2=pulsing (lap flash, 2s) | n/a | n/a |
| **Height** | default 4px (configurable 1-20px via `OVERLAY_HEIGHT`) | same | n/a | n/a |
| **Lap visual** | none | 2-layer: dark goldenrod `#B8860B` base (completed laps) + bright gold fill (current lap). 2s pulse flash on lap completion. | n/a | n/a |

**Standing bar behavior**:
- Bar fills 0->100% over `standing_target_mins` (default 15 min)
- On lap completion (reaching target), bar resets and a 2-second pulse flash plays
- `lap_flash_until` prevents double-flash on same lap
- `clear_standing()` called on transition away from standing

**State transitions**:
- Sitting -> Standing: `clear_standing()`, then `hide()` overlay, then standing bar starts
- Standing -> Sitting: `clear_standing()`, then sitting bar `show()` with progress
- To Walking/Away: `clear_standing()`, `hide()`

Source: `overlay_renderer.rs`, `overlay_standing.rs`, `tray_controller.rs:69-84,109-127`, `colors.rs:17-36`

### 2.3 Widget (OneBar) — Layout Hierarchy

The popup widget renders top-to-bottom (reordered in E002-T08):

```
┌──────────────────────────────────────────┐
│ ● sitting @ desk (72 cm)              ⚙  │  OneBarHeader (state dot + label)
├──────────────────────────────────────────┤
│ 25:00 / 40:00            previously:     │  OneBarTimer (big number + bar)
│ ████████████████████░░░░░░░░░░░░░░░░░░░ │  stood 12m ✓
├──────────────────────────────────────────┤
│ Today                                    │  KpiStrip ("Today" + 4 badges
│ ↕ Standing 12%  ⇄ Changes 1.2/h  ...    │   + HealthWidget, see 2.9)
│ Steps 8,412 ♥ 61 phone ↻                 │
├──────────────────────────────────────────┤
│ ▓▓▓▓░░▓▓▓▓▓▓▓▓░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓░▓▓▓▓▓▓│  OneBarTimeline (+ live block)
│  8         9        10        11         │  (hour markers; click → Analyst)
└──────────────────────────────────────────┘
```

**Header**: `● {state} @ desk ({height} cm) ⚙` — dot color matches state (gold=standing, green/yellow/red=sitting, gray=away).

Source: `OneBarWidget.tsx`

### 2.4 Widget (OneBar) — Timer (state-aware)

| Property | Sitting | Standing / Walking / Away |
|----------|---------|--------------------------|
| **State label** | `sitting` | `standing` / `walking` / `away` |
| **Elapsed** | `currentSessionSecs` | `breakSecs` (time since last state change) |
| **Total** | `limitSecs` (sit limit) | `standLimitSecs` (standing target) |
| **Big number** | `25:00 / 40:00` (elapsed / total) | `08:30 / 15:00` (break / stand target) |
| **Overtime sign** | `+` prefix when `limitRemaining < 0` | n/a |
| **Progress bar color** | `sitting` scheme (green→yellow→red) | `standing` scheme (gold) |
| **Previous session** | shows if available: `previously: stood 12m ✓` or `away 5m` | same |

Source: `OneBarTimer.tsx:22-71`

### 2.5 Widget (OneBar) — KPI Strip (added E001)

Four daily metrics rendered as color-coded badges above the timeline.
Computed server-side by `MetricEngine` (Rust), transported via `DashboardState` IPC.

| KPI ID | Label | Formula | Green | Yellow | Red |
|--------|-------|---------|-------|--------|-----|
| `standing_pct` | Standing | `standing_secs / (sitting + standing) × 100` | ≥15% | 10-15% | <10% |
| `position_rate` | Changes/h | `position_changes / hours_worked` | ≥1.0/h | 0.5-1.0/h | <0.5/h |
| `hourly_breaks` | Breaks | hours with ≥5 min away / total hours | all covered | 1-2 missed | ≥3 missed |
| `longest_session` | Session | `longest_computer_session_secs / 60` | <45m | 45-75m | >75m |

**Badge rendering**: border color = level color (`--signal-ok` / `--signal-warn` / `--signal-alert`).
Value text color matches level. Personal best shows `PB` tag.

**Tooltips**: each badge has a hover tooltip explaining the metric and its thresholds.

**Early data**: metrics show `—` until `kpi_early_data_threshold_mins` (30 min) of data collected.

**Data flow**: `SessionState` → `MetricEngine.compute_all()` → `DashboardState.metrics` → IPC → `useDesk().metrics` → `KpiStrip`.

Source: `metrics/mod.rs`, `metrics/*.rs`, `KpiStrip.tsx`

### 2.6 Widget (OneBar) — Coach Message (REMOVED in E001)

> OneBarCoach was removed in E001. The KPI strip and inline progress bar now communicate
> session status visually. No text-based coach messages remain in the widget.

### 2.7 Widget (OneBar) — Temperature Background

| State/Condition | Temperature | Background Color | Effect |
|----------------|-------------|------------------|--------|
| Away | `away` | `var(--panel-base)` = `#141210` | Neutral |
| Standing/Walking, breakResetProgress >= 1.0 | `reset` | `#0e110c` | Green-shifted + glow |
| Standing/Walking, breakResetProgress < 1.0 | `standing` | `#0e100c` | Slight green shift |
| Sitting, limitRatio < 0.5 | `calm` | `var(--panel-base)` = `#141210` | Neutral |
| Sitting, 0.5 <= limitRatio < 0.8 | `warm` | `#12100c` | Slight warm shift |
| Sitting, 0.8 <= limitRatio < 1.0 | `hot` | `#140e0c` | Warmer |
| Sitting, limitRatio >= 1.0 | `burning` | `#16100c` | Noticeably warm |

Source: `temperature.ts:26-37`, `DESIGN.md:93-103`

### 2.8 Timeline Blocks

| State | Block CSS class | Visual color |
|-------|----------------|--------------|
| Sitting | `--sitting` | Red-toned |
| Standing | `--standing` | Green-toned |
| Walking | `--standing` (same as Standing) | Green-toned |
| Away | `--away` | Gray-toned |
| Current session (last, open-ended) | additional `--current` class | Glowing right edge |

Empty state: shows "No sessions yet" when no sessions recorded.

Hover tooltip: `HH:MM — {state} {duration}` (e.g., `09:15 — sitting 23m`).

**Click opens the Analyst window.** The timeline strip is a compressed view of
one day; clicking it invokes the `open_analyst_window` Tauri command, which
opens the full Analyst window (section 11) — the "magnified" version of the
same data. The container carries `role="button"` and `tabIndex={0}`, so Enter
and Space do the same thing for keyboard users, and the hover title reads
"Click to open Analyst — full timeline view". If the command fails, the click
is a no-op and the failure is logged to the console only; nothing in the popup
changes.

> **Known gap (2026-09-06):** the click/keyboard handler is attached only to
> the *empty-state* container (`OneBarTimeline.tsx:71-81`). Once the day has
> sessions, the populated container (`OneBarTimeline.tsx:124`) renders without
> it, so the timeline stops being clickable exactly when it is worth opening.
> Documented here rather than silently fixed — the fix touches a file outside
> E018-T08's write set.

Source: `OneBarTimeline.tsx:13-17,66-86,124`

### 2.9 Health Widget (steps + optional heart rate)

A compact "health today" badge slotted **inside** the KPI strip (2.5), so it
wraps in the same flex flow as the four metric badges and reads as a fifth
badge. Unlike those four, its numbers do not come from the app's own engine —
they come from whichever health source is registered, merged by
`HealthAggregator` and read through `get_health_today` / `refresh_health_now`.

Renamed from **Steps Widget (Google Fit)** in E021-T04: the source is no
longer Google Fit specifically, so the badge names the metric, not the vendor
([ADR 020](ADR/020-health-source-inlet.md)).

| Situation | What the user sees |
|---|---|
| No source configured | `Health  connect health source` — muted; tooltip names both routes in: `GOOGLE_REFRESH_TOKEN` in `.env`, or `POST /display/health` |
| Configured, no snapshot yet | `Steps  —` |
| Fresh snapshot (< 1 h) | `Steps  8,412` + the source label (`Google Fit`, `phone`, or the raw `source_id`) + a `↻` refresh button. Count uses the OS locale's thousands separator |
| Source supplies heart rate | an extra `♥ 61` badge; absent when no source sent one — never `0` |
| Stale snapshot (> 1 h) | same count, dimmed; tooltip: "Last successful refresh > 1h ago (stale)" |
| Refresh token revoked (`auth_revoked`) | `Health  reconnect google fit` — tooltip gives the exact repair command, `node apps/desk/scripts/google-fit-auth.cjs` |
| Transient error (network, API) | last known count plus a small red dot; tooltip carries the error message |
| Google Fit is the source | the tooltip appends **"Google Fit ends late 2026"** — a dated warning, not a removal (E021-D6) |
| Remote display (phone browser) | **the same data as the desktop**, minus the `↻` button. Before E021 the widget bailed out whenever it was not running under Tauri, so the phone showed nothing |

**Transports.** One hook, `useHealth.ts`, two behaviours: on the desktop it
polls over IPC (first fetch ~500 ms after mount, then 5 min on success, with a
1 → 2 → 5 → 10 → 30 min backoff on transient failure that stays at 30); on the
phone it reads `health` out of the WS snapshot and **never polls**.
`auth_revoked` stops the desktop polling entirely — retrying without the user
re-consenting cannot succeed. The `↻` button forces a refresh and is disabled
while one is in flight.

**Why it is here.** Standing at the desk and walking are both breaks, but the
app's own sensor only sees the desk. Steps are the one signal that
distinguishes "stood still" from "actually moved", so the badge sits next to
the ergonomic KPIs rather than in the Analyst window.

Setup, the `.env` keys and the push contract: root `CLAUDE.md` §Health
sources, and [`docs/REMOTE_DISPLAY.md`](../docs/REMOTE_DISPLAY.md).

Source: `HealthWidget.tsx`, `useHealth.ts`, `OneBarWidget.tsx:69-71`,
`KpiStrip.tsx:14`, `health_source.rs`, `google_fit_service.rs`,
`remote_routes_health.rs`, `commands_health.rs`

---

## 3. Alert Escalation Flow

The alert system activates when the user sits past their limit.

### State Diagram

```
┌──────┐    progress >= 1.0    ┌────────┐    2 min elapsed    ┌────────┐
│ Idle │──────────────────────►│ Stage1 │────────────────────►│ Stage2 │
│      │                       │ (pulse)│                      │ (popup)│
└──────┘                       └────────┘                      └────┬───┘
   ▲                              ▲  │                              │
   │ progress < 1.0               │  │ progress < 1.0               │ user
   │ (resolved)                   │  │ (resolved)                   │ dismisses
   │                              │  ▼                              ▼
   │                              │ ┌──────┐                   ┌─────────┐
   │                              └─┤ Idle │                   │ Snoozed │
   │                                └──────┘                   └────┬────┘
   │                                                                │
   │  progress < 1.0 (resolved)      snooze timer expires           │
   ├─────────────────────────────────◄──────────────────────────────┘
   │
   │  on_standing() at any stage
   ├────── resets to Idle, clears snooze_index
```

### Escalation Stages

| Stage | Trigger | UI Effect | Duration |
|-------|---------|-----------|----------|
| **Idle** | Default state | Normal overlay bar (solid variant 0) | Until limit reached |
| **Stage1** | `progress >= 1.0` (sitting limit reached) | Overlay bar switches to pulsing variant (variant=2) | 120 seconds (2 min) |
| **Stage2** | Stage1 active for >= 120s | WinAPI popup window with message | Until dismissed or resolved |
| **Stage3-5** | Not implemented (T017) | Reserved for future escalation | n/a |
| **Snoozed** | User dismisses Stage2 popup | Returns to normal overlay, popup hidden | Deescalating cooldown |

### Snooze Behavior

Snooze durations deescalate (get longer) with each consecutive dismiss:

| Dismiss # | Snooze Duration | Message Tone |
|-----------|----------------|--------------|
| 1st | 5 minutes | Neutral: "Time for a stretch!" or "Your body needs a break" |
| 2nd | 15 minutes | Neutral |
| 3rd+ | 30 minutes | Positive: "Even 2 min standing helps blood flow" or "Quick stand = fresh mind" |
| 4th+ | 60 minutes (capped) | Positive |

**Tone shift threshold**: After 3 dismisses (`tone_shift_threshold=3`), messages switch from neutral to positive/encouraging.

**Standing resets everything**: Any transition to Standing state resets the alert to Idle and clears snooze_index back to 0. Actions: `StopPulse` + `DismissPopup`.

Source: `alert_manager.rs`, `alert_config.rs:32-57`, `alert_actions.rs`

---

## 4. Points System

### Score Accumulation (per tick, ~1/s)

| State | Points per minute | Notes |
|-------|-------------------|-------|
| **Sitting** | -0.5 pts/min | Penalty for sitting (configurable: `pts_sitting_per_min`) |
| **Standing** | +1.0 pts/min | Reward for standing (configurable: `pts_standing_per_min`) |
| **Walking** | 0 pts/min | Neutral |
| **Away** | 0 pts/min | Neutral |

### Lap Bonus

When the user stands continuously for `standing_target_mins` (default 15 min), they earn a **+5.0 pts** bonus (`pts_session_bonus`). This can repeat: standing for 30 min = 2 laps = +10 pts bonus total.

**Lap tracking**:
- `standing_session_secs` increments every second while standing
- `current_lap = standing_session_secs / (standing_target_mins * 60)`
- When `current_lap > lap_bonus_awarded_for_lap`, award bonus and update tracker
- Resets on state exit from Standing (sit down, walk away)

### Daily Score

- Resets to 0.0 at midnight (daily reset check, throttled to 60s intervals)
- Displayed in tray tooltip: `+38` or `-12`
- Displayed in widget via `todayScore` prop

Source: `session_manager.rs:208-229`, `config.rs:22-31`

---

## 5. Session Lifecycle

### Which counter the UI reads

One counter drives everything the user reads as session progress: the
credited `sitting_seconds`, exposed to the UI as `limit_used_secs`
(`limitUsedSecs` in TypeScript). Timer, overlay fill, colour band and the
sit-limit alert all take that one value, so the number and its colour can
never disagree.

`sitting_seconds_total` and `standing_seconds` are raw daily totals for KPI
and reporting. Compare raw with raw; never a credited value with a raw one.

`secs_since_last_break` (seconds since the last position change) exists for
the Debug tab only. It ignores break credit and restarts at zero on every
position change — that is exactly the reset E015 removed from the UI. See
[ADR 008](ADR/008-proportional-break-credit.md) revision 2026-09-06.

### Session Creation

A sitting session begins when the user transitions **to Sitting** from any other state:
- `sitting_started = now` timestamp recorded
- Break credit for the break just ended is applied to `sitting_seconds` (below)

### Session Accumulation

While sitting, seconds are computed live (not committed until exit):
- `live_sitting = sitting_seconds + (now - sitting_started)` — this is what
  `limit_used_secs` reports
- `live_break = (now - break_started)` (computed on demand for standing/walking/away)

### Session Completion

A sitting session ends when transitioning **from Sitting** to any other state:
- Elapsed time committed: `sitting_seconds += elapsed`
- `CompletedSession` created with `started_at`, `ended_at`, `duration_secs`
- Saved to SQLite via `db_sessions::insert_session()`, with the `break_credit`
  applied on that transition stored on the row
- `alert_fired` reset to false for the next sitting stint

### Break Credit Rules

Applied when returning to Sitting after a break (Standing, Walking, or Away).
Proportional, per [ADR 008](ADR/008-proportional-break-credit.md) — no tiers,
no cliff at 5 or 10 minutes:

| Break Duration | Credit Type | Effect on `sitting_seconds` |
|---------------|-------------|----------------------------|
| < `break_min_secs` (default 60s) | `None` | No change — session continues as if uninterrupted |
| >= `break_min_secs`, credit smaller than the counter | `Partial` | `sitting_seconds -= break_secs × break_credit_multiplier` |
| >= `break_min_secs`, credit reaches the counter | `Full` | `sitting_seconds` floors at 0 |

`break_credit_multiplier` is per ergonomic profile: **3.0 in `standard`**
(15 min break cancels 45 min of sitting), 2.0 in `default`, `relaxed`,
`strict` and `demo`. Both knobs live under `limits` in
`profiles/ergonomic/*.json`.

Source: `session_breaks.rs:117-155`

### Timeline Representation

Sessions appear as proportional colored blocks in the timeline:
- Width = `(session.duration_secs / totalDaySecs) * 100%`, minimum 0.5%
- Current (open) session: last entry with `end=null`, gets `--current` glow class
- Hover shows tooltip: `HH:MM — {state} {duration}`

### Daily Reset

Checked every 60 seconds. On new day:
- `sitting_seconds`, `standing_seconds`, `position_changes` reset to 0
- `continuous_computer_secs`, `longest_computer_session_secs` reset to 0
- `daily_score` reset to 0
- All notification flags cleared
- Standing session tracking reset
- `desk:daily-reset` event emitted

Source: `session_breaks.rs:11-95`, `session_types.rs:12-17`, `session_manager.rs:176-205`

---

## 6. Color Language

### Communication Architecture (updated 2026-03-30)

All visual signals are driven by **CommunicationPolicy** which reads from a JSON communication profile. Colors have a unified dictionary — each color means one thing everywhere.

### Color Dictionary (global, immutable — profiles do NOT change these)

| Color | Hex | Meaning |
|-------|-----|---------|
| **None** | — | No action needed. Tray: no dot visible. |
| **Yellow** | `#ffc107` | Heads-up: change position soon |
| **Red** | `#f44336` | Change position now |
| **Gray** | `#808080` | System issue (sensor disconnected) |
| **Neutral** | `#2c2920` | Subtle progress indicator (no urgency) |

**Green and gold are removed.** Sitting at a computer is never "green." Standing has no special color — it uses neutral progress bar. Absence of signal = OK.

### KPI Badge Colors (independent from communication policy)

| Token | Hex | Usage |
|-------|-----|-------|
| `--signal-ok` | `#4caf50` | KPI metric = good (kept for metric evaluation only) |
| `--signal-warn` | `#ffc107` | KPI metric = warning |
| `--signal-alert` | `#f44336` | KPI metric = bad |

### Signal Matrix (default communication profile)

**Sitting** (limit from ergonomic profile, default 40 min):

| Phase | Tray | Overlay | Popup header | Notification |
|-------|------|---------|--------------|--------------|
| Baseline (0–30 min) | nothing | neutral bar | "Sitting (12:34)" | — |
| Warning (30–40 min) | yellow dot | yellow bar | yellow | — |
| Limit (40–45 min) | red dot | red bar | red | toast |
| Overdue (45+ min) | red blink 3×/10s | red pulsing | red | popup |

**Standing** (limit from ergonomic profile, default 20 min):

| Phase | Tray | Overlay | Popup header | Notification |
|-------|------|---------|--------------|--------------|
| Baseline (0–15 min) | nothing | neutral progress + lap | "Standing" | — |
| Warning (15–20 min) | yellow dot | yellow bar | yellow | — |
| Limit (20–25 min) | red dot | red bar | red | toast |
| Overdue (25+ min) | red blink 3×/10s | red pulsing | red | popup |

**Away/Walking**: nothing visible, overlay hidden.
**Sensor disconnected**: gray blink 3×/10s, toast once.

### Profile System

Two independent profile types control behavior:
- **Ergonomic Profile** (`profiles/ergonomic/*.json`) — limits, scoring, KPI thresholds, break credit multiplier
- **Communication Profile** (`profiles/communication/*.json`) — escalation timing, channels, blink patterns, messages

Profiles are hot-reloadable: edit JSON → app picks up changes in ~1s.

### Break Credit (proportional)

Each second of break cancels `break_credit_multiplier` seconds of sitting —
**3.0 in `standard`**, 2.0 in `default`, `relaxed`, `strict` and `demo`.
Configurable in ergonomic profile: `limits.break_min_secs` and `limits.break_credit_multiplier`.
It reduces the credited counter the UI reads (`limit_used_secs`); see §5.

### Temperature Backgrounds

| Temperature | Background | Trigger |
|-------------|-----------|---------|
| `calm` | `#141210` (panel-base) | Sitting, limitRatio < 0.5 |
| `warm` | `#12100c` | Sitting, 0.5 <= limitRatio < 0.8 |
| `hot` | `#140e0c` | Sitting, 0.8 <= limitRatio < 1.0 |
| `burning` | `#16100c` | Sitting, limitRatio >= 1.0 (overtime) |
| `standing` | `#0e100c` | Standing/Walking |
| `away` | `#141210` (panel-base) | Away state |

Source: `communication_policy.rs`, `colors.rs`, `temperature.ts`

---

## 7. Time-Based Scenarios

### Scenario 1: User Sits All Day (Never Stands)

```
08:00  State: Away -> Sitting (desk lowered)
       Overlay: visible, green bar at 0%
       Temperature: calm
       KPIs: all "—" (early data threshold not reached)
       Score: starts at 0, decreasing at -0.5/min

08:22  limitRatio crosses 0.5
       Temperature: warm
       KPIs: Standing 0% (red), Changes 0/h (red), Session 22m (green)
       Overlay: still green (< 60%)

08:27  limitRatio crosses 0.6 (27/45)
       Overlay: yellow #ffc107

08:36  limitRatio crosses 0.8 (36/45)
       Temperature: hot

08:38  limitRatio crosses 0.85 (38.25/45)
       Overlay: red #f44336
       Tray icon: red dot

08:45  limitRatio reaches 1.0 — ALERT
       Temperature: burning
       KPIs: Session 45m (yellow), Standing 0% (red)
       Toast notification: "Time to stand up!"
       Alert Stage1 starts: overlay pulses (variant=2)

08:47  Alert Stage2 (2 min after Stage1)
       WinAPI popup: "Time for a stretch!"
       Overlay continues pulsing

08:47  User dismisses popup
       Snooze #1: 5 minutes
       Overlay stops pulsing (variant=0)

08:52  Snooze expires -> Stage1 again
       Overlay pulses

08:54  Stage2 again -> popup: "Your body needs a break"

08:54  User dismisses popup
       Snooze #2: 15 minutes

09:09  Snooze expires -> Stage1 -> Stage2 (after 2 min)

09:11  Popup: "Your body needs a break" (cycles neutral messages)

09:11  User dismisses popup
       Snooze #3: 30 minutes (tone shifts to positive)

       Score at 08:45 (45 min sitting): -22.5 pts
       Score continuing to decrease at -0.5/min
```

### Scenario 2: User Alternates Sitting/Standing Regularly

```
08:00  Sitting starts
       Overlay: green bar, calm temperature

08:30  User raises desk -> Standing (after 5s debounce)
       Overlay: gold bar fills over 15 min target
       Temperature: standing
       Score at 08:30: -15 pts (30 min * -0.5)

08:40  Standing for 10 min -> breakResetProgress >= 1.0
       Temperature: reset (green glow)

08:45  Standing for 15 min -> lap complete
       Overlay: 2-second pulse flash, then resets to 0%
       Lap bonus: +5 pts
       Score: -15 + 15*(+1.0) + 5 = +5 pts

08:45  User lowers desk -> Sitting
       Break credit: 15 min * 3.0 = 45 min, more than the counter holds
       -> Full, sitting_seconds floors at 0
       Overlay: green bar at 0%, calm temperature

09:15  30 min sitting, limitRatio = 0.67
       Overlay: yellow
       KPIs: Standing 12% (yellow), Changes 1.3/h (green)

09:20  User raises desk -> Standing again
       Break credit from the 35 min sitting: none yet (no break applied until return)
       Gold bar starts filling again

09:30  10 min standing -> 30 min of credit earned
       User sits -> 35 min sitting - 30 min credit = 5 min left on the timer
       Credit type: Partial. The timer shows 5 min, not 0 — no reset on sit.

       Pattern continues through the day.
       Typical daily score: positive, growing with each standing lap.
```

### Scenario 3: User Sits, Gets Alerts, Eventually Stands

```
08:00  Sitting starts, calm, green bar

08:45  Limit reached (45 min)
       Alert Stage1: bar pulses red
       Toast: "Time to stand up!"
       KPIs: Session 45m (yellow)

08:47  Stage2: popup "Time for a stretch!"
       User dismisses -> 5 min snooze

08:52  Stage1 again -> bar pulses
08:54  Stage2 -> popup "Your body needs a break"
       User dismisses -> 15 min snooze

09:09  Stage1 again -> bar pulses
09:11  Stage2 -> popup "Even 2 min standing helps blood flow" (positive tone)
       User dismisses -> 30 min snooze

09:41  Stage1 -> bar pulses
09:43  Stage2 -> popup "Quick stand = fresh mind"

09:44  USER FINALLY STANDS
       Alert: on_standing() -> Idle, snooze_index=0
       Overlay: stops pulsing, switches to gold standing bar

       Score at this point: ~50 min overtime * -0.5/min = about -40 pts total

09:49  Only 5 min standing
       User sits -> Break credit: Partial (5 min * 3.0 = 15 min)
       Effective sitting_seconds: was ~105 min, now ~90 min
       Still over limit! limitRatio > 1.0
       Alert system immediately re-enters Stage1

09:59  10 more min sitting (~100 min) -> user stands again, this time 20 min

10:19  20 min * 3.0 = 60 min of credit
       sitting_seconds: 100 - 60 = 40 min, just under the limit
       Timer shows 40 min on the return to sitting — not 0
       Score recovering with +1.0/min standing + lap bonus

       A deep overshoot is not cleared by one short break: with multiplier
       3.0 it takes a break of a third of the accumulated sitting time.
```

---

## 8. Configuration Knobs

### Session Limits

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `sit_limit_mins` | 45 | 10-90 | Sitting session limit before alerts fire. Controls overlay progress bar fill rate and limitRatio. |
| `stand_limit_mins` | 15 | 5-60 | Standing session limit (triggers "consider sitting" nudge when exceeded). |
| `standing_target_mins` | 15 | 5-60 | Duration for gold bar to fill one lap. Reaching it awards `pts_session_bonus`. |
| `stand_max_mins` | 90 | 30-120 | Maximum continuous standing before protective nudge (ceiling, not goal). |

### Calibration

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `sitting_mm` | 720 | 400-900 | Sensor distance when desk is in sitting position. |
| `standing_mm` | 1050 | 900-1400 | Sensor distance when desk is in standing position. |
| `desk_thickness_mm` | 30 | (unclamped) | Subtracted from sensor reading: `desk_height = sensor_mm/10 - thickness_mm/10`. |

Midpoint for state detection = `(sitting_mm + standing_mm) / 2 / 10` = 88.5 cm by default.

Validation: if `sitting_mm >= standing_mm`, both reset to defaults with a warning.

### Points

| Parameter | Default | Effect |
|-----------|---------|--------|
| `pts_sitting_per_min` | -0.5 | Points per minute while sitting (negative = penalty). |
| `pts_standing_per_min` | +1.0 | Points per minute while standing. |
| `pts_session_bonus` | +5.0 | Bonus awarded each time a standing lap completes. |

### KPI Thresholds (added E001)

| Parameter | Default | Effect |
|-----------|---------|--------|
| `kpi_standing_green_pct` | 15.0 | Standing % threshold for green level. |
| `kpi_standing_yellow_pct` | 10.0 | Standing % threshold for yellow level (below = red). |
| `kpi_changes_green` | 1.0 | Position changes/h threshold for green. |
| `kpi_changes_yellow` | 0.5 | Position changes/h threshold for yellow (below = red). |
| `kpi_break_yellow_missed` | 2 | Missed hourly breaks before yellow. |
| `kpi_break_red_missed` | 3 | Missed hourly breaks before red. |
| `kpi_session_green_mins` | 45 | Screen time (min) below which = green. |
| `kpi_session_yellow_mins` | 75 | Screen time (min) below which = yellow (above = red). |
| `kpi_early_data_threshold_mins` | 30 | Minutes before KPIs show real values (shows "—" before). |

### Notifications

| Parameter | Default | Effect |
|-----------|---------|--------|
| `notify_inactivity` | true | Alert when no position change for 60 minutes. |
| `notify_daily_posture_balance` | true | Alert when sitting > 2x standing time today. |
| `notify_praise_halfway` | true | Praise when standing reaches 50% of `stand_limit_secs`. |
| `notify_webhook_enabled` | false | Master switch for the outbound phone push (Settings → More → Phone notifications). Off means `WebhookNotifier::from_env_or_config` returns `None` and nothing is sent, whatever the URL says. |
| `notify_webhook_url` | *(unset)* | Endpoint the push goes to, ntfy-shaped (`POST <base>/<topic>`, JSON `{title, message, priority, tags}`). Falls back to `DESK_NOTIFY_WEBHOOK_URL` from `.env` when the field is empty. |
| `voice_ai_model` | *(unset)* | OpenRouter model for the voice reply. Empty = `google/gemini-2.5-flash-lite`. Ignored entirely when `OPENROUTER_API_KEY` is unset — no key means no reply, not an error. |

### Overlay (Environment Variables, read at startup)

| Env Var | Default | Values | Effect |
|---------|---------|--------|--------|
| `OVERLAY_DATA` | `demo` (debug) / `live` (release) | `demo`, `live`, `mock` | Data source for the overlay bar. |
| `OVERLAY_MODE` | `opaque` | `opaque`, `layered` | WinAPI render mode. |
| `OVERLAY_VARIANT` | `0` | `0` (solid), `1` (gradient), `2` (pulsing) | Initial visual style. |
| `OVERLAY_HEIGHT` | `4` | 1-20 | Bar height in pixels. |

### Other

| Parameter | Default | Effect |
|-----------|---------|--------|
| `active_widget` | `one-bar` | Which widget layout is rendered in the floating window. |
| `notification_backend` | `toast` | `toast` (native), `popup` (WinAPI), or `both`. |
| `show_welcome_on_startup` | true | Whether to show the welcome popup on first launch. |

Source: `config.rs:11-153`

---

## 9. Notification Summary

Beyond the alert escalation system, these one-shot notifications fire:

| Notification | Trigger | Title | Body |
|-------------|---------|-------|------|
| Sit limit reached | `limit_used_secs >= session_limit_secs` (credited counter, once per stint) | "Time to stand up!" | "You've been sitting for 40 minutes. Take a break." |
| Stand limit reached | `break_seconds >= stand_limit_secs` while standing (once) | "You've been standing a while" | "Ready to sit down for a bit?" |
| Inactivity | No position change for 60 min | "No position change in 60 minutes" | "Time to move." |
| Posture balance | `sitting_seconds_total >= 6h` AND `sitting_seconds_total > standing_seconds * 2` — both raw daily totals, never the credited counter | "You've been sitting most of today" | "Consider standing for a while." |
| Praise halfway | Standing reaches 50% of standing target | "Halfway through your standing goal!" | "Keep it up." |
| Standing target reached | Standing reaches 100% of standing target | "Standing target reached!" | "Great break! You stood for the full target duration." |

Source: `serial_periodic.rs:36-141`, `session_breaks.rs:98-157`

### Off-screen mirror (E021, opt-in)

These are desktop toasts. When `notify_webhook_enabled` is on and a URL is
configured, `notify_webhook.rs` also pushes the sit-limit alert — and every
voice acknowledgement — to the user's own ntfy-shaped endpoint. Android shows
that as a phone notification and mirrors it to a paired watch, which is the
whole of the watch integration: **the watch shows, it never runs our code**
([ADR 021](ADR/021-voice-in-on-phone-watch-as-glance.md)).

The push is fire-and-forget: spawned on tokio, never awaited by the caller,
5 s per attempt, one retry on 5xx or timeout, no retry on 4xx. A dead webhook
delays no toast and fails no request. Default is **off** — nothing leaves the
machine until the user configures it.

---

## 10. Computer Time (Unified Cycle)

Standing phase: when `continuous_computer_secs >= max_continuous_computer_secs`:
- One toast: random nudge from `screen_break_nudge.messages` pool
- No tray/overlay change (reserved for sitting/standing limits)
- Fires once per computer session (resets on 5+ min Away)

Activity status in StateIndicator:
- idle < 30s → "Active" (subtle gray)
- idle >= 30s → "Idle Xm Ys" (yellow text)
- Away state → no desk height shown

---

## 11. Analyst Window

A standalone Tauri window dedicated to inspecting and exploring everything the
app writes to disk: minute snapshots, the events log, completed sessions, the
config store, and live signals. Opens via tray menu → **Open Analyst**.

URL: `index.html#/analyst` (the tray launches the labelled `analyst` window
defined in `tauri.conf.json`). DEV-only mockup with fixtures lives at
`index.html#/mockup/analyst`.

### Tabs

#### Catalog
Sortable, filterable table over the static data-source catalog returned by
`get_data_catalog`. Each row describes one source: name, kind
(`sqlite | file | json | store | serial | signal | events`), on-disk location,
retention rule, and the schema of one record. Click a row's *fields* details
to inspect column names and types. Use the filter box to narrow by name,
kind, or location.

The Catalog answers "what data does this app actually keep, and where does it
live on my machine?" — the entry point for anyone wanting to debug, back up,
or analyse history outside the app.

#### Explorer
A 5-chart grid driven by a single date-range picker (defaults to the last
seven days). Every range change debounces by 250 ms before re-querying the
backend, so dragging the date input does not spam the disk.

| Chart | Source | What it shows |
|-------|--------|---------------|
| Desk height timeline | `get_snapshots_range` (snapshots) | Sensor reading over the full range. Downsampled to ~1000 buckets when more than 1000 points fall in range — the curve stays smooth even for a full week of minute samples. Higher = standing. |
| State Gantt by day | `get_snapshots_range` (snapshots) | One row per day, coloured by state across the 24-hour axis. Quick visual of when the user was sitting (burgundy), standing (green), walking (blue), or away (gray). |
| Daily score trajectory | `get_snapshots_range` (snapshots) | One line per day, max posture score by hour. Reveals consistency: bumpy lines = chaotic days, smooth = stable rhythm. |
| Break credit histogram | `get_sessions_range` (sessions) | Bar chart of the three break-credit outcomes defined by ADR 008 — none, partial, full — read from the `break_credit` column the engine wrote for that session, not guessed from its duration. Sessions recorded before the column existed carry `null`, which reads as unknown, not as "none". |
| KPI trends (7 days) | derived from snapshots (last value per day) | Small multiples for Standing %, Position changes, Longest session, and Daily score. Most recent day on the right. |

Colours follow the canonical Color Dictionary (section 6): burgundy = sitting,
green = standing, blue = walking, gray = away, gold = score. The legend on
the Gantt is the source of truth.

### When to open
- After a long session: compare today's Gantt against yesterday's.
- Suspecting a logging bug: jump to Catalog → click the file path of the
  affected source.
- Tuning ergonomic or communication profiles: watch the KPI trend over the
  week before/after a profile switch.
- Verifying daily score logic: trajectory chart shows hour-by-hour buildup.

### Performance notes
- Range queries are debounced 250 ms.
- The height timeline downsamples to ~1000 average-per-bucket points; the
  full input list is never rendered as SVG path commands.
- Chart inputs are memoised on `(from, to, data.length)` so identical data
  does not re-trigger React renders.

---

## 12. Voice Dictation (phone display only)

`VoiceCapture.tsx` renders on `/display` — the phone dashboard — and **never
in the desktop popup**, which already has a keyboard. It is how the user
answers the app from across the room.
Decision record: [ADR 021](ADR/021-voice-in-on-phone-watch-as-glance.md).

### What the user sees

| Element | When | Behaviour |
|---------|------|-----------|
| Token sheet | No `desk_token` in `localStorage` | Asks for the shared secret once. Stored in `localStorage` (wrapped in try/catch — private mode throws), sent as `X-Desk-Token` on every post. Without it the inlet answers `401`. |
| Textarea + Send | Always | **The primary path.** The user taps the phone keyboard's own mic (Gboard / Samsung) and dictates into the field — OS dictation, which works on any origin. |
| Mic button | Only when `window.SpeechRecognition` exists **and** `navigator.permissions.query({name:"microphone"})` is not `denied` | Progressive enhancement. Where the browser cannot do it the button is **absent**, never present-and-broken. `/display` is plain HTTP, and Chrome wants a secure context for this API. |
| Acknowledgement | After the post | Arrives asynchronously on the WebSocket as `desk:voice-ack`, not in the POST response. Shows the parsed intent and the AI reply when there is one. |

Double-clicking Send is guarded — the second click while a post is in flight
does nothing.

### What a sentence does

`voice_intent.rs::parse` maps the transcript offline (Polish + English
regexes, no API key, no network):

| Intent | Example phrases | Effect on the app |
|--------|-----------------|-------------------|
| `snooze` | "drzemka 5", "odłóż o 10 minut", "snooze", "remind me in 20" | Sets `CommunicationPolicy`'s existing snooze fields — the same ones the alert popup's snooze button sets. Minutes clamped to **1–180**, default **5**. |
| `walk_start` | "idę na spacer", "wychodzę", "going for a walk" | **Nothing but the record.** Does not change `DeskState`, does not touch the session engine. |
| `walk_end` | "wracam", "koniec spaceru", "I'm back" | Same — recorded only. Matched *before* `walk_start`, because "wracam ze spaceru" contains a walk. |
| `note` | anything unrecognised | The fallback, by design: no transcript is ever dropped for not matching a rule. |

A spoken "I'm walking" is deliberately **not** allowed to set the state.
[ADR 015](ADR/015-pure-ergo-engine.md) keeps the session engine a pure core;
letting a sentence set position would give it two contradictory sources of
truth. A manual override is a future engine feature with its own ADR.

### Where a dictation ends up

Every sentence lands in three places, in this order:

1. **`events.log`** — a `VOICE <intent> <transcript truncated to 80 chars>`
   line, written *first*, so a later failure still leaves the record
   (`.claude/rules/logging.md`);
2. **`voice_notes`** in SQLite — transcript, intent, language, optional AI
   reply, bucketed by the local day of `captured_at_ms`. Listed by
   `list_voice_notes(day)` and visible in Analyst → Catalog as the
   `voice_notes` source;
3. **the acknowledgement** — broadcast as `desk:voice-ack`, then pushed to the
   webhook (§9) so the reply reaches the phone and the watch.

### Degraded paths

Each leg is independently optional, and the request still returns `200` with
the parsed intent when one is missing:

| Missing | Result |
|---------|--------|
| `DESK_REMOTE_TOKEN` unset on the PC | `503` — the inlet is **closed**, not open. This is the only case where nothing is recorded. |
| Wrong / missing `X-Desk-Token` | `401` |
| Body over 4 KiB | `413` |
| Transcript empty or over 2000 chars | `422` |
| No database open | Note is logged and acknowledged; `note_id` is absent |
| No `OPENROUTER_API_KEY` | Acknowledged with no `reply` — the AI is off, not broken |
| Webhook off or unreachable | Everything else still happens; the ack still shows on the phone |
