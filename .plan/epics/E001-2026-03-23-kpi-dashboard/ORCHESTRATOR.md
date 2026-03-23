# E001 ORCHESTRATOR — Timer UX + KPI Dashboard + MetricEngine

## Parallelism Strategy

Each wave has parallel tasks that **never edit the same file**. Each parallel task runs in its own git worktree. Sequential tasks run after their wave's parallel tasks merge.

Total: **10 tasks**, 3 waves, max **3 subagents parallel**.

---

## Wave 1: Timer UX Foundation (absorbs T045)

### Wave 1a — parallel (2 agents)

**E001-T01: New ProgressBar + Animation Components**
- Status: [ ]
- Agent: worktree
- Creates (NEW files only):
  - `src/components/ProgressBar.tsx` (~80 lines) — unified bar, props: elapsed/total/variant/colorScheme/shimmer
  - `src/components/ProgressBar.css` (~60 lines) — bar styles, 3 color schemes, pulse keyframe, shimmer
  - `src/components/ProgressBar.test.tsx` (~80 lines) — overlay/inline variants, div-by-zero guard, color classes
  - `src/hooks/useTimerAnimations.ts` (~40 lines) — fade on state change, shimmer on reset crossing 1.0
  - `src/hooks/useTimerAnimations.test.ts` (~50 lines) — fade timing, shimmer trigger, no-stack
- Depends on: nothing
- Conflict zone: NONE (all new files)

**E001-T02: WidgetProps + useWidgetData Extensions**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src/types.ts` — add `elapsed: number`, `total: number`, `colorScheme: "sitting"|"standing"|"gray"` to WidgetProps
  - `src/hooks/useWidgetData.ts` — compute elapsed/total/colorScheme per state mapping:
    - Sitting → currentSessionSecs / limitSecs / "sitting"
    - Standing/Walking → breakSecs / breakResetThreshold / "standing"
    - Away → breakSecs / breakResetThreshold / "gray"
- Depends on: nothing
- Conflict zone: types.ts, useWidgetData.ts (NO other task touches these in Wave 1)

### Wave 1b — sequential (1 agent, after 1a merges)

**E001-T03: Timer UX Integration + Cleanup**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src/widgets/one-bar/OneBarTimer.tsx` — use elapsed/total from props (not limitRemaining), remove duration from state label, add tooltip, apply useTimerAnimations
  - `src/widgets/OneBarWidget.tsx` — import new ProgressBar (inline variant) replacing old one-bar__progress divs
  - `src/App.tsx` — replace ScreenProgressBar with ProgressBar(variant="overlay"), remove `state==="Sitting"` gate, use elapsed/total/colorScheme from widgetProps
  - `src/widgets/one-bar/one-bar.css` — add `.one-bar--away` desaturation, remove `.one-bar__progress*` classes, remove `.one-bar__session-duration`
  - Update existing tests (~6 breaking): OneBarWidget.test.tsx, OneBarCoach.test.tsx, temperature.test.ts (test factories need new fields)
- Deletes:
  - `src/components/ScreenProgressBar.tsx`
- Depends on: E001-T01 (ProgressBar component), E001-T02 (new WidgetProps fields)
- Verify: `pnpm test:unit` passes, app renders correctly with `pnpm tauri:dev:demo`

---

## Wave 2: Rust MetricEngine Backend

### Wave 2a — solo (1 agent, foundational)

