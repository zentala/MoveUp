# CODEX_PROMPT.md — How to Use Codex for This Project

Use this file as the initial instruction for Codex.

---

## Main Instruction

You are implementing **Healthy Balance**, an Electron + React + TypeScript desktop tray/menu-bar app.

Read these files before coding:

```txt
PRD.md
DESIGN_SYSTEM.md
ARCHITECTURE.md
TASKS.md
TEST_CASES.md
```

Follow them strictly.

---

## Engineering Rules

- Use TypeScript strict mode.
- Avoid `any`.
- Keep domain logic outside React components.
- Keep Electron main process separate from renderer.
- Renderer must not access Node.js APIs directly.
- Use preload IPC bridge.
- Use local JSON persistence for MVP.
- Use CSS variables for design tokens.
- Use pure functions for timeline calculations.
- Add tests for calculation logic.
- Do not implement real hardware integration yet.
- Do not add cloud sync, auth, analytics, or telemetry.

---

## Preferred Stack

```txt
Electron
React
TypeScript
Vite
Zustand
Vitest
CSS variables
```

---

## First Task

Create the app shell.

Implement:

- Electron app bootstrap.
- React renderer.
- Preload bridge.
- Tray/menu-bar icon.
- Popover window.
- Popover open/close behavior.
- Platform detection.
- Basic Status screen placeholder.
- Light/dark/system theme tokens.

Do not implement notifications or timeline logic in the first task unless the shell is complete.

---

## Second Task

Implement domain types and persistence.

Add:

- `WorkMode`
- `TimelineSegment`
- `AppSettings`
- `DailySummary`
- defaults
- settings persistence
- timeline persistence
- corrupt JSON recovery
- typed preload API

---

## Third Task

Implement mode tracking and timeline calculations.

Add:

- Zustand store
- `setMode`
- `calculateDailySummary`
- `switchMode`
- `clampSegmentToDay`
- unit tests from `TEST_CASES.md`

---

## Fourth Task

Implement UI.

Add:

- Status screen
- CurrentModeCard
- DeskHeightCard
- SummaryCard
- QuickActionBar
- DailyTimeline compact placeholder or real component

---

## Fifth Task

Implement reminders.

Add:

- reminder scheduler
- native notification bridge
- snooze
- quiet hours
- standing completion notification
- tests

---

## Definition of Done for Every Task

Before finishing each task:

- Run typecheck.
- Run tests.
- Keep code formatted.
- Avoid unrelated changes.
- Mention files changed.
- Mention any skipped requirement clearly.
