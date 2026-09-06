---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 111
agent: ts-dev
wave: 5
parallel: []
depends-on: [E017, E018, E019]
blocked-by: ""
---

# E022 — Cross-device phone relay (Pro feature)

Research brief: [`../../reports/2026-09-06-brief-cross-device-relay.md`](../../reports/2026-09-06-brief-cross-device-relay.md).
Vision source: [`../../vision/2026-03-25-premium-tier-definition.md`](../../vision/2026-03-25-premium-tier-definition.md)
§Epic E014 (stale numbering — this is the real epic). Original LAN decision:
[ADR 001](../../../.arch/ADR/001-remote-display-web-kiosk.md). Board deck
(Polish): [`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

Today the phone dashboard works only on the same Wi-Fi, without any
authentication, and cannot change anything. This epic adds a **cloud relay**
so the phone shows the desk from anywhere, a **pairing-code + device-token
auth model** (the app's first auth surface — no accounts, no passwords, no
e-mail), and a **scoped two-way control set** (acknowledge an alert, change
sit/stand limits, switch profile). The relay is a Cloudflare Worker with one
Durable Object per desk, holding only the latest snapshot in memory — no
ergonomics history ever leaves the PC. The desktop connects **outbound only**
(no firewall rule, no port forward). The LAN path stays as the free tier,
read-only, behind a settings toggle, and adopts the same versioned message
envelope so one React client speaks to both. 16 tasks, 111 points, 6 waves,
three parallel chains (relay Worker / desktop Rust / phone React). Security
review (pipeline step 5) is a named task and cannot be skipped. Pro
entitlement is a license key checked by the relay; minting keys is a script
until billing (vision E010-T04) exists.

## Problem

From the brief, verified against source:

- **LAN-only.** `remote_server.rs:44-67` binds `0.0.0.0:3390`; the phone must
  be on the same network. `docs/REMOTE_DISPLAY.md` tells the user to find
  their PC's IP and open a firewall port. The vision doc's whole T01-T03 is
  "works outside LAN".
- **Zero authentication.** `ws_handler` (`remote_server.rs:85-98`) checks only
  the client count. `api_handler` (`:151-154`) answers anyone. A roommate,
  guest or office colleague on the same Wi-Fi sees the ergonomics data.
- **One-way only.** `DisplayEvent` (`ws_broadcaster.rs:30-54`) has no inbound
  variant; the receive loop ignores everything (`remote_server.rs:137-142`,
  `Some(Ok(_)) => {}`); `useRemoteDesk.ts:143-145` makes `calibrate`,
  `setSitLimit`, `setStandLitmit` no-ops that only `console.warn`.
- **No cross-device state.** When the PC is off, the phone shows
  "Reconnecting..." forever (`ConnectionOverlay.tsx:45-59`); there is nowhere
  to read the last known state from.
- **Connection story is "type an IP".** Not a paid-feature experience.

## Decisions and ADRs

Read before planning: `.plan/decisions.jsonl` (E015-D1..E020-D3, none touch
the remote path), ADR 001 (web kiosk — kept, extended, not superseded), ADR
005 (open core — the relay is exactly the "cloud is closed/paid" half), ADR
017 (ts-rs codegen — reused for protocol DTOs), ADR 018 (PM3 ownership —
unaffected, the relay is not a local service), ADR 019 (release store —
unaffected).

Five decisions, each with the alternatives weighed in the next section:

- **D1 — Relay runs on Cloudflare Workers + Durable Objects (+ D1 for
  credentials).** New ADR 020. The vision doc named this; it still holds
  eleven months later because the relay is a *customer-facing* service, and
  `knowdlege/my-severs.md` reserves self-hosting for Paweł's own services
  ("self-host by default; Cloudflare DNS + Workers + D1 + R2 + Pages already
  used, independently of own infra"). `pve01.lan` sits behind NAT with
  autosleep and no SLA — a paying user's phone cannot depend on it.
- **D2 — Auth is pairing code → long-lived device tokens, no accounts.** New
  ADR 021. The desk registers once with a license key and receives a
  `desk_token`; each phone pairs once with an 8-character code shown on the
  desktop (QR or typed) and receives a `viewer_token`. Tokens are opaque
  random bytes; the relay stores only SHA-256 hashes. Revocation is a list in
  desktop Settings. No e-mail, no password, no user table — the first auth
  surface of this app is deliberately the smallest one that is still
  revocable. Accounts (vision E010-T01 magic link) can be layered on later by
  making a license key the thing an account owns.
- **D3 — Two-way control v1 is exactly three commands:** `ack_alert`,
  `set_limits {sit_min?, stand_min?}`, `switch_profile {kind, name}`. Each
  maps to a function that already exists and already validates its input
  (`AlertPopup::dismiss` + `CommunicationPolicy::dismiss`;
  `commands_config.rs:78-102` `set_session_limit`/`set_stand_limit` via
  `SessionManager::set_*_minutes`; `commands_profiles.rs:161-206`
  `switch_*_profile`). Not in v1: calibration (physical, needs the person at
  the desk), `save_settings` (arbitrary `AppConfig` write), profile editing,
  DB restore. Commands travel only over the relay path; the LAN path stays
  read-only (see D4).
- **D4 — LAN mode stays as the free tier, read-only, behind a toggle, on the
  same envelope.** `remote_server.rs` is modified in place, not forked: it
  wraps `DisplayEvent` in the v1 envelope, keeps ignoring inbound messages,
  and gains `AppConfig.remote_lan_enabled` (default `true` — today's users
  keep working). The zero-auth-on-LAN gap is *narrowed* (toggle) not closed;
  closing it (LAN pairing) is filed as a follow-up in `.plan/BACKLOG.md`,
  because the Pro story is "anywhere + control + revocable devices", and the
  free story is "your own Wi-Fi, view only", which the tier table already
  says (`premium-tier-definition.md:30`).
- **D5 — Relay hostname is `relay.desk.zentala.io`**, a subdomain of the
  product domain (`~/code/desk.zentala.io/CNAME` → `desk.zentala.io`). It is
  a constant `RELAY_DEFAULT_URL` overridable in Settings, so staging and
  self-hosted relays work without a rebuild. Deploying to that hostname is an
  outward-facing action and goes through `consent-broker` (T16).

Decisions D1-D5 are appended to `.plan/decisions.jsonl` in T14; ADR 020 and
021 are written there too. ADR numbers assume 019 is the highest at plan time
(`ls .arch/ADR/` on 2026-09-06). If the sibling epic E021 (planned in
parallel today) claims 020 first, T14 takes the next free integers and fixes
cross-references — it does not fight over the number (same caveat E020 used).

## Alternatives

### Relay infrastructure (D1)

| | A. Minimum — self-hosted axum relay on `pve01.lan` | B. Target — Cloudflare Workers + Durable Objects (this plan) | C. Managed pub/sub (Ably / Pusher) |
|---|---|---|---|
| Summary | Reuse `remote_server.rs`'s axum shape as a standalone relay binary in an LXC, port-forwarded from `router.lan` | One Worker; one Durable Object per desk holds sockets + last snapshot; D1 holds hashed credentials | Third-party channel service; desk publishes, phone subscribes |
| Effort | M | M | S |
| Risk | H (uptime, NAT, autosleep, single home IP exposed to customers) | M (new runtime, DO hibernation API to learn) | L technically, H commercially (per-message pricing, vendor lock, their auth model, not ours) |
| Pros | Same language as the app; zero cloud bill; full control | Global edge, WebSocket hibernation makes idle desks free, D1 is SQLite (matches the app), `wrangler dev` runs the whole thing locally for tests; vision E010 already picks this stack, so history/sync later share it | Fastest to a demo |
| Cons | A paid feature on a home server behind NAT; `my-severs.md` explicitly reserves self-hosting for own services; every outage is a refund | Vendor-specific DO API; TypeScript relay next to a Rust app (mitigated: protocol fixtures shared by both) | Cannot enforce our pairing model; message cost scales with the 1 Hz snapshot stream |
| Reuses | `remote_server.rs`, `ws_broadcaster.rs` | Cloudflare account and Pages already in use (`desk.zentala.io`); ADR 017 codegen for DTOs; `ws_broadcaster` channel as the desktop's event source | nothing |

**Recommendation: B.** A is not cheaper once "customer-facing" is taken
seriously — it is cheaper to *build* and ruinous to *operate*. C hands the
auth model to a vendor, and auth is the decision this epic is actually about.
B's vendor risk is contained: the protocol is plain JSON over WebSocket and
the fixtures in `tests/fixtures/relay-protocol/` are the contract, so a
self-hosted relay (A) remains possible later as a *second implementation of
the same contract*, which is how the LAN path already relates to it.

### Authentication model (D2)

| | A. Minimum — pairing code + device tokens (this plan) | B. Target — accounts (magic-link e-mail, JWT) | C. Long-lived shared secret typed on both sides |
|---|---|---|---|
| Summary | Desk registers with license key → `desk_token`; phone enters a short-lived code → `viewer_token`; both revocable | Vision E010-T01: user logs in on desktop and phone; tokens are JWTs issued per login | One passphrase configured on the desktop, typed on the phone, used as bearer |
| Effort | M | L (e-mail delivery, user table, consent screens, GDPR data-subject flows) | S |
| Risk | L | M | H (no revocation without changing the passphrase everywhere; passphrase in URL/logs) |
| Pros | No PII stored; revocation per device; works for a kiosk browser that has no keyboard beyond a one-time code; QR makes it a 10-second flow | Multi-desk, multi-user, billing-ready | Trivial |
| Cons | A license key is still needed for entitlement, and someone has to mint it (script until Stripe) | Stores e-mail addresses — first PII in a privacy-first product; 5× the work; blocks this epic on E010 | No per-device revoke; brute-forceable if short |
| Reuses | `uuid`, `rand` crates already present; `tauri-plugin-store` for non-secret settings | nothing existing | nothing |

**Recommendation: A**, explicitly *as a foundation B can sit on*: an account,
when it comes, is "the thing that owns license keys and lists desks". Nothing
in A's tables has to be thrown away for that. A is also the only option that
matches the tier table's promise "privacy-first, data stays local".

### Two-way control scope (D3)

| | A. Minimum — `ack_alert` only | B. Target — `ack_alert` + `set_limits` + `switch_profile` (this plan) | C. Full settings mirror (`save_settings` remotely) |
|---|---|---|---|
| Summary | Phone dismisses the popup | Phone also changes the two limits and picks a profile | Phone can write any `AppConfig` field |
| Effort | S | M | M |
| Risk | L | L (each command calls an existing validated function) | M (calibration and store writes from an untrusted network path) |
| Pros | Smallest attack surface | Covers the three things a person actually wants to do from the sofa; each maps 1:1 to an existing Tauri command with clamping already in place | "Everything works from the phone" |
| Cons | A "Pro" feature that can only press one button | Three allowlisted names to maintain | Calibration from a phone is meaningless (the person is not at the desk); a generic write path is exactly what a security reviewer will reject |
| Reuses | `AlertPopup::dismiss`, `CommunicationPolicy::dismiss` | + `SessionManager::set_limit_minutes`/`set_stand_limit_minutes`, `commands_profiles::switch_*` | `commands_config::save_settings` |

**Recommendation: B.** Named allowlist, bounded arguments, every command
answered with a `command_result`, executed through the same functions the
desktop UI uses.

### LAN mode (D4)

| | A. Replace LAN with relay-only | B. Keep LAN as free tier on the shared envelope (this plan) | C. Keep LAN untouched, add relay as a parallel code path |
|---|---|---|---|
| Effort | S | S | M |
| Risk | H (removes a working free feature; a cloud dependency for a local dashboard contradicts ADR 001 and the privacy positioning) | L | M (two envelopes, two clients, `useRemoteDesk` forks) |
| Recommendation | no | **yes** | no — the whole point of the envelope is one client for both transports |

## Architecture

```
 PC (Tauri)                                    Cloudflare                          Phone
 ┌───────────────────────────┐                 ┌───────────────────────────┐        ┌──────────────────┐
 │ session/tray → ws_tx ─────┼─► remote_server │  Worker  /v1/*  (REST)    │        │ React build      │
 │  (broadcast<String>)      │   (LAN, :3390)  │   ├─ register / pair /    │        │  /app  (served   │
 │           │               │                 │   │  viewers  → D1 (hashes)│        │   by the Worker) │
 │           └──► relay_client ── wss outbound ─►  └─ /desks/{id}/ws ──► DO │◄─ wss ─┤ RelayTransport   │
 │   relay_auth (keyring)    │                 │      DeskRoom (per desk)   │        │  or LanTransport │
 │   relay_commands ◄────────┼── command ──────┤      • sockets: 1 desk,    │        │  (same reducer)  │
 │   → existing commands     │                 │        N viewers           │        └──────────────────┘
 └───────────────────────────┘                 │      • last snapshot (RAM)│
                                               │      • pairing codes (RAM)│
                                               └───────────────────────────┘
```

- **Desktop never listens on the internet.** `relay_client.rs` is one more
  subscriber of the existing `ws_tx` broadcast channel (`ws_broadcaster.rs:58-61`)
  — every event the LAN clients get, the relay gets, already serialized.
- **DeskRoom** (Durable Object, WebSocket Hibernation API): accepts one desk
  socket (a newer desk connection replaces the older one with close 4409) and
  up to `max_viewers` viewer sockets. Fans desk `event` messages out to
  viewers; routes viewer `command` to the desk and `command_result` back to
  the originating viewer; keeps the last `snapshot` in memory so a viewer that
  connects while the desk is offline sees stale data marked `desk_online:
  false` instead of "Reconnecting..." (this is the whole of "cross-device state
  sync" for v1 — no config store in the cloud).
- **D1** holds `licenses`, `desks`, `viewers` — hashes and metadata only.
- **Privacy**: the relay logs no payloads, stores no history, and the
  snapshot dies with the DO's memory. Documented in `docs/PRIVACY.md` (T14).

### Protocol — envelope (v1)

Every WebSocket message in both directions, on both transports:

```json
{ "v": 1, "type": "event", "id": "8f1c…", "ts": 1757170000123, "payload": { } }
```

`v` integer protocol version; `type` string; `id` UUID v4 (echoed in replies);
`ts` sender's Unix ms (used for the latency measurement, never for logic);
`payload` type-specific object, `null` where none. Unknown `type` → ignored
by clients, `error{code:"unknown_type"}` from the relay. Unknown `v` →
close 4400.

| type | direction | payload |
|---|---|---|
| `hello` | client → relay | `{ role: "desk"\|"viewer", desk_id, token, client: { app: "moveup-desktop"\|"moveup-viewer", version } }` — must arrive within 5 s of upgrade, else close 4401 |
| `welcome` | relay → client | `{ role, desk_id, desk_online, viewer_count, snapshot: RemoteDisplayState\|null, snapshot_ts }` |
| `event` | desk → relay → viewers | `DisplayEvent` as serialized today (`{event, payload}` from `ws_broadcaster.rs`), unchanged inside the envelope |
| `desk_status` | relay → viewers | `{ online: bool, since: unix_ms }` — on desk connect/disconnect |
| `command` | viewer → relay → desk | `{ name, args }`; relay adds `viewer_id` before forwarding |
| `command_result` | desk → relay → that viewer | `{ command_id, ok, error: { code, message }\|null }` — `command_id` is the `id` of the `command` envelope |
| `ping` / `pong` | either | `null`; clients ping every 25 s, relay answers; relay closes after 60 s of silence |
| `error` | relay → client | `{ code, message }` non-fatal (rate limit, unknown type, bad args) |

Close codes (WebSocket, 4xxx application range): `4400` protocol error,
`4401` unauthenticated (missing/late/invalid `hello`), `4402` entitlement
expired or license revoked, `4403` token revoked, `4409` replaced by a newer
desk connection, `4429` rate limited, `4503` relay shedding load.

Commands (`command.name` allowlist, args validated on the relay *and* on the
desk):

| name | args | desk executes |
|---|---|---|
| `ack_alert` | `{}` | `CommunicationPolicy::dismiss()` + `AlertPopup::dismiss()` — same as the user clicking the popup |
| `set_limits` | `{ sit_min?: 5..=240, stand_min?: 1..=120 }` at least one | `SessionManager::set_limit_minutes` / `set_stand_limit_minutes` (existing clamps still apply) |
| `switch_profile` | `{ kind: "ergonomic"\|"communication", name: [a-z0-9_-]{1,32} }` | `commands_profiles::switch_ergonomic_profile` / `switch_communication_profile` logic (refactored to an app-handle-free function so both the IPC command and the relay call it) |

Rate limits: 10 commands / minute / viewer (relay, `4429`-style `error`, not a
close); the desk additionally ignores a command whose `ts` is older than 30 s.

### Protocol — REST (Worker, JSON, `Authorization: Bearer <token>` where noted)

| Method + path | Auth | Body → Response | Notes |
|---|---|---|---|
| `POST /v1/desks/register` | none | `{ license_key, desk_name, app_version }` → `201 { desk_id, desk_token, plan, expires_at }` | key checked against `licenses` (hash), `max_desks` enforced; 5/min/IP |
| `POST /v1/desks/{desk_id}/pairings` | desk | `{}` → `201 { code, expires_at }` | 8 chars from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`, TTL 300 s, single use, kept in the DO (never D1); at most one active code per desk (a new one replaces it) |
| `POST /v1/pair` | none | `{ desk_id, code, device_name }` → `201 { viewer_id, viewer_token, desk_name }` | 5/min/IP; DO locks pairing for 15 min after 10 wrong codes; `409` when `max_viewers` reached |
| `GET /v1/desks/{desk_id}/viewers` | desk | → `200 [{ viewer_id, device_name, paired_at, last_seen, online }]` | drives the Settings list |
| `DELETE /v1/desks/{desk_id}/viewers/{viewer_id}` | desk | → `204` | DO closes that viewer's socket with `4403` |
| `DELETE /v1/desks/{desk_id}` | desk | → `204` | "disable relay" on the desktop: revokes everything |
| `GET /v1/desks/{desk_id}/ws` | via `hello` | WebSocket upgrade | token is **never** in the URL |
| `GET /app/*` | none | static React build | Workers static assets; same build as the desktop, `RelayTransport` selected by `#/pair` / stored token |
| `GET /healthz` | none | `200 { ok: true, version }` | for the deploy gate |

Token format: `mu_d_<43 base64url chars>` (desk) and `mu_v_<…>` (viewer) —
32 random bytes; the prefix lets secret scanners and the
`secret-emission-guard` hook recognise them. Stored as `sha256(token)` in D1;
compared in constant time. Errors are `{ error: { code, message } }`, codes
from a fixed list (`invalid_license`, `license_exhausted`, `bad_code`,
`code_expired`, `pairing_locked`, `viewer_limit`, `unauthorized`,
`rate_limited`, `not_found`).

### Desktop outbound connection contract (`relay_client.rs`)

1. Runs only when `AppConfig.relay_enabled` and a `desk_token` exists in the
   OS credential store (`keyring` crate → Windows Credential Manager; the
   `desk_id` and non-secret metadata live in `tauri-plugin-store`).
2. Connects to `wss://{relay_url}/v1/desks/{desk_id}/ws`, sends `hello`
   within 1 s, expects `welcome` within 5 s; any other first message → treat
   as failed attempt.
3. On `welcome`: immediately sends one `event{snapshot}` built by
   `remote_display_state::build` so the DO has fresh state; then forwards
   every message from its `ws_tx` subscription verbatim inside an `event`
   envelope. `Lagged(n)` on the broadcast receiver → re-send a snapshot, log
   at debug (same handling as `remote_server.rs:125-127`).
4. `ping` every 25 s; missing `pong` for 60 s → reconnect.
5. Reconnect with exponential backoff 1 s → 60 s, ±20 % jitter, reset on
   `welcome`. `4402`/`4403` stop the loop and surface a `RelayStatus::
   Unentitled`/`Revoked` state to Settings — no silent retry against a dead
   credential. `4409` stops the loop too (another instance of the app owns
   this desk) and shows it.
6. Incoming `command`: allowlist + bounds check (`relay_commands.rs`), execute
   under the same locks the IPC commands use, reply `command_result` with the
   command's `id` within 2 s. Every command is written to the event log
   (`events.log`, `REMOTE <name> viewer=<id> ok|err`) — the audit trail the
   security review asks for.
7. Status is observable: `get_relay_status` Tauri command returns
   `{ enabled, state: disabled|connecting|online|unentitled|revoked|replaced|error, since, viewers_online, last_error }`; the Settings section renders it and the Debug tab dumps it.

### Phone client connection / pairing flow

1. User opens Settings → Remote on the desktop, clicks **Pair a phone**.
   Desktop calls `POST /pairings`, shows the code as text and as a QR encoding
   `https://relay.desk.zentala.io/app/#/pair?d=<desk_id>&c=<code>`.
2. Phone scans (opens the viewer app at `/app`) or opens `/app` and types
   `desk_id` (shown next to the code) and the code.
3. `RelayTransport.pair()` → `POST /v1/pair` → stores `{ relay_url, desk_id,
   viewer_id, viewer_token, desk_name }` in `localStorage` key
   `moveup.relay.v1` (wrapped in try/catch per the browser-storage rule).
4. Connects, sends `hello{role:"viewer"}`, receives `welcome` with the last
   snapshot and `desk_online`; the reducer applies the snapshot exactly as the
   LAN path does today (`useRemoteDesk.ts:47-54`).
5. `ConnectionOverlay` gains a fourth state: relay connected + desk offline →
   banner "Desk offline since HH:MM — showing last known state" over
   dimmed data (distinct from "Reconnecting..." which now means *the phone*
   lost the relay).
6. Control: `RemoteControls` renders only when the transport reports
   `capabilities.control === true` (relay: true, LAN: false). Each button
   sends `command`, shows a pending state until `command_result`, and shows
   the error message on `ok: false`.
7. `#/pair` also exposes **Forget this desk** (clears storage; the desktop
   revokes from its own list — the phone cannot revoke itself server-side,
   which is intentional: the desk owns its viewers).
8. Transport selection in `useRemoteDesk`: `#/pair` or a stored relay record
   → `RelayTransport`; otherwise `LanTransport` (today's behaviour,
   `ws://${location.host}/display/ws`).

## Architecture impact

- **First auth surface of the app.** Until now every write path was local
  IPC behind the Tauri webview. This epic adds a network write path guarded
  by bearer tokens the app did not have a place to store. Consequences the
  code must reflect, not just the docs: secrets go to the OS credential store
  (`keyring`), never to `tauri-plugin-store` JSON; every remote write is
  allowlisted and logged; the LAN path stays read-only so the unauthenticated
  surface never gains a write. ADR 021 records this as the pattern any later
  cloud feature (history sync, coaching) must follow — "a credential the
  relay hashed, a command the desk allowlisted".
- **New supervision boundary: local app ↔ cloud relay.** ADR 020. The relay
  is not a PM3 service (ADR 018 unaffected); its lifecycle is `wrangler
  deploy` behind `consent-broker`, its health is `/healthz`, and the desktop
  treats it as unreliable by design (backoff, stale snapshot on the phone).
- **New dependencies**: Rust `tokio-tungstenite` (with `rustls-tls-webpki-roots`)
  and `keyring`; TS `qrcode` (desktop Settings), `zod` (envelope validation,
  shared by the viewer and the relay), `wrangler` + `@cloudflare/workers-types`
  + `@cloudflare/vitest-pool-workers` in `relay/` only. Listed in ADR 020/021.
- **New top-level directory `relay/`** — a standalone pnpm project with its
  own lockfile (the root workspace declares no `packages:` and `.giter.yaml`
  guards only the root `node_modules`; T12 adds `relay/node_modules` to
  `worktree.guards`). It imports the shared protocol schema from
  `src/remote/protocol.ts` through a tsconfig path alias so the schema exists
  once.
- **`remote_server.rs` changes shape but not role**: same routes, envelope
  added, toggle added, still read-only.
- **`.arch/ARCHITECTURE.md`** gets a "Remote display — two transports, one
  contract" section; `CLAUDE.md` Remote Display section rewritten;
  `PROJECT.xml` updated (new files, new IPC commands, new test layer);
  `docs/REMOTE_DISPLAY.md` rewritten around pairing; `docs/PRIVACY.md` states
  what the relay holds.
- **Version**: this epic bumps the minor per `.claude/rules/versioning.md`
  (0.7.0, or the next free minor if E021 ships first).

## Scope

**In**: `relay/**` (new), `src-tauri/src/remote_protocol.rs`,
`relay_client.rs`, `relay_auth.rs`, `relay_commands.rs`, `relay_status.rs`,
`commands_relay.rs` (+ `_tests.rs` siblings), `remote_server.rs`
(envelope + toggle), `ws_broadcaster.rs` (no variant change; doc only),
`config.rs` (`relay_enabled`, `relay_url`, `remote_lan_enabled`),
`commands.rs` (`AppState.relay`), `commands_profiles.rs` (extract
app-handle-free switch functions), `lib.rs`, `Cargo.toml`, `src/remote/**`
(new), `src/hooks/useRemoteDesk.ts` (+ tests + helpers),
`src/components/ConnectionOverlay.tsx`, `src/components/settings/RemoteSection.tsx`
(new), `src/App.tsx` (`#/pair` route), `tests/fixtures/relay-protocol/**`
(new), `vite.config.ts` (dev proxy for `/display`, see note), `.giter.yaml`,
`justfile`, `package.json`, `.arch/ADR/020-*.md`, `021-*.md`,
`.arch/ARCHITECTURE.md`, `CLAUDE.md`, `PROJECT.xml`, `docs/REMOTE_DISPLAY.md`,
`docs/PRIVACY.md`, `.plan/decisions.jsonl`.

**Out**: accounts / e-mail login (vision E010-T01), Stripe billing
(E010-T04) — license keys are minted by `relay/scripts/mint-license.mjs`
until then; history/cloud sync (E011); Tauri Mobile native app (ADR 001
Phase 2, still not this epic); LAN pairing / LAN auth (follow-up, backlog);
calibration from the phone; the three dev-mode remote-display gaps in
`.plan/BACKLOG.md` ("Dev-mode remote display", tied to E015-T05) **except**
the Vite `/display` proxy, which T12 needs for the local end-to-end check and
therefore lands here (the backlog entry is closed by T12, the other two stay).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Protocol contract: `remote_protocol.rs` (envelope struct, `hello`/`welcome`/`command`/`command_result` types, close-code constants, command allowlist + arg bounds as data), ts-rs export, `src/remote/protocol.ts` (zod schemas), fixtures in `tests/fixtures/relay-protocol/*.json`, a Rust test and a vitest that both round-trip every fixture | 5 | ts-dev | 0 |
| T02 | Relay Worker core: `relay/` project (`wrangler.toml`, `package.json`, vitest workers pool), `DeskRoom` DO with hibernation WS, `hello`→`welcome`, desk/viewer roles, `event` fan-out, `desk_status`, in-memory last snapshot, ping/pong + idle close, close codes, `/healthz`, `/app/*` static assets | 13 | ts-dev | 1 |
| T03 | Relay auth + REST: D1 migrations (`licenses`, `desks`, `viewers`), `register`, `pairings`, `pair`, viewers list/revoke, desk delete, SHA-256 token storage + constant-time compare, per-IP and per-desk rate limits, pairing lockout, `scripts/mint-license.mjs` | 13 | ts-dev | 2 |
| T04 | Relay command routing: viewer `command` → allowlist/arg validation → desk; `command_result` back to the originating viewer only; 10/min/viewer limit; `error` envelopes | 5 | ts-dev | 3 |
| T05 | Desktop relay client: `relay_client.rs` (tokio-tungstenite + rustls), `ws_tx` subscription, `hello`/`welcome`, snapshot on welcome, ping/pong, backoff with jitter, close-code handling → `RelayStatus`; mock WS server in tests | 8 | ts-dev | 1 |
| T06 | Desktop credentials + pairing: `relay_auth.rs` (`keyring` for `desk_token`; store for `desk_id`/metadata), `register`, `start_pairing`, `list_viewers`, `revoke_viewer`, `disable_relay` (REST via `reqwest`), `AppConfig.relay_enabled/relay_url`, Tauri commands in `commands_relay.rs`, `get_relay_status` | 8 | ts-dev | 2 |
| T07 | Desktop command execution: `relay_commands.rs` allowlist → `ack_alert`, `set_limits`, `switch_profile` through existing functions (extract app-handle-free `switch_*` from `commands_profiles.rs`); stale-`ts` rejection; `events.log` `REMOTE` lines; `command_result` | 5 | ts-dev | 3 |
| T08 | Phone transport abstraction: `src/remote/transports/{lan,relay}.ts` behind one `Transport` interface, `useRemoteDesk` consumes it, envelope v1 parsing on both, `capabilities.control`, `ConnectionOverlay` fourth state (desk offline), stored-record detection | 8 | ts-dev | 1 |
| T09 | Phone pairing + controls UI: `#/pair` route (`PairScreen.tsx`: QR deep link parse, manual code entry, forget desk), `RemoteControls.tsx` (ack, sit/stand ±, profile picker) with pending/error states, wired to `RelayTransport.sendCommand` | 8 | ts-dev | 2 |
| T10 | Desktop Settings → Remote section: enable toggle, license key entry (sent once, never displayed again), relay status line, **Pair a phone** (code + QR via `qrcode`), paired devices list with revoke, LAN toggle; mockup scenario in `src/test/scenarios.ts` first (`ux-design-flow.md`) | 8 | ts-dev | 3 |
| T11 | LAN path on the shared envelope: `remote_server.rs` wraps `DisplayEvent` in the v1 envelope, honours `remote_lan_enabled`, keeps the receive loop read-only; `remote_server_tests.rs` updated; `docs/REMOTE_DISPLAY.md` LAN section notes the toggle | 3 | ts-dev | 1 |
| T12 | Wiring + local end-to-end: `just relay-dev/relay-test/relay-deploy`, `.giter.yaml` guard, Vite `/display` dev proxy, `scripts/relay-e2e.mjs` (starts `wrangler dev`, registers a desk with a test license, pairs a fake viewer, asserts an `event` arrives and a `command` round-trips with p95 latency < 500 ms over 100 messages), `PROJECT.xml` | 5 | ts-dev | 4 |
| T13 | **Security review — pipeline step 5, not skippable**: threat model in `reports/threat-model.md`, `security-reviewer` agent pass over `relay/`, `relay_*.rs`, `commands_relay.rs`, `remote_server.rs`; `review-loop` until zero confirmed findings (max 3 rounds, then escalate with the disputed list); fixes land in this task; manual checklist executed and recorded | 5 | main | 5 |
| T14 | Docs + ADRs: ADR 020 (relay on Cloudflare DO), ADR 021 (pairing-code device-token auth), `.arch/ARCHITECTURE.md`, `CLAUDE.md` Remote Display section, `docs/REMOTE_DISPLAY.md` rewrite, `docs/PRIVACY.md`, `decisions.jsonl` D1-D5, backlog entry for LAN pairing, `scripts/check-e022-t14-docs.mjs` | 3 | main | 6 |
| T15 | Verify + browser pass: `verify` agent over every acceptance criterion; one `browser` dispatch (three pages: desktop Settings → Remote with a code shown; phone `/app/#/pair` → paired dashboard with a live snapshot; a `set_limits` from the phone visible on the desktop popup); evidence records `current` | 2 | verify | 6 |
| T16 | Deploy relay to `relay.desk.zentala.io` (staging first): `wrangler deploy` + D1 migration + one minted Founder license — outward-facing, through `consent-broker`; `/healthz` and the T12 e2e script against the live host | 2 | main | 6 |

Wave points: W0=5, W1=32 (T02+T05+T08+T11, four disjoint write sets), W2=29
(T03+T06+T09), W3=18 (T04+T07+T10), W4=5, W5=5, W6=7. All ≤ 40. Total 111.

**Minimum shippable cut** if the epic must be halved: W0–W2 minus controls
(T01, T02, T03, T05, T06, T08, T09 without `RemoteControls`, T11, T12, T13,
T14, T15, T16 = 79 points) ships "view your desk from anywhere, paired,
revocable". T04/T07/T10's control half is the second release. The plan is
written as one epic because the control path is where the security review
earns its keep, and reviewing it once, with the whole surface present, is
cheaper than twice.

## Test strategy

Four shadow paths (happy / nil / empty / error) per `rules/testing.md`, named
per seam. Mocks stub only the *external* boundary (network, clock,
credential store); every in-repo seam gets at least one real-vs-real test.

- **T01 protocol fixtures** — the contract test that keeps Rust, viewer and
  relay honest. happy: every `tests/fixtures/relay-protocol/*.json` parses in
  Rust (`serde`) and in TS (`zod`) and re-serialises byte-equal after key
  sorting. nil: `payload: null` on `ping` is valid; missing `payload` is not.
  empty: `command.args: {}` valid for `ack_alert`, invalid for `set_limits`
  (needs ≥ 1 key). error: `v: 2` rejected with a typed error in both
  languages; an `event` whose inner `DisplayEvent` is unknown is *accepted*
  by the envelope layer (forward compatibility) and rejected only by the
  reducer. **Fails today**: no envelope exists. Files:
  `src-tauri/src/remote_protocol_tests.rs`, `src/remote/protocol.test.ts`.
- **T02 DeskRoom** (vitest workers pool, real DO in miniflare). happy: desk
  hello → viewer hello → desk `event` → viewer receives it, `welcome` carries
  the snapshot, `desk_status` on desk drop. nil: viewer connects with no desk
  ever seen → `welcome.snapshot: null, desk_online: false`. empty: desk sends
  `event` with 0 viewers → no error, snapshot cached. error: second desk
  socket → first gets 4409; no `hello` in 5 s → 4401; `v: 2` → 4400; 61 s
  without ping → closed. Chaos: DO eviction between messages (miniflare
  restart) → viewer reconnects and gets the same snapshot (it lives in the
  DO's memory, so after eviction it is `null` — assert that, and that the
  desk's reconnect re-populates it).
- **T03 auth** (vitest + D1 in miniflare). happy: register → pair → hello
  with viewer token accepted. nil: `register` without `license_key` → 400
  `invalid_license`. empty: `code: ""` → 400 `bad_code`, no lockout counter
  increment. error: 10 wrong codes → `pairing_locked` for 15 min (fake clock);
  expired code → `code_expired`; revoked viewer's next `hello` → 4403 and an
  open socket is closed; `max_viewers` → 409; token compared via
  `crypto.subtle.timingSafeEqual` (assert the plaintext token never appears
  in any D1 row: query all tables, grep for the token). Hostile QA: the same
  pairing code used twice concurrently → exactly one 201.
- **T04 command routing**. happy: viewer `command` → desk receives with
  `viewer_id`; desk `command_result` → only that viewer gets it. nil: command
  with no desk online → immediate `command_result{ok:false, error.code:
  "desk_offline"}` from the relay. empty: `args: {}` on `set_limits` →
  `error{bad_args}` never forwarded. error: 11th command in a minute →
  `error{rate_limited}`; unknown name → `error{unknown_command}`.
- **T05 desktop client** (`cargo test`, mock WS server on a random port with
  tokio-tungstenite). happy: connects, `hello`, receives `welcome`, sends
  snapshot, forwards a broadcast message inside an `event` envelope. nil: no
  `desk_token` in the (mocked) credential store → client never starts,
  status `disabled`. empty: `ws_tx` has no messages for 25 s → exactly one
  `ping` sent. error: server closes 4403 → status `revoked`, **no reconnect**
  within 5 s (assert zero further connection attempts); server closes 1006 →
  reconnect after ~1 s then ~2 s (fake sleep, assert jitter within ±20 %).
  Chaos: server drops mid-`hello` → counted as one failed attempt, backoff
  doubles.
- **T06 credentials**. happy: `register` stores `desk_token` in the keyring
  mock and `desk_id` in the store; `start_pairing` returns code + QR payload
  string. nil: `register` with a rejected license → `Unentitled`, nothing
  stored. empty: `list_viewers` with none paired → `[]` rendered as "No
  phones paired" (not a blank list). error: `disable_relay` when the relay
  is unreachable → local credential is still deleted and the failure is
  logged (the desk must be able to turn itself off without the cloud).
  Keyring mock: `keyring` has a mock credential store feature — use it; the
  real Windows store is exercised once in T15's manual pass.
- **T07 command execution**. happy: `set_limits{sit_min: 30}` → session
  limit is 1800 s afterwards and `command_result{ok:true}`. nil:
  `switch_profile{name}` for a missing profile → `ok:false, error.code:
  "not_found"`, current profile unchanged. empty: `set_limits{}` rejected
  before touching the session. error: `ts` older than 30 s → `ok:false,
  "stale"`; a name outside the allowlist → `ok:false, "unknown_command"`,
  logged as `REMOTE DENIED`. Real-vs-real: the test drives
  `relay_commands::execute` against a real `SessionManager`, not a trait
  mock.
- **T08 transports** (vitest, fake WebSocket as in
  `remoteDesk.test-helpers.ts`). happy: relay transport parses an envelope
  `event` and dispatches to the reducer identically to the LAN transport
  (one test feeds the same fixture through both and asserts equal reducer
  state). nil: no stored relay record and not on `#/pair` → LAN transport.
  empty: `welcome.snapshot: null` → reducer stays at initial state, overlay
  shows "desk offline", not "reconnecting". error: 4403 → transport clears
  the stored record and routes to `#/pair` with a "This phone was
  unpaired" message; malformed JSON → warned and ignored (existing
  behaviour kept).
- **T09 pairing UI**. happy: deep link `#/pair?d=…&c=…` prefills and
  auto-submits; success navigates to the dashboard. nil: `#/pair` with no
  params → manual form. empty: submitting an empty code → inline validation,
  no request. error: `bad_code` → message under the field, code input
  cleared; `pairing_locked` → message with the retry time. Controls:
  pending state until `command_result`; `ok:false` shows `error.message`.
- **T10 Settings section** — mockup scenarios first (`ux-design-flow.md`):
  `relay-disabled`, `relay-online-2-viewers`, `relay-unentitled`,
  `relay-pairing-code-shown`. Unit: revoke calls the command with the right
  `viewer_id` and removes the row optimistically, restores it on error; the
  license key input never renders a stored value.
- **T11 LAN envelope**. happy: existing `remote_server_tests.rs` pass with
  the envelope; a LAN client on the new transport renders. nil:
  `remote_lan_enabled: false` → server not started (assert bind never
  called). error: an inbound `command` on the LAN socket → ignored, logged at
  debug, connection stays open (read-only path proven by test, not by
  comment).
- **T12 end-to-end** (`scripts/relay-e2e.mjs` against `wrangler dev`):
  register → pair → 100 events → p95 desk→viewer latency < 500 ms measured
  from envelope `ts`; one `set_limits` round-trip < 1 s. Flake risk: marked
  time-dependent; runs in `just relay-test` but not in the unit gate.
- **T13 security** — manual checklist, each item recorded in the threat
  model with the command or observation used: unauthenticated WS →
  4401; token in URL query → 401 (the route ignores it); revoked viewer
  socket closes within 1 s; brute-force lockout; `desk_token` absent from
  every file under `%APPDATA%\io.zntl.desk` (grep); relay logs contain no
  payload fields (grep `wrangler tail` output during e2e); command from a
  viewer of desk A cannot reach desk B (two rooms in the test); CSP of the
  viewer build allows `wss:` only to the configured relay origin.
- **Pyramid check**: ~60 unit (Rust + TS + Worker), ~12 integration (DO/D1
  in miniflare, mock WS server), 1 scripted e2e, 1 browser pass. Not
  inverted. Time-dependent tests (backoff, TTL, lockout) use injected
  clocks; only the e2e latency assertion touches wall time.

## Evidence contract

Records under `evidence/records/`, schema per `rules/evidence.md`. Raw
output, screenshots and `wrangler tail` captures under `evidence/.local/`.

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| protocol-fixtures | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_protocol` and `npx vitest run --config vite.config.ts src/remote/protocol.test.ts` | both pass; fixture count asserted > 0 in each | `T01-protocol-fixtures.json` |
| relay-room | test | `pnpm --dir relay exec vitest run test/room` | pass | `T02-relay-room.json` |
| relay-auth | test | `pnpm --dir relay exec vitest run test/auth test/http` | pass; plaintext-token-absent assertion included | `T03-relay-auth.json` |
| relay-commands | test | `pnpm --dir relay exec vitest run test/commands` | pass | `T04-relay-commands.json` |
| desk-client | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_client` | pass; includes the no-reconnect-after-4403 case | `T05-desk-client.json` |
| desk-auth | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_auth commands_relay` | pass | `T06-desk-auth.json` |
| desk-commands | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_commands` | pass; real `SessionManager` | `T07-desk-commands.json` |
| phone-transports | test | `npx vitest run --config vite.config.ts src/remote src/hooks/useRemoteDesk` | pass; same-fixture-both-transports test included | `T08-phone-transports.json` |
| phone-pairing-ui | test | `npx vitest run --config vite.config.ts src/remote/PairScreen src/remote/RemoteControls` | pass | `T09-phone-pairing-ui.json` |
| settings-remote | test | `npx vitest run --config vite.config.ts src/components/settings/RemoteSection` | pass | `T10-settings-remote.json` |
| lan-envelope | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_server` | pass; inbound-command-ignored case included | `T11-lan-envelope.json` |
| e2e-local | test | `node scripts/relay-e2e.mjs` | exit 0; prints p95 latency and it is < 500 ms; 100 events counted | `T12-e2e-local.json` |
| security-review | review | `review-loop` over the listed files + manual checklist in `reports/threat-model.md` | zero confirmed findings after ≤ 3 rounds, or escalation list; every checklist row has a recorded observation | `T13-security-review.json` |
| docs-consistency | manual | `node scripts/check-e022-t14-docs.mjs` | exit 0: ADR 020/021 exist, referenced from ARCHITECTURE.md and CLAUDE.md; `docs/REMOTE_DISPLAY.md` mentions pairing; PRIVACY.md mentions the relay | `T14-docs-consistency.json` |
| browser-visual | visual | `browser` agent, three pages, landmarks: pairing code visible in Settings; phone dashboard shows `limit_used_secs` moving; desktop popup reflects a phone-sent `set_limits` | verdict with screenshots in `evidence/.local/` | `T15-browser-visual.json` |
| relay-live | deployment | `curl https://relay.desk.zentala.io/healthz` and `node scripts/relay-e2e.mjs --relay https://relay.desk.zentala.io --license <minted>` (license via `password-broker inject`) | `{ok:true}`; e2e exit 0 | `T16-relay-live.json` |

A missing record is `missing`, an empty search is `unknown` — never `0`,
never a pass.

## Acceptance criteria

1. A phone on mobile data, not on the PC's network, shows the live desk
   state within 1 s of a state change, after pairing with a code from the
   desktop — no IP, no firewall rule, no port forward (T02/T03/T05/T06/T08/
   T09; proven by `e2e-local` and `browser-visual`).
2. Every relay connection without a valid `hello` token is closed with
   4401; a revoked viewer is disconnected within 1 s and cannot reconnect
   (T03; `relay-auth`, `security-review`).
3. The plaintext `desk_token` exists only in the OS credential store; the
   relay's D1 holds hashes only; no token ever appears in a URL, a log line
   or a `tauri-plugin-store` file (T03/T06/T13; `relay-auth`,
   `security-review`).
4. From the phone, `ack_alert`, `set_limits` and `switch_profile` work and
   every other command name is rejected with `unknown_command` and logged
   `REMOTE DENIED`; the desktop applies the same clamps as its own UI
   (T04/T07/T09; `desk-commands`, `relay-commands`).
5. With the PC off, a paired phone shows the last known state with a
   visible "desk offline since HH:MM" banner, not "Reconnecting..." (T02/T08;
   `relay-room`, `phone-transports`).
6. The LAN path still works for a user who never enables the relay, is
   read-only (an inbound `command` is ignored, proven by test), and can be
   turned off in Settings (T11; `lan-envelope`).
7. Relay and desktop refuse to start or stay connected on an expired or
   revoked license (4402) and the Settings section says so in words, not by
   silently retrying (T05/T06/T10; `desk-client`, `settings-remote`).
8. Desk→viewer p95 latency < 500 ms over 100 events on `wrangler dev`, and
   the live relay passes the same script (T12/T16; `e2e-local`,
   `relay-live`).
9. The security review ran, its checklist has an observation per row, and
   zero confirmed findings remain (or an explicit escalation list exists)
   (T13; `security-review`) — this criterion has no skip path.
10. ADR 020 and ADR 021 exist and are linked from `.arch/ARCHITECTURE.md`
    and `CLAUDE.md`; `docs/REMOTE_DISPLAY.md` documents pairing; `PRIVACY.md`
    states what the relay stores; `PROJECT.xml` lists the new files and
    commands (T14; `docs-consistency`).
