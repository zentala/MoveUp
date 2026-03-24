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
