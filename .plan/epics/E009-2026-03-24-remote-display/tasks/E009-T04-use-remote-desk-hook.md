---
id: E009-T04
epic: E009
status: completed
created: 2026-03-24
branch: feat/E009-T04-use-remote-desk
depends_on: [E009-T03]
title: useRemoteDesk() React Hook
---
# E009-T04: useRemoteDesk() React Hook

## What
Create a React hook that provides the same `UseDeskResult` interface as
`useDesk()`, but connects via WebSocket instead of Tauri IPC. Create a
wrapper `useDeskAuto()` that auto-selects the right hook based on runtime
environment.

## Context

### Current data flow (Tauri IPC)
```
useDesk() → invoke("get_dashboard_state") every 1s (polling)
           → listen("desk:state-changed") (event)
           → listen("desk:device-connected") (event)
           → returns UseDeskResult
```

### New data flow (WebSocket)
```
useRemoteDesk() → new WebSocket("ws://HOST:PORT/display/ws")
                → receives JSON events (same payloads)
                → returns UseDeskResult (identical interface)
```

### Environment detection
```typescript
const isTauri = !!window.__TAURI_INTERNALS__;
// true  → Tauri desktop app → useDesk()
// false → browser/kiosk → useRemoteDesk()
```

## Implementation

### New file: `src/hooks/useRemoteDesk.ts` (~100 lines)

```typescript
import { useState, useEffect, useRef, useCallback } from "react";
import type { UseDeskResult } from "./useDeskTypes";
import type { DeskState, SessionStateDto, MetricSnapshot } from "@/types";

/** WebSocket event shape (matches Rust DisplayEvent serialization). */
interface WsEvent {
  event: string;
  payload: any;
}

/**
 * Connects to the desk backend via WebSocket.
 * Same interface as useDesk() but works in any browser.
 *
 * Auto-reconnects with exponential backoff (1s → 2s → 4s → max 10s).
 * Falls back to REST polling if WebSocket fails.
 */
export function useRemoteDesk(): UseDeskResult {
  // ... state variables (same as useDesk)
  const [wsConnected, setWsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const wsConnectedRef = useRef(false); // ENG REVIEW FIX: ref for interval closure
  const reconnectDelay = useRef(1000);

  // Derive WS URL from current page location
  const wsUrl = `ws://${window.location.host}/display/ws`;
  const apiUrl = `/display/api`;

  useEffect(() => {
    let mounted = true;

    function connect() {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setWsConnected(true);
        wsConnectedRef.current = true; // ENG REVIEW FIX: update ref too
        reconnectDelay.current = 1000; // Reset backoff
      };

      ws.onmessage = (e) => {
        // REVIEW FIX: wrap in try/catch to handle malformed JSON gracefully
        let msg: WsEvent;
        try {
          msg = JSON.parse(e.data);
        } catch {
          console.warn("Remote display: received malformed WS message, ignoring");
          return;
        }
        switch (msg.event) {
          case "snapshot":
            applySnapshot(msg.payload);
            break;
          case "desk:state-changed":
            applyStateChanged(msg.payload);
            break;
          case "desk:device-connected":
            setConnected(true);
            break;
          case "desk:device-lost":
            setConnected(false);
            break;
          case "heartbeat":
            break; // Keep-alive, no action
        }
      };

      ws.onclose = () => {
        setWsConnected(false);
        wsConnectedRef.current = false; // ENG REVIEW FIX: update ref too
        if (mounted) {
          // Exponential backoff reconnect
          setTimeout(connect, reconnectDelay.current);
          reconnectDelay.current = Math.min(
            reconnectDelay.current * 2,
            10000
          );
        }
      };

      ws.onerror = () => ws.close();
    }

    connect();

    // REST polling fallback (every 2s, only when WS is disconnected)
    // ENG REVIEW FIX: use wsConnectedRef (not wsConnected state) to avoid stale closure
    const fallbackInterval = setInterval(() => {
      if (!wsConnectedRef.current) {
        fetch(apiUrl)
          .then(r => r.json())
          .then(applySnapshot)
          .catch(() => {}); // Silently fail
      }
    }, 2000);

    return () => {
      mounted = false;
      clearInterval(fallbackInterval);
      wsRef.current?.close();
    };
  }, []);

  // applySnapshot: maps SessionStateDto fields to React state
  // applyStateChanged: maps StateChangedPayload to React state
  // (same mapping as useDesk's fetchState and state-changed listener)

  // Return same UseDeskResult shape
  // calibrate, setSitLimit, setStandLimit → no-op (read-only display)
  return { ... };
}
```

### New file: `src/hooks/useDeskAuto.ts` (~15 lines)

```typescript
import { useDesk } from "./useDesk";
import { useRemoteDesk } from "./useRemoteDesk";
import type { UseDeskResult } from "./useDeskTypes";

