# Desk App — Sprint Orchestrator (UX Communication System)

You are the coordinating agent for the UX Communication System sprint.
**Do not implement code directly.** Manage agent waves.

---

## Project Context

| | |
|---|---|
| **Repo root** | `C:/code/zntl-tray` |
| **App dir** | `apps/desk/` |
| **Main branch** | `main` |
| **Stack** | Tauri 2 + React + TypeScript + Rust |
| **Package manager** | pnpm |
| **Pre-commit hook** | Rejects `.rs`/`.ts`/`.tsx` files > 250 lines — **applies to every agent** |

### Key files to read before starting
- `apps/desk/CLAUDE.md` — project conventions
- `apps/desk/TASKS.md` — priority board
- `apps/desk/.agent/vision/2026-03-20-standing-gamification.md` — gamification vision
- `apps/desk/.agent/vision/2026-03-20-ux-communication-vision.md` — UX vision

---

## Architecture — Key Decisions (READ BEFORE EACH WAVE)

### 1. Rust owns all state
- `SessionManager` (Rust) is the single source of truth for all counters
- Frontend receives data ONLY via Tauri commands or events (`desk:state-changed`, `desk:distance`)
- `AppState` in `commands.rs` contains: `session`, `overlay`, `alert_manager`, `alert_popup`

### 2. Config persistence via tauri-plugin-store
- `AppConfig` loaded from store in `config.rs` via `AppConfig::load(store)`
- All fields have `#[serde(default)]` — safe for missing keys
- **After T027:** `standing_target_mins` has `#[serde(alias = "stand_limit_mins")]` — automatic migration

### 3. EventBus — no direct imports between modules
- Events: `desk:distance` (~every 1s), `desk:state-changed` (on state transition)
- `tray_controller.rs` subscribes to both and updates overlay + tray + alert manager

### 4. File size limit: 250 lines
- Pre-commit hook blocks commits if any file exceeds 250 lines
- `session.rs` is 1348 lines — T027 MUST be the first wave
- After T027: `session.rs` → `session_types.rs` + `session_manager.rs` + `session_tests.rs`

### 5. Overlay color + variant are independent
- `overlay.update(progress, color_rgb)` — sets color and progress
- `overlay.set_variant(v)` — sets animation (0=solid, 1=gradient, 2=pulsing)
- Call `update()` BEFORE `set_variant(2)` so pulsing uses the correct color
- Variant 2 is used by: alert Stage1 (red) AND lap flash (gold) — depends on last color set

### 6. Score accumulation pattern
- `accumulate_score_tick(config)` — new SessionManager method, called from `serial.rs` every ~1s
- Pattern: same as `check_notification_conditions(config)` — config as parameter
- Score lives in `SessionState.daily_score: f32`, reset at daily reset

### 7. Standing progress bar lap counting (T022 + T028 coordination)
- `standing_session_secs` = current continuous standing session (resets when user sits)
- `standing_seconds` = cumulative standing today (never resets mid-day)
- T022 uses `standing_session_secs` for bar progress and flash detection
- T022 uses `standing_seconds` only for the left lap indicator (total laps today)
- Both T022 and T028 need `standing_session_secs` in `SessionStateDto` — T028 adds it

---

## Wave Plan

```
Wave 1 (sequential):  T027 — BLOCKS EVERYTHING
Wave 2 (parallel):    T023 + T024 + T029 — P1 bugs, independent files
Wave 3 (parallel):    T022 + T028 + T025 — P2 features
Wave 4 (parallel):    T030 + T016 — fixes + optional icon
Wave 5a (sequential): T031 — widget architecture (BLOCKS T032, T033)
Wave 5b (parallel):   T032 + T033 — One Bar + Timeline Zen widgets
```

---

## Wave 1 — T027: Split oversized Rust files (P0, REQUIRED BEFORE ALL OTHERS)

