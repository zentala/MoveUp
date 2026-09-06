---
id: "E014-T03"
title: "T03 health-good-probe"
status: pending
priority: high
effort: 5
dependencies: ["E014-T02"]
tags: [E014]
created_at: 2026-09-05
---

# T03 — health-good-probe

See HANDOFF.md → T03 ("health-good probe") and PLAN.md decision D5 for
scope, files and verification. Points: 5. New module:
`src-tauri/src/health_probe.rs` + sibling `health_probe_tests.rs`. Three
test cases: process-alive-only (must fail), serves-then-dies-early (must
fail), both conditions met (must pass).
