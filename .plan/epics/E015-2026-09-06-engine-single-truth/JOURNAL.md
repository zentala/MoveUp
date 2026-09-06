# E015 — JOURNAL

## 2026-09-06 — first AO run failed on write_set, HANDOFF amended

Run `E015-20260906-0408` (wave 0: T01+T03 parallel). Both `ts-dev` workers
finished and each task's own `cargo test -- e015_` passed, but AO's merge
gate refused both with `write_set_out_of_scope_write` before anything
touched the integration branch — nothing was lost, this is the gate working
as designed.

Files the original HANDOFF's `write_set`/`claims` missed (inspected each
diff before widening — every one is a genuine, necessary touch, not scope
creep):

- **T01** (removing `current_session_secs`): the field is constructed or
  read in more places than the HANDOFF's "Mental model" listed —
  `commands_catalog_sources.rs` (catalog field description),
  `metrics/tests.rs`, `session_daily.rs`, `session_manager.rs` (struct
  literals + the DTO builder itself), `ws_broadcaster.rs` (test fixture).
  Also a glob bug: the write_set had `session_tests_*.rs` but the actual
  file is `session_tests.rs` (no trailing `_`), so the glob never matched
  it even though it's exactly the kind of file this epic exists to touch.
- **T03**: `session_breaks.rs` — T03's own task text already says "PostureBalance
  uses `sitting_seconds_total` vs `standing_seconds` (both raw)", i.e. this file
  was always in scope for T03, just missing from its `claims`/`write_set`.
  `serial_periodic.rs` — needed to wire `insert_session` →
  `insert_session_with_credit`, otherwise the new `break_credit` column T03
  adds would never be populated (the "vertical slice needs a wiring task"
  trap the AO skill already warns about). `db_tests_break_credit.rs` — a new
  test file for the new column, never named in the original write_set.

**Real collision found, not just a listing gap**: T01 and T03 both write to
`session_breaks.rs` (T01 drops the dead field write, T03 fixes the
PostureBalance comparison) — genuinely disjoint line ranges, but AO's
same-wave claim policy doesn't check line ranges, and running them
concurrently against the same file is exactly what the AO skill's own
recovery table warns costs a re-run. Fixed by sequencing: `E015-T03` now
`depends_on: ["E015-T01"]` instead of running parallel to it. Wave shape is
now T01 alone, then T02+T03 parallel, then T04 — HANDOFF.md and its `## AO`
block updated in the same commit as this journal entry.

Nothing had merged when this failure was found (status was `needs_attention`
before any worker touched the integration branch), so no rollback was
needed — cleaned up all 3 worktrees from the failed run
(`E015-20260906-0408`) with `wt-remove` and restarted with a new run id.
