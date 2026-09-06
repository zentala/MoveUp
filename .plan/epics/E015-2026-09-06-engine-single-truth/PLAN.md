# E015 — One truth for the sitting counter

Status: planned (2026-09-06). Version bump on start: 0.6.0.
Source review: [`../../reports/_review-2026-09-06/engine.md`](../../reports/_review-2026-09-06/engine.md).
Board deck (Polish): [`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

The engine applies proportional break credit correctly (ADR 008), but the
popup timer and progress bar read `current_session_secs`, a second counter
that is zeroed on every return to sitting. The user sees a reset that the
engine never made. This epic makes one credited counter the only number any
UI reads, deletes the second counter, fixes the test that enshrines the
reset, closes two adjacent seams (PostureBalance mixing raw and credited
counters; `break_credit` missing on `SessionRow`), and brings ADR 008 and
UX-FLOW back in line with the code. 21 points, one wave of 3 parallel
tasks plus one sequential closing task.

## Problem

- `session_types.rs:112-114` documents `current_session_secs` as "resets on
  Sitting entry, after break credit. Use this for the session progress
  timer in the UI." That comment contradicts ADR 008.
- `useDesk.ts:106,170` and `useRemoteDesk.ts:74,96` store it under the name
  `sittingSeconds`; `OneBarTimer.tsx:28,59-68` renders it as the session
  timer, while `temperature.ts:29-32` colours the same widget from the
  credited field.
- `session_tests_timers_live.rs:160-164` asserts the reset as correct.
- History (`plans-and-history.md` §2): the raw/credited split was
  introduced in `72bb935` and rewritten nine times in March 2026, never
  with a rule for who reads which field.

## Decisions and ADRs

- ADR 008 (proportional credit) — applied; this epic makes the UI obey it.
  Revised in T04 if D1 changes the default multiplier.
- ADR 009 (day break credit) — unchanged.
- ADR 011 (unified cycle) — unchanged.
- `.plan/decisions.jsonl` — none exists; T04 creates it with D1 and D2.
- Decided by Paweł 2026-09-06: **D1** default multiplier 3.0 in `standard`;
  **D2** delete `current_session_secs`, add `secs_since_last_break` for the
  Debug tab only; **D4** E015 before the public release. Recorded in
  HANDOFF.md and, by T04, in `.plan/decisions.jsonl`.

On tests: the owner's criticism stands. Today's suite asserts what its
author believed, reset included. This epic deletes or inverts those tests
and adds tests on the user-visible path (the DTO the UI reads), not on
helpers. New Rust tests carry the `e015_` prefix so AO can run exactly them.

## Alternatives

| | A. Minimum | B. Target architecture | C. Full engine rewrite |
|---|---|---|---|
| Summary | Repoint the two hooks and the widget at the credited field; fix the test | A + delete the second counter, fix PostureBalance and SessionRow, sync docs | New `ErgoEngine::step()` with injected clock, one snapshot type, signals |
| Effort | S (5) | M (21) | XL (52+) |
| Risk | L | M | H |
| Pros | Ships in an hour; answers the complaint | Closes the seam that caused nine rewrites; no fourth "elapsed" can appear | Cleanest model |
| Cons | The seam stays; the next feature reads the wrong field again | Touches DTO, TS types, tests, docs | Re-creates the same bug class mid-migration; loses the 740-test net |
| Reuses | everything | `apply_break_credit`, `limitUsedSecs`, existing tests | little |

**Recommendation: B.** A is what every previous fix did, and each time the
seam reopened. C is E020, sequenced after release. B is the smallest change
that removes the second counter from the type system, so the compiler
enforces the rule.

## Scope

In: `session_types.rs`, `session_breaks.rs`, `session_live.rs`,
`session_reading.rs`, `session_tests_*.rs`, `db_sessions.rs` (+ migration),
`notification_service.rs` (PostureBalance), `src/types.ts`, `useDesk.ts`,
`useRemoteDesk.ts`, `remoteDesk.test-helpers.ts`, `OneBarTimer.tsx`,
`OneBarTimeline.tsx`, `useWidgetData.ts`, `DebugSection.tsx`,
`SessionProgress.floating.test.tsx` (dead component; delete with E018 or
update here), `.arch/ADR/008`, `.arch/UX-FLOW.md`, `CLAUDE.md` Session Logic.

Out: clock injection, persistence unification, config extraction, signals
(E020); frontend reducer merge (E018).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Rust: delete `current_session_secs` from `SessionState`, `SessionStateDto`, `StateChangedPayload`; expose one `limit_used_secs` (credited) and, only if D2 says keep, `secs_since_last_break`; fix/replace tests asserting the reset; add a real-path test: sit 30 min → stand 2 min → sit → DTO shows 30·60 − 120·m | 8 | ts-dev (Rust) | 1 |
| T02 | TS: hooks and widgets read the credited field; rename state honestly; drift test that `SessionStateDto` keys in `types.ts` match Rust (`serde` JSON fixture) | 5 | ts-dev | 1 |
| T03 | PostureBalance compares raw with raw (`sitting_seconds_total` vs `standing_seconds`); `break_credit` persisted on `SessionRow` with migration; Analyst reads it instead of guessing | 5 | ts-dev (Rust) | 1 |
| T04 | Docs: ADR 008 revision (multiplier per D1, "one counter" rule), UX-FLOW §credit table, CLAUDE.md Session Logic, `decisions.jsonl` with D1/D2; `.arch/ARCHITECTURE.md` session section | 2 | main | 2 |
| T05 | Verify: `cargo test`, `pnpm test:unit`, browser pass on popup: sit → stand 2 min → sit, timer shows credited value and matches bar colour | 1 | verify + browser | 2 |

Wave 1 = 18 points, under the 40-point ceiling. T01 and T02 share the DTO
contract; T01 lands first in the worktree, T02 rebases on it.

## Test strategy

- Unit (Rust): the reset assertion at `session_tests_timers_live.rs:160-164`
  is inverted to assert credit; new scenario test in
  `session_tests_scenarios.rs` runs the full `on_reading` path with
  backdated timestamps (existing pattern) and checks the DTO, not a helper.
  Fails today because the DTO field does not exist.
- Unit (TS): `useRemoteDesk` test feeds a payload with credited value and
  asserts the rendered timer; `OneBarTimer` test asserts number and colour
  come from the same field. Fails today.
- Drift: JSON fixture emitted by a Rust test (`serde_json::to_string` of a
  DTO) is read by a Vitest test that checks key set equality with
  `types.ts`. Fails today (no fixture).
- Integration: existing `tests/integration` needs a running app; not
  required for this epic. Browser check in T05 is the user-path evidence.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| credit-dto | test | `cargo test --lib -- credited_counter` | pass | `evidence/records/T01-credit-dto.json` |
| ts-drift | test | `pnpm test:unit -- dto-drift` | pass | `evidence/records/T02-ts-drift.json` |
| popup-visual | visual | browser agent, sit→stand 2 min→sit | timer ≠ 0, equals bar colour band | `evidence/records/T05-popup-visual.json` |

## Architecture impact

`.arch/ARCHITECTURE.md` session section: one counter, delete mention of the
UI timer field. ADR 008 revised (status stays accepted, revision note). No
new component, no new data flow. Sub-task in T04.

## Acceptance criteria

1. No field named `current_session_secs` exists in Rust or TS.
2. After a 2-minute stand, the popup timer shows `previous − 120·m`, not 0.
3. Number and colour of `OneBarTimer` derive from one field.
4. PostureBalance can fire for a user who takes breaks.
5. `SessionRow.break_credit` is populated for new sessions.
6. ADR 008, UX-FLOW, CLAUDE.md describe the same single model.
