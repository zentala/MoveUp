# ws — incubator monorepo

`ws/` holds side projects that support MoveUp but are not part of the desktop
app. Each top-level folder is one project with its own `AGENTS.md`, `.plan/`
and `packages/*`. The MoveUp root (`../`) does not build, lint or test
anything here; `ws/` is its own pnpm workspace root.

| Project | What | Plan |
|---|---|---|
| [`iconforge/`](iconforge/AGENTS.md) | Text brief → IconSpec JSON → deterministic SVG/PNG icons | [`iconforge/.plan/`](iconforge/.plan/STATE.md) |

## Commands (run from `ws/`)

```bash
pnpm install
pnpm typecheck        # tsc over every project's packages
pnpm test             # vitest over */packages/*/{src,test}/**/*.test.ts
pnpm icon <cmd> ...   # IconForge CLI (see iconforge/README.md)
```

## Adding a project

1. `mkdir <name>/packages/<pkg>/src`, package name `@<name>/<pkg>`, `"exports": { ".": "./src/index.ts" }`.
2. Packages ship TypeScript source; `tsx` and `vitest` run it directly. No build step until something is published.
3. Give the project `AGENTS.md`, `CLAUDE.md` (`@AGENTS.md`) and `.plan/STATE.md`, and add a row to the table above.

## Conventions

- Files, code and commits in English. Spec documents copied from elsewhere keep their original language under `docs/spec/`.
- Strict TypeScript (`noUncheckedIndexedAccess`), ESM, `.ts` import extensions.
- Tests sit next to source as `*.test.ts`.
