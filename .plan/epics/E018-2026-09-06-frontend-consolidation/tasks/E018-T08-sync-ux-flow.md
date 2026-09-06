---
id: E018-T08
epic: E018
status: pending
priority: low
effort: trivial
dependencies: []
tags: [docs]
created_at: 2026-09-06
points: 1
agent: main
branch: feat/E018-T08-sync-ux-flow
---

# E018-T08: Sync `.arch/UX-FLOW.md` for the Steps widget and timeline→Analyst click

## Objective

`.arch/UX-FLOW.md` (the mandated single source of truth per
`.claude/rules/ux-flow-sync.md`) was last updated 2026-05-16, before the
Google Fit `StepsWidget` (mounted 2026-05-18/19/21) and the
click-timeline-to-open-Analyst interaction (2026-05-21) existed
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #8).

## Tasks

- [ ] Add a section (or extend an existing widget-inventory section)
  describing the Steps widget: what it shows (`src/components/StepsWidget.tsx`),
  where it's mounted (`OneBarWidget`), what happens with no Google Fit
  connection configured (reconnect CTA per the component's own behavior),
  and the data source (Google Fit, `fitness.activity.read` scope — see
  root `CLAUDE.md` §Google Fit Integration). Use the words "Steps" and
  "Google Fit" explicitly so the verification script below can find them.
- [ ] Add a line describing the click-timeline-to-Analyst interaction:
  clicking the popup timeline (`src/widgets/one-bar/OneBarTimeline.tsx`)
  opens the full Analyst window (`OneBarWidget.tsx:73-78`).
- [ ] Add `scripts/check-e018-t08-ux-flow.mjs` (template below).

```js
// scripts/check-e018-t08-ux-flow.mjs
import { readFileSync } from "node:fs";

const doc = readFileSync(".arch/UX-FLOW.md", "utf8").toLowerCase();
const problems = [];
if (!doc.includes("steps")) problems.push("no mention of the Steps widget");
if (!doc.includes("google fit")) problems.push("no mention of Google Fit");
if (!/timeline[^\n]{0,80}analyst|click[^\n]{0,80}timeline/.test(doc)) {
  problems.push("no mention of the click-timeline-to-Analyst interaction");
}
if (problems.length > 0) {
  console.error("FAIL:", problems.join("; "));
  process.exit(1);
}
console.log("OK: UX-FLOW.md mentions Steps, Google Fit, and timeline-to-Analyst");
```

## Acceptance Criteria

- `node scripts/check-e018-t08-ux-flow.mjs` exits 0.
- `.arch/UX-FLOW.md` describes the Steps widget and the timeline-click
  interaction in enough detail that a new reader understands both without
  reading the source.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Test strategy T08.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #8.
- Rule: `~/.claude/rules/ux-flow-sync.md`.
