---
id: E001
created: 2026-03-23
status: reviewed
title: Timer UX + KPI Dashboard + MetricEngine Platform
review_mode: EXPANSION
review_date: 2026-03-23
absorbs: T045 (Popup Timer UX Redesign)
---

# E001: Timer UX + KPI Dashboard + MetricEngine Platform

> **Note:** This epic absorbs T045 (Popup Timer UX Redesign) as Wave 1.
> T045 coach message changes are SKIPPED — coach is removed entirely in Wave 3 (replaced by KPI strip).

## Waves Overview
- **Wave 1 (T045):** Timer UX — elapsed/total, unified ProgressBar, Away=gray, animations
- **Wave 2:** MetricEngine + KPI computations (Rust) + gap handling + persistence
- **Wave 3:** KPI Strip UI → replaces OneBarCoach, tray tooltip, debug tab, personal bests, timeline markers

## Problem Statement

The desk app tracks ergonomic data (sitting/standing time, position changes, breaks) but presents it poorly:

1. **Key metrics are hidden** — position changes appear only in a coach message ("Half limit used. 3 changes today") that's easy to miss and not always shown.
2. **Text descriptions are unreadable** — "Half of the limit used" doesn't communicate urgency. Users want visual progress, not sentences.
3. **No relative metrics** — raw counts (e.g., "3 position changes") are meaningless without context (was it a 4-hour or 12-hour day?).
4. **No break tracking** — the system tracks standing as a break from sitting, but doesn't track breaks from the COMPUTER (eye relief, movement).
5. **No max-session guardrail** — user can sit at the computer for 4+ hours (even standing) without any warning about eye strain / continuous screen time.

## User Outcome

At a glance, the user sees 4 color-coded KPIs telling them:
- Am I standing enough? (Standing %)
- Am I changing positions enough? (Changes/hour)
- Am I taking screen breaks? (Hourly breaks coverage)
- Am I at the computer too long? (Longest continuous session)

Green = good. Yellow = watch out. Red = act now.

Like a car dashboard — one glance and you know.

## Core Philosophy

**Position changes > standing duration.** The goal is movement variety, not static standing. Standing for 4 hours straight is barely better than sitting 4 hours straight.

**Standing ≡ Away as "break."** Both count as position changes. 5+ CONTINUOUS minutes of either = 1 change.

**Everything relative.** No raw counts without time context. Percentages, rates per hour, ratios.

## Key Decisions (from CEO review)

1. **5 continuous minutes** = position change / break (not cumulative)
2. **MetricEngine platform** — deklaratywny system metryk via Rust trait, not hardcoded 4 KPI
3. **Coach messages → KPI strip** — OneBarCoach znika, zastąpiony przez KPI strip
4. **KPI strip at top of widget** — first thing user sees
5. **hours_worked** = od pierwszego sensor reading dnia
6. **Metryki stateless** — stan w SessionManager, metryki to czyste `compute(state) → result`
7. **Combined IPC** — `get_dashboard_state()` = SessionStateDto + Vec<MetricSnapshot>
8. **Persist HourlyBreakTracker** co godzinę do tauri-plugin-store
9. **Gap handling:** < 5 min = continue last state, 5-10 min = Away, > 10 min = long break (full reset)
10. **Kompaktowy KPI layout:** colored dot + value per metric (~50px each)
11. **Debug tab** shows raw metric values
12. **All 5 delight items** built into v1 (not deferred)

## Architecture: MetricEngine Platform

```rust
/// Each metric is a self-contained, stateless computation unit
pub trait Metric: Send + Sync {
    fn id(&self) -> &str;           // "standing_pct"
    fn label(&self) -> &str;        // "Standing"
    fn compute(&self, state: &SessionState, config: &AppConfig) -> MetricResult;
}

pub struct MetricResult {
    pub value: f64,                 // 0.15 (15%)
    pub display: String,            // "15%"
    pub level: MetricLevel,         // Green / Yellow / Red
    pub is_personal_best: bool,     // ★ marker
}

pub enum MetricLevel { Green, Yellow, Red }

pub struct MetricEngine {
    metrics: Vec<Box<dyn Metric>>,
}

impl MetricEngine {
    pub fn compute_all(&self, state: &SessionState, config: &AppConfig) -> Vec<MetricSnapshot> {
        self.metrics.iter().map(|m| MetricSnapshot {
            id: m.id().into(),
            label: m.label().into(),
            result: m.compute(state, config),
        }).collect()
    }
}
```

