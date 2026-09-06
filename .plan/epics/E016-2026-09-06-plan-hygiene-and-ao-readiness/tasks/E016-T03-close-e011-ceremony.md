---
id: "E016-T03"
title: "Close E011 ceremony; mark E003-T07 and E013 superseded"
status: pending
priority: high
effort: small
dependencies: ["E016-T02"]
tags: [docs, plan-hygiene]
created_at: 2026-09-06
---

# Close E011's closing ceremony; supersession notes for E003-T07 and E013

## Objective

E011's five tasks are all `[x]` and merged, but its own
`.plan/epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md:44-51`
"Final integration" checklist still has five unchecked boxes, and
`.plan/HISTORY.md` does not exist at all despite 12+ closed epics sitting in
`.plan/DONE.md`. Two related loose threads need marking, not silent
dropping: `E003-T07` (GitHub Releases CI/CD, `.plan/BACKLOG.md:69`) was
folded into E013's scope but never marked superseded there, and E013 itself
is being split going forward (its planning stalled; per the 2026-09-06
architecture review's epic sequencing, its scope moves into two new epics).

**Depends on E016-T02** because both tasks edit `.plan/BACKLOG.md` — this
task must run after T02's merge lands so it edits the post-merge file, not a
version T02 will later overwrite.

## Tasks

- [ ] Create `.plan/HISTORY.md` (per `rules/plan-arch-structure.md`'s
      "Persistent knowledge" section: condensed epic narrative + links to
      each epic's `JOURNAL.md`/`DONE.md`/ADRs — not a copy of `DONE.md`'s
      task list). Write one entry each for E011 (Autostart Hardening,
      2026-05-07: self-heal autostart, EventLogger reliability fix,
      `--minimized` launch) and E012 (Analyst Dashboard, 2026-05-16: Catalog
      + Explorer window, range-query commands, 5 charts). Link each entry to
      its epic's `PLAN.md` and `DONE.md` section.
- [ ] Annotate E011's `ORCHESTRATOR.md` "Final integration" checklist: for
      each still-unchecked line, either check it if it is genuinely true
      today, or replace the checkbox with a one-line note explaining why it
      is moot (e.g. "version bump to 0.4.0 / tag v0.4.0 — moot, version
      progressed to 0.5.0 via E012 before this ceremony closed; a
      retroactive v0.4.0 tag would not reflect real history"). Do not check
      a box for something that did not happen.
- [ ] Triage E011's own `IMPROVEMENTS.md`: it currently has zero entries.
      Add one line recording that the triage happened and found nothing to
      promote to `.plan/IMPROVEMENTS.md` (an empty triage is still a
      completed triage — record it, do not leave it silently unaddressed).
- [ ] In `.plan/BACKLOG.md`, find the `E003-T07` line and append
      " — superseded by E013 (see E013/PLAN.md status note)." without
      deleting the original line's content.
- [ ] In `.plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md`,
      add a status note near the top (below the frontmatter, or update
      `status:` to `superseded` in the frontmatter with a body note
      explaining why status and content can disagree during a split):
      "Superseded 2026-09-06: this epic's planned waves 1-2 (release
      readiness: docs, license, CI, signing decision) move to E017; waves
      3-5 (PM3 supervision, rollback, moveup.internal health checks) move to
      E014. This file is retained for its acceptance-criteria research; do
      not resume implementation against it directly."
- [ ] Run `node scripts/check-e016-t03.mjs` and fix until it exits 0.

## Acceptance Criteria

- `node scripts/check-e016-t03.mjs` exits 0.
- `.plan/HISTORY.md` exists with E011 and E012 entries.
- E011's `ORCHESTRATOR.md` checklist has no unexplained open box.
- E011's `IMPROVEMENTS.md` records the triage happened.
- `.plan/BACKLOG.md`'s E003-T07 line says "superseded".
- E013's `PLAN.md` names both split targets, E017 and E014.

## Verify

```
node scripts/check-e016-t03.mjs
```
