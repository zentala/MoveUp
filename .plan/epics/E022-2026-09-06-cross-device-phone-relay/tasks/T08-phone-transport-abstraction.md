---
id: E022-T08
status: pending
---
# E022-T08: Phone transport abstraction
## Acceptance
src/remote/transports/{types.ts, lan.ts, relay.ts, index.ts} (Transport { connect, close, sendCommand?, capabilities, onMessage, onStatus }), envelope parse via protocol.ts, stored record helpers src/remote/storage.ts (moveup.relay.v1, try/catch), useRemoteDesk.ts consumes a transport and exposes capabilities + deskOnline, ConnectionOverlay.tsx fourth state (data-testid="conn-overlay-desk-offline"), useDesk.ts gets the two new constant fields, useDeskTypes.ts updated. Verify: npx vitest run --config vite.config.ts src/remote src/hooks/useRemoteDesk src/components/ConnectionOverlay.
