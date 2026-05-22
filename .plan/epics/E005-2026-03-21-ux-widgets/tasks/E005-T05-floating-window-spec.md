---
id: E005-T05
epic: E005
status: completed
created: 2026-03-21
completed: 2026-03-21
original_id: T029
title: T029 — Floating Window: Spec, Expected Behavior & Tests
---
# T029 — Floating Window: Spec, Expected Behavior & Tests

**Status:** open
**Priority:** P1 (spec + test coverage before any UI bug fixes)
**Branch:** feat/T029-floating-window-spec
**Note:** This task is SPEC + TESTS only. Do not fix bugs here — document what's broken,
write tests that fail, then fix in a follow-up task.

---

## Why This Task Exists

Two confirmed bugs + suspected broad undertesting:
1. **Sitting timer shows wrong value** — user sat 5 min ago, UI shows 50
2. **Last standing duration never shown** — after Standing->Sitting transition

## Implementation Plan

1. Read and document current behavior for all 9 scenarios (A-I)
2. Write failing Rust unit tests (scenarios B, C, D, F)
3. Write failing TypeScript tests (scenario G)
4. Extend StateChangedPayload with `last_break_secs`, `last_sitting_secs`, `break_credit`
5. Document what remains for follow-up T030

---

## Acceptance Criteria

- [ ] Document actual vs expected for all 9 scenarios
- [ ] Failing Rust tests written for scenarios B, C, D, F
- [ ] Failing TypeScript test written for scenario G
- [ ] `StateChangedPayload` extended with transition fields
- [ ] No behavior changes (spec + tests only)
- [ ] Create follow-up task T030