Frontend receives `Vec<MetricSnapshot>` and renders dynamically — adding new metrics = implement trait + register.

## Scope

### In scope (core):

1. **MetricEngine (Rust)** — trait + engine + 4 initial metrics
2. **KPI Strip component** — top of widget, renders Vec<MetricSnapshot> dynamically
3. **4 initial metrics:**
   - StandingPercentMetric
   - PositionChangeRateMetric
   - HourlyBreakCoverageMetric
   - LongestSessionMetric
4. **Continuous computer time tracker** — new timer in SessionManager
5. **HourlyBreakTracker** — per-clock-hour binary break check + persistence
6. **Extended position_changes** — count 5+ continuous min Away bouts
7. **Remove OneBarCoach** — replaced by KPI strip
8. **Combined IPC:** `get_dashboard_state()` → SessionStateDto + Vec<MetricSnapshot>
9. **Snapshot logger extension** — metric values in JSON snapshots
10. **Gap handling on app restart** — persist state, detect gaps, apply rules
11. **Debug tab metrics** — raw metric values in Settings → Debug

### In scope (delight):

12. **Micro-animation** on KPI color change (pulse/glow effect)
13. **Tray tooltip with KPI summary** — "↕ 72cm | 15%↑ | 1.2/h | 5/7h"
14. **KPI tooltip with yesterday comparison** — hover → "Standing 15% (was 12% yesterday)"
15. **Personal best markers** — ★ when metric is weekly best (stored in plugin-store)
16. **Timeline hour markers** — "day start" line + hourly gridlines in OneBarTimeline

### Out of scope (deferred):

- Gamification scoring changes (daily_score stays as-is)
- Notification/alert triggers from KPI thresholds (future epic)
- Historical trend charts (needs SQLite time-series)
- Settings UI for KPI thresholds (configure via AppConfig for now)
- Break credit system changes

## Acceptance Criteria

1. KPI strip visible on main widget at all times (all states)
2. All metrics update every 1s poll cycle
3. Color thresholds configurable via AppConfig
4. Continuous-session timer resets correctly after 5 continuous min Away
5. Position change = 5 continuous min standing OR away
6. Hourly break = binary per clock hour (≥5 continuous min Away)
7. OneBarCoach removed, replaced by KPI strip
8. MetricEngine trait → new metrics without UI changes
9. App restart restores HourlyBreakTracker + handles gap
10. KPI color transitions have micro-animation
11. Tray tooltip shows KPI summary
12. Hover on KPI shows yesterday comparison
13. ★ appears on personal bests
14. Timeline has hour markers
15. Unit tests: each metric, MetricEngine, HourlyBreakTracker, continuous timer, gap handling
16. TS tests: KpiStrip rendering, color logic

## Data Flow

```
SessionManager (Rust)
    ├── existing: sitting_secs, standing_secs, position_changes
    ├── NEW: continuous_computer_secs (resets after 5 continuous min Away)
    ├── NEW: longest_computer_session_secs (daily max)
    ├── NEW: HourlyBreakTracker (per-clock-hour Away tracking, persisted)
    ├── NEW: away_bout_secs (current continuous Away duration)
    ├── NEW: first_reading_at (for hours_worked)
    ├── NEW: last_tick_ts (for gap detection on restart)
    └── snapshot() → SessionState (internal, all fields)
            ↓
MetricEngine.compute_all(state, config)
    ├── StandingPercentMetric → {value: 0.15, display: "15%", level: Green, pb: false}
    ├── PositionChangeRateMetric → {value: 1.2, display: "1.2/h", level: Green, pb: true ★}
    ├── HourlyBreakCoverageMetric → {value: 0.71, display: "5/7h", level: Green, pb: false}
    └── LongestSessionMetric → {value: 2820, display: "47m", level: Yellow, pb: false}
            ↓
IPC: get_dashboard_state() → { session: SessionStateDto, metrics: Vec<MetricSnapshot> }
            ↓
useDesk() hook (polls every 1s)
            ↓
Widget renders:
    ┌──────────────────────────────────────────┐
    │  KPI Strip (top)                         │
    │  ● 15%   ● 1.2/h★  ● 5/7h   ● 47m     │
    │  (grn)   (grn)      (grn)    (ylw)      │
    ├──────────────────────────────────────────┤
    │  State: sitting for 5m23s               │
    │  Timer: 39:37 / 45:00                   │
    │  ████████████░░░░░░ (progress bar)      │
    ├──────────────────────────────────────────┤
    │  Timeline: |▓▓▓░░░▓▓▓░▓▓▓▓|            │
    │           9  10  11  12  13  (hours)    │
    └──────────────────────────────────────────┘

Tray tooltip: "↕ 72cm | 15%↑ | 1.2/h★ | 5/7h | 47m"
```

