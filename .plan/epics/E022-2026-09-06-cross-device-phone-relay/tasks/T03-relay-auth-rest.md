---
id: E022-T03
status: done
updated: 2026-09-08
evidence: 264ece8
---
# E022-T03: Relay auth + REST
## Acceptance
migrations/0001_init.sql (licenses(key_hash PK, plan, max_desks, max_viewers, expires_at, created_at), desks(desk_id PK, license_key_hash, token_hash, desk_name, app_version, created_at, last_seen), viewers(viewer_id PK, desk_id, token_hash, device_name, paired_at, last_seen, revoked_at)), the six REST routes from PLAN.md, pairing codes in the DO (this.pairing = {code_hash, expires_at, attempts}), rate limits (per-IP via a small DO or the RateLimiter binding -- pick one, document in relay/README.md), lockout, scripts/mint-license.mjs (prints one key, inserts its hash via wrangler d1 execute). Verify: pnpm --dir relay exec vitest run test/auth test/http.
