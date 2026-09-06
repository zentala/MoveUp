---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 8
agent: main
wave: 1
parallel: []
depends-on: []
blocked-by: ""
---

# E016 — Plan hygiene and AO readiness

## TLDR

Four files claim to be "the backlog" and disagree with each other, `.plan/STATE.md`
still contradicts its own frontmatter in the body text, E011 is code-complete but
its closing ceremony was never finished, two build-artifact directories are
tracked in git, and the repo has none of the four files the Agent Orchestrator
(AO) needs before it can run a future epic unattended. This epic fixes all five
in one short, docs-and-config-only session — no application code changes — so
that `.plan/` can be trusted again and E017+ can be dispatched through AO.
8 points, one wave (one task has an internal dependency), agent `main` throughout.

## Problem

Five independent, verified defects, all sourced from the 2026-09-06 full
architecture review (`.plan/reports/2026-09-06-pelny-przeglad-architektury-i-release.md`
§6 E016) and its two backing sub-reports:

1. **`.plan/STATE.md` self-contradicts.** Its YAML frontmatter (fixed earlier
   today) already says `active_epic: none (E011 and E012 code complete...)`,
   but the markdown body below it still says *"E011 (autostart hardening) —
   active, scaffolded, T03 done. Wave 1 (T01+T02) and Wave 2-3 pending"* and
   lists four E011 tasks as `⏳` pending that are in fact all `[x]` done per
   `.plan/DONE.md`. This is the single most-read status file in the repo and
   it is wrong about the most recently closed epic
   (`.plan/reports/_review-2026-09-06/plans-and-history.md` §1 row 1, §3
   E011 row). It also carries an unreconciled E010 "human tasks remaining"
   count: the body says 3 (T02, T14, T18); root `BACKLOG.md:266-280` lists
   10 items under "Collect from User — blocking launch" (Stripe, Plausible,
   GSC, OG image, stats export, etc.) that are not on that list of 3
   (`plans-and-history.md` row 12).
2. **Four files claim to be "the backlog."** Root `BACKLOG.md` (339 lines,
   legacy, missing every 2026-09 finding), root `TASKS.md` (40 lines, already
   a redirect stub), root `ORCHESTRATOR.md` (3 lines, dead pointer to an
   archived 2026-03-21 sprint file), and `.plan/BACKLOG.md` (366 lines, the
   one actually maintained and linked from epics). `plans-and-history.md`
   §1 recommends `.plan/BACKLOG.md` as canonical and merging the unique
   content (chiefly the "Collect from User" checklist) before deleting the
   other three.
