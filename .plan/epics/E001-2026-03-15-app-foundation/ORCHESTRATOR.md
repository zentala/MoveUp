---
epic: E001
created: 2026-03-15
status: done
---
# E001 Orchestrator — App Foundation

## Wave Structure

### Wave 1 — Foundation (1 agent, sequential)
- [x] E001-T01 — Config store + rusqlite (was 0002)

### Wave 2 — Parallel (2 agents)
- [x] E001-T02 — Daily reset (was 0009)
- [x] E001-T03 — Settings panel UI (was 0001-settings-panel)

### Wave 3 — Parallel (2 agents, merge conflict in session.rs)
- [x] E001-T04 — Notification toggles (was 0003)
- [x] E001-T05 — Stand reminder limit (was 0004)
- [x] E001-T06 — Position changes counter (was 0005)
- [x] E001-T07 — Fix standing_secs (was 0006)

### Wave 4 — Parallel (3 agents, no conflicts)
- [x] E001-T08 — Test framework (was 0001-test-framework)
- [ ] E001-T09 — position_changes DB schema (was 0007) → BACKLOG
- [-] E001-T10 — Dynamic tray icon (was 0010) — CANCELLED, replaced by T016
- [ ] E001-T11 — Rail pulse animation (was 0011) → BACKLOG
- [ ] E001-T12 — Yesterday delta arrow (was 0012) → BACKLOG

## Merge Conflict Notes

Wave 3 agents both modified `session.rs`. Changes were purely additive (different field names):
- Agent C added: `last_position_change_at`, `stand_limit_secs`, `stand_alert_fired`, notification flags
- Agent D added: `position_changes: u32`
Resolution: take both sets of fields.

## Original Orchestration

See `archive/COORDINATOR.md` for the full original coordinator document.
