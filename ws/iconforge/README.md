# IconForge

IconForge turns a text brief into a consistent, editable line icon. The
model never writes SVG — it writes an **IconSpec** (JSON over a closed set
of primitives), which is validated and compiled by deterministic local code.
See [`AGENTS.md`](AGENTS.md) for the full pipeline diagram and rules, and
[`docs/spec/`](docs/spec/README.md) (Polish) for the PRD, architecture and
schema — the schema doc is the contract.

## Quick start

From the `ws/` workspace root:

```bash
pnpm icon validate iconforge/examples/icons/moveup/desk.json
pnpm icon render   iconforge/examples/icons/moveup/desk.json --out out/desk
pnpm icon sheet    iconforge/examples/icons/moveup --out out/sheet
pnpm icon generate "height-adjustable desk, raised" --out out/gen   # needs OPENROUTER_API_KEY
```

`out/` is gitignored — every command output lands there.

## Commands

Run `pnpm icon <command> --help` for the full option list.

### `validate`

```bash
pnpm icon validate <spec.json...> [--profile id|path] [--json]
```

Prints one line per diagnostic: `path [shapeId] severity code: message`.
Exit 0 if every file is valid, 1 otherwise.

### `render`

```bash
pnpm icon render <spec.json> --out <dir> [--profile id|path]
```

Renders one IconSpec. On validation errors it prints diagnostics, writes
nothing, and exits 1. On success it writes:

- `icon.svg` — `stroke="currentColor"`, no metadata inlined
- `spec.json` — normalized copy of the input
- `preview-24-light.png`, `preview-24-dark.png`, `preview-512-light.png`, `preview-512-dark.png`
- `report.json` — diagnostics, quality summary, profile id, schema version, SVG sha256

Identical spec + profile always produce a byte-identical `icon.svg`.

### `sheet`

```bash
pnpm icon sheet <dir|files...> --out <dir> [--profile id|path]
```

Builds a comparison sheet for a set of icons: `sheet.html` (self-contained,
inline SVGs, no external assets or scripts — rows at 24/48/96px, light and
dark panels, a warning-count badge per icon) and `sheet.png` (the same grid
rasterized in one image; labels use system fonts when available). Invalid
icons are listed as rejected with their first diagnostic, never silently
skipped.

### `generate`

```bash
pnpm icon generate "<brief>" --out <dir> [--profile id|path] [--examples <dir>] \
  [--model <id>] [--max-rounds 1..3] [--max-cost <usd>] [--timeout-ms <ms>]
```

Runs the planner → validate → reviewer → reviser loop (`@iconforge/agent`)
and writes the same artifacts as `render`, plus `history.json` (every round:
spec, diagnostics, defects, token/cost usage — never the API key or headers)
and `brief.txt`. Reads `OPENROUTER_API_KEY` from the environment; missing
key exits 2 before any network call.

### `revise`

```bash
pnpm icon revise <spec.json> --defects <defects.json|"free text"> --brief "..." --out <dir>
```

Revises an existing IconSpec against a list of defects (a JSON array of
`{ shapeId, observation, change }`, or free text treated as one defect) and
writes the same artifacts as `generate`.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | validation errors |
| 2 | usage or configuration error (bad flags, missing `OPENROUTER_API_KEY`) |
| 3 | provider or budget failure (timeout, cost cap, planner failure) |

## Profile resolution

`--profile` accepts a builtin id (currently `outline-24-v1`, from
`BUILTIN_PROFILES`) or a path to a profile JSON file (parsed with
`parseProfile`). Without `--profile`, the spec's own `profile` field is used.
An unknown id or an invalid profile file is a usage error (exit 2).

## Tests

```bash
pnpm vitest run iconforge/packages/cli   # this package only
pnpm test                                 # full workspace
pnpm typecheck
```
