# MoveUp relay

## TLDR

A Cloudflare Worker with one Durable Object per desk. The desktop app connects
outbound over WebSocket, phones connect the same way, and the room fans desk
events out to the phones while keeping the newest snapshot in memory. Devices
authenticate with tokens minted from a licence key; D1 stores their hashes and
nothing else. No ergonomics history is ever stored here.

**State: T02 (room, router, protocol), T03 (credentials, pairing, REST) and T04
(command routing) are in.** A phone can pair, watch, and send the three
allowlisted commands.

## Commands

```sh
pnpm install --frozen-lockfile   # standalone project, own lockfile
pnpm test                        # every suite
pnpm exec vitest run test/auth test/http   # credentials and REST
pnpm typecheck
pnpm dev                         # wrangler dev

# one licence key, its hash stored in the local D1; stdout is the key alone
node scripts/mint-license.mjs --local --max-viewers 5
```

`relay/` is a **standalone pnpm project**, not a member of the app's workspace:
its own `pnpm-workspace.yaml` anchors the root so pnpm does not walk up to the
app's. Never add it to the app's `packages:` — the relay ships to Cloudflare,
the app ships to a desktop installer, and the trees have nothing in common.

## Layout

| Path | Holds |
|---|---|
| `src/index.ts` | router: `/healthz`, `/app/*`, the WS upgrade, the REST routes |
| `src/room/desk-room.ts` | the `DeskRoom` Durable Object — protocol only |
| `src/room/room-admin.ts` | pairing and socket revocation, as functions |
| `src/room/pairing.ts` | the code/lockout state machine, clock injected |
| `src/room/rpc.ts` | the private `/__room/*` channel between REST and the room |
| `src/room/commands.ts` | viewer -> desk command routing and every refusal it can answer with |
| `src/room/command-table.ts` | the reply table (`command_id` -> viewer, 30 s) and the per-viewer throttle |
| `src/room/messages.ts` | envelope building and the parse-failure decision |
| `src/room/sockets.ts` | per-socket state, kept in the socket's attachment |
| `src/auth/tokens.ts` | minting, hashing and `verifyToken` |
| `src/auth/licenses.ts` | entitlement lookup and the fixed error-code list |
| `src/http/` | responses, body reading, rate limiting, the six REST routes |
| `migrations/0001_init.sql` | `licenses`, `desks`, `viewers` — hashes only |
| `scripts/mint-license.mjs` | issue a key, store its hash via `wrangler d1 execute` |
| `test/{room,http,auth,commands}/` | vitest, running inside workerd against a real D1 |

The protocol itself is **not** here. `@app/remote/protocol` (an alias for
`../src/remote/protocol.ts`, E022-T01) is the one schema the desk, the phone and
the relay all validate against; the alias is declared in both `tsconfig.json`
and `vitest.config.ts`.

## Credentials

Three secrets, three lifetimes:

| Secret | Shape | Lives | Stored as |
|---|---|---|---|
| Licence key | `mu_lic_<43 base64url>` | until it expires | `sha256` in `licenses.key_hash` |
| Desk token | `mu_d_<43 base64url>` | until the desk is deleted | `sha256` in `desks.token_hash` |
| Viewer token | `mu_v_<43 base64url>` | until revoked | `sha256` in `viewers.token_hash` |
| Pairing code | 8 chars, `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` | 5 minutes | **nowhere** — the room's memory |

A stored hash is never used as a SQL selector: rows are fetched by `desk_id` and
their hashes compared in constant time, and the viewer scan does not break on a
match. `test/auth/tokens.test.ts` dumps every table and asserts no plaintext
token appears in any row — and asserts the row count, so an empty database
cannot pass that check by having nothing to find.

A relay with no `DB` binding refuses **every** credential. It does not fall back
to a shape check: a well-formed string is not a credential.

## Rate limits

Two unauthenticated routes are throttled per client address: `POST /v1/desks/register`
and `POST /v1/pair`, both **5 per minute**. On top of that the room locks pairing
for **15 minutes after 10 wrong codes**, which is per desk rather than per address
and therefore survives an attacker rotating IPs.

**A Durable Object, not the platform `RateLimiter` binding.** The binding is
cheaper and its counters are per-colo, but it cannot be exercised under
`vitest-pool-workers`: a test could assert only that a limit was configured, not
that it holds. Brute-force protection on a pairing code is the one thing in this
relay that has to be provably tested, so the limiter is `src/http/rate-limit.ts`
— one Durable Object per `<route>:<ip>` bucket, whose whole state is the list of
hits still inside the window.

That window is memory only, so an eviction forgives a caller's history. For a
throttle the cost of a forgiven minute is one extra burst; the alternative is a
storage write per request. The pairing **lockout** is different and deliberately
so: it is held by the room for the whole of the room's life, and issuing a fresh
code does not clear it.

Commands are throttled separately, at **10 per minute per viewer**, and refused
with a non-fatal `error{rate_limited}` rather than a close — a phone that taps
too fast should slow down, not be logged out. That counter is a third mechanism
again: it rides in the socket's own attachment (`src/room/commands.ts`), so a
Durable Object eviction between two commands cannot forgive a burst. The check
runs *before* the allowlist, so an invalid command name still costs a token.

## Command routing

A `command` from a viewer is validated against the shared allowlist, stamped
with the sender's `viewer_id`, and forwarded to the desk socket **with its own
`id` and `ts` untouched** — the id is what the reply is matched on, and the
timestamp is what lets the desk refuse a command that sat in a tunnel for a
minute.