**Why it blocks:** Pre-commit hook rejects files >250 lines. Multiple files exceed this:
- `session.rs` = 1348 lines (every task touches it)
- `db.rs` = 467 lines (T024 may touch it)
- `serial.rs` = 464 lines (T024 modifies it)
- `commands.rs` = 264 lines (T029 modifies it)
- `overlay_tests.rs` = 281 lines

**CEO review (2026-03-21):** T027 scope expanded to split ALL oversized files, not just session.rs.

```bash
cd /c/code/zntl-tray
git branch feat/T027-split-session-rs main
git worktree add .claude/worktrees/T027-split-session-rs feat/T027-split-session-rs
```

**Agent T027 works in:** `.claude/worktrees/T027-split-session-rs/`

**Spec:** `apps/desk/.claude/tasks/T027-split-session-rs.md`

**What it does:**
1. Splits `session.rs` (1348L) into:
   - `session_types.rs` — structs, enums, DTOs (~80L)
   - `session_manager.rs` — SessionManager + impl (~200L)
   - `session_tests.rs` — all `#[cfg(test)]` content (~800L)
   - `session.rs` → thin re-export or deleted
2. Splits `serial.rs` (464L) into:
   - `serial.rs` — main reader loop (~200L)
   - `serial_notifications.rs` — notification check loop (~150L)
   - `serial_parser.rs` — parse_distance + device detection (~80L)
3. Splits `db.rs` (467L) into:
   - `db.rs` — connection + schema (~150L)
   - `db_sessions.rs` — session CRUD (~150L)
   - `db_summary.rs` — get_today_summary + queries (~150L)
4. Splits `commands.rs` (264L) into:
   - `commands_session.rs` — session state + calibration commands
   - `commands_config.rs` — settings get/save
   - `commands.rs` → thin re-export
5. Adds to `config.rs`:
   - `standing_target_mins: u32` with `#[serde(default=15, alias="stand_limit_mins")]`
   - `stand_max_mins: u32` with `#[serde(default=90)]`
   - `active_widget: String` with `#[serde(default="one-bar")]`
6. Adds to `SessionStateDto`:
   - `limit_used_secs: i64` — computed by Rust, includes break credit. Goes negative = overtime.
7. Updates all imports in: `lib.rs`, `serial.rs`, `commands.rs`, `tray_controller.rs`

**Verification before merge:**
```bash
cargo test  # MUST pass — this is a pure refactor
```

**Merge after Wave 1:**
```bash
cd /c/code/zntl-tray
git checkout main
git merge feat/T027-split-session-rs --no-ff -m "refactor(desk): split session.rs + add standing_target_mins config"
git worktree remove .claude/worktrees/T027-split-session-rs
git branch -d feat/T027-split-session-rs
```

---

## Wave 2 — T023 + T024 + T029 (P1 bugs, parallel)

Launch **all three agents in one message** after Wave 1 is merged.

### Agent T023 — Fix tooltip while standing

```bash
git branch feat/T023-tooltip-fix main
git worktree add .claude/worktrees/T023-tooltip-fix feat/T023-tooltip-fix
```

**Spec:** `apps/desk/.claude/tasks/T023-fix-tooltip-standing.md`

**Files:** `tray_controller.rs` + `tray.rs` only

**No conflict with T024** — T024 does not touch these files.

### Agent T024 — Notifications debug + fix

```bash
git branch feat/T024-notifications-fix main
git worktree add .claude/worktrees/T024-notifications-fix feat/T024-notifications-fix
```

**Spec:** `apps/desk/.claude/tasks/T024-notifications-debug-and-fix.md`

**Files:** `session_manager.rs` (thresholds), `serial.rs` (notification loop),
`commands.rs` (trigger command), `config.rs` (notification_backend field)

**Important:** `trigger_test_notification` WITHOUT `#[cfg(debug_assertions)]` — production command
with 60s rate limit via `AtomicI64`.

