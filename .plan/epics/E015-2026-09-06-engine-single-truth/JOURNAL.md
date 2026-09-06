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

## 2026-09-06 — second AO run: T01 merged, T02/T03 hit two more gaps

Run `E015-20260906-0425`. T01 merged clean this time. T02 and T03 both
landed in `needs_attention`:

- **T02 — another `write_set_out_of_scope_write`**: `src/widgets/OneBarWidget.test.tsx`
  is one directory above `src/widgets/one-bar/**`, so the original glob
  never matched it, even though it is exactly the test file that has to
  change when the widget's data field is renamed. Widened
  `src/widgets/one-bar/**` to `src/widgets/**` in both `write_set` and
  `claims`.
- **T03 — `dirty_worktree`, root cause is NOT a repo hook** (the AO skill's
  recovery table assumes a hook; this repo genuinely has none — confirmed
  again by absence of `.claude/settings.json`/`.husky/`). Real cause: the
  user-level git config has `core.autocrlf=true`, so a *fresh* `git
  worktree add` checkout converts LF blobs to CRLF on disk, while this
  repo's main checkout (checked out earlier, before autocrlf normalization
  applied) stayed LF — so T03's worker worktree showed `.plan/…/HANDOFF.md`,
  its own task file, and both `src-tauri/gen/schemas/*.json` as modified
  purely from line-ending churn (`git diff` showed 0 real lines changed;
  `git status --porcelain` still flagged them dirty). Fixed at the repo
  level: `git config --local core.autocrlf false` in the main checkout —
  verified with a scratch detached worktree (`git status --porcelain`
  empty after the config change, non-empty before). This is a config fix,
  not a `.gitattributes` commit, so it only protects worktrees created on
  this machine with this local config; a proper `.gitattributes` (`* text=auto`
  + `git add --renormalize .`) would fix it for every clone and is filed
  as a backlog item below instead of done inline mid-run.
- Cleaned up all 4 worktrees from this run (T01's already-merged work
  included — the dirty_worktree recovery path is "restart, not resume",
  and the fix here is a repo-wide git config change that a fresh worktree
  from a NEW run picks up automatically, so redoing T01 costs one Rust
  test run, not more design work) and started a third run,
  `E015-20260906-0508`.

Backlog (this repo, not filed elsewhere — it's MoveUp's own git hygiene,
not an AO defect): add `.gitattributes` with `* text=auto` and run
`git add --renormalize .` once, so line-ending behavior is enforced by
the repo instead of by a local `core.autocrlf` override that only this
machine has. (Importance: Low, Points: 2)

## 2026-09-06 — promoted, T05 browser check blocked (honest status)

AO run `E015-20260906-0449` (third attempt) promoted clean to `main` at
`7a8bbf3`: all 4 tasks merged, then full `cargo test --lib` (504 pass),
`pnpm test:unit` (255 pass), `pnpm typecheck` and `node scripts/check-e015-docs.mjs`
all green on the merged integration branch before promote. Version bumped
to 0.6.0 (`cdb4bf3`) and tagged `v0.6.0`. Evidence records written for
`E015-T01-credit-dto` and `E015-T02-ts-drift` (both `pass`, current at
`cdb4bf3`).

**`E015-T05`'s browser/visual check (`popup-visual` in PLAN.md's evidence
contract) did NOT complete** — dispatched the `browser` agent to watch a
sit→stand-2min→sit cycle render in the popup; it could not reach a live
rendered UI at all (two pre-existing dev-mode gaps, both filed to
`.plan/BACKLOG.md`: no Vite proxy for `/display`, and `OVERLAY_DATA=mock`
never drives the real session engine, only the disconnected native overlay
bar). It DID confirm, from the real `/display/api` endpoint, that the
backend now serves `limit_used_secs` (not the deleted field) — real
data-level evidence, but not the pixel-level "timer number matches bar
colour" check the acceptance criteria call for.

**No `popup-visual` evidence record exists — this is correctly `missing`,
not fabricated as `pass`.** Per `rules/evidence.md` and the "never claim it
works before you've seen it" rule, PLAN.md acceptance criterion 2 ("After a
2-minute stand, the popup timer shows previous − 120·m, not 0") is proven
at the engine level by the `e015_` Rust scenario test (exact sit-30min /
stand-2min / sit-again path, asserting the DTO value) and by the live field
name change, but is NOT visually confirmed in the running app. Reporting
this as an open gap rather than closing the epic as fully verified.
