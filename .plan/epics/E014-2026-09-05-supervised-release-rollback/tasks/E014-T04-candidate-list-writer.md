---
id: "E014-T04"
title: "T04 candidate-list-writer"
status: pending
priority: high
effort: 3
dependencies: ["E014-T01", "E014-T02"]
tags: [E014]
created_at: 2026-09-05
---

# T04 — candidate-list-writer

See HANDOFF.md → T04 ("ordered candidate list writer") and PLAN.md decision
D6 for scope, files and verification. Points: 3. New module:
`src-tauri/src/candidate_list.rs` + sibling `candidate_list_tests.rs`.
Shadow paths from PLAN.md's Test strategy: nil (fresh install, one entry),
empty (no builds, empty list, no crash), error handled by the probe not
the list writer.
