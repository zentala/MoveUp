---
id: E005-T07
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T028
---
# T028 — Points system (in-memory, daily score)

**Status:** open
**Priority:** P2
**Depends on:** T022 (standing progress bar), T027 (split session.rs)
**Branch:** feat/T028-points-system

---

## Goal

Track a daily posture score. The score is the motivational "why" behind the gold bar.
In-memory only (resets at midnight — same as other daily counters).

## Scoring Rules

```
+1.0  per minute standing
+5.0  per complete standing session (standing_target_mins reached)
-0.5  per minute sitting
 0    walking / away (neutral)
```

## Key Decisions

- Bonus +5 per LAP (not per session) — `lap_bonus_awarded_for_lap` tracks last lap with bonus
- `standing_session_secs` resets when user sits (not same as `standing_seconds`)
- Score accumulates in `on_reading()` context (serial thread) — NOT in `update_overlay_progress()`
- Adds `standing_session_secs: i64` to `SessionStateDto` — T022 needs it from snapshots

---

## Acceptance Criteria

- [ ] Score accumulates per second during sitting/standing
- [ ] +5 bonus awarded when standing_target_mins reached per session
- [ ] No double-award for same session
- [ ] Score resets at midnight
- [ ] Tray tooltip shows score
- [ ] `cargo test` + `pnpm test:unit` pass
