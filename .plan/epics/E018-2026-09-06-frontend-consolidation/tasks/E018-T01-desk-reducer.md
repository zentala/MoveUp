---
id: E018-T01
epic: E018
status: pending
priority: high
effort: large
dependencies: []
tags: [frontend, refactor]
created_at: 2026-09-06
points: 8
agent: ts-dev
branch: feat/E018-T01-desk-reducer
---

# E018-T01: Shared desk-state reducer + two transport adapters

## Objective

Replace the near-duplicate state machines in `src/hooks/useDesk.ts` (Tauri
IPC) and `src/hooks/useRemoteDesk.ts` (WebSocket) with one pure reducer
module shared by both, and make `useDesk.ts` unit-testable for the first
time (today: 0.78% statement coverage per
[`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #3/#4).

## Context

`src/hooks/useDesk.ts:37-247` and `src/hooks/useRemoteDesk.ts:43-252` each
hand-roll `useState` for the same ~20 fields and independently derive
`previousSession`, `todaySessions`, `breakResetProgress`. Both were hand-
edited to add `continuous_computer_secs` (ADR 011) — proof the duplication
already costs double work. `useRemoteDesk.ts` is 94.6% covered because
`WebSocket` is fakeable in jsdom; `useDesk.ts` cannot be faked the same way
because it calls `@tauri-apps/api/core`'s `invoke` and
`@tauri-apps/api/event`'s `listen` directly.

## Tasks

- [ ] Create `src/hooks/deskReducer.ts`: a pure `(state: DeskState, action: DeskAction) => DeskState`
  covering the fields both hooks expose via `UseDeskResult`
  (`src/hooks/useDeskTypes.ts`) — do not change that interface's shape.
  Include the derived selectors (`previousSession`, `todaySessions`,
  `breakResetProgress`) as pure functions of the reducer's state, not new
  `useState`/`useMemo` scattered in each hook.
- [ ] `src/hooks/useDesk.ts`: replace its `useState` calls with
  `useReducer(deskReducer, initialDeskState)`; keep its own `invoke`
  polling (`get_dashboard_state` every 1s, `get_today_summary` every 10s)
  and `listen` subscriptions, translating each Tauri payload into a
  `deskReducer` action instead of duplicating the derivation logic inline.
- [ ] `src/hooks/useRemoteDesk.ts`: same treatment for its WebSocket +
  REST-fallback transport.
- [ ] `src/hooks/deskReducer.test.ts` (new): pure unit tests, no Tauri, no
  WebSocket — feed actions, assert resulting state and derived selectors.
- [ ] `src/hooks/useDesk.test.ts` (new): mock `@tauri-apps/api/core` and
  `@tauri-apps/api/event` with `vi.mock`, the way
  `useRemoteDesk.test.ts` already mocks `WebSocket`; assert the hook
  reaches the same `UseDeskResult` shape as `useRemoteDesk` for an
  equivalent payload.
- [ ] Run the existing `useRemoteDesk.test.ts` unmodified against the
  refactored hook — it must still pass without edits to its assertions
  (only to its internal mocking if the hook's transport call sites moved).

## Four shadow paths (see `~/.claude/rules/testing.md`)

- **happy** — a `state-changed` action updates `sittingSeconds`/`transition`.
- **nil** — initial state before any event: `previousSession === null`.
- **empty** — `todaySummary.sessions.length === 0` → `todaySessions: []`.
- **error** — a rejected `invoke("get_dashboard_state")` leaves `connected`
  false and does not throw out of the hook (matches today's `catch` in
  `fetchState`).

## Acceptance Criteria

- `deskReducer.ts` exists and both hooks dispatch into it; no duplicated
  derivation logic remains between `useDesk.ts` and `useRemoteDesk.ts`.
- `useDesk.test.ts` exists and exercises the four shadow paths above.
- `UseDeskResult`'s public shape (`src/hooks/useDeskTypes.ts`) is unchanged
  — `useWidgetData.ts` and all widgets keep working without edits.
- `npx vitest run --config vite.config.ts src/hooks` passes.
- All new/changed files ≤ 250 lines.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Problem, §Test strategy T01.
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — reducer/adapter split.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) findings #3, #4, §Recommended target structure item 1.