/** Auto-selects useDesk (Tauri) or useRemoteDesk (browser) based on runtime. */
export function useDeskAuto(): UseDeskResult {
  // This check is stable at module load — safe for hook rules
  if (window.__TAURI_INTERNALS__) {
    return useDesk();
  }
  return useRemoteDesk();
}
```

### Changes to `src/hooks/useWidgetData.ts`

Replace:
```typescript
import { useDesk } from "@/hooks/useDesk";
```
With:
```typescript
import { useDeskAuto } from "@/hooks/useDeskAuto";
```

And change `useDesk()` call to `useDeskAuto()`.

### Changes to `src/types.ts` or `src/global.d.ts`

Add type declaration for `__TAURI_INTERNALS__`:
```typescript
declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}
```

## Key Implementation Details

### State mapping
The `snapshot` event payload is now `RemoteDisplayState` (not just `SessionStateDto`):
```typescript
interface RemoteDisplayState {
  session: SessionStateDto;  // same as get_session_state
  metrics: MetricSnapshot[]; // same as get_dashboard_state.metrics
  today: TodaySummaryDto;    // same as get_today_summary
}
```
Map `payload.session` fields exactly like `useDesk.fetchState()` does:
- `payload.session.state` → `setState()`
- `payload.session.desk_height_cm` → `setDeskHeightCm()`
- `payload.session.current_session_secs` → `setSittingSeconds()`
- etc.

Map `payload.metrics` → `setMetrics()`
Map `payload.today` → `setTodaySummary()`

### Read-only mode
In remote mode, calibrate/setSitLimit/setStandLimit should be no-ops:
```typescript
const calibrate = useCallback(async () => {
  console.warn("Calibration not available in remote display mode");
}, []);
```

### Connection state
Two levels of "connected":
1. `wsConnected` — is the WebSocket to the PC alive?
2. `connected` — is the sensor connected to the PC?

Both should be visible in the UI (T06 handles this).

### Reconnection
Exponential backoff: 1s → 2s → 4s → 8s → 10s (cap).
On successful reconnect, server sends fresh snapshot → state is immediately correct.

## Testing

### Unit tests (`src/hooks/useRemoteDesk.test.ts`):
1. Parses `snapshot` event and updates all state fields (session + metrics + today)
2. Parses `desk:state-changed` event correctly
3. Handles `desk:device-connected` / `desk:device-lost`
4. Ignores `heartbeat` events without error
5. Returns no-op functions for calibrate/setSitLimit/setStandLimit
6. `useDeskAuto` returns `useDesk` when `__TAURI_INTERNALS__` exists
7. `useDeskAuto` returns `useRemoteDesk` when `__TAURI_INTERNALS__` is undefined
8. **Malformed JSON WS message → caught, logged, ignored (no exception)**
9. **Reconnect backoff: verify delay doubles (1s→2s→4s) and caps at 10s**
10. **Handles `desk:daily-reset` event → resets daily counters**

### Mock WebSocket for tests:
Use a mock WebSocket class that simulates server messages.
Existing test infrastructure in `src/test/` can be extended.

## Definition of Done
- [ ] `useRemoteDesk.ts` connects via WS with auto-reconnect
- [ ] `useDeskAuto.ts` auto-selects correct hook
- [ ] `useWidgetData.ts` uses `useDeskAuto()` instead of `useDesk()`
- [ ] State updates work identically to `useDesk()` — including metrics and today summary
- [ ] Read-only mode (no calibrate/settings from remote)
- [ ] **JSON.parse wrapped in try/catch — malformed messages logged and ignored**
- [ ] **Handles `desk:daily-reset` event**
- [ ] Unit tests for all event types + auto-selection + malformed JSON + backoff
- [ ] Existing tests still pass (no breaking changes)