## KPI Definitions

### 1. StandingPercentMetric
- Formula: `standing_seconds / (sitting_seconds + standing_seconds) × 100`
- Excludes Away/Walking from denominator (only "at desk" time)
- Edge case: denominator = 0 → display "—"
- Edge case: first 30 min → display "—" (too early)
- Thresholds: ≥15% green, 10-15% yellow, <10% red

### 2. PositionChangeRateMetric
- Formula: `position_changes / hours_worked`
- `hours_worked` = (now - first_reading_at) in fractional hours
- Position change = 5+ continuous min of Standing or Away (extended from current Sit↔Stand only)
- Edge case: first 30 min → display "—"
- Edge case: hours_worked = 0 → display "—"
- Thresholds: ≥1.0/h green, 0.5-1.0/h yellow, <0.5/h red

### 3. HourlyBreakCoverageMetric
- Per clock hour: was there ≥5 continuous min Away?
- Only counts completed hours where user was active
- Display: `X/Y` (X hours with break / Y completed work hours)
- Edge case: no completed hours → display "—"
- Persisted via tauri-plugin-store (survives restart)
- Thresholds: all covered = green, 1-2 missed = yellow, ≥3 missed = red

### 4. LongestSessionMetric
- Timer: time at computer (Sitting + Standing + Walking) without 5+ continuous min Away
- Resets: after 5 continuous min Away
- Tracks daily max (`longest_computer_session_secs`)
- Display: minutes (e.g., "47m", "1h12m")
- Thresholds: <45m green, 45-75m yellow, >75m red

## Gap Handling on App Restart

```
App starts → read last_tick_ts from store
    ↓
gap = now - last_tick_ts
    ↓
gap < 5 min?  → assume last known state continued
                 (add gap seconds to sitting/standing as appropriate)
    ↓
5 min ≤ gap < 10 min? → assume Away (short break)
                         → HourlyBreakTracker: mark as break
                         → continuous_computer_secs: reset
                         → position_changes: +1 (if was Sitting before gap)
    ↓
gap ≥ 10 min? → assume Away (long break / full reset)
                → same as above, plus full break credit applied
    ↓
Log: "RESTART gap=Xm, assumed=<state>" to event log
```

**Persistence (every tick to tauri-plugin-store):**
- `last_tick_ts`: DateTime
- `last_state`: DeskState
- `hourly_break_tracker`: serialized HourlyBreakTracker
- `continuous_computer_secs`: i64
- `longest_computer_session_secs`: i64
- `personal_bests`: HashMap<String, f64> (metric_id → best value this week)

## Technical Design

### Rust — New files:
- `src-tauri/src/metrics/mod.rs` — MetricEngine, Metric trait, MetricResult, MetricLevel
- `src-tauri/src/metrics/standing_pct.rs` — StandingPercentMetric
- `src-tauri/src/metrics/position_rate.rs` — PositionChangeRateMetric
- `src-tauri/src/metrics/hourly_breaks.rs` — HourlyBreakCoverageMetric
- `src-tauri/src/metrics/longest_session.rs` — LongestSessionMetric
- `src-tauri/src/metrics/tests.rs` — unit tests for all metrics
- `src-tauri/src/gap_handler.rs` — gap detection + state restoration on restart
- `src-tauri/src/hourly_break_tracker.rs` — HourlyBreakTracker struct + persistence

### Rust — Modified files:
- `session_types.rs` — add new fields (continuous_computer_secs, longest, away_bout_secs, first_reading_at)
- `session_reading.rs` — tick continuous timer, update HourlyBreakTracker on Away transitions
- `session_breaks.rs` — extend position_changes for 5-min Away bouts
- `session_daily.rs` — reset new fields + HourlyBreakTracker on daily reset
- `commands.rs` — replace get_session_state() with get_dashboard_state(), add metrics
- `snapshot_logger.rs` — add metric snapshots to JSON output
- `tray_controller.rs` — update tray tooltip format with KPI summary
- `config.rs` — add KPI threshold fields to AppConfig