3. **E011's closing ceremony was never finished.** All five of its tasks are
   `[x]` and merged (`.plan/DONE.md`), but its own
   `ORCHESTRATOR.md:44-51` "Final integration" checklist still has five
   unchecked boxes (build gate, manual smoke, version bump, tag,
   `HISTORY.md` entry, IMPRO triage) — and `.plan/HISTORY.md` does not exist
   at all despite 12+ closed epics in `DONE.md`
   (`_review-2026-09-06/release.md` "Should-fix" list). Two loose threads
   from the same era also need marking, not silent dropping:
   `E003-T07` (GitHub Releases CI/CD) was folded into E013's scope but never
   marked superseded in its own backlog line (`.plan/BACKLOG.md:69`), and
   E013 itself stalled at "planning" with only an abandoned task branch
   (`plans-and-history.md` §3) — its scope is being split going forward
   (waves 1-2 into E017 release-readiness, waves 3-5 into E014 rollback
   supervision, per the review's epic sequencing in §6 of the parent report).
4. **Two build-artifact directories are tracked in git.** `coverage/` (test
   coverage HTML/JSON) and `test-performance-report/` (18 files total,
   `release.md` "Verbatim command results" → Tracked build artifacts) are
   committed despite `.gitignore` already excluding `dist/`. They drift on
   every test run and bloat every clone.
5. **The repo has none of AO's five preconditions.** `skills/ao/SKILL.md`
   §1 requires (among others) a `.giter.yaml` with `worktree.copy`/
   `worktree.guards`, and no repo hook writing into a worktree after a
   worker's commit. This repo has neither `.giter.yaml` nor a root
   `justfile` (confirmed: `find . -maxdepth 1 -iname ".giter.yaml"` and
   `-iname "justfile"` both empty), and no `.claude/settings.json` hooks or
   `.husky/` directory exist in this repo at all (confirmed by direct
   listing 2026-09-06), so precondition 4 (no post-commit tree writes) is
   already satisfied — nothing to guard, but this must be recorded, not
   assumed, because E017 is the first epic planned to run through `ao run`.

## Decisions and ADRs

`.plan/decisions.jsonl` does not exist yet in this repo. No existing ADR in
`.arch/ADR/` covers plan-file hygiene, backlog consolidation, or AO
onboarding — this is process/config work, not an architecture change, so no
new ADR is created by this epic (see "Architecture impact" below).

## Alternatives

**A — Minimum: fix the five defects in place, no tooling change.**
Effort: S. Risk: L.
- Pros: smallest diff; every fix is independently verifiable; no new
  concepts introduced.
- Cons: none identified — this is not a design problem, it is five
  independent cleanup facts already fully specified by the review.
- Reuses: existing `.plan/BACKLOG.md`, existing `.gitignore`, existing
  `rules/just.md` template.

**B — Target: same fixes, plus a script that re-checks backlog-file drift
and AO-precondition health on every future `done.`.**
Effort: M. Risk: L.
- Pros: prevents this exact rot (four backlog files, contradictory STATE.md)
  from recurring silently.
- Cons: scope creep for a hygiene epic — building a checker is itself a
  small feature with its own test strategy and maintenance cost; the review
  that found these problems took a dedicated sweep, and a cheap automated
  check could give false confidence between sweeps.
- Reuses: same as A, plus a new `scripts/check-plan-hygiene.mjs`.

**RECOMMENDATION: A.** The five defects are fully enumerated, none require a
design decision, and the epic's own purpose is to restore trust in `.plan/`
quickly so E017 can start — building a new checker script belongs in a
`.plan/BACKLOG.md` entry for later triage (added by T02/T03 below), not in
the critical path of the hygiene fix itself. Alternative B is not rejected on
merit, only on scope: nothing here eliminates it as a future idea.

## Scope in/out

**In scope:**
- Correct `.plan/STATE.md` body text (frontmatter is already correct).
- Merge root `BACKLOG.md` + `TASKS.md` unique content into `.plan/BACKLOG.md`
  verbatim; delete root `BACKLOG.md`, `TASKS.md`, `ORCHESTRATOR.md`; fix any
  `README.md`/`CONTRIBUTING.md` links that pointed at them.
- Close E011's ceremony on paper: `.plan/HISTORY.md` created with E011 and
  E012 entries, E011's own `IMPROVEMENTS.md` triaged (it currently has zero
  entries — confirmed empty, so triage is "nothing to promote," recorded as
  such), E011 `ORCHESTRATOR.md` "Final integration" checklist annotated with
  what already happened vs. what is retroactively moot (version bump/tag —
  version is already past 0.4.0 at 0.5.0; a retroactive `v0.4.0` tag is not
  useful, per `release.md` item 11 — recorded, not performed).
- Mark `E003-T07` superseded-by-E013(→E017) in `.plan/BACKLOG.md`, and add a
  status note to E013's `PLAN.md` recording the wave split (1-2 → E017,
  3-5 → E014).
- Untrack `coverage/` and `test-performance-report/`, add both to
  `.gitignore`.
- Add root `.giter.yaml` (worktree copy/guards) and root `justfile`
  (setup/dev/build/test/check/typecheck/lint/clean, wrapping existing
  `package.json` scripts per `rules/just.md`).

**Out of scope:**
- Any application code change (Rust or TypeScript source).
- Running `pnpm tauri:build`, retagging `v0.4.0`, or any other action E011's
  checklist calls for that is now moot — recorded as moot, not performed.
- Fixing `eslint` (dead in `pnpm lint` today) — `just lint` wraps it as-is;
  the fix is E018 scope per the parent report's epic sequencing.
- Building a plan-hygiene drift checker (Alternative B) — filed to backlog
  instead.
- Any decision about E013's actual remaining work content — only the
  supersession/split note is written here; E017 and E014 own the real scope.

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| E016-T01 | Fix `.plan/STATE.md` body (E011/E012 status, E010 human-task reconciliation) | 1 | main | 1 |
| E016-T02 | Merge root `BACKLOG.md`+`TASKS.md` into `.plan/BACKLOG.md`; delete root `BACKLOG.md`/`TASKS.md`/`ORCHESTRATOR.md`; fix README/CONTRIBUTING links | 2 | main | 1 |
| E016-T03 | Close E011 ceremony (HISTORY.md, IMPRO triage note, E003-T07 + E013 supersession notes) | 2 | main | 1 (depends on T02) |
| E016-T04 | Untrack `coverage/` and `test-performance-report/`, extend `.gitignore` | 1 | main | 1 |
| E016-T05 | AO readiness: `.giter.yaml` + root `justfile` | 2 | main | 1 |

