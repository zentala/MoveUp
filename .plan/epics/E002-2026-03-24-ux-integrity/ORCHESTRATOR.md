# E002 — Orchestrator

## Wave 0: Context (no code changes, documentation only)
- [x] **E002-T01** — Update UX-FLOW.md: sync with E001, add target state machine, KPI definitions
  - Blocks: all other tasks (everyone needs to read the same truth)

## Wave 1: State Machine Fix (Rust backend)
- [x] **E002-T02** — Away state: inactive → Away regardless of desk height
  - Modify `session_reading.rs`: `!active → DeskState::Away`
  - Mark `Walking` as future (comment in enum, keep variant)
  - Away = new session start (not continuation of old sitting)
  - Suppress notifications during Away (no "time to stand" at empty desk)
  - Blocks: T03, T04, T05, T06
- [x] **E002-T03** — Away sessions to DB + live timeline
  - `handle_state_exit(Away)` → `CompletedSession(state="Away")`
  - Frontend: gray blocks in timeline
  - **Live current session in timeline** — show what's happening NOW as a growing block,
    not only after state transition. Current state = rightmost block, extends in real-time.
  - Depends on: T02
- [x] **E002-T04** — Away flow tests (~15 Rust tests)
  - Low+inactive → Away, High+inactive → Away
  - `away_bout_secs` accumulation, 5-min position_change
  - `continuous_computer_secs` reset after 5 min Away
  - Screen time KPI resets after 5 min Away
  - Notifications suppressed during Away
  - Away → Sitting = new session (current_session_secs = 0)
  - Depends on: T02

## Wave 2: UI Fixes (TypeScript + Rust)
- [x] **E002-T05** — KPI rename + icons on all badges
  - "Session" → "Screen time" (Rust: `longest_session.rs` label field)
  - Add line icons to ALL KPI badges:
    - 👁 eye → Screen time (protecting eyesight)
    - ↕ arrows → Position changes/h
    - ☕ cup or ⏸ pause → Hourly breaks
    - 🧍 person → Standing %
  - Use Unicode line icons (no image assets needed)
  - Add "Today" header label above KPI strip
  - Depends on: T02 (Away must work for screen time to reset)
- [x] **E002-T06** — Overlay/popup color unification
  - Define ONE color palette (Rust `colors.rs` as source of truth)
  - TypeScript `ProgressBar.tsx` uses same hex values
  - Same thresholds: <60% green, 60-85% yellow, ≥85% red
  - Standing: same gold gradient in both
  - Away: gray in both
  - Depends on: T01 (UX-FLOW defines the canonical colors)
- [x] **E002-T07** — Standing lap 2-layer visual
  - Overlay: completed lap as darker gold base, new lap fills on top
  - Popup ProgressBar: same visual (CSS gradient or layered div)
  - Depends on: T06
- [x] **E002-T08** — Popup layout redesign + self-descriptive UI
  - **New visual hierarchy (top to bottom):**
    1. Header: `● sitting @ desk (72 cm) ⚙` — state=bold+state-color, height=muted
    2. Timer: big number `25:00 / 40:00` + single progress bar
    3. KPI strip: "Today" header + 4 badges with icons
    4. Timeline: at bottom, with live block for current session
  - **Remove duplicate bar**: inline bar stays (under timer), overlay bar stays (top of screen).
    Remove the overlay ProgressBar from INSIDE App.tsx popup — it's redundant with the
    WinAPI overlay. One bar per context: OS-level (overlay_renderer) + popup-level (inline).
  - **State color consistency**: standing = gold EVERYWHERE (label, dot, bar, timeline, glow).
    Sitting red = EVERYWHERE. Away gray = EVERYWHERE.
  - **Tooltips on every element**: state dot, timer, bar, timeline blocks, KPI badges
  - Depends on: T05, T06, T07

## Wave 3: Notification Centralization (Rust backend)
- [x] **E002-T09** — NotificationService: central routing module
  - New file: `notification_service.rs`
  - Move 6 toast conditions from `serial_periodic.rs`
  - Connect `AlertManager` as notification source
  - Wire `notification_backend` config (toast / popup / both)
  - **Suppress ALL notifications when state = Away**
  - Depends on: T02 (Away affects inactivity notification logic)
- [x] **E002-T10** — NotificationService tests
  - Backend switch: toast vs popup vs both
  - Once-per-day gates work centrally
  - No notifications fire during Away
  - Depends on: T09

## Wave 4: DevEx & Docs
- [x] **E002-T11** — Process guard: restore auto-kill old desk.exe
  - cross-env doesn't support process guard
  - Option A: Node.js script (cross-platform)
  - Option B: PowerShell one-liner before tauri dev
  - Must work from `pnpm tauri:dev`
- [x] **E002-T12** — Regression tests for popup standing timer
  - TS test: Standing state → timer shows breakSecs/standLimitSecs
  - TS test: Sitting→Standing transition → bar turns gold
  - TS test: Away state → bar gray, timer shows 0
  - Partially done (test updated this session), extend coverage
- [x] **E002-T13** — Final UX-FLOW.md verification
  - Read running app, compare with UX-FLOW.md
  - Fix any remaining discrepancies
  - Depends on: all previous tasks

## Backlog (not this epic, dopisane do BACKLOG.md)
- Time range selector for stats (today / 7d / 30d) — needs DB history first
- Cross-platform activity detection (Linux/macOS)
- Custom popup redesign (WebviewWindow instead of WinAPI)

## Dependencies

```
T01 (UX-FLOW) ──→ T02 (Away state) ──→ T03 (Away DB + live timeline)
                                    ──→ T04 (Away tests)
                                    ──→ T05 (KPI rename + icons)
                                    ──→ T09 (Notifications)
               ──→ T06 (Colors) ──→ T07 (Lap visual)
                               ──→ T08 (UI audit)
T11 (Process guard) — independent
T12 (Regression tests) — independent
T13 (Final verification) — last
```

## Parallel opportunities

- Wave 1: T02 is sequential (core change)
- Wave 2: T05 + T06 can run in parallel after T02. T08 after both.
- Wave 3: T09 can start after T02
- Wave 4: T11, T12 can run anytime in parallel