**No conflict with T023** — T023 does not touch session/serial/commands.

### Agent T029 — Floating Window Spec + Tests

```bash
git branch feat/T029-floating-window-spec main
git worktree add .claude/worktrees/T029-floating-window-spec feat/T029-floating-window-spec
```

**Spec:** `apps/desk/.claude/tasks/T029-floating-window-spec-and-tests.md`

**Files:**
- `session_types.rs` — add `last_break_secs`, `last_sitting_secs`, `break_credit` to `StateChangedPayload`
- `session_tests.rs` — new failing tests for sitting timer behavior (scenarios B, C, D, F)
- `src/components/SessionProgress.tsx` — read only (no changes)
- `src/hooks/useDesk.ts` — read only (no changes)
- `src/components/SessionProgress.test.tsx` — new TypeScript tests (scenario G will fail — field missing)

**Important:** SPEC + TESTS ONLY. No bug fixes. If a test reveals a bug, add a `// BUG:` comment
and let it fail. Root causes get fixed in T030 (separate task, created at the end of this one).

**No conflict with T023** (tray_controller/tray.rs only).
**No conflict with T024** (session_manager/serial/commands/config).
**Possible conflict with T028 (Wave 3):** both may add fields to `session_types.rs`.
Merge coordinator: take both sets of fields — they are additive, no overlap.

### Merge Wave 2
```bash
cd /c/code/zntl-tray
git checkout main
git merge feat/T023-tooltip-fix --no-ff -m "fix(desk): update tooltip every second while standing"
git merge feat/T024-notifications-fix --no-ff -m "fix(desk): add notification debug trigger + lower thresholds"
git merge feat/T029-floating-window-spec --no-ff -m "test(desk): floating window spec + failing tests"
git worktree remove .claude/worktrees/T023-tooltip-fix
git worktree remove .claude/worktrees/T024-notifications-fix
git worktree remove .claude/worktrees/T029-floating-window-spec
git branch -d feat/T023-tooltip-fix feat/T024-notifications-fix
```

---

## Wave 3 — T022 + T028 + T025 (P2, parallel)

Launch **all three agents in one message** after Wave 2 is merged.

### Agent T022 — Standing Progress Bar

```bash
git branch feat/T022-standing-bar main
git worktree add .claude/worktrees/T022-standing-bar feat/T022-standing-bar
```

**Spec:** `apps/desk/.claude/tasks/T022-standing-progress-bar.md`

**Files:**
- `colors.rs` — add `color_for_standing()`
- `overlay_renderer.rs` — new fields in `OverlayState`, new methods
- `overlay_opaque.rs` — render gold bar + tick_lap_flash
- `overlay_layered.rs` — same
- `tray_controller.rs` — wire standing mode in `update_overlay_progress()`

**Key architectural decisions for T022:**
- Lap flash state lives in `OverlayState` (NOT AppState) as `lap_flash_until: Option<Instant>`
- Add `last_flashed_lap: u32` to `OverlayState` — prevents re-flash when standing up after a completed lap
- Add `maybe_flash_lap(session_lap: u32)` method — idempotent, compares against `last_flashed_lap`
- `clear_standing()` resets `last_flashed_lap = 0` so each new session can earn its own flash
- Add `tick_lap_flash(now: Instant)` method — enables unit testing without Thread::sleep
- Background thread (`run_event_loop`) calls `tick_lap_flash(Instant::now())` each frame
- **Progress uses `standing_session_secs` (current session), NOT `standing_seconds` (total today)**
  - `standing_session_secs` is added to `SessionStateDto` by T028 — coordinate if merging before T028
  - Use `standing_seconds` only for the left lap indicator (total completed laps today)
- Color: `color_for_standing(lap_progress)` → `#DAA520` → `#FFD720`
- Lap indicator: 8px bright-gold segment on the left per completed lap

**No conflict with T028** (T028 = session_manager/serial/config) or T025 (frontend/Rust commands).

