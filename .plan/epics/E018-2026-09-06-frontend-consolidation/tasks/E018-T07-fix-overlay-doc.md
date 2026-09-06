---
id: E018-T07
epic: E018
status: pending
priority: low
effort: trivial
dependencies: []
tags: [docs]
created_at: 2026-09-06
points: 1
agent: main
branch: feat/E018-T07-fix-overlay-doc
---

# E018-T07: Correct `.claude/rules/overlay.md`'s dev-launcher references

## Objective

`.claude/rules/overlay.md` documents a `scripts/tauri-dev.sh` that does not
exist and a `pnpm tauri:dev --force` invocation that `package.json` does
not pass through to the real PowerShell launcher
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #15).

## Context

Actual state: `scripts/tauri-dev.ps1` is the only launcher (its own header
says "Replaces tauri-dev.sh for Windows PowerShell environments"), invoked
with a `-Force` switch, not a `--force` flag passed through `pnpm`.

## Tasks

- [ ] In `.claude/rules/overlay.md` line 30, replace
  `pnpm tauri:dev --force            # live + auto-kill previous instance`
  with the real invocation:
  `powershell -ExecutionPolicy Bypass -File scripts/tauri-dev.ps1 -Force  # live + auto-kill previous instance`.
- [ ] In `.claude/rules/overlay.md` lines 33 and 37, replace both
  `scripts/tauri-dev.sh` references with `scripts/tauri-dev.ps1`.
- [ ] Add `scripts/check-e018-t07-overlay-doc.mjs` (template below) and
  keep it — it is the task's own verification and a standing regression
  guard against the doc drifting again.

```js
// scripts/check-e018-t07-overlay-doc.mjs
import { readFileSync } from "node:fs";

const doc = readFileSync(".claude/rules/overlay.md", "utf8");
const problems = [];
if (doc.includes("tauri-dev.sh")) problems.push("stale 'tauri-dev.sh' reference still present");
if (!doc.includes("tauri-dev.ps1 -Force")) problems.push("missing corrected 'tauri-dev.ps1 -Force' invocation");
if (problems.length > 0) {
  console.error("FAIL:", problems.join("; "));
  process.exit(1);
}
console.log("OK: overlay.md launcher references are correct");
```

## Acceptance Criteria

- `node scripts/check-e018-t07-overlay-doc.mjs` exits 0.
- No occurrence of `tauri-dev.sh` remains in `.claude/rules/overlay.md`.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Test strategy T07.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #15.
