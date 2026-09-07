# ADR 022: The phone relay runs on Cloudflare Workers + Durable Objects, and holds no history

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E022 (cross-device phone relay) — tasks T02, T03, T04, T05, T12, T16
- **Related**: [ADR 001](001-remote-display-web-kiosk.md) (LAN kiosk — extended, not superseded),
  [ADR 005](005-open-core-software-model.md) (the relay is the paid half),
  [ADR 017](017-ts-rs-for-rust-ts-codegen.md) (protocol DTOs),
  [ADR 018](018-pm3-app-ownership-split.md) (unaffected — the relay is not a PM3 service),
  [ADR 023](023-pairing-code-device-token-auth.md) (who is allowed to connect),
  [E022](../../.plan/epics/E022-2026-09-06-cross-device-phone-relay/PLAN.md)

## Context

ADR 001 put the phone dashboard on an embedded HTTP + WebSocket server bound to
`0.0.0.0:3390`. That works, and it stops working the moment the phone leaves the
flat. Three facts follow from the binding, and they are separate problems:

- **The phone must share the PC's network.** `docs/REMOTE_DISPLAY.md` used to
  open by telling the user to run `ipconfig` and add a Windows Firewall rule.
  That is a setup story for a homelab, not for a paid feature.
- **When the PC is off there is nothing to read.** The phone showed
  "Reconnecting…" forever, because the only holder of desk state was the
  process that had just gone away.
- **Nothing can travel toward the PC.** Making the LAN socket writable would
  mean adding a write path to a surface with no authentication at all — the
  worst possible order in which to add the app's first remote control.

So the epic needed a third party that is always up, that both sides connect
*out* to, and that can hold the last snapshot while the desk sleeps. The
question this ADR answers is what that third party is.

## Decision

A **Cloudflare Worker** at `relay.desk.zentala.io`, with **one Durable Object
per desk** (`DeskRoom`) and **D1** for credential rows.

- `DeskRoom` accepts one desk socket and up to `max_viewers` viewer sockets
  through the **WebSocket Hibernation API** (`ctx.acceptWebSocket`, tags `desk`
  and `viewer:<id>`), so an idle desk costs no duration billing. It fans desk
  `event` messages out to viewers, routes a viewer `command` to the desk and the
  matching `command_result` back to the one viewer that asked, and answers
  `ping`.
- **The room's only state is the latest snapshot, in memory.** No history, no
  ergonomics rows, no config store in the cloud. A viewer that connects while
  the desk is offline gets that snapshot with `desk_online: false` — stale data
  that says it is stale, instead of a spinner. When the DO is evicted the
  snapshot is gone and `welcome.snapshot` is `null`; that is the documented
  behaviour, not a bug to paper over.
- **D1 holds `licenses`, `desks`, `viewers`** — hashes and metadata, never a
  plaintext token and never a reading (see ADR 023).
- **The desktop connects outbound only.** `relay_client.rs` is one more
  `subscribe()` on the existing `ws_tx` broadcast channel
  (`ws_broadcaster.rs`), so the relay receives exactly what a LAN client
  receives, already serialized. No inbound port, no firewall rule, no port
  forward.
- **One envelope on both transports.** `{v, type, id, ts, payload}`, defined
  once in `remote_protocol.rs`, ts-rs exported, mirrored in zod as
  `src/remote/protocol.ts`, and asserted by the JSON fixtures in
  `tests/fixtures/relay-protocol/`. The LAN server wraps its events in the same
  envelope, so one React client speaks to both.
- **The relay is not a PM3 service.** Its lifecycle is `wrangler deploy` behind
  `consent-broker`, its health is `GET /healthz`, and the desktop treats it as
  unreliable by design: 25 s ping, 60 s pong timeout, 1 s → 60 s backoff with
  ±20 % jitter.

## Alternatives

| | A. Self-hosted axum relay on `pve01.lan` | B. Cloudflare Workers + Durable Objects (chosen) | C. Managed pub/sub (Ably, Pusher) |
|---|---|---|---|
| Summary | Reuse the `remote_server.rs` shape as a standalone binary in an LXC, port-forwarded from the router | One Worker; one DO per desk holds the sockets and the last snapshot; D1 holds hashed credentials | A third-party channel service; the desk publishes, the phone subscribes |
| Effort | M | M | S |
| Risk | High — uptime, NAT, autosleep, one home IP exposed to customers | Medium — a new runtime and the hibernation API to learn | Low technically, high commercially |
| Pros | Same language as the app, no cloud bill, full control | Global edge; hibernation makes idle desks free; D1 is SQLite, which the app already speaks; `wrangler dev` runs the whole thing locally for tests; the vision doc's later cloud work (history sync) shares the stack | Fastest path to a demo |
| Cons | A paid feature on a home server behind NAT; `my-severs.md` reserves self-hosting for Paweł's own services; every outage is a refund | Vendor-specific DO API; a TypeScript relay beside a Rust app | The vendor's auth model replaces ours, and auth is the decision this epic is about; per-message pricing against a 1 Hz snapshot stream |

**Chosen: B.** A is cheaper to build and ruinous to operate: `pve01.lan` sits
behind NAT with autosleep and no SLA, and a paying user's phone cannot depend on
that. C hands the pairing model to a vendor. B's vendor risk is contained on
purpose — the protocol is plain JSON over WebSocket and the fixtures are the
contract, so a self-hosted relay stays possible later as a *second
implementation of the same contract*, which is exactly how the LAN path already
relates to it.

## Consequences

- **A new top-level `relay/` directory**, a standalone pnpm project with its own
  lockfile. The root workspace declares no `packages:`, and `.giter.yaml`'s
  `worktree.guards` now covers `relay/node_modules`. It imports the shared
  schema from `src/remote/protocol.ts` through a tsconfig path alias, so the
  envelope exists once.
- **A cloud bill and a deploy gate.** Deploying to `relay.desk.zentala.io` is
  outward-facing and goes through `consent-broker`; `just relay-deploy` is the
  recipe, `/healthz` and `scripts/relay-e2e.mjs` are the gate.
- **The desktop must survive a dead relay in silence.** Terminal close codes
  (`4402` unentitled, `4403` revoked, `4409` replaced) stop the reconnect loop
  and surface a named state in Settings — a dead credential is never retried
  forever, and "relay down" never blocks the ergonomics engine, which does not
  know the relay exists.
- **The LAN path stays, read-only, behind `remote_lan_enabled`.** The free tier
  keeps working offline; the unauthenticated surface never gains a write.
- **New dependencies**: Rust `tokio-tungstenite` (rustls-webpki-roots),
  `keyring`; TypeScript `qrcode`, `zod`; in `relay/` only, `wrangler`,
  `@cloudflare/workers-types`, `@cloudflare/vitest-pool-workers`.
- **Privacy is a property of the design, not a promise.** The relay logs no
  payloads and stores no history, so there is no cloud copy of the ergonomics
  record to leak, subpoena or export. Stated in
  [`docs/PRIVACY.md`](../../docs/PRIVACY.md).
