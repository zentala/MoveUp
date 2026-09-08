---
id: E022-T02
status: done
updated: 2026-09-08
evidence: 494f9f6
---
# E022-T02: Relay Worker core
## Acceptance
Create relay/ per Mental model (package.json, pnpm-lock.yaml, wrangler.toml, tsconfig.json, vitest.config.ts, migrations/0001_init.sql, src/index.ts router, src/room/desk-room.ts, src/room/messages.ts, src/auth/tokens.ts, src/auth/licenses.ts, src/http/*.ts, scripts/mint-license.mjs, test/{room,auth,http,commands}/*.test.ts). DeskRoom DO using the WebSocket Hibernation API (acceptWebSocket, webSocketMessage, webSocketClose, tags desk/viewer:<id>), hello timeout via DO alarm, welcome, event fan-out, desk_status, last snapshot in this.snapshot, ping/pong + 60s idle close, all close codes, /healthz, /app/* assets, router stubs for the T03 REST paths returning 501. Auth in this task is a verifyToken hook that T03 fills -- but the 4401 path must already exist and be tested. Verify: pnpm --dir relay install --frozen-lockfile && pnpm --dir relay exec vitest run test/room.
