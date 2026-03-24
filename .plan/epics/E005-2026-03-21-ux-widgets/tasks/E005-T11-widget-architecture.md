---
id: E005-T11
epic: E005
status: done
created: 2026-03-21
completed: 2026-03-22
original_id: T031
---
# T031 — Widget Architecture for Floating Window

**Priority:** P1
**Depends on:** T027 (session.rs split), T030 (floating window fixes)
**Blocks:** T032, T033
**Wave:** 5

---

## Goal

Replace the hardcoded floating window UI with a pluggable widget system.
Core app provides data; widgets handle presentation.

## Key Decisions

- Extend useDesk instead of new useWidgetData hook
- Shared `<SessionTimeline>` component for all widgets
- `limit_used_secs` computed in Rust — single source of truth
- `limitRemaining` goes negative when over limit — no clamping
- Widget registry is a static TS map, no dynamic loading

---

## Acceptance Criteria

- [ ] App.tsx uses widget system, no hardcoded session UI
- [ ] SettingsPanel has widget picker (dropdown)
- [ ] PlaceholderWidget shows all data from WidgetProps
- [ ] `active_widget` persisted in AppConfig
- [ ] All existing tests still pass
- [ ] No file > 250 lines
