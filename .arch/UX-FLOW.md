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
| **Progress** | `sitting_seconds / session_limit_secs` (0.0-1.0, fills left to right) | lap progress within standing_target_mins (resets each lap) | n/a | n/a |
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
│ Today                                    │  KpiStrip ("Today" + 4 badges)
│ ↕ Standing 12%  ⇄ Changes 1.2/h  ...    │
├──────────────────────────────────────────┤
│ ▓▓▓▓░░▓▓▓▓▓▓▓▓░░▓▓▓▓▓▓▓▓▓▓▓▓▓▓░▓▓▓▓▓▓│  OneBarTimeline (+ live block)
│  8         9        10        11         │  (hour markers)
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

Source: `OneBarTimeline.tsx:12-102`

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

### Session Creation

A sitting session begins when the user transitions **to Sitting** from any other state:
- `sitting_started = now` timestamp recorded
- `current_session_secs` reset to 0 (always a fresh session timer)

### Session Accumulation

While sitting, seconds are computed live (not committed until exit):
- `live_sitting = sitting_seconds + (now - sitting_started)`
- `live_current_session = current_session_secs + (now - sitting_started)`
- `live_break = (now - break_started)` (computed on demand for standing/walking/away)

### Session Completion

A sitting session ends when transitioning **from Sitting** to any other state:
- Elapsed time committed: `sitting_seconds += elapsed`
- `current_session_secs` reset to 0
- `CompletedSession` created with `started_at`, `ended_at`, `duration_secs`
- Saved to SQLite via `db_sessions::insert_session()`
- `alert_fired` reset to false for the next sitting stint

### Break Credit Rules

Applied when returning to Sitting after a break (Standing, Walking, or Away):

| Break Duration | Credit Type | Effect on `sitting_seconds` |
|---------------|-------------|----------------------------|
| < 5 min (300s) | `None` | No change — session continues as if uninterrupted |
| 5-9 min (300-599s) | `Partial` | Subtract 1200s (20 min) from sitting_seconds, min 0 |
| >= 10 min (600s) | `Full` | Reset sitting_seconds to 0 |

Constants: `BREAK_SHORT_SECS=300`, `BREAK_LONG_SECS=600`, `SHORT_BREAK_CREDIT_SECS=1200`

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

### Semantic Signal Tokens (CSS) — updated E002-T06

| Token | Hex Value | Meaning | Where Used |
|-------|-----------|---------|------------|
| `--signal-ok` | `#4caf50` | Green — sitting within limit | KPI badges, progress bar |
| `--signal-warn` | `#ffc107` | Yellow — approaching limit | KPI badges, progress bar |
| `--signal-alert` | `#f44336` | Red — limit exceeded | KPI badges, progress bar |
| `--signal-up` | `#DAA520` | Gold — standing state | Standing indicators, bar |
| `--signal-away` | `#808080` | Gray — away/idle | Away indicators |

### Overlay Bar Colors (Rust — `colors.rs`)

| Progress Range | Color | Hex | RGB |
|---------------|-------|-----|-----|
| < 60% | Green | `#4caf50` | (76, 175, 80) |
| 60-85% | Yellow | `#ffc107` | (255, 193, 7) |
| >= 85% | Red | `#f44336` | (244, 67, 54) |

### Standing Bar Colors (Rust — `colors.rs`)

Interpolates from goldenrod to bright gold as standing lap progresses:

| Progress | Color | RGB |
|----------|-------|-----|
| 0% | Goldenrod | (218, 165, 32) `#DAA520` |
| 50% | Mid-gold | (~236, ~190, 32) |
| 100% | Bright gold | (255, 215, 32) `#FFD720` |

### Tray Icon Dot Colors (Rust — `tray.rs`)

| State + Progress | Color | Hex |
|-----------------|-------|-----|
| Sitting < 60% | Green | `#00C864` |
| Sitting 60-85% | Amber | `#C89600` |
| Sitting > 85% | Red | `#C80000` |
| Standing | Gold | `#DAA520` |
| Walking / Away | Gray | `#808080` |

Note: tray dot colors differ from overlay bar colors. Both use the same thresholds (60%, 85%) but different hex values.

### Temperature Backgrounds

| Temperature | Background | Trigger |
|-------------|-----------|---------|
| `calm` | `#141210` (panel-base) | Sitting, limitRatio < 0.5 |
| `warm` | `#12100c` | Sitting, 0.5 <= limitRatio < 0.8 |
| `hot` | `#140e0c` | Sitting, 0.8 <= limitRatio < 1.0 |
| `burning` | `#16100c` | Sitting, limitRatio >= 1.0 (overtime) |
| `standing` | `#0e100c` | Standing/Walking, break not yet full |
| `reset` | `#0e110c` | Standing/Walking, break >= full reset threshold |
| `away` | `#141210` (panel-base) | Away state |

Source: `DESIGN.md:57-103`, `colors.rs`, `tray.rs:150-158`, `temperature.ts`

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
       Break credit: Full (15 min >= 10 min)
       sitting_seconds reset to 0
       Overlay: green bar at 0%, calm temperature

09:15  30 min sitting, limitRatio = 0.67
       Overlay: yellow
       KPIs: Standing 12% (yellow), Changes 1.3/h (green)

09:20  User raises desk -> Standing again
       Break credit from the 35 min sitting: none yet (no break applied until return)
       Gold bar starts filling again

09:30  10 min standing -> full reset earned
       User sits -> sitting_seconds = 0 again

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
       User sits -> Break credit: Partial (5-9 min)
       sitting_seconds reduced by 1200s (20 min)
       Effective sitting_seconds: was ~105 min, now ~85 min
       Still over limit! limitRatio > 1.0
       Alert system immediately re-enters Stage1

09:59  User stands again, this time for 12 min

10:11  Full break credit earned
       sitting_seconds reset to 0
       User can sit fresh for 45 min
       Score recovering with +1.0/min standing + lap bonus
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
| Sit limit reached | `sitting_seconds >= session_limit_secs` (once per stint) | "Time to stand up!" | "You've been sitting for 40 minutes. Take a break." |
| Stand limit reached | `break_seconds >= stand_limit_secs` while standing (once) | "You've been standing a while" | "Ready to sit down for a bit?" |
| Inactivity | No position change for 60 min | "No position change in 60 minutes" | "Time to move." |
| Posture balance | `sitting_seconds > standing_seconds * 2` today | "You've been sitting most of today" | "Consider standing for a while." |
| Praise halfway | Standing reaches 50% of standing target | "Halfway through your standing goal!" | "Keep it up." |
| Standing target reached | Standing reaches 100% of standing target | "Standing target reached!" | "Great break! You stood for the full target duration." |

Source: `serial_periodic.rs:36-141`, `session_breaks.rs:98-157`
