---
formatVersion: 1
type: handoff
status: todo
---

# E023 Handoff — relay security hardening

## TLDR

Read only this file and [`PLAN.md`](PLAN.md). Six independent fixes (T01-T06)
run as subagents in one parallel wave, each in its own worktree (`wt-add`),
then T07 verifies. 13 points → subagents, not AO (CLAUDE.md "Próg 13
punktów"). Bump to **0.7.0** before T01 (`.claude/rules/versioning.md`;
if E024 started first and took 0.7.0, take 0.8.0). Deploy nothing.

## Mental model

- Relay code is a Cloudflare Worker in `relay/`. Tests run on miniflare via
  `just relay-test`; `relay/test/helpers.ts:43` injects `CF-Connecting-IP`
  (random UUID by default) — T04 must make `ip: null` omit the header.
- Pairing flow: `relay/src/http/pairing.ts:36-89`. Pre-check at `:61-65`,
  code redeemed in the room at `:67`, viewer row inserted at `:78-84`. T02
  replaces the plain `INSERT` with a conditional one and checks
  `meta.changes === 1`.
- Room frames enter at `relay/src/room/desk-room.ts:165` (`webSocketMessage`)
  and are parsed at `relay/src/room/messages.ts:92`. `raw` is
  `ArrayBuffer | string`: measure bytes (`byteLength` / `TextEncoder`), not
  string length.
- Desk side: `ws_url` (`src-tauri/src/relay_client.rs:82-90`) only rewrites
  schemes. Add a pure `relay_base_allowed(base) -> Result<(), RelayUrlError>`
  next to it and call it where the client decides to start (near `should_run`,
  `:100`), reporting through the existing relay status `last_error`.
- Do not touch: `relay_commands.rs` (replay fix already done), token hashing,
  the command allowlist, `remote_server.rs` (LAN path).

## Tasks

- [ ] **T01** (2, ts-dev, Rust) — F1: `relay_base_allowed`, loopback-only
  `ws://`/`http://`, refusal surfaces as `last_error`. Tests `e023_` in
  `relay_client_tests.rs`: remote `ws://` refused, remote `http://` refused,
  loopback `ws://` allowed, `wss://` allowed, empty → default URL.
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e023_`.
  [task](tasks/T01-desk-refuse-plain-ws.md)
- [ ] **T02** (3, ts-dev) — F2: conditional viewer insert, `409 viewer_limit`
  when `changes === 0`. Test: concurrent pair race in `pair.test.ts`.
  Verify: `just relay-test`. [task](tasks/T02-pair-viewer-cap-atomic.md)
- [ ] **T03** (1, ts-dev) — F3: `type="password"`, `autoComplete="off"` on the
  licence key input. Test in `RemoteSection.test.tsx`.
  Verify: `npx vitest run --config vite.config.ts src/components/settings/RemoteSection`.
  [task](tasks/T03-licence-key-masked.md)
- [ ] **T04** (2, ts-dev) — F4: trust only `CF-Connecting-IP`; missing →
  `500 relay_misconfigured`. Tests in `router.test.ts`; `helpers.ts` gains
  `ip: null`. Verify: `just relay-test`. [task](tasks/T04-client-ip-fail-closed.md)
- [ ] **T05** (2, ts-dev) — F5: `MAX_FRAME_BYTES = 16 * 1024` checked before
  parse, close `1009`. Tests in `handshake.test.ts` (over, exact, empty).
  Verify: `just relay-test`. [task](tasks/T05-room-frame-size-limit.md)
- [ ] **T06** (1, ts-dev) — F6: unknown and expired licence both
  `403 invalid_license`, same message. Test in `register.test.ts`.
  Verify: `just relay-test`. [task](tasks/T06-register-uniform-error.md)
- [ ] **T07** (2, verify + security-reviewer) — full `just check`,
  `just relay-test`; `security-reviewer` confirms F1-F6 closed; update the
  review report statuses to `NAPRAWIONE` with commit shas; append D1-D5 to
  `.plan/decisions.jsonl`; set E023 `done` in `epics/INDEX.md`.
  [task](tasks/T07-verify-and-rereview.md)

## Waves

Fala 1: T01 + T02 + T03 + T04 + T05 + T06 — disjoint files except
`relay/test/helpers.ts` (T04 only; T02/T05/T06 use it read-only). 6 agents ≤ 8.
Fala 2: T07. Order reason: T07 judges the merged result.

Merge each task to `main` as soon as its verify passes (worktrees.md
"Immediate integration rule"). After the wave, run `just relay-test` and
`just check` on `main` before T07.

## Done means

All seven acceptance criteria in PLAN.md hold, every new test was seen failing
against reverted code, the review report shows F1-F6 fixed, INDEX row `done`,
`.plan/HISTORY.md` entry written.
