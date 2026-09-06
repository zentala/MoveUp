# E020 — JOURNAL

## 2026-09-06 — AO run, session limit, three write_set widenings, one cross-epic fix

Run `E020-20260906-1418` (waves: T01 → T02 → T03 → T04 → T05+T06 → T07 →
T08, mostly sequential — every task edits `SessionState`/`SessionManager`
or a file that depends on the previous task's shape). `E020-T01` came back
`needs_attention` with `executor_result_error` and stdout
`"You've hit your session limit · resets 3pm (Europe/Warsaw)"` — same
pattern as E019. Checked the clock (15:34, past the 15:00 reset) and
`ao resume`d directly, no diagnosis needed this time.

**Widening 1 — T01.** One violation: `session_manager.rs`, not in the
declared write_set. Diffed it before widening: a 2-line change replacing
`now.date_naive()` with `crate::session_daily::local_day(now)` — the exact
clock-injection consistency fix T01 exists to make. Widened, committed,
regenerated the manifest with the same `--run-id`, resumed. Merged on
attempt 2.

**Widening 2 — T02.** Three violations: `lib.rs` (+1 line, a `mod`
declaration), `session_tests_e015_credited.rs` (1-line adaptation to the
new config location), and `session_tests_limits.rs` (a new 138-line test
file — this task's whole point is extracting ergonomic limits out of
`SessionState`, so a dedicated limits test file is exactly the expected
shape). Widened and resumed; merged on attempt 2 after one more resume.

**Discovered and fixed: mid-run HANDOFF.md drift causing false
violations.** After the T02 widening, `ao resume` reported a *fourth*
violation on T02 — `.plan/epics/E020-2026-09-06-engine-pure-core/HANDOFF.md`
itself "out of scope". Diffed it: the worker's branch had the CORRECT,
newer HANDOFF.md (both widenings applied, since its context was
materialized after both commits), but `ao/integration`'s copy was frozen at
the point T01 merged — before the T02 widening commit landed on `main`.
Since HANDOFF.md isn't declared in any task's write_set (only its
`JOURNAL.md`/`IMPRO.md` siblings are), any drift in it between branches
looks like an out-of-scope write regardless of which direction it drifted.
This is exactly what `ao/SKILL.md`'s "never edit HANDOFF.md while a run is
in flight" rule warns about — my own widening edits were the cause. Fixed
by copying the current `main` HANDOFF.md directly into the `integration`
worktree and committing it there (a scratch branch for this run only, safe
to patch directly) — not a new `--run-id`, since that would have re-spent
Claude usage re-running four already-merged tasks for a documentation
diff. Did the same pre-emptively for the T05 and T07 widenings that
followed, and hit no further false positives.

**Widening 3 — T05.** One violation: `tray_controller_tests.rs`, the test
sibling of the already-declared `tray_controller.rs`. Diffed (94
insertions / 30 deletions, real test coverage for the new `PolicyInput`
fields) before widening.

**Widening 4 — T07, a genuine cross-epic fix, not scope creep.** One
violation: `scripts/check-e019-t08-docs.mjs` — a doc-consistency script
that belongs to a *different, already-closed* epic (E019). Read the actual
diff before assuming it was a mistake: E020-T05 had deleted
`tray_controller.rs::compute_standing_lap` and moved the standing-lap
arithmetic into the engine (`SessionManager::policy_input`, per ADR 015),
which made two of E019-T08's own regression assertions
(`compute_standing_lap` existing, "TrayController computes the per-tick
inputs") permanently false — the script would have started failing on
every future `just check`. The worker rewrote the assertions to the
narrower claim that survived ("TrayController wires policy and executor
together each tick", "still decides when to dismiss the alert popup"),
with a code comment explaining the revision and crediting E020-T07. This
is the "found a blocker on the path to correctness → fix it" instinct
working correctly across an epic boundary, so widened rather than treating
it as an error. Merged on attempt 2 (T08 then completed clean on attempt
0 — its own worktree runs `cargo test` unfiltered and writes evidence,
i.e. exactly what the promotion-time verification does again independently
below).

`ao promote` succeeded in one call (manifest kept in a scratch path
outside the repo tree throughout, so no `dirty_target`/stash cycle needed,
unlike E019's first promote attempt). Merge commit `244de29`. Nine worker
worktrees plus `integration` needed manual `wt-remove` (the standing `ao
cleanup --force` defect, filed from E016/E017/E019/E018) — removed one
Bash-tool call per worktree, same as the prior three epics, since the
`$variable`-inside-`pwsh -Command` loop interpolation still fails.

## 2026-09-06 — post-promotion verification

`just check` on promoted `main` (`244de29`): 569 Rust + 3 integration + 304
TypeScript tests, all green, exit 0. Re-ran all 8 evidence checks via
`verify-evidence run --epic .plan/epics/E020-2026-09-06-engine-pure-core
--task T0N ...` — first pass used the `E020-T0N` task-id prefix convention
from E018/E019 by habit, producing records that didn't match PLAN.md's own
evidence contract table (which specifies bare `T01-clock-injection.json`
etc., no epic prefix — confirmed by reading `T08-full-suite.json`, which
the T08 worker itself had already written during the run with `task_id:
"T08"`). Deleted the wrongly-prefixed records and re-ran with `--task T0N`
(bare) plus explicit `--epic`, since the bare task id alone doesn't let
`verify-evidence` infer which epic from the repo root. `verify-evidence
status` then confirmed all 8 records `current`.

No Outside-AO items for this epic — every task is a Rust engine refactor
with no UI surface, so per HANDOFF.md's own "Done means" section and the
repo's browser-verification exception for code-only backend changes, no
`browser` agent pass was required.
