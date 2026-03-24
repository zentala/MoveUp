---
id: E005-T06
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T022
---
# T022 — Standing Progress Bar (gold, 0->standing_target_mins)

**Status:** open
**Priority:** P2
**Depends on:** T027 (split session.rs + add standing_target_mins to config)
**Branch:** feat/T022-standing-progress-bar

---

## Goal

When the user stands up, show a gold progress bar at the top of the screen that fills from
0 -> `standing_target_mins` (default 15 min). When the bar reaches 100%, flash gold for 2s (lap
celebration), then start lap 2. Currently the bar is hidden while standing.

## Key Decisions

- Lap flash state lives in `OverlayState` (NOT AppState)
- `tick_lap_flash(now: Instant)` method enables unit testing without Thread::sleep
- `maybe_flash_lap(session_lap)` is idempotent (prevents re-flash on same lap)
- `clear_standing()` resets `last_flashed_lap = 0` so each new session can earn its own flash
- Progress uses `standing_session_secs` (current session), NOT `standing_seconds` (total today)
- Color: `color_for_standing(lap_progress)` -> goldenrod `#DAA520` -> bright gold `#FFD720`

---

## Acceptance Criteria

- [ ] When standing, top bar turns gold and fills over `standing_target_mins`
- [ ] At target reached: 2s gold pulse, then lap 2 starts
- [ ] Sitting -> bar immediately returns to red sitting behavior
- [ ] All ~16 tests pass
- [ ] `cargo test` + `pnpm test:unit` pass
