# Unified Work Cycle & Activity Status — Design Spec

*Date: 2026-03-31 | ADR: [011](.arch/ADR/011-unified-sit-stand-walk-cycle.md)*

## Problem

The app tracks sitting time (40 min limit) but not continuous screen/computer time. Standing at the desk resets the sitting timer but the user is still staring at the monitor. Medical consensus: 60 min continuous screen time is the maximum before eye strain symptoms increase significantly ([research report](../../.plan/reports/screen-time-eye-health-research.md)).

Adding a separate screen time alert system would conflict with existing visual channels (tray yellow/red already means "sitting limit"). Two overlapping escalation systems on the same channels = UX confusion.

## Solution: Unified Three-Phase Work Cycle

One coherent flow instead of two separate alert systems.

### Phase 1: Sit (0–40 min)

Existing behavior, one message change:

- Overlay bar: neutral → yellow (30 min) → red (40 min)
- Tray dot: none → yellow → red → blink
- Toast at limit: **"Time for a change — stand up or step away from the screen for a few minutes"** (was: "Time to change position")
- The message now explicitly offers walking away as an equal alternative to standing

### Phase 2: Stand (computer_time keeps ticking)

Existing standing behavior (lap progress, break credit accumulation) plus:

- `continuous_computer_secs` continues incrementing (Sitting + Standing both count)
- When `continuous_computer_secs >= max_continuous_computer_secs` (default 60 min = 3600s):
  - **One gentle toast** with encouraging message from `screen_break_nudge` pool
  - Examples: "You've been at the screen a while — perfect moment to grab water", "Your eyes would love a 5-min break", "Quick stretch? 2 minutes is all it takes"
  - **No tray/overlay escalation** — those channels are reserved for sitting limit
  - Toast fires once per computer session (flag `screen_break_nudge_fired`, resets on computer_time reset)
- Standing UI otherwise unchanged (neutral lap bar, no dot)

### Phase 3: Walk Away (Away state, 5+ min)

- `continuous_computer_secs` resets to 0 (existing behavior, currently hardcoded at 300s)
- `sitting_seconds` gets break credit (existing behavior)
- Both timers reset — user returns to a fresh Phase 1
- This is the "premium" break: fixes posture AND eyes

### What We Don't Do

- No second escalation axis on tray/overlay
- No separate "Activity" KPI badge (screen time ≈ sitting time, redundant)
- No micro-break reminders (20-20-20 rule too granular for desk sensor)
- No complex multi-channel computer time escalation

## Feature 2: Activity Status in UI

### StateIndicator Enhancement

Current: `"Standing ↕ 72cm"`
New: `"Standing ↕ 72cm · Active"` or `"Standing ↕ 72cm · Idle 2m 30s"`

- Active (idle < 60s): subtle gray text "Active"
- Idle (idle >= 30s): yellow text "Idle Xm Ys" (timer increments live)
- Away: `"Away · Idle 5m 12s"` (no desk height — irrelevant when Away)

### Required Data

New fields in `SessionStateDto`:
- `idle_secs: i64` — current system idle time from `get_idle_seconds()`, refreshed every 1s in `accumulate_ongoing()`
- `away_bout_secs: i64` — already exists in SessionState, just missing from DTO

New field in `useDesk` hook return:
- `idleSecs: number`
- `awayBoutSecs: number`

### Feature Toggle

`AppConfig.show_activity_status: bool` (default: true)
Toggle in Settings → More. When off, StateIndicator shows current format (no Active/Idle).

## Ergonomic Profile Changes

New fields in `limits`:
```json
{
  "max_continuous_computer_secs": 3600,
  "computer_break_reset_secs": 300
}
```

- `max_continuous_computer_secs`: triggers screen break nudge toast (default 60 min)
- `computer_break_reset_secs`: replaces hardcoded 300s in `session_reading.rs` (default 5 min)

All existing profiles get these fields with defaults. No breaking change.

## Communication Profile Changes

