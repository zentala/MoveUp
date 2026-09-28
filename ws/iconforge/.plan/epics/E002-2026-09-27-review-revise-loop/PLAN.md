# E002 — Automated review→revise loop on existing icons

Created 2026-09-27 after the operator's review of the E001 pilot sheet.

## Why

Operator feedback on the pilot: the approach works as a proof of concept, but
(1) some icons need more primitives than the planner allowed, and the count
should be chosen per icon; (2) strokes end slightly too long or too short, so
joints show round-cap bumps and notches ("ragged"); (3) heart-rate is jagged;
(4) height-sensor is ugly; (5) can primitives be rounded, not only straight lines.

## Approach

- New additive `path` primitive (segments: line / quad / cubic / arc) — one
  continuous stroke with proper joins. Spec addendum in `docs/spec/ICON_SPEC.md`.
- Quality rules `join.near-miss` / `join.overshoot` measure what the operator saw.
- Dynamic shape budget in the planner/reviser prompts instead of a fixed 12.
- CLI `improve`: vision reviewer → reviser rounds on an existing spec, never
  accepting a revision with geometry errors; writes before/after + sheet.
- Run on a few icons first (heart-rate, height-sensor, settings, sitting); the
  coordinator judges before/after from the renders before widening.

## Tasks

| ID | Task | Package | Importance | Points | Wave | Status |
|---|---|---|---|---|---|---|
| T01 | `path` primitive: schema (coordinator) + renderer | schema, renderer | High | 3 | 1 | done |
| T02 | `path` bounds + `join.near-miss`/`join.overshoot` + `flatten` | quality | High | 3 | 1 | done |
| T03 | Prompts (path, joins, dynamic budget, warnings to reviewer) + `improveIcon` + CLI `improve` + `revise` test | agent, cli | High | 5 | 1 | done |
| T04 | Live run on 4 icons, coordinator evaluation, decide on widening | examples | High | 2 | 2 | done |

Epic total: **13 points** → subagents (policy threshold ≤13).

## Architecture impact

IconSpec v1 gains one additive shape type; no version bump (every v1 document
stays valid). No MoveUp app architecture impact.
