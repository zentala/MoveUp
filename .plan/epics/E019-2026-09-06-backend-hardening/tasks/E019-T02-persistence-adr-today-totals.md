---
id: "002"
title: "T02 persistence-adr-today-totals"
status: pending
priority: high
effort: 8
tags: [E019]
created_at: 2026-09-06
---

# T02 — persistence-adr-today-totals

See HANDOFF.md → T02 for scope, files, and verification. Points: 8.
Does not touch `session_types.rs`, `db_sessions.rs`, or
`session_persistence.rs` (E015-owned) — only calls their existing public
functions.
