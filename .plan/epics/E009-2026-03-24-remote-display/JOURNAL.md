# E009 — Journal: Remote Display

## Planning 2026-03-24
- Epic created from brainstorming session
- Architecture: web kiosk (PC serves React + WS to phone browser)
- Decision: axum for HTTP+WS server (already using tokio)
- Decision: same React build, hook abstraction for Tauri vs WS
- Future phases documented in vision (Tauri Mobile, standalone BLE)
- 7 tasks across 4 waves, ~16h estimated
- ADR 001 created (web kiosk over Tauri Mobile, PWA, standalone)

## CEO Plan Review 2026-03-24 (HOLD SCOPE)
- **3 architecture issues found, all resolved:**
  1. MetricEngine not in WS broadcast → broadcast full RemoteDisplayState (session+metrics+today)
  2. Static files don't exist in dev mode → reverse proxy to Vite (:1443)
  3. TodaySummaryDto missing from WS → included in snapshot payload
- **4 error-handling gaps fixed:**
  1. Port binding panic → graceful error log, app continues
  2. Mutex poison → unwrap_or_else recovery for read-only access
  3. JSON.parse in TS → try/catch
  4. Vite not running → helpful error page
- **1 security fix:** max 10 WS clients (AtomicUsize counter)
- **2 test gaps added:** port-in-use test, malformed JSON test
- **1 stale diagram noted:** ARCHITECTURE.md needs update after E009
- All task files updated with review fixes

## Eng Plan Review 2026-03-25 (SMALL CHANGE)
- **Issue 1 (CRITICAL): SQLite query every 1s in hot path**
  - Original plan queried DB for TodaySummaryDto on every tick
  - Fix: cache TodaySummaryDto in AppState, refresh only on state transitions
  - Zero SQLite in the broadcast hot path — all data from memory
- **Issue 2: Stale closure bug in useRemoteDesk**
  - REST fallback interval captured wsConnected (state) instead of ref
  - Fix: wsConnectedRef.current used in setInterval closure
- **Issue 3: No test for Vite proxy failure**
  - Fix: added Scenario 5 manual test in T07
- All task files updated with eng review fixes

## Session 2026-03-25 01:15 — Implementation Complete

- **Goal**: Implement all 7 tasks of E009 Remote Display epic
- **Done**:
  - Wave 0: version bump to 0.3.0 (aa4f7fc)
  - Wave 1: ws_broadcaster.rs + remote_server.rs — axum HTTP+WS on :3390 (5049c8a)
  - Wave 2: useRemoteDesk.ts + useDeskAuto.ts + responsive CSS (918f2e7)
  - Wave 3: ConnectionOverlay.tsx + docs + ARCHITECTURE.md update (f801782)
  - Split oversized files: commands_welcome.rs, tray_helpers.rs (250-line limit)
- **Decisions**: today_cache in AppState (no SQLite in hot path), wsConnectedRef for interval closure
- **Findings this session**: 0
- **Improvements logged**: 0
- **Test totals**: Rust 339 (+9), TypeScript 189 (+21) = 528 total
- **Next**: Dogfood on real phone, visual verification, manual e2e scenarios
