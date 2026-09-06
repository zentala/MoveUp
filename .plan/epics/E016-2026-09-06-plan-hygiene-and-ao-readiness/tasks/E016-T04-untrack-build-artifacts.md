---
id: "E016-T04"
title: "Untrack coverage/ and test-performance-report/"
status: pending
priority: medium
effort: small
dependencies: []
tags: [repo-hygiene]
created_at: 2026-09-06
---

# Untrack build-artifact directories

## Objective

`coverage/` (test-coverage HTML/JSON, 18 files) and `test-performance-report/`
are committed to git despite `.gitignore` already excluding `dist/`. They
drift on every test run and bloat every clone. Confirmed today:
`git ls-files | grep -E '^(coverage|test-performance-report)/'` returns 18
matches.

## Tasks

- [ ] `git rm -r --cached coverage test-performance-report`
- [ ] Add `coverage/` and `test-performance-report/` to `.gitignore`
      (alongside the existing `dist/` entry).
- [ ] Run `node scripts/check-e016-t04.mjs` and fix until it exits 0.

## Acceptance Criteria

- `git ls-files` shows zero files under `coverage/` or
  `test-performance-report/`.
- `.gitignore` contains both entries.
- The directories still exist on disk (only untracked, not deleted) so a
  local `pnpm test:coverage` run still writes output somewhere sane.

## Verify

```
node scripts/check-e016-t04.mjs
```
