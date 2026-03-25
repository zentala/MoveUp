---
id: E009-T06
epic: E009
status: done
completed: 2026-03-25
created: 2026-03-24
branch: feat/E009-T06-reconnect-ui
depends_on: [E009-T04, E009-T05]
---
# E009-T06: Connection Status & Reconnect UI

## What
Show connection status on the remote display — the user needs to know if:
1. WebSocket to PC is connected (network level)
2. Sensor is connected to PC (device level)
3. Reconnection is in progress

## Context
The phone sits on the desk as an always-on display. WiFi may drop,
PC may restart, sensor may disconnect. The display must communicate
its state clearly without user intervention.

## Implementation

### Connection states
```
WS connected + sensor connected  → normal UI (green dot or no indicator)
WS connected + sensor lost       → "Sensor disconnected" banner
WS disconnected (reconnecting)   → full-screen "Reconnecting..." overlay
WS disconnected (failed)         → "Cannot reach PC" with IP/port info
```

### New component: `src/components/ConnectionOverlay.tsx` (~60 lines)

Only rendered in remote display mode. Shows:

**Reconnecting state:**
```
┌────────────────────────────────┐
│                                │
│     ⟳ Reconnecting...         │
│     Last connected: 14:32      │
│                                │
└────────────────────────────────┘
```
Semi-transparent overlay on top of the (stale) dashboard data.
Dashboard remains visible underneath (user can still see last known state).

**Sensor disconnected:**
Small banner at top or bottom:
```
┌────────────────────────────────┐
│ ⚠ Sensor disconnected          │
├────────────────────────────────┤
│ [normal dashboard content]     │
└────────────────────────────────┘
```

### Integration

In `App.tsx` (or the widget wrapper), when in remote mode:
```tsx
{isRemote && <ConnectionOverlay wsConnected={wsConnected} sensorConnected={connected} />}
```

`wsConnected` comes from `useRemoteDesk()` — needs to be added to `UseDeskResult`:
```typescript
// Add to UseDeskResult interface:
wsConnected?: boolean; // undefined in Tauri mode, true/false in remote mode
```

### Styling
- Use existing design system variables (`--panel-bg`, `--ink-*`, `--beam-*`)
- Reconnecting overlay: `background: rgba(0,0,0,0.7)`, centered text
- Sensor banner: subtle, non-intrusive, matches app style

## Testing

### Unit tests:
1. `ConnectionOverlay` renders nothing when both connected
2. Shows reconnecting overlay when `wsConnected=false`
3. Shows sensor banner when `wsConnected=true, sensorConnected=false`
4. Shows reconnecting (not sensor banner) when both disconnected

### Manual testing:
1. Kill the desktop app → phone shows "Reconnecting..."
2. Restart desktop app → phone auto-reconnects, overlay disappears
3. Disconnect sensor (unplug USB) → phone shows "Sensor disconnected"
4. Reconnect sensor → banner disappears

## Definition of Done
- [x] `ConnectionOverlay` component exists and is rendered in remote mode
- [x] Reconnecting state shows semi-transparent overlay
- [x] Sensor disconnected shows subtle banner
- [x] Normal state shows no extra UI
- [x] `wsConnected` exposed in `UseDeskResult`
- [x] Unit tests pass
- [ ] Visual verification on phone-sized viewport
