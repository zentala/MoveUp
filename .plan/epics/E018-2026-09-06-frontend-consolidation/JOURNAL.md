# E018 — JOURNAL

## 2026-09-06 — AO run, three write_set widenings, one transient flake, promoted

Run `E018-20260906-1209` (wave 0: T01,T03,T04,T06,T07,T08 parallel; wave 1:
T02,T05,T09; wave 2: T10; wave 3: T11). Five of six wave-0 tasks merged
clean. `E018-T03` came back `needs_attention` with
`write_set_out_of_scope_write`.

**Widening 1 — T03.** The worker correctly deleted a dead component
(`SessionProgress`) and *both* of its test files, but the plan's write_set
only declared one of them (`SessionProgress.floating.test.tsx`) — it did not
know `SessionProgress.test.tsx` existed as a separate file. Verified via
`git diff` that both changes were pure deletions (0 insertions), confirming
the worker's action was correct and the plan's scope was just incomplete.
Widened `write_set`/`claims` in HANDOFF.md for T03, committed, regenerated
the manifest with the *same* `--run-id` (not a fresh one — the run's other
five wave-0 merges were already recorded against this run-id and re-running
them would have re-spent Claude usage for no reason), then `ao resume`.
Completed cleanly on attempt 1.

**Wave 1 — a transient flake plus a much bigger widening.** `E018-T05`
(`pnpm build` verification) failed with exit 1 shortly after `E018-T02`
started building in parallel — re-ran `pnpm build` by hand in T05's own
worktree immediately after and got exit 0 with no changes, so treated it as
build-cache contention between the two parallel workers rather than a real
regression (same category as the E017 "Windows app control policy" flake:
diagnosed by direct re-run, not assumed). `ao resume` picked it up and T05
merged clean on attempt 2 (T02 was still running at the time so the resume
call only advanced T05; T02 needed a real fix, below).

`E018-T02` (ts-rs codegen) came back with **23** `write_set_out_of_scope_write`
violations — an order of magnitude more than T03's one. Read every violated
path against the actual diff before touching anything: all 18 unique files
were legitimate fallout of wiring `#[derive(TS)]` codegen through the real
DTO surface — new `.arch/ADR/017-ts-rs-for-rust-ts-codegen.md`, an
`.arch/ARCHITECTURE.md` update, a `.plan/decisions.jsonl` entry, `Cargo.lock`
(new dependency), five more Rust modules that reference the annotated
structs (`db.rs`, `lib.rs`, `metrics/mod.rs`, `serial_parser.rs`,
`session_dto_fixture_tests.rs`), and `deskReducer`/`useDesk`/`OneBarTimeline`
files that consume the generated types. Confirmed no collision risk before
widening: `deskReducer.ts` was created by `E018-T01` (`depends_on:
["E018-T01"]`, already merged), so T02 editing it afterward is the intended
sequential dependency, not a same-wave race. Widened T02's write_set to the
full real footprint, committed, regenerated, resumed — merged on attempt 2
(one earlier resume in between only advanced T05, as noted above).

**Wave 2 — a third, smaller widening.** `E018-T10` (Recharts KPI donuts)
hit 3 violations: `pnpm-lock.yaml` (a companion to the already-declared
`package.json` for the new `recharts` dependency — plan declared the
manifest but forgot its lockfile), `src/analyst/explorer-day-kpis.test.ts`
(the natural test sibling of the already-declared `.ts` file), and
`CLAUDE.md` (a genuine, non-scope-creep documentation update — diffed it
before widening: it updates the Analyst-window feature list and ADR index
to mention the new donuts and ADR 016). Widened, committed, regenerated,
resumed — merged on attempt 1. Wave 3 (`E018-T11`) then merged clean with
no violations.

`ao promote` succeeded in one call (`dirty_target`-free — the manifest for
this run was kept in a scratch path outside the repo tree from the start,
per the lesson from E019's promote needing a stash/unstash cycle for the
same reason). Merge commit `7566de7`. Twelve worker worktrees plus
`integration` needed manual `wt-remove` (the same `ao cleanup --force`
defect already filed from E016/E017/E019) — the per-directory bash loop hit
the same `$variable`-inside-`pwsh -Command` interpolation failure seen
before, so removed them one Bash-tool call per worktree instead of looping.

## 2026-09-06 — post-promotion verification and Outside-AO items

`just check` on promoted `main` (`7566de7`): 534 Rust + 3 integration + 304
TypeScript tests, all green, exit 0. New ESLint warnings appeared (T04's
scope) but did not fail the gate — warnings, not errors, by the plan's own
"baseline downgrades" design (six rules pinned to `warn` because they flag
pre-existing code T04 could not touch; filed to `.plan/BACKLOG.md` by the
T04 worker itself with per-site file:line references).

**Coverage threshold (Outside AO, decision D4)**: T05 could not reach
80/80/75% within its budget — first enforced measurement was
84.96/82.81/73.07/72.04 (lines/statements/functions/branches). Per the
HANDOFF's own instruction, lowering a threshold instead of raising coverage
is Paweł's call to confirm, "surfaced as a normal PR/commit note, not a
question in chat" — the worker left a dated code comment in `vite.config.ts`
explaining the numbers and why; I additionally filed the raise-it-back-up
follow-up to `.plan/BACKLOG.md` since a code comment alone doesn't show up
in a backlog sweep.

**Browser pass on the Analyst UI (Outside AO)**: dispatched the `browser`
agent. Result: full PASS on all 4 checkpoints (date header + nav arrows,
sticky day-nav via `getComputedStyle` confirming `position: sticky`, 3
Recharts KPI donuts with arc + centre number, and a `MutationObserver`-
confirmed `kpi-donut-pulse` class toggling ~730ms after changing the
selected day) — console clean, zero JS errors. One honest substitution: the
pass ran against `pnpm dev` → `/#/mockup/analyst` (fixture data), not
`pnpm tauri:dev`'s live window as PLAN.md's evidence contract literally
specifies, because the live window needs the physical VL53L1X sensor this
session did not have attached. The mockup route renders the exact same
React components/CSS as the live window, so the layout/visual claims are
covered; only the Tauri-IPC data plumbing into that UI is unverified by this
pass. Recorded honestly in the evidence record's procedure text rather than
silently upgraded to "live-verified".

**Evidence records**: none existed after promotion (unlike a task whose
`ao` executor writes its own record, these 8 code/test-based checks plus the
2 manual-script checks needed `verify-evidence run` after the fact, same as
E017). `verify-evidence run -- npx ...` failed with `spawnSync npx ENOENT`
/ `EINVAL` on this Windows shell — worked once switched to
`pnpm exec vitest ...` instead of `npx vitest ...`. The one `class: visual`
check (`ui-polish-visual`) has no re-runnable command by design, so wrote
its record by hand using `verify-evidence`'s own `fingerprint.mjs`
(`sourceFingerprint`/`sha256`) imported directly, rather than fabricating a
fake passing command — `verify-evidence status` then confirmed all 12
records `current`.

## Not done this session

- E012's `ORCHESTRATOR.md` one-line pointer (per HANDOFF's "Done means") —
  deferred to the same closeout pass as `.plan/DONE.md`/`HISTORY.md`/`STATE.md`.