New section:
```json
{
  "screen_break_nudge": {
    "enabled": true,
    "messages": [
      "You've been at the screen a while — perfect moment to grab water",
      "Your eyes would love a 5-minute break",
      "Quick stretch? 2 minutes is all it takes",
      "Step away for a moment — your focus will be sharper when you return",
      "Screen break time — look out the window for a minute"
    ]
  }
}
```

Message is chosen randomly from the pool. One toast per computer session.

## AppConfig Changes

```json
{
  "enable_computer_time_tracking": true,
  "show_activity_status": true
}
```

Both toggles in Settings → More tab.

## State Machine Changes

### session_reading.rs — accumulate_ongoing()

- Add `self.state.idle_secs = get_idle_seconds()` (refresh every tick)
- Replace hardcoded `300` with `self.state.computer_break_reset_secs`

### session_types.rs

- Add `idle_secs: i64` to `SessionState`
- Add `computer_break_reset_secs: i64` to `SessionState` (loaded from ergonomic profile)
- Add `max_continuous_computer_secs: i64` to `SessionState` (loaded from ergonomic profile)

### SessionStateDto

- Add `idle_secs: i64`
- Add `away_bout_secs: i64`
- Add `continuous_computer_secs: i64` (already in DTO but not used by frontend)

### communication_policy.rs

- New flag: `screen_break_nudge_fired: bool` (resets when `continuous_computer_secs` resets)
- In `evaluate()`: after existing sitting/standing logic, check if Standing + `continuous_computer_secs >= max_limit` + not fired → emit toast from nudge pool
- Nudge only fires when Standing (not Sitting — sitting has its own escalation)

### tray_controller.rs

- Pass `continuous_computer_secs` to PolicyInput (or read from session snapshot)
- No change to `elapsed_secs` logic — it stays as sitting_seconds / break_seconds

## Sitting Limit Message Change

In all communication profiles, update the sitting limit toast:
- **Before**: `"Time to change position"`
- **After**: `"Time for a change — stand up or step away from the screen"`

## Test Strategy

### Unit Tests (Rust)

- `continuous_computer_secs` accumulates during Sitting AND Standing
- `continuous_computer_secs` resets after `computer_break_reset_secs` of Away
- `computer_break_reset_secs` is configurable (not hardcoded 300)
- Screen break nudge fires once when standing + computer_time >= limit
- Screen break nudge does NOT fire when sitting (sitting has own escalation)
- Screen break nudge flag resets when computer_time resets
- `idle_secs` field populated in SessionState
- `away_bout_secs` appears in SessionStateDto

### Unit Tests (TypeScript)

- StateIndicator shows "Active" when idle < 60s
- StateIndicator shows "Idle Xm Ys" when idle >= 30s
- StateIndicator hides activity when `show_activity_status = false`
- Away state shows "Away · Idle Xm Ys" without desk height
- Settings toggle for computer time tracking renders and persists

### Integration Tests

- Full cycle: sit 40 min → standing toast says "stand or step away" → stand 20 min → computer nudge toast fires → away 5 min → computer_time resets

## Files to Modify

### Rust (src-tauri/src/)
- `session_types.rs` — new fields in SessionState + SessionStateDto
- `session_reading.rs` — idle_secs refresh, configurable reset threshold
- `session_breaks.rs` — no change (break credit logic unchanged)
- `communication_policy.rs` — screen break nudge logic + flag
- `communication_policy_helpers.rs` — nudge message selection
- `tray_controller.rs` — pass computer_secs to policy input
- `ergonomic_profile.rs` — new limit fields
- `communication_profile.rs` — new screen_break_nudge section

### Profiles (src-tauri/profiles/)
- `ergonomic/*.json` — add max_continuous_computer_secs, computer_break_reset_secs
- `communication/*.json` — add screen_break_nudge section, update sitting toast message

### Frontend (src/)
- `hooks/useDesk.ts` — expose idleSecs, awayBoutSecs
- `components/StateIndicator.tsx` — add Active/Idle display
- `components/SettingsPanel.tsx` (or More tab) — add two toggles
- `types/` — update TypeScript interfaces for new DTO fields

### Config
- `AppConfig` type — add enable_computer_time_tracking, show_activity_status