**Likely conflict with T023 (already on main after Wave 2):** Both T022 and T023 modify
`update_overlay_progress()` in `tray_controller.rs` — T023 extracted `update_tooltip()` and
restructured the early-return block; T022 inserts a Standing branch before that same early return.
`on_state_changed()` may also conflict (T023 fixes state display, T022 adds `clear_standing()` call).

**Merge conflict resolution for T022:**
- Keep T023's `update_tooltip()` extraction (already on main)
- Insert T022's Standing branch BEFORE the early return, AFTER the `update_tooltip()` call
- In `on_state_changed()`: keep T023's state display fix + add T022's `overlay.clear_standing()` call

### Agent T028 — Points System

```bash
git branch feat/T028-points main
git worktree add .claude/worktrees/T028-points feat/T028-points
```

**Spec:** `apps/desk/.claude/tasks/T028-points-system.md`

**Files:**
- `session_types.rs` — add `daily_score: f32`, `standing_session_secs: i64`,
  `lap_bonus_awarded_for_lap: u32` to `SessionState`; add `daily_score: f32` and
  `standing_session_secs: i64` to `SessionStateDto` (T022 also needs the latter)
- `session_manager.rs` — add `accumulate_score_tick(config: &AppConfig)`
- `config.rs` — add `pts_standing_per_min`, `pts_session_bonus`, `pts_sitting_per_min`
- `serial.rs` — call `sess.accumulate_score_tick(&config)` after `on_reading()`
- `tray_controller.rs` — add score to tooltip (format: `🏆 +38`)
- `src/App.tsx` or `src/components/TodayStats.tsx` — display score in floating window

**Key architectural decisions for T028:**
- Bonus +5 per LAP (not per session) — `lap_bonus_awarded_for_lap` tracks the last lap with a bonus
- `standing_session_secs` resets when user sits (not the same as `standing_seconds`)
- Score accumulates in `on_reading()` context (serial thread) — NOT in `update_overlay_progress()`
- `daily_score` shown in tray tooltip: `↕ 114.0 cm — Standing 8:32  🏆 +38`
- **Add `standing_session_secs: i64` to `SessionStateDto`** — T022 needs it from snapshots

**No conflict with T022** (overlay/colors/tray render) or T025 (welcome popup).

**Potential conflict with T023:** both modify `tray_controller.rs`.
- T023: extracts `update_tooltip()`, changes label format
- T028: adds score to label in `update_tooltip()`
- **Merge conflict resolution:** Take T028's tooltip format (contains score) + T023's `update_tooltip()` structure.

### Agent T025 — Welcome Popup

```bash
git branch feat/T025-welcome-popup main
git worktree add .claude/worktrees/T025-welcome-popup feat/T025-welcome-popup
```

**Spec:** `apps/desk/.claude/tasks/T025-welcome-popup.md`

**Files:**
- `src/components/WelcomePopup.tsx` — new React component
- `src/welcome.tsx` — new entry point
- `welcome.html` — new Vite HTML entry
- `vite.config.ts` — add `welcome` to rollup inputs
- `tauri.conf.json` — add window config for "welcome"
- `src-tauri/src/commands.rs` — add `dismiss_welcome()` + `show_welcome()` commands
- `src-tauri/src/config.rs` — add `show_welcome_on_startup: bool`
- `src-tauri/src/lib.rs` — check flag on startup, call `show_welcome_window()`

**Key architectural decisions for T025:**
- Uses `WebviewWindowBuilder` (NOT WinAPI) — full CSS/React, draggable via `data-tauri-drag-region`
- Button "Przetestuj powiadomienia 🔔" calls `trigger_test_notification` (T024 command, production)
- `show_welcome_on_startup: bool` with `#[serde(default = "bool_true")]` — default true
- Window: 480×420px, always-on-top, centered, `visible: false` in tauri.conf.json (shown by Rust)

