---
id: E009-T07
epic: E009
status: pending
created: 2026-03-24
branch: feat/E009-T07-e2e-test
depends_on: [E009-T06]
---
# E009-T07: End-to-End Testing & Documentation

## What
Verify the full flow works end-to-end, write integration tests,
and create user documentation for phone setup.

## End-to-End Test Scenarios

### Scenario 1: Basic connection
1. Start app: `pnpm tauri:dev`
2. Open `http://localhost:3390/display` in browser
3. Verify: dashboard shows, state updates in real-time
4. Inject a reading: `inject_reading` via another browser tab
5. Verify: phone display updates within 1s

### Scenario 2: State transitions
1. App running in demo/mock mode
2. Phone connected to `/display`
3. State cycles through sitting→standing→away
4. Verify: phone UI reflects each transition (colors, progress, KPIs)

### Scenario 3: Reconnection
1. Phone connected and showing dashboard
2. Kill the desktop app process
3. Verify: phone shows "Reconnecting..."
4. Restart desktop app
5. Verify: phone reconnects and shows current state

### Scenario 4: Multiple clients
1. Open `/display` in two browser tabs simultaneously
2. Verify: both receive same events, no conflicts

### Scenario 5: Vite proxy failure (dev mode only)
1. Stop Vite dev server (kill `pnpm dev`)
2. Keep Tauri running (`pnpm tauri:dev` — Tauri process stays up)
3. Open `http://localhost:3390/display` in browser
4. Verify: helpful error page shown ("Vite dev server not running. Start with: pnpm dev")
5. Start Vite again, refresh → dashboard loads

## Integration Tests

### New file: `tests/integration/remote-display.test.ts`

```typescript
import { describe, it, expect } from "vitest";

describe("Remote Display Server", () => {
  it("GET /display/api returns valid session state", async () => {
    const res = await fetch("http://localhost:3390/display/api");
    expect(res.ok).toBe(true);
    const data = await res.json();
    expect(data).toHaveProperty("state");
    expect(data).toHaveProperty("sitting_seconds");
    // NOTE: REST endpoint returns SessionStateDto only (not full RemoteDisplayState)
    // Full state is only available via WS snapshot. This is acceptable for debug endpoint.
  });

  it("GET /display serves HTML", async () => {
    const res = await fetch("http://localhost:3390/display");
    expect(res.ok).toBe(true);
    const text = await res.text();
    expect(text).toContain("<!DOCTYPE html>");
  });

  it("WebSocket connects and receives full snapshot", async () => {
    // Use ws library or native WebSocket
    const ws = new WebSocket("ws://localhost:3390/display/ws");
    const firstMessage = await new Promise((resolve) => {
      ws.onmessage = (e) => resolve(JSON.parse(e.data));
    });
    expect(firstMessage.event).toBe("snapshot");
    // Verify full RemoteDisplayState payload
    expect(firstMessage.payload).toHaveProperty("session");
    expect(firstMessage.payload).toHaveProperty("metrics");
    expect(firstMessage.payload).toHaveProperty("today");
    expect(firstMessage.payload.session).toHaveProperty("state");
    ws.close();
  });

  it("WebSocket receives heartbeat within 7 seconds", async () => {
    // REVIEW FIX: use 7s (not 6s) for timing margin to avoid flakiness
    const ws = new WebSocket("ws://localhost:3390/display/ws");
    const messages: any[] = [];
    ws.onmessage = (e) => messages.push(JSON.parse(e.data));
    await new Promise((r) => setTimeout(r, 7000));
    const hasHeartbeat = messages.some((m) => m.event === "heartbeat");
    expect(hasHeartbeat).toBe(true);
    ws.close();
  });
});
```

**Note:** These tests require the app to be running (`pnpm tauri:dev`),
same as existing integration tests.

## Documentation

### New file: `docs/REMOTE_DISPLAY.md`

Contents:
1. **What it does** — turn a phone into a desk dashboard
2. **Requirements** — phone + WiFi + kiosk browser app
3. **Setup steps:**
   - Install Fully Kiosk Browser on Android (free version)
   - Find PC's local IP: `ipconfig` on Windows → WiFi adapter → IPv4
   - In Fully Kiosk: set URL to `http://<PC-IP>:3390/display`
   - Configure: fullscreen, landscape, auto-start, screen always on
4. **Firewall setup** — allow port 3390 inbound on Windows Firewall
5. **Troubleshooting:**
   - "Cannot connect" → check same WiFi, check firewall, check IP
   - "Reconnecting..." → desktop app not running or WiFi dropped
   - "Sensor disconnected" → check USB cable on PC
6. **Configuration** — `DESK_REMOTE_PORT` env var for custom port

### Fully Kiosk Browser recommended settings:
- Start URL: `http://<IP>:3390/display`
- Orientation: Landscape
- Screen always on: Yes
- Fullscreen mode: Yes
- Autostart on boot: Yes (optional)
- Status bar: Hidden
- Navigation bar: Hidden

## Changes to CLAUDE.md

Add a brief section about the remote display:
```markdown
## Remote Display (Phone Dashboard)
- Embedded HTTP+WS server on `:3390` (configurable via `DESK_REMOTE_PORT`)
- Same React UI served to browsers; `useDeskAuto()` hook selects WS or Tauri IPC
- See `docs/REMOTE_DISPLAY.md` for phone setup instructions
- Key files: `remote_server.rs`, `ws_broadcaster.rs`, `useRemoteDesk.ts`
```

## Definition of Done
- [ ] All 5 e2e scenarios pass manually (including Vite proxy failure)
- [ ] Integration tests in `tests/integration/remote-display.test.ts`
- [ ] `docs/REMOTE_DISPLAY.md` written with setup instructions
- [ ] CLAUDE.md updated with remote display section
- [ ] **`.arch/ARCHITECTURE.md` updated** — add remote_server, ws_broadcaster to component map
- [ ] **`.arch/ARCHITECTURE.md` data flow updated** — add WS broadcast path
- [ ] PROJECT.xml updated if maintained
