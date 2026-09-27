# IconForge — agent instructions

An agent turns a text brief into consistent, editable line icons. The model
never writes SVG: it writes an **IconSpec** (JSON over a closed set of
primitives), which is validated and compiled by deterministic local code.

```
brief + examples + profile → planner LLM → IconSpec → validate → compile → SVG → PNG
                                              ↑                                  ↓
                                          reviser LLM ← defects ← reviewer LLM (vision)
```

The source specification (Polish) is in [`docs/spec/`](docs/spec/README.md):
PRD, ARCHITECTURE, ICON_SPEC, TASKS, RESEARCH and the original AGENTS brief.
**It is the contract.** Change `docs/spec/ICON_SPEC.md` with a reason before
changing the schema in code.

## Packages

| Package | Responsibility | Depends on |
|---|---|---|
| `@iconforge/schema` | Zod IconSpec v1 + StyleProfile, `parseIconSpec`, diagnostics with shape id + path | zod |
| `@iconforge/renderer` | primitives → normalized, allowlisted SVG; SVG → PNG | schema, @resvg/resvg-js |
| `@iconforge/quality` | geometry rules: bounds incl. half stroke, limits, grid, gaps | schema |
| `@iconforge/agent` | provider-agnostic LLM adapter, planner/reviewer/reviser prompts, budgets | schema |
| `@iconforge/cli` | `validate`, `render`, `generate`, `revise`, `sheet` | all of the above |

Profiles and hand-checked icons live in `examples/`. Run outputs go to `out/` (gitignored).

## Rules

- Never execute code, HTML or SVG returned by a model. Model output is data and only ever enters through `parseIconSpec`.
- Never ask a model for `path d`. The renderer may emit `<path>` for arcs and curves.
- Style lives in profiles, semantics in prompts, geometry in IconSpec. The model does not set stroke width.
- Do not repair model coordinates silently. Invalid geometry returns diagnostics, never a broken SVG.
- Identical IconSpec + profile → byte-identical SVG (fixed precision, fixed attribute order).
- Exported SVG: no scripts, no `foreignObject`, no `<image>`, no URIs, no `style`, no event handlers, no unknown attributes.
- Look at renders at 24 px and 512 px on light and dark before calling an icon good. Report concrete defects, not "looks nice".
- API keys come from the environment (`OPENROUTER_API_KEY`) and never land in `out/` artifacts.
- Confirm a new dependency with a small prototype and a licence check before adding it.

## Commands (from `ws/`)

```bash
pnpm test
pnpm typecheck
pnpm icon validate iconforge/examples/icons/moveup/desk.json
pnpm icon render   iconforge/examples/icons/moveup/desk.json --out out/desk
pnpm icon sheet    iconforge/examples/icons/moveup --out out/sheet
pnpm icon generate "height-adjustable desk, raised" --out out/gen   # needs OPENROUTER_API_KEY
```

## Plan

Work tracking: [`.plan/STATE.md`](.plan/STATE.md), epics in [`.plan/epics/INDEX.md`](.plan/epics/INDEX.md).
Linked as a sub-plan from the MoveUp repository `.plan/epics/INDEX.md`.
