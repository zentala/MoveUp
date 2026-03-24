---
id: E005-T12
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T032
---
# T032 — Widget "One Bar"

**Priority:** P2
**Depends on:** T031 (widget architecture)
**Wave:** 5 (parallel with T033)

---

## Goal

Implement the "One Bar" widget: horizontal popup with timeline, unified progress bar,
temperature escalation, and one-line coach.

## Core Concept

One bar, one meaning: progress bar = limit usage. Fills when sitting, drains when standing/away.

Temperature escalation: calm -> warm -> hot -> burning -> standing -> away -> reset.

---

## Acceptance Criteria

- [ ] Widget renders in all 3 states (sitting/standing/away)
- [ ] Temperature escalation visible
- [ ] Progress bar has ONE meaning: fills when sitting, drains when standing/away
- [ ] Big number shows limitRemaining
- [ ] Coach sentence is context-appropriate
- [ ] No file > 250 lines
