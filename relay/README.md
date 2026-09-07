# MoveUp relay

## TLDR

A Cloudflare Worker with one Durable Object per desk. The desktop app connects
outbound over WebSocket, phones connect the same way, and the room fans desk
events out to the phones while keeping the newest snapshot in memory. No
ergonomics history is ever stored here.

**This is E022-T02: the room, the router and the protocol plumbing. It is not
deployable yet** — `verifyToken` checks a token's *shape*, not a credential, and
every REST route answers `501`. Credentials, pairing and D1 land in
[T03](../.plan/epics/E022-2026-09-06-cross-device-phone-relay/HANDOFF.md);
command routing lands in T04.

## Commands

```sh
pnpm install --frozen-lockfile   # standalone project, own lockfile
pnpm test                        # every suite
pnpm exec vitest run test/room   # the room only
pnpm typecheck
pnpm dev                         # wrangler dev
```

`relay/` is a **standalone pnpm project**, not a member of the app's workspace:
its own `pnpm-workspace.yaml` anchors the root so pnpm does not walk up to the
app's. Never add it to the app's `packages:` — the relay ships to Cloudflare,
the app ships to a desktop installer, and the trees have nothing in common.

## Layout

| Path | Holds |
|---|---|
| `src/index.ts` | router: `/healthz`, `/app/*`, the WS upgrade, the REST stubs |
| `src/room/desk-room.ts` | the `DeskRoom` Durable Object |
| `src/room/messages.ts` | envelope building and the parse-failure decision |
| `src/room/sockets.ts` | per-socket state, kept in the socket's attachment |
| `src/auth/tokens.ts` | `verifyToken` — the seam T03 fills |
| `src/auth/licenses.ts` | entitlement lookup and the fixed error-code list |
| `src/http/` | responses, health, assets, REST route table |
| `migrations/0001_init.sql` | `licenses`, `desks`, `viewers` — hashes only |
| `test/{room,http,auth,commands}/` | vitest, running inside workerd |

The protocol itself is **not** here. `@app/remote/protocol` (an alias for
`../src/remote/protocol.ts`, E022-T01) is the one schema the desk, the phone and
the relay all validate against; the alias is declared in both `tsconfig.json`
and `vitest.config.ts`.

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
| `4402` | entitlement expired *(T03)* |
| `4403` | token revoked *(T03)* |
| `4409` | replaced by a newer desk connection |
| `1001` | idle past `IDLE_TIMEOUT_MS` — not the client's fault, so reconnecting is right |

A bad payload, an unknown message type or a not-yet-routed `command` gets a
non-fatal `error` frame carrying the offending frame's `id`, and the socket
stays open.

## Configuration

`wrangler.toml` `[vars]`, mirrored in `vitest.config.ts` with shorter timeouts:

| Var | Default | Meaning |
|---|---|---|
| `RELAY_VERSION` | `0.1.0` | reported by `/healthz` so a redeploy is visible |
| `HELLO_TIMEOUT_MS` | `5000` | grace period for `hello` after the upgrade |
| `IDLE_TIMEOUT_MS` | `60000` | silence before the room closes a socket; clients ping every 25 s |

The `ASSETS` binding points at `../dist`, which exists only after a `vite
build`. When it is missing, `/app/*` answers `503 assets_unavailable` rather
than `404` — a missing build and a missing page are different problems.