Total: 8 points, one wave (T03 depends on T02 landing first because both
touch `.plan/BACKLOG.md`).

## Test strategy

This epic touches only documentation, plan files, and repo config — no
application code. Every task's proof is a `node -e` assertion against file
content or `git ls-files`/`git status` output, per `rules/evidence.md`'s
`class: manual` allowance for a check that states what human/mechanical
observation satisfies it, made mechanical here because every claim is a
string-presence or file-absence fact.

| Task | Kind | Assertion that FAILS today | File |
|---|---|---|---|
| T01 | doc | `scripts/check-e016-t01.mjs` fails today: body contains stale `"active, scaffolded, T03 done"` line, contradicts frontmatter | `scripts/check-e016-t01.mjs` |
| T02 | doc | `scripts/check-e016-t02.mjs` fails today: root `BACKLOG.md`/`TASKS.md`/`ORCHESTRATOR.md` still exist | `scripts/check-e016-t02.mjs` |
| T03 | doc | `scripts/check-e016-t03.mjs` fails today: canonical history file does not exist; E003-T07 line not marked superseded | `scripts/check-e016-t03.mjs` |
| T04 | doc | `scripts/check-e016-t04.mjs` fails today: `git ls-files` lists 18 files under the two tracked build-artifact directories | `scripts/check-e016-t04.mjs` |
| T05 | doc | `scripts/check-e016-t05.mjs` fails today: neither `.giter.yaml` nor `justfile` exist at repo root | `scripts/check-e016-t05.mjs` |

Each script exits 0 with a `PASS:` line naming what it checked, or exits 1
with a `FAIL:` line and one bullet per problem — no check can pass on zero
matches without saying so explicitly, per the "silence never means success"
rule.

## Evidence contract

| check_id | class | procedure | expected | record_path |
|---|---|---|---|---|
| state-md-consistent | test | `node scripts/check-e016-t01.mjs` | exit 0, `PASS:` line | `evidence/records/E016-T01-state-md-consistent.json` |
| backlog-consolidated | test | `node scripts/check-e016-t02.mjs` | exit 0, `PASS:` line | `evidence/records/E016-T02-backlog-consolidated.json` |
| e011-ceremony-closed | test | `node scripts/check-e016-t03.mjs` | exit 0, `PASS:` line | `evidence/records/E016-T03-ceremony-closed.json` |
| build-artifacts-untracked | test | `node scripts/check-e016-t04.mjs` | exit 0, `PASS:` line | `evidence/records/E016-T04-artifacts-untracked.json` |
| ao-preconditions-present | test | `node scripts/check-e016-t05.mjs` | exit 0, `PASS:` line | `evidence/records/E016-T05-ao-preconditions.json` |

## Architecture impact

None. No `.plan/ARCH.md` change, no new ADR — this is process/documentation
hygiene and repo tooling (justfile, `.giter.yaml`), not a system architecture
change.

## Acceptance criteria

- [ ] `.plan/STATE.md` frontmatter and body agree on E011/E012 status; the
      E010 human-task count discrepancy (3 vs. 10) is resolved with an
      explanatory note, not silently dropped.
- [ ] Root `BACKLOG.md`, `TASKS.md`, `ORCHESTRATOR.md` no longer exist;
      every entry they carried that was not already in `.plan/BACKLOG.md`
      is present there verbatim (no summarizing, no dropped items).
- [ ] `.plan/HISTORY.md` exists with entries for E011 and E012.
- [ ] E011's own `IMPROVEMENTS.md` triage is recorded as done (it has zero
      entries to promote).
- [ ] `E003-T07` in `.plan/BACKLOG.md` is marked superseded by E013→E017.
- [ ] E013's `PLAN.md` carries a status note: waves 1-2 → E017, waves 3-5 →
      E014.
- [ ] `git ls-files` shows zero files under `coverage/` or
      `test-performance-report/`; both are in `.gitignore`.
- [ ] Root `.giter.yaml` exists with `worktree.copy` and `worktree.guards`.
- [ ] Root `justfile` exists, `just --list` exits 0, and covers
      setup/dev/build/test/check/typecheck/lint/clean mapped to existing
      `package.json` scripts.
- [ ] `HANDOFF.md`'s `## AO` block validates against `skills/ao/SKILL.md`
      §1 preconditions 1-3 and 5 (precondition 4 is already satisfied: no
      repo hook writes to the tree post-commit, confirmed by absence of
      `.claude/settings.json` and `.husky/` in this repo).
