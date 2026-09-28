# E001 journal

## 2026-09-27

- Unpacked spec, scaffolded `ws/` pnpm workspace (root `ws/`, packages `*/packages/*`), packages ship TS source run by tsx/vitest.
- T01 schema written by the coordinator (it is the shared contract): zod 4 `strictObject`, finite numbers, positive radii, arc sweep `0 < |s| <= 360`, kebab-case ids, duplicate-id check, diagnostics carry `path` + innermost `shapeId`. Smoke-tested: NaN / negative radius / unknown key each produce a located diagnostic.
- Location decision: `ws/` inside the MoveUp repo (operator said "tutaj"); pilot domain = MoveUp concepts.
- Wave 1 (parallel ts-dev): renderer (25 tests), quality (26 tests, 12 rule codes), agent (22 tests; OpenRouter + mock provider, one schema retry, hard cap 3 rounds). Wave 2: CLI (7 tests) and 20 pilot icons (general-purpose agent, looked at its own renders).
- Coordinator verification: `pnpm typecheck` clean, `vitest` 80/80, two `render` runs of desk.json byte-identical, `sheet` wrote 20 icons / 0 rejected.
- Coordinator's own review of the sheet flags 7 icons (sitting, steps, heart-rate, settings, snooze, walking≈standing) — filed to BACKLOG as phase-3 input.
- Agent decisions to keep an eye on: quality checks grid/coord-range on raw (untransformed) coordinates; unparseable reviewer output counts as "no defects" plus a warning; no `ajv` — agent hand-rolled a mini JSON-schema validator for one equivalence test.
