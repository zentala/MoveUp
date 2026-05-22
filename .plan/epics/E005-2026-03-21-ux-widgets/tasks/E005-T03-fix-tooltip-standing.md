---
id: E005-T03
epic: E005
status: completed
created: 2026-03-21
completed: 2026-03-21
original_id: T023
title: T023 — Fix tooltip while standing
---
# T023 — Fix tooltip while standing

**Status:** open
**Priority:** P1 (bug)
**Branch:** feat/T023-fix-tooltip-standing

---

## Problem Description

When user stands up, the tray tooltip freezes and shows wrong information:

1. **Timer frozen**: tooltip only updates on `desk:state-changed` events (state transitions). While standing, no event fires every second -> tooltip stays stuck at the moment of transition.
2. **Wrong counter**: tooltip always shows `sitting_secs` even when standing. So "Standing (2:03)" means "you had been sitting for 2:03 when you stood up" — not the standing duration.

User report: "shows 2:03 the whole time I'm standing, never changes."

---

## Fix

1. Update `on_state_changed()` — pick duration based on state
2. Update `update_overlay_progress()` — move tooltip update BEFORE the early return
3. Extract `update_tooltip()` helper, add `update_tray_tooltip()` to `tray.rs`

---

## Acceptance Criteria

- [ ] Tooltip updates every second while standing
- [ ] While standing: shows `↕ 114.0 cm — Standing 7:32` (incrementing)
- [ ] While sitting: shows `↕ 72.0 cm — Sitting 12:34` (unchanged behavior)
- [ ] `cargo test` passes