**E001-T04: SessionManager Extensions**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src-tauri/src/session_types.rs` — add fields:
    - `continuous_computer_secs: i64`
    - `longest_computer_session_secs: i64`
    - `away_bout_secs: i64`
    - `first_reading_at: Option<DateTime<Utc>>`
    - `last_tick_ts: Option<DateTime<Utc>>`
  - `src-tauri/src/session_reading.rs` — tick continuous_computer_secs (when not Away), track away_bout_secs, reset continuous after 5+ min Away, increment position_changes on 5+ min Away→Sitting
  - `src-tauri/src/session_breaks.rs` — extend position_changes logic for Away bouts
  - `src-tauri/src/session_daily.rs` — reset new fields on daily reset
  - `src-tauri/src/config.rs` — add KPI threshold fields (8 new fields with defaults)
  - Update SessionStateDto in session_types.rs — add continuous_computer_secs, longest_computer_session_secs to snapshot()
  - Add/update tests in session_tests_*.rs
- Depends on: nothing (pure Rust, no frontend)
- Verify: `cargo test` passes

### Wave 2b — parallel (2 agents, after 2a merges)

**E001-T05: HourlyBreakTracker + GapHandler**
- Status: [ ]
- Agent: worktree
- Creates (NEW files):
  - `src-tauri/src/hourly_break_tracker.rs` (~120 lines):
    - `HourlyBreakTracker` struct — tracks per-clock-hour Away minutes
    - `record_away(secs)` — add Away seconds to current hour
    - `check_hour(hour) -> bool` — was there 5+ continuous min Away?
    - `hours_with_break() -> u8`, `hours_worked() -> u8`
    - `serialize() -> String` / `deserialize(s) -> Self` — for tauri-plugin-store
    - Unit tests: record, check, serialize roundtrip, hour boundary, daily reset
  - `src-tauri/src/gap_handler.rs` (~80 lines):
    - `detect_gap(last_tick_ts, last_state) -> GapResult`
    - GapResult: Continue(secs) | ShortBreak(secs) | LongBreak(secs)
    - Apply gap to SessionState
    - Log RESTART event to event_logger
    - Unit tests: <5m, 5-10m, >10m, missing last_tick
- Modifies:
  - `src-tauri/src/lib.rs` — register new modules
- Depends on: E001-T04 (new SessionState fields)
- Conflict zone: lib.rs (only mod declarations — easy merge)

**E001-T06: MetricEngine + 4 Metrics**
- Status: [ ]
- Agent: worktree
- Creates (NEW files):
  - `src-tauri/src/metrics/mod.rs` (~50 lines) — Metric trait, MetricEngine, MetricResult, MetricLevel, threshold_level() utility
  - `src-tauri/src/metrics/standing_pct.rs` (~40 lines) — StandingPercentMetric impl
  - `src-tauri/src/metrics/position_rate.rs` (~45 lines) — PositionChangeRateMetric impl
  - `src-tauri/src/metrics/hourly_breaks.rs` (~40 lines) — HourlyBreakCoverageMetric impl
  - `src-tauri/src/metrics/longest_session.rs` (~35 lines) — LongestSessionMetric impl
  - `src-tauri/src/metrics/tests.rs` (~120 lines) — unit tests for all 4 metrics + engine
- Modifies:
  - `src-tauri/src/lib.rs` — register metrics module
- Depends on: E001-T04 (SessionState fields to compute from)
- Conflict zone: lib.rs (mod declaration only — easy merge with T05)

### Wave 2c — sequential (1 agent, after 2b merges)

**E001-T07: Rust IPC + Persistence Wiring**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src-tauri/src/commands.rs` — new `get_dashboard_state()` IPC returning SessionStateDto + Vec<MetricSnapshot>
  - `src-tauri/src/snapshot_logger.rs` — add metric values to JSON snapshots
  - `src-tauri/src/tray_controller.rs` — update tray tooltip format with KPI summary
  - Startup code — call gap_handler on launch, restore HourlyBreakTracker from store
  - Periodic — persist HourlyBreakTracker + last_tick_ts to store every tick
- Depends on: E001-T04, E001-T05, E001-T06 (all Rust components ready)
- Verify: `cargo test` passes, `pnpm tauri:dev` shows metrics in Rust logs

---

## Wave 3: KPI Frontend

### Wave 3a — parallel (3 agents)

**E001-T08: KpiStrip Component**
- Status: [ ]
- Agent: worktree
- Creates (NEW files):
  - `src/widgets/one-bar/KpiStrip.tsx` (~80 lines) — renders Vec<MetricSnapshot> as colored dots + values, micro-animation on color change, tooltip with yesterday comparison
  - `src/widgets/one-bar/kpi-strip.css` (~60 lines) — compact layout, dot colors, pulse animation, tooltip styling
  - `src/widgets/one-bar/KpiStrip.test.tsx` (~60 lines) — render 4 metrics, empty array, colors, animation class, tooltip, personal best ★
- Depends on: nothing (uses MetricSnapshot interface — can mock)
- Conflict zone: NONE (all new files)

**E001-T09: Timeline Hour Markers**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src/widgets/one-bar/OneBarTimeline.tsx` — add "day start" marker line + hourly gridlines
  - `src/widgets/one-bar/one-bar.css` — timeline marker styles (`.one-bar__timeline-hour`, `.one-bar__timeline-start`)
- Depends on: nothing
- Conflict zone: one-bar.css (only adds new classes in timeline section — non-overlapping with other changes)

**E001-T10: Debug Tab Metrics**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src/components/settings/DebugSection.tsx` — add "Metrics" section showing raw MetricSnapshot values
- Depends on: nothing (uses useDesk which will have metrics after T07)
- Conflict zone: NONE (only file is DebugSection.tsx)

### Wave 3b — sequential (1 agent, after 3a + Wave 2 merges)

**E001-T11: Final Integration + Coach Removal**
- Status: [ ]
- Agent: worktree
- Modifies:
  - `src/types.ts` — add MetricSnapshot, MetricLevel, DashboardState interfaces
  - `src/hooks/useDesk.ts` — switch from get_session_state() to get_dashboard_state(), expose metrics array
  - `src/hooks/useWidgetData.ts` — pass metrics to WidgetProps
  - `src/widgets/OneBarWidget.tsx` — add KpiStrip at top, remove OneBarCoach import
  - `src/widgets/one-bar/one-bar.css` — KPI strip layout at top of widget
  - Personal bests: store/read best values in useDesk via get_dashboard_state
