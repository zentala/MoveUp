---
id: "E016-T02"
title: "Consolidate four backlog files into .plan/BACKLOG.md"
status: pending
priority: high
effort: small
dependencies: []
tags: [docs, plan-hygiene]
created_at: 2026-09-06
---

# Consolidate root `BACKLOG.md` + `TASKS.md` into `.plan/BACKLOG.md`

## Objective

Four files currently claim to be "the backlog": root `BACKLOG.md` (339
lines), root `TASKS.md` (40 lines, already a redirect stub), root
`ORCHESTRATOR.md` (3 lines, dead pointer to an archived 2026-03-21 sprint
file), and `.plan/BACKLOG.md` (366 lines, actively maintained, the one
`rules/plan-arch-structure.md` names as canonical). Root `BACKLOG.md`
duplicates roughly 80% of `.plan/BACKLOG.md`'s section headers verbatim but
is missing every 2026-09 finding and carries one section not present
anywhere else: "🔴 Collect from User (zentala) — blocking launch" (10 human
action items: screenshots, sensor/desk photos, hero GIF, marketing videos,
usage stats export, OG image, Stripe account, Plausible account, Google
Search Console) plus a "Landing Page — Post-Launch" and "i18n" section.

**Per the global "nic nie przycinaj" rule: move every entry verbatim, do not
summarize, do not drop anything as "not worth it."** If a section already
exists near-verbatim in `.plan/BACKLOG.md`, skip it (it is not lost, it is
already there) — but anything unique to root `BACKLOG.md` must land in
`.plan/BACKLOG.md` byte-for-byte (copy the section, do not retype it).

## Tasks

- [ ] Diff root `BACKLOG.md` against `.plan/BACKLOG.md` section by section.
      Sections confirmed unique to root (verify no equivalent exists):
      "🔴 Collect from User (zentala) — blocking launch", "Landing Page —
      Post-Launch", "i18n — Landing Page Translations". Append each,
      verbatim, to `.plan/BACKLOG.md` under a clearly labeled heading (keep
      the original section title so provenance is traceable).
- [ ] Check every other root `BACKLOG.md` section (Tauri Plugins, App Icon
      SVG, Alert System Future, Sensor and Readings, Logging and
      Observability, Timeline Full Window, KPI Time Range Selector, Activity
      Tracking, Future Features, Removed) against `.plan/BACKLOG.md`; if any
      line exists in root but not in `.plan/BACKLOG.md`, append it there too.
      Do not re-append content that is already present.
- [ ] Delete root `BACKLOG.md`.
- [ ] Delete root `TASKS.md` (already fully redirects to `.plan/` — nothing
      to migrate; its "Open Tasks"/"Epics" tables are a stale mirror of
      `.plan/BACKLOG.md` and `.plan/DONE.md`).
- [ ] Delete root `ORCHESTRATOR.md` (3-line dead pointer; the archived sprint
      file it points to, `.claude/journals/2026-03-21-ux-communication-sprint.md`,
      is not touched — this only removes the stale root pointer).
- [ ] `grep` `README.md` and `CONTRIBUTING.md` for links to `BACKLOG.md`,
      `TASKS.md`, or `ORCHESTRATOR.md`; repoint any hit to `.plan/BACKLOG.md`.
- [ ] Run `node scripts/check-e016-t02.mjs` and fix until it exits 0.

## Acceptance Criteria

- `node scripts/check-e016-t02.mjs` exits 0.
- Root `BACKLOG.md`, `TASKS.md`, `ORCHESTRATOR.md` no longer exist.
- `.plan/BACKLOG.md` contains the "Collect from User" section and its 10
  items, plus the Stripe/Plausible/GSC lines, verbatim.
- `README.md`/`CONTRIBUTING.md` have no dangling links to the deleted files.

## Verify

```
node scripts/check-e016-t02.mjs
```
