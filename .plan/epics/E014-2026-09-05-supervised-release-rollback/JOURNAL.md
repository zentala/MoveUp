# E014 — JOURNAL

## 2026-09-06 — writing the `## AO` block, waves 1-2 only

This epic had a `PLAN.md` and `HANDOFF.md` with full decisions (D1-D7) and
a wave/points breakdown, but no `## AO` block and no `tasks/` directory —
it predates the AO-dispatch convention fixed in later epics. Wrote both
before dispatching:

- **Version bump already done.** T09's original brief included "bump to
  0.6.0" — E015 already did that and tagged `v0.6.0` on 2026-09-06. Dropped
  that sub-item from T09's scope; it would have been a no-op edit.
- **`.plan/ARCH.md` → `.arch/ARCHITECTURE.md`.** PLAN.md predates E016's fix
  of the architecture-doc convention. Used the real path.
- **T01/T02 sequenced, not parallel**, against the wave table's own
  suggestion. Every epic run through AO since E018 hit the same collision:
  two tasks that both add a `mod` line to `lib.rs` cannot share a wave
  without a guaranteed write-set clash. Made T02 `depends_on: ["E014-T01"]`
  and chained T03/T04 the same way — five sequential tasks instead of two
  waves of two. Costs some wall-clock, saves a guaranteed retry.
- **Module names were not specified in PLAN.md** (only test-file names were:
  `release_store_tests.rs`, `health_probe_tests.rs`). Chose
  `release_store.rs`, `last_known_good.rs`, `health_probe.rs`,
  `candidate_list.rs` — one file per D5/D6 concept, each with its sibling
  `_tests.rs`, matching this repo's established test-file convention.
- **No `tasks/` directory existed.** `ao-handoff-manifest.mjs` requires one
  task-reference file per task id to materialize into each worker's
  worktree. Wrote five minimal ones (frontmatter + a pointer back to
  HANDOFF.md's own per-task description), matching the format already used
  by E019/E020's task files.

## 2026-09-06 — AO run, one verification-command fix

Run `E014-20260906-2012` (five sequential tasks). `E014-T09` came back
`needs_attention` with `verification_command_failed`, but `changedPaths`
showed no scope violation at all — the worker wrote exactly the two
declared ADRs plus `.arch/ARCHITECTURE.md`. Ran the verification command by
hand in the worker's worktree and it passed cleanly, which meant the
failure was in how the manifest's inline `node -e "..."` one-liner survived
the YAML → shell round trip on this machine, not in the worker's output.
Replaced the fragile inline command with a real script,
`scripts/check-e014-t09-docs.mjs` (existence + cross-reference checks,
same shape as E018/E019/E020's `check-eXXX-tYY-docs.mjs` scripts), wired it
into T09's `write_set`/`verification`, committed, and — having learned from
E020's mid-run HANDOFF drift — synced the fix directly into the
`ao/integration` branch before regenerating the manifest and resuming, so
no later task would see a stale copy and flag a false out-of-scope
violation. `ao resume` then completed all five tasks cleanly with zero
further intervention.

`ao promote` succeeded in one call (manifest kept outside the repo tree
throughout). Merge commit `d12412a`. Six worker worktrees plus
`integration` needed manual `wt-remove` (the standing `ao cleanup --force`
defect, filed from four prior epics this session).

## 2026-09-06 — post-promotion verification

`just check` on promoted `main` (`d12412a`): 620 Rust + 3 integration + 304
TypeScript tests, all green, exit 0. This epic's `PLAN.md` never had an
evidence-contract table (predates that convention), so wrote all 5 records
from scratch via `verify-evidence run --epic ... --task T0N ...` —
`verify-evidence status` then confirmed all 5 `current`.

## Not done this session

- Waves 3-4 (T05-T08, T10) — still blocked on two items in
  `pm3-mcp/.plan/BACKLOG.md` plus a third pre-existing PM3 item (`pm3d` has
  no ONLOGON Scheduled Task, owner E000-A4) that blocks T10 specifically.
  Not re-checked this session; HANDOFF.md's own instruction is not to work
  around this with a private watchdog.