- Deletes:
  - `src/widgets/one-bar/OneBarCoach.tsx` — replaced by KPI strip
  - `src/widgets/one-bar/OneBarCoach.test.tsx` — tests for removed component
- Depends on: ALL previous tasks
- Verify: full `pnpm test:unit`, `cargo test`, visual check with `pnpm tauri:dev:demo`

---

## Execution Timeline

```
Wave 1a:  ┌─ T01 (ProgressBar + animations) ─────┐
          │                                        ├─ merge ─┐
          └─ T02 (types + useWidgetData) ─────────┘          │
Wave 1b:                                              T03 (integration) ─── merge
                                                                               │
Wave 2a:                                              T04 (session_*.rs) ──── merge
                                                                               │
Wave 2b:  ┌─ T05 (HourlyBreak + GapHandler) ─────┐                           │
          │                                        ├─ merge ─┐               │
          └─ T06 (MetricEngine + 4 metrics) ──────┘          │               │
Wave 2c:                                              T07 (IPC wiring) ────── merge
                                                                               │
Wave 3a:  ┌─ T08 (KpiStrip component) ───────────┐                           │
          ├─ T09 (Timeline hour markers) ─────────┤─ merge ─┐               │
          └─ T10 (Debug tab metrics) ─────────────┘          │               │
Wave 3b:                                              T11 (final integration) ─ DONE
```

**Max parallel agents: 3** (Waves 1a and 3a)
**Total tasks: 11**
**Critical path: T01/T02 → T03 → T04 → T05/T06 → T07 → T08/T09/T10 → T11**

## File Ownership Matrix

| File | T01 | T02 | T03 | T04 | T05 | T06 | T07 | T08 | T09 | T10 | T11 |
|------|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|-----|
| ProgressBar.tsx (NEW) | ✏️ | | | | | | | | | | |
| ProgressBar.css (NEW) | ✏️ | | | | | | | | | | |
| useTimerAnimations.ts (NEW) | ✏️ | | | | | | | | | | |
| types.ts | | ✏️ | | | | | | | | | ✏️ |
| useWidgetData.ts | | ✏️ | | | | | | | | | ✏️ |
| OneBarTimer.tsx | | | ✏️ | | | | | | | | |
| OneBarWidget.tsx | | | ✏️ | | | | | | | | ✏️ |
| App.tsx | | | ✏️ | | | | | | | | |
| one-bar.css | | | ✏️ | | | | | | ✏️* | | ✏️ |
| ScreenProgressBar.tsx | | | 🗑️ | | | | | | | | |
| session_types.rs | | | | ✏️ | | | | | | | |
| session_reading.rs | | | | ✏️ | | | | | | | |
| session_breaks.rs | | | | ✏️ | | | | | | | |
| session_daily.rs | | | | ✏️ | | | | | | | |
| config.rs | | | | ✏️ | | | | | | | |
| hourly_break_tracker.rs (NEW) | | | | | ✏️ | | | | | | |
| gap_handler.rs (NEW) | | | | | ✏️ | | | | | | |
| metrics/ (NEW) | | | | | | ✏️ | | | | | |
| lib.rs | | | | | ✏️ | ✏️ | | | | | |
| commands.rs | | | | | | | ✏️ | | | | |
| snapshot_logger.rs | | | | | | | ✏️ | | | | |
| tray_controller.rs | | | | | | | ✏️ | | | | |
| KpiStrip.tsx (NEW) | | | | | | | | ✏️ | | | |
| kpi-strip.css (NEW) | | | | | | | | ✏️ | | | |
| OneBarTimeline.tsx | | | | | | | | | ✏️ | | |
| DebugSection.tsx | | | | | | | | | | ✏️ | |
| useDesk.ts | | | | | | | | | | | ✏️ |
| OneBarCoach.tsx | | | | | | | | | | | 🗑️ |

✏️ = modify, 🗑️ = delete, ✏️* = adds new classes only (non-overlapping section)

**Conflict notes:**
- `lib.rs` touched by T05 + T06 (both add `mod` declarations) — trivial merge
- `one-bar.css` touched by T03 + T09 + T11 — different sections, but T09 is in Wave 3a (after T03 merged)
- `types.ts` + `useWidgetData.ts` touched by T02 + T11 — different waves, no conflict

## Verification Checkpoints

After each wave merge:
- **Wave 1:** `pnpm test:unit` + visual check (`pnpm tauri:dev:demo`) — timer shows elapsed/total, Away is gray
- **Wave 2:** `cargo test` — all 101 existing + ~30 new Rust tests pass
- **Wave 3:** `pnpm test:unit` + `cargo test` + visual check — KPI strip visible, colored, updates live
