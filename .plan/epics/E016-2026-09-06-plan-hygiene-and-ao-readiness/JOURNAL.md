# E016 — JOURNAL

## 2026-09-06 — AO run E016-20260906-0348

Ran E016 through AO end-to-end (`ao ready` → `plan` → `run` → `promote` →
`cleanup`), operator = main session, not a worker.

- Manifest: `node ao-handoff-manifest.mjs .../E016-2026-09-06-plan-hygiene-and-ao-readiness --run-id E016-20260906-0348 --out <scratchpad>/ao/E016-20260906-0348.json`.
  Wave plan: `W0: T01,T02,T04,T05 | W1: T03`.
- `run` completed clean: all 5 tasks `merged`, each own verification script
  passed (`node scripts/check-e016-t0N.mjs`), status `ready_for_promotion`.
- Before promote, ran all 5 check scripts together on the merged integration
  worktree as the epic's own gate (docs/config-only epic — `just check`
  against the full app is not the relevant gate here, per HANDOFF's explicit
  "no application code changes" scope). All 5 passed.
- **Intervention 1 — dirty_target on first `ao promote`.** Root cause:
  pre-existing uncommitted `coverage/**` diffs in the main checkout (present
  before this session started, unrelated to E016 — later untracked by
  T04 itself). Fix: `git stash push -u`, then `git stash pop` + `git restore`/
  `git clean -fd` scoped to `coverage/` and `test-performance-report/` only
  (kept the AO manifest files). This is exactly the documented `dirty_target`
  recovery in `skills/ao/SKILL.md` §3.
- **Intervention 2 — same `dirty_target` symptom recurred** after the
  coverage cleanup, caused by an operator mistake: I had written the AO
  manifest/ledger pointer files into `.plan/ao/` inside the repo tree itself,
  which is not gitignored here, so the untracked directory alone made
  `git status --porcelain` non-empty and AO's dirty-target check treated
  that as a real conflict. Fix: moved `.plan/ao/*` out of the repo into the
  session scratchpad; repo tree came back fully clean and `promote` then
  succeeded on the next attempt (commit `0995f42`).
- **Intervention 3 — `ao promote` without `--ledger` returned
  `status: not_found`** even though `run` used the same run id. Re-ran with
  `--ledger` pointed at `.dispatch/runs-<runId>.json` and it resolved fine.
  Backlog candidate for dispatch.internal: `promote`'s usage line lists
  `--ledger` as optional, but omitting it silently fails to find an
  otherwise-valid run instead of erroring or falling back to the default
  ledger path.
- **Intervention 4 — `ao cleanup --force` reported `cleanup: "retained"`
  for all 5 worker worktrees and the integration worktree**, i.e. it did not
  actually delete them (confirmed: directories still present on disk after
  `--force`). Fell back to the skill's documented escape hatch, `wt-remove`,
  which succeeded for all 6 worktrees. Two sub-issues hit along the way,
  both operator/environment, not AO's fault:
  - `wt-remove` is a PowerShell 7 script (`#requires -Version 7.0`); calling
    it via the bare `powershell` binary (Windows PowerShell 5.1) fails with
    a version-requirement error — must invoke via `pwsh`.
  - Backslash-escaped Windows paths built with bash variable interpolation
    (`"...\\$T-attempt-0"`) silently produced a literal `$T` instead of
    substituting the loop variable, under this session's shell. Forward-slash
    paths (`"$P"` built with `/`) interpolated correctly and PowerShell
    accepts them natively — used that instead.
  Backlog candidate for dispatch.internal: `ao cleanup --force` should either
  actually remove a fully-merged worker/integration worktree or report a
  concrete reason it left it retained (right now it reports success-shaped
  `"cleanup":"retained"` with no cause).
- Final state: all 6 AO-created git worktrees removed; `ao/worker/*` and
  `ao/integration/*` branches kept (harmless, already merged, `wt-remove`'s
  own default). Main checkout clean. All 5 `check-e016-t0N.mjs` scripts pass
  against `main` at `0995f42`.