**No conflict with T022 or T028** — T025 adds new frontend files + new commands, does not modify
existing command handlers.

### Merge Wave 3

Known conflict: T023 and T028 both modified `tray_controller.rs`. T023 was already merged in Wave 2,
so this is a merge conflict between T023's changes (now on main) and T028's new changes.

```bash
cd /c/code/zntl-tray
git checkout main

# Merge T022 first — no conflicts expected
git merge feat/T022-standing-bar --no-ff -m "feat(desk): standing progress bar + lap flash"

# Merge T028 — conflict expected in tray_controller.rs with T023's changes
git merge feat/T028-points --no-ff -m "feat(desk): points system + score in tooltip"
# If conflict in tray_controller.rs:
# → Keep T028's tooltip format (it includes score)
# → Keep T023's update_tooltip() function structure (clean extraction)

# Merge T025
git merge feat/T025-welcome-popup --no-ff -m "feat(desk): welcome popup on first launch"

# Cleanup
git worktree remove .claude/worktrees/T022-standing-bar
git worktree remove .claude/worktrees/T028-points
git worktree remove .claude/worktrees/T025-welcome-popup
git branch -d feat/T022-standing-bar feat/T028-points feat/T025-welcome-popup
```

---

## Wave 4 — T030 + T016 (parallel where possible)

### Agent T030 — Floating Window Fixes (P1, run regardless of T026)

```bash
git branch feat/T030-floating-window-fix main
git worktree add .claude/worktrees/T030-floating-window-fix feat/T030-floating-window-fix
```

**Spec:** `apps/desk/.claude/tasks/T030-floating-window-fix.md`

**Files:**
- `session_types.rs` — add `current_session_secs` to `SessionState` + `SessionStateDto`
- `session_manager.rs` — populate `current_session_secs`, fix `load_today_totals`
- `session_tests.rs` — fix/pass tests written by T029
- `src/hooks/useDesk.ts` — use `current_session_secs`, save `lastBreakSecs`, extend timer
- `src/components/SessionProgress.tsx` — use correct field
- `src/components/TransitionBanner.tsx` — NEW: 30s transition message
- `src/App.tsx` — wire TransitionBanner, Walking/Away state section

**Key decisions for T030:**
- `current_session_secs` = timer the user sees (resets on session start, after break credit)
- `sitting_seconds` = daily accumulator for TodayStats (unchanged semantics)
- `load_today_totals()` sets `sitting_seconds` from DB but `current_session_secs = 0`
- TransitionBanner shows for 30s then auto-clears (useEffect + setTimeout)
- Uses `last_break_secs` + `break_credit` from `StateChangedPayload` (added by T029)

**No conflict with T016** (tray icon — completely separate files).

### Agent T016 — Tray Icon Redesign (P2, optional — requires T026 assets)

T016 depends on PNG assets from T026 (app icon design).
T026 requires manual work (creating/finding a desk silhouette icon).

**If T026 assets are ready:**

```bash
git branch feat/T016-tray-icon main
git worktree add .claude/worktrees/T016-tray-icon feat/T016-tray-icon
```

**Spec:** `apps/desk/.claude/tasks/T016-tray-icon-redesign.md`

**If T026 assets are NOT ready:** Skip T016 in Wave 4. T016 is standalone and can run separately.

### Merge Wave 4

```bash
cd /c/code/zntl-tray
git checkout main
git merge feat/T030-floating-window-fix --no-ff -m "fix(desk): fix session timer + add transition banner"
git merge feat/T016-tray-icon --no-ff -m "feat(desk): tray icon white base + colored dot"  # if T016 ran
git worktree remove .claude/worktrees/T030-floating-window-fix
git worktree remove .claude/worktrees/T016-tray-icon  # if T016 ran
git branch -d feat/T030-floating-window-fix feat/T016-tray-icon
```

---

