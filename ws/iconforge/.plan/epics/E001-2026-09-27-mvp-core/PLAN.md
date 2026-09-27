# E001 — IconForge MVP core (spec phases 0–2)

Created 2026-09-27. Source: `docs/spec/TASKS.md` phases 0–2, `docs/spec/PRD.md`.

## Goal

A working offline pipeline (IconSpec → validate → SVG → PNG → comparison sheet)
plus the LLM loop (generate / revise) behind a provider adapter, and a pilot set
of 20 MoveUp-domain icons authored as IconSpec so the style can be judged on one sheet.

## Out of scope

Phase 3 blind human evaluation, semantic macros (`person`, `chair`), React editor, Figma.

## Tasks

| ID | Task | Package | Importance | Points | Wave | Status |
|---|---|---|---|---|---|---|
| T01 | Monorepo scaffold + IconSpec v1 / profile schema + diagnostics | schema | High | 3 | 0 | done |
| T02 | Deterministic renderer, allowlist, fixed precision; PNG via resvg | renderer | High | 5 | 1 | done |
| T03 | Geometry quality rules (bounds + half stroke, limits, grid, gaps) | quality | High | 3 | 1 | done |
| T04 | LLM adapter (OpenRouter + mock), planner/reviewer/reviser, budgets, one schema retry | agent | Medium | 5 | 1 | done |
| T05 | CLI validate/render/generate/revise/sheet + HTML/PNG sheet | cli | High | 5 | 2 | done |
| T06 | 20 MoveUp pilot icons as IconSpec, visually checked at 24/512 px | examples | Medium | 3 | 2 | done |

Epic total: **24 points**. Execution policy routes >13 to AO; the human operator
asked for a subagent swarm for this bootstrap, so this epic runs as
**direct subagent execution** (not an AO run), max 4 in parallel.

## Architecture impact

New, self-contained project under `ws/iconforge/`. No change to the MoveUp app
architecture (`.arch/ARCHITECTURE.md`) and no MoveUp ADR triggered.

## Acceptance (from PRD)

- Same IconSpec + profile → byte-identical SVG.
- Each primitive has a valid example that renders; tests cover arc 360°, stroke bounds, bad coordinates, unknown field, determinism, rejection of unsafe fields.
- Invalid geometry → diagnostics with shape id and path, no SVG.
- Sheet of the pilot set at 24 px and 512 px on light and dark backgrounds.
