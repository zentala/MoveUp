# Versioning — Epic-Driven Releases

## Scheme: Semantic Versioning (0.MAJOR.MINOR)

Pre-1.0 convention:
- **0.MAJOR.0** — new epic started (bump before first task of epic)
- **0.MAJOR.MINOR** — hotfix or maintenance within an epic

Examples: `v0.1.0` (E001), `v0.2.0` (E002), `v0.2.1` (hotfix during E002).

## Rule: every epic = version bump

Before starting the first task of a new epic:
1. Bump version in all three files:
   - `package.json` → `"version": "0.X.0"`
   - `src-tauri/tauri.conf.json` → `"version": "0.X.0"`
   - `src-tauri/Cargo.toml` → `version = "0.X.0"`
2. Commit: `chore(desk): bump version to 0.X.0 for E00X`
3. Tag: `git tag -a v0.X.0 -m "v0.X.0 — E00X: <epic title>"`

## Hotfixes between epics

If a fix lands outside an epic (E000-maintenance), bump MINOR:
- `v0.2.0` → `v0.2.1` → `v0.2.2` etc.
- Tag after commit: `git tag -a v0.X.Y -m "v0.X.Y — fix: <description>"`

## Tag format

- Annotated tags only (`git tag -a`, not lightweight)
- Message: `v0.X.Y — E00N: <epic title>` or `v0.X.Y — fix: <description>`
- Push tags with: `git push origin --tags`