## Wave 5 — T031 → (T032 + T033) — Widget System

> **Product context:** App is a coach, not a tracker. Frequent position changes > standing duration.
> See `.agent/vision/2026-03-21-product-vision-coach-and-business.md`

```
Wave 5a (sequential):  T031 — widget architecture (BLOCKS T032, T033)
Wave 5b (parallel):    T032 + T033 — two widget implementations
```

### Architecture — Key Decisions for Wave 5

### 8. Widget system
- `WidgetProps` is the single interface between core and presentation
- Core computes `limitRemaining` (decreasing counter) — widgets don't compute session logic
- `active_widget: String` in AppConfig, persisted via tauri-plugin-store
- Widget registry is a static TS map, no dynamic loading
- Each widget is `FC<WidgetProps>` — pure presentation component
- `useWidgetData()` hook wraps `useDesk()` + `useTimer()` + computed derived values
- "Stop" button removed from UI (dev command, not user-facing)
- ScreenProgressBar (overlay) stays outside widget system — it's a separate concern

### 9. One Bar concept (T032)
- **One bar, one meaning:** progress bar = limit usage. Fills when sitting, drains when standing/away
- Big number = `limitRemaining`, NOT `sittingSeconds` or `breakSeconds`
- Temperature escalation: 7 CSS classes (calm/warm/hot/burning/standing/away/reset)
- Coach: ONE sentence, context-dependent. Coach says what to DO, not statistics
- Timeline: proportional blocks, hover → tooltip, ghost rhythm lines
- Away = legitimate break, same as standing (gray on timeline, drains the bar)
- Break ≥15 min (standing OR away) = full reset

### Agent T031 — Widget Architecture

```bash
git branch feat/T031-widget-architecture main
git worktree add .claude/worktrees/T031-widget-architecture feat/T031-widget-architecture
```

**Spec:** `apps/desk/.claude/tasks/T031-widget-architecture.md`

**Files:**
- `src/types.ts` — add WidgetProps, PreviousSession, WidgetRegistration
- `src/widgets/registry.ts` — NEW: widget map + resolver
- `src/hooks/useWidgetData.ts` — NEW: compute derived props
- `src/widgets/PlaceholderWidget.tsx` — NEW: dev verification widget
- `src/App.tsx` — refactor: replace hardcoded UI with widget system
- `src-tauri/src/config.rs` — add `active_widget: String`
- `src/components/SettingsPanel.tsx` — add widget picker dropdown

**Verification:**
```bash
cargo test
pnpm test:unit
```

**Merge T031:**
```bash
cd /c/code/zntl-tray
git checkout main
git merge feat/T031-widget-architecture --no-ff -m "refactor(desk): widget architecture for floating window"
git worktree remove .claude/worktrees/T031-widget-architecture
git branch -d feat/T031-widget-architecture
```

### Agent T032 — Widget "One Bar" (parallel with T033)

```bash
git branch feat/T032-widget-one-bar main
git worktree add .claude/worktrees/T032-widget-one-bar feat/T032-widget-one-bar
```

**Spec:** `apps/desk/.claude/tasks/T032-widget-one-bar.md`

**Files (all NEW):**
- `src/widgets/OneBarWidget.tsx`
- `src/widgets/one-bar/OneBarTimeline.tsx`
- `src/widgets/one-bar/OneBarTimer.tsx`
- `src/widgets/one-bar/OneBarCoach.tsx`
- `src/widgets/one-bar/temperature.ts`
- `src/widgets/one-bar/one-bar.css`
- `src/widgets/registry.ts` — register

**No conflict with T033** — T032 and T033 create different files.

### Agent T033 — Widget "Timeline Zen" (parallel with T032)

```bash
git branch feat/T033-widget-timeline-zen main
git worktree add .claude/worktrees/T033-widget-timeline-zen feat/T033-widget-timeline-zen
```

