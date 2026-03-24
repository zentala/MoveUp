# E009 — Orchestrator: Remote Display (Web Kiosk)

## Task Index

| ID | Task | Status | Depends on | Wave |
|----|------|--------|------------|------|
| E009-T01 | Version bump to 0.3.0 | [ ] | — | 0 |
| E009-T02 | WebSocket broadcaster module | [ ] | — | 1 |
| E009-T03 | HTTP + WebSocket server (axum) | [ ] | T02 | 1 |
| E009-T04 | useRemoteDesk() React hook | [ ] | T03 | 2 |
| E009-T05 | Responsive layout for phone | [ ] | T04 | 2 |
| E009-T06 | Connection status & reconnect UI | [ ] | T04, T05 | 3 |
| E009-T07 | E2E testing & documentation | [ ] | T06 | 3 |

## Execution Waves

### Wave 0 — Prep (5 min)
**T01: Version bump** — trivial, do first.

### Wave 1 — Backend (parallel-capable)
**T02 + T03** — T02 creates the broadcast channel, T03 creates the server.
T03 depends on T02 (needs `ws_tx`), so do T02 first, then T03.
Alternatively: implement both in one branch if single developer.

**Estimated effort:** T02 ~2h, T03 ~4h

**Verification checkpoint:** After wave 1, run `pnpm tauri:dev` and verify:
- Log line: "Remote display server listening on 0.0.0.0:3390"
- `curl http://localhost:3390/display/api` returns JSON
- `websocat ws://localhost:3390/display/ws` shows events

### Wave 2 — Frontend (parallel-capable)
**T04 + T05** — T04 creates the hook, T05 adapts CSS.
T05 depends on T04 (needs `useDeskAuto` working to test), but CSS work
can start in parallel if developer stubs the hook.

**Estimated effort:** T04 ~3h, T05 ~2h

**Verification checkpoint:** After wave 2, open `http://localhost:3390/display`
in Chrome DevTools with phone viewport. Should show live dashboard data updating.

### Wave 3 — Polish & Ship
**T06** — Connection overlay (needs T04+T05 done).
**T07** — E2E tests + docs (needs everything done).

**Estimated effort:** T06 ~2h, T07 ~3h

**Final verification:** Full e2e test on actual phone or phone emulator.

## Total Estimated Effort
~16 hours for a mid-level developer familiar with Rust + React.

## Key Files Reference

Developers should read these before starting:

| File | Why |
|------|-----|
| `src-tauri/src/tray_controller.rs` | Understand existing event flow |
| `src-tauri/src/commands.rs` | `AppState` struct to extend |
| `src-tauri/src/session_types.rs` | All DTOs and types |
| `src/hooks/useDesk.ts` | Hook interface to replicate |
| `src/hooks/useDeskTypes.ts` | `UseDeskResult` type definition |
| `src/hooks/useWidgetData.ts` | Where to swap `useDesk` → `useDeskAuto` |
| `src/types.ts` | All TypeScript types/payloads |
| `.plan/epics/E009-.../PLAN.md` | Full architecture and rationale |

## Eng Review Fixes (2026-03-25)

| Task | Fix applied |
|------|-------------|
| T02 | **No SQLite in hot path** — cache `TodaySummaryDto` in `AppState`, refresh only on state transitions |
| T02 | Add `today_cache: Arc<Mutex<TodaySummaryDto>>` to `AppState` |
| T04 | Fix stale closure bug: use `wsConnectedRef` (useRef) instead of `wsConnected` state in setInterval |
| T07 | Add Scenario 5: Vite proxy failure manual test |

## CEO Review Fixes (2026-03-24)

| Task | Fix applied |
|------|-------------|
| T02 | Broadcast `RemoteDisplayState` (session + metrics + today), not just `SessionStateDto` |
| T02 | Listen for and broadcast `desk:daily-reset` event |
| T03 | Dev mode: reverse proxy to Vite (:1443) instead of static files |
| T03 | Graceful port binding failure (log error, don't crash app) |
| T03 | Max 10 WS clients (AtomicUsize counter, reject with 503) |
| T03 | Log active client count on connect/disconnect |
| T03 | Mutex poison recovery for read-only WS access |
| T03 | Vite not running → helpful error page (not blank) |
| T04 | JSON.parse wrapped in try/catch (malformed WS messages) |
| T04 | Handle `desk:daily-reset` event |
| T04 | Snapshot payload is now `{ session, metrics, today }` |
| T07 | Heartbeat test uses 7s margin (not 6s) to avoid flakiness |
| T07 | Update `.arch/ARCHITECTURE.md` after E009 ships |

## Risks & Mitigations

| Risk | Wave | Mitigation |
|------|------|------------|
| axum version incompatibility with tokio | 1 | Use axum 0.8 which matches tokio 1.x (already in Cargo.toml) |
| Static file path differs in dev vs prod | 1 | Dev: proxy to Vite. Prod: `DESK_REMOTE_DIST` env var + sensible defaults |
| Port already in use | 1 | Graceful error log, app continues without remote display |
| `useDesk` import scattered in many files | 2 | Only `useWidgetData.ts` imports it directly |
| Malformed WS message crashes phone UI | 2 | JSON.parse in try/catch, log and ignore |
| Phone browser doesn't support WebSocket | 3 | REST polling fallback in `useRemoteDesk` |
| Windows Firewall blocks port | 3 | Document firewall rule in `docs/REMOTE_DISPLAY.md` |
| WS connection flood (DoS) | 1 | Max 10 clients, reject 11th with 503 |