### Frontend — New files:
- `src/widgets/one-bar/KpiStrip.tsx` — renders Vec<MetricSnapshot> dynamically
- `src/widgets/one-bar/kpi-strip.css` — styling: colored dots, values, animations, tooltips

### Frontend — Modified files:
- `src/hooks/useDesk.ts` — switch to get_dashboard_state(), expose metrics
- `src/hooks/useWidgetData.ts` — pass metrics to WidgetProps
- `src/widgets/one-bar/OneBarWidget.tsx` — add KpiStrip at top, remove OneBarCoach import
- `src/widgets/one-bar/OneBarTimeline.tsx` — add hour markers + "day start" line
- `src/types.ts` — add MetricSnapshot, MetricLevel, DashboardState interfaces
- `one-bar.css` — adjust layout for KPI strip at top
- `src/components/DebugSection.tsx` — add metric values

### Frontend — Removed files:
- `src/widgets/one-bar/OneBarCoach.tsx` — replaced by KPI strip

### Config additions (AppConfig):
```rust
// KPI thresholds (all configurable, sensible defaults)
pub kpi_standing_green_pct: f32,       // 15.0
pub kpi_standing_yellow_pct: f32,      // 10.0
pub kpi_changes_green: f32,            // 1.0
pub kpi_changes_yellow: f32,           // 0.5
pub kpi_break_yellow_missed: u8,       // 2
pub kpi_break_red_missed: u8,          // 3
pub kpi_session_green_mins: u32,       // 45
pub kpi_session_yellow_mins: u32,      // 75
pub kpi_early_data_threshold_mins: u32, // 30 (show "—" before this)
```

## State Machine: Continuous Computer Timer

```
         ┌──────────────────────────────────┐
         │        AT COMPUTER               │
         │  (Sitting / Standing / Walking)  │
         │                                  │
         │  continuous_computer_secs += 1   │
         │  (every tick)                    │
         │                                  │
         │  longest = max(longest, current) │
         └──────────┬───────────────────────┘
                    │ state → Away
                    ▼
         ┌──────────────────────────────────┐
         │           AWAY                   │
         │  away_bout_secs += 1             │
         │  (every tick)                    │
         │                                  │
         │  if away_bout_secs < 300:        │
         │    → continuous_computer still   │
         │      paused (not reset)          │
         │                                  │
         │  if away_bout_secs >= 300:       │
         │    → continuous_computer = 0     │
         │    → position_change += 1        │
         │    → hourly_break: mark hour     │
         └──────────┬───────────────────────┘
                    │ state → Sitting/Standing
                    ▼
         ┌──────────────────────────────────┐
         │  if was_away_bout >= 300s:       │
         │    continuous_computer = 0       │
         │    (fresh start)                 │
         │  else:                           │
         │    continuous_computer resumes   │
         │    (short bathroom break)        │
         └──────────────────────────────────┘
```

## Error Handling

All metrics return `display: "—"` and `level: Green` when data is insufficient (first 30 min, division by zero). No crashes, no silent failures. The "—" is a deliberate design choice — user sees "no data yet" rather than misleading numbers.

Gap detection logs to event log (`RESTART gap=Xm, assumed=<state>`) for debugging.

## Test Strategy

### Rust unit tests (~30 tests):
- MetricEngine: compute_all returns correct count, handles empty state
- StandingPct: normal case, div-by-zero, early day (→ "—"), thresholds
- PositionRate: normal, div-by-zero, early day, boundary values
- HourlyBreaks: basic coverage, hour boundary, persist/restore, no completed hours
- LongestSession: normal growth, reset after 5min Away, 4:59 Away (no reset), daily max
- HourlyBreakTracker: record, check, serialize/deserialize, daily reset
- ContinuousTimer: tick, pause (short Away), reset (long Away)
- GapHandler: <5 min gap, 5-10 min gap, >10 min gap, missing last_tick_ts
- Extended position_changes: 5-min Away increments, 4:59 Away doesn't

### TypeScript tests (~10 tests):
- KpiStrip: renders 4 metrics, handles empty array, correct colors
- KpiStrip: "—" display for missing data
- KpiStrip: micro-animation CSS class on color change
- KpiStrip: tooltip shows yesterday comparison
- KpiStrip: ★ marker on personal best
- Timeline: hour markers render correctly