**Spec:** `apps/desk/.claude/tasks/T033-widget-timeline-zen.md`

**Files (all NEW):**
- `src/widgets/TimelineZenWidget.tsx`
- `src/widgets/timeline-zen/ZenTimeline.tsx`
- `src/widgets/timeline-zen/ZenStatus.tsx`
- `src/widgets/timeline-zen/timeline-zen.css`
- `src/widgets/registry.ts` — register

**Merge conflict with T032 in `registry.ts`:** Both add an entry. Take both — they're additive.

### Merge Wave 5

```bash
cd /c/code/zntl-tray
git checkout main

# T032 first (default widget)
git merge feat/T032-widget-one-bar --no-ff -m "feat(desk): One Bar widget — horizontal, temperature, coach"

# T033 — possible merge conflict in registry.ts (additive, take both entries)
git merge feat/T033-widget-timeline-zen --no-ff -m "feat(desk): Timeline Zen widget — minimalist, big timeline"

# Cleanup
git worktree remove .claude/worktrees/T032-widget-one-bar
git worktree remove .claude/worktrees/T033-widget-timeline-zen
git branch -d feat/T032-widget-one-bar feat/T033-widget-timeline-zen
```

---

## Agent message template

```
You are an agent implementing [TASK_ID] in the zntl-tray/apps/desk project.

Work EXCLUSIVELY in the directory:
  /c/code/zntl-tray/.claude/worktrees/[WORKTREE_NAME]/

Absolute path to your app dir:
  /c/code/zntl-tray/.claude/worktrees/[WORKTREE_NAME]/apps/desk/

Read BEFORE writing any code (in this order):
1. apps/desk/.claude/tasks/[TASK_FILE].md — FULL task spec
2. apps/desk/CLAUDE.md — conventions (commit style, linting, tests)
3. apps/desk/.claude/ORCHESTRATOR.md — architecture and key decisions
4. Every file you will modify — Read first, always

Key rules:
- Pre-commit hook rejects files > 250 lines — enforce this
- cargo test must pass before commit
- Commit only your files (git add specific files, NOT git add -A)
- Do NOT merge yourself — coordinator merges after wave completes
- Commit format: feat(desk): [description] or fix(desk): [description]

After finishing:
- cargo test — must pass
- pnpm test:unit — must pass (if you touched TypeScript)
- Commit with specific files
- Report: "T[NNN] ready to merge"
```

---

## Pre-Wave 1 Checklist

- [ ] You are on `main` branch and `git status` is clean
- [ ] `pnpm install` in `apps/desk/` is done
- [ ] `cargo check` in `apps/desk/src-tauri/` passes
- [ ] You have read `T027-split-session-rs.md` in full
- [ ] You understand T027 = pure refactor — zero behavior changes, only file splitting

---

## Dependency Graph

```
T027 (Wave 1)
  │
  ├──► T023 (Wave 2, parallel) ──────────────────────┐
  │                                                  │ (all Wave 2 merged before Wave 3)
  ├──► T024 (Wave 2, parallel)                        │
  │                                                  │
  └──► T029 (Wave 2, parallel) ──────────────────────┤
         │ (session_types.rs: adds StateChangedPayload fields)
         │ (session_tests.rs: writes failing tests → T030 created at end)
         │
         ▼ (T029 merged → T030 created as follow-up fix task)
       T025 (Wave 3, parallel)
       T022 (Wave 3, parallel) ◄── needs standing_session_secs from T028
       T028 (Wave 3, parallel) ────────────────► tray_controller merge conflict with T023
         │
         ▼
       T016 (Wave 4, after T026 assets)
       T030 (Wave 4, fix floating window bugs — depends on T029 + T027)
         │
         ▼
       T031 (Wave 5a, sequential — widget architecture)
         │
         ├──► T032 (Wave 5b, parallel) — One Bar widget
         └──► T033 (Wave 5b, parallel) — Timeline Zen widget
```