The room keeps a bounded table of forwarded commands, `command_id -> viewer_id`,
for 30 seconds. A `command_result` from the desk is delivered to that one viewer
and to nobody else; broadcasting it would tell every paired phone what another
phone just did.

Everything is answered, because a phone that gets nothing back cannot tell a
slow command from a lost one:

| Situation | Answer to the sender |
|---|---|
| no desk connected | `command_result{ok:false, error.code:"desk_offline"}` |
| unknown name | `error{unknown_command}` |
| bad or missing arguments | `error{bad_args}` |
| over the per-viewer limit | `error{rate_limited}` |
| a desk sending `command`, or a viewer sending `command_result` | `error{forbidden}` |
| a result whose `command_id` expired or was already answered | `error{unknown_command_id}` to the desk |
| a result whose viewer left first | `error{viewer_gone}` to the desk |

Arguments are validated here **and** again on the desk. That is not redundancy:
the relay's copy is what refuses a bad command without waking the PC, and the
desk's copy is what keeps the relay untrusted (ADR 023).

## REST routes

All JSON. Errors are `{ error: { code, message } }` with `code` from the list in
`auth/licenses.ts`.

| Method + path | Auth | In → out |
|---|---|---|
| `POST /v1/desks/register` | none | `{license_key, desk_name?, app_version?}` → `201 {desk_id, desk_token, plan, expires_at}` |
| `POST /v1/desks/{id}/pairings` | desk | `{}` → `201 {code, expires_at}` |
| `POST /v1/pair` | none | `{desk_id, code, device_name?}` → `201 {viewer_id, viewer_token, desk_name}` |
| `GET /v1/desks/{id}/viewers` | desk | → `200 [{viewer_id, device_name, paired_at, last_seen, online}]` |
| `DELETE /v1/desks/{id}/viewers/{viewerId}` | desk | → `204`, socket closed 4403 |
| `DELETE /v1/desks/{id}` | desk | → `204`, every socket closed, every row gone |

Desk routes take `Authorization: Bearer mu_d_…`. Every failure on a desk route is
`401 unauthorized` whatever the cause — telling an unauthenticated caller *why*
its token failed tells it which desk ids exist. The distinction is kept where it
matters, on the socket, where the close code drives the client's retry decision.

`/__room/*` is the room's private channel (`src/room/rpc.ts`). Durable Object
stubs are not addressable from the internet, and the Worker refuses to become the
bridge that would make them so: a public request for that prefix is `404`.

## Two design points worth knowing before you edit the room

**Socket state lives in the attachment, not in accept-time tags.** The room uses
the WebSocket Hibernation API, so the object can be evicted between two messages
and rebuilt with its sockets intact — and only `serializeAttachment` survives
that. A socket's role could not be a tag anyway: tags are fixed at
`acceptWebSocket`, and the role arrives one round trip later in `hello`. Putting
the role in the upgrade URL to make it taggable would have leaked connection
metadata into a place the protocol deliberately keeps empty, so role lookup is a
filter over `getWebSockets()`. One desk and a few viewers make that free.

**The snapshot is memory only.** It dies with the object and is never written to
storage. A viewer that reconnects after an eviction gets `snapshot: null` until
the desk sends the next one, which it does on every `welcome`. That is the whole
of "cross-device state sync" for v1, and it is what lets `docs/PRIVACY.md` say
the relay stores nothing about a person's day.

## Close codes

From `@app/remote/protocol`; never written as literals.

| Code | Sent when |
|---|---|
| `4400` | unparseable frame, wrong protocol version, or not an envelope |
| `4401` | no `hello` within `HELLO_TIMEOUT_MS`, a frame before `hello`, or a token that fails verification |
| `4402` | the desk's licence is missing or expired |
| `4403` | the viewer was revoked, or the desk disabled the relay |
| `4409` | replaced by a newer desk connection |
| `1001` | idle past `IDLE_TIMEOUT_MS` — not the client's fault, so reconnecting is right |

A bad payload, an unknown message type or a refused `command` gets a non-fatal
`error` frame carrying the offending frame's `id`, and the socket stays open.

## Configuration

`wrangler.toml` `[vars]`, mirrored in `vitest.config.ts` with shorter timings:

| Var | Default | Test | Meaning |
|---|---|---|---|
| `RELAY_VERSION` | `0.1.0` | `test` | reported by `/healthz` so a redeploy is visible |
| `HELLO_TIMEOUT_MS` | `5000` | `250` | grace period for `hello` after the upgrade |
| `IDLE_TIMEOUT_MS` | `60000` | `700` | silence before the room closes a socket |
| `PAIRING_TTL_MS` | `300000` | `1000` | how long a pairing code lives |
| `PAIRING_LOCKOUT_MS` | `900000` | `2000` | how long pairing stays locked |
| `PAIRING_MAX_ATTEMPTS` | `10` | `10` | wrong codes before the lockout |

Bindings: `DESK_ROOM` and `RATE_LIMITER` (Durable Objects), `DB` (D1),
`ASSETS` (the viewer build at `../dist`, which exists only after a `vite build` —
when it is missing `/app/*` answers `503 assets_unavailable` rather than `404`,
because a missing build and a missing page are different problems).

`wrangler.toml` carries a placeholder `database_id`. It is filled from
`wrangler d1 create moveup-relay` at deploy time (T16); until then `wrangler
deploy` fails loudly rather than shipping a relay whose credentials cannot be
checked.
