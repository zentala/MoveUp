---
id: E009-T01
epic: E009
status: pending
created: 2026-03-24
branch: feat/E009-T01-version-bump
---
# E009-T01: Bump version to 0.3.0

## What
Bump version in all three files for the new epic.

## Files to change
1. `package.json` → `"version": "0.3.0"`
2. `src-tauri/tauri.conf.json` → `"version": "0.3.0"`
3. `src-tauri/Cargo.toml` → `version = "0.3.0"`

## Commit
`chore(desk): bump version to 0.3.0 for E009`

## Tag
`git tag -a v0.3.0 -m "v0.3.0 — E009: Remote Display"`
