---
id: "E016-T01"
title: "Fix .plan/STATE.md body to match its frontmatter"
status: pending
priority: high
effort: small
dependencies: []
tags: [docs, plan-hygiene]
created_at: 2026-09-06
---

# Fix `.plan/STATE.md` body

## Objective

`.plan/STATE.md`'s YAML frontmatter was already corrected today (says
`active_epic: none (E011 and E012 code complete...)`), but the markdown body
below it still says E011 is "active, scaffolded, T03 done" with "Wave 1
(T01+T02) and Wave 2-3 pending" and lists T01/T02/T04/T05 as `⏳` — all five
are in fact `[x]` done per `.plan/DONE.md`. This is the file every agent
reads first; a self-contradicting file is worse than a stale one because it
looks authoritative in two directions at once.

Also reconcile the E010 "human tasks remaining" count: the body's "E010
Progress" section says 3 (T02 real usage data, T14 marketing videos, T18
hero GIF/video). Root `BACKLOG.md:266-280`'s "Collect from User — blocking
launch" checklist lists 10 items (adds Stripe account, Plausible account,
Google Search Console, OG image, real usage stats export, sensor/desk
photos). These are not the same list — the 3 are E010's own numbered tasks;
the 10 are a broader "things only Paweł can do" checklist that overlaps
with but is not identical to those 3 tasks. Do not silently pick one number;
state both and how they relate.

## Tasks

- [ ] Rewrite the "## E011 progress" section: mark T01/T02/T04/T05 as `✅`
      done (matching `.plan/DONE.md`), remove the "Wave ... pending" framing,
      note the ceremony closure done in E016-T03.
- [ ] Rewrite the "## Status" bullet for E011 to say "done, code-complete
      2026-05-07, ceremony closed via E016" instead of "active, scaffolded".
- [ ] In "## E010 Progress", keep the 3-task list but add one sentence:
      "See `.plan/BACKLOG.md` 'Collect from User' for 7 additional
      business/infra items (Stripe, Plausible, GSC, OG image, stats export)
      not counted as E010 tasks but also outstanding." (Written after E016-T02
      has merged that section into `.plan/BACKLOG.md` — if T02 has not yet
      landed when this runs, point at root `BACKLOG.md:266-280` instead and
      leave a one-line TODO to update the pointer once T02 merges.)
- [ ] Confirm the version line already says `v0.5.0` (it does, per today's
      earlier partial fix) — do not touch if already correct.
- [ ] Run `node scripts/check-e016-t01.mjs` and fix until it exits 0.

## Acceptance Criteria

- `node scripts/check-e016-t01.mjs` exits 0.
- No sentence in `.plan/STATE.md` claims an E011 task is pending.
- The E010 3-vs-10 discrepancy has an explicit one-sentence reconciliation,
  not a silent pick of one number.

## Verify

```
node scripts/check-e016-t01.mjs
```
