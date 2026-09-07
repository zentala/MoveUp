# ADR 023: Pairing code → device tokens, and an allowlist for every remote write

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E022 (cross-device phone relay) — tasks T03, T06, T07, T09, T10, T13
- **Related**: [ADR 022](022-relay-on-cloudflare-durable-objects.md) (where these credentials are checked),
  [ADR 001](001-remote-display-web-kiosk.md) (the LAN path this does *not* cover),
  [ADR 005](005-open-core-software-model.md) (a license key is the entitlement),
  [ADR 015](015-pure-ergo-engine.md) (a remote command is an adapter call, never a new engine input),
  [ADR 020](020-health-source-inlet.md) (the shared-secret gate this supersedes for the relay path),
  [E022](../../.plan/epics/E022-2026-09-06-cross-device-phone-relay/PLAN.md)

## Context

Until this epic every write into the app was local IPC behind the Tauri webview.
The only network write was `POST /display/health` and `POST /display/voice`,
guarded by one shared `DESK_REMOTE_TOKEN` typed into a `.env` file (ADR 020).
That gate is adequate for a LAN inlet the owner configures once; it does not
survive the relay, for three reasons:

- **One secret cannot be revoked per device.** Losing a phone would mean
  changing the secret on every other phone.
- **There is no identity in a shared secret**, so `command_result` cannot be
  routed back to the viewer that asked, and an audit line cannot name who did
  what.
- **A remote control path is exactly where a generic write is worst.** The
  relay carries commands that change the user's limits and profile. Whatever
  guards them has to be narrower than "authenticated", not wider.

The obvious answer — accounts with e-mail and passwords — is the vision doc's
E010-T01, and it would make this epic's first act "store a user's e-mail
address" in a product whose positioning is privacy-first.

## Decision

**Pairing code → long-lived per-device tokens. No accounts, no passwords, no
e-mail.** Plus an **allowlist** for the only three things a viewer may do.

1. **The desk registers once** with a license key (minted by
   `relay/scripts/mint-license.mjs` until billing exists) and receives
   `desk_id` + `desk_token`.
2. **Each phone pairs once.** The desktop asks the relay for a code — 8
   characters from `ABCDEFGHJKLMNPQRSTUVWXYZ23456789` (no `I`, `O`, `0`, `1`),
   TTL 300 s, single use, held in the Durable Object's memory and never written
   to D1. The phone types it or scans a QR of
   `…/app/#/pair?d=<desk_id>&c=<code>` and receives `viewer_id` +
   `viewer_token`. Ten wrong codes lock pairing on that desk for 15 minutes.
3. **Tokens are opaque random bytes**: `mu_d_` / `mu_v_` plus 43 base64url
   characters from 32 random bytes. The prefix exists so secret scanners and
   the `secret-emission-guard` hook can recognise one. D1 stores only
   `sha256(token)`, compared in constant time. **A token never appears in a
   URL** — the WebSocket carries it in the `hello` message after the upgrade,
   never in a query string that would land in a proxy log.
4. **Secrets live in the OS credential store.** `relay_auth.rs` puts
   `desk_token` in Windows Credential Manager through the `keyring` crate;
   `tauri-plugin-store` holds only `desk_id`, `relay_url` and flags. On the
   phone, the viewer record sits in `localStorage` under `moveup.relay.v1`,
   which is the phone's own risk to carry and revocable from the desk.
5. **Revocation is the desk's, not the phone's.** Settings → Remote lists the
   paired devices; revoking one deletes its row and closes its socket with
   `4403`. The phone's "Forget this desk" clears local storage only —
   deliberately, because the desk owns its viewers.
6. **Every remote write is allowlisted, bounded and logged.**
   `COMMAND_ALLOWLIST` in `remote_protocol.rs` is data, not a match arm:
   `ack_alert` (no args), `set_limits` (`sit_min` 5..=240, `stand_min` 1..=120,
   at least one), `switch_profile` (`kind` ∈ {ergonomic, communication}, `name`
   a slug ≤ 32 chars). It is checked on the relay **and again on the desk**;
   a command whose `ts` is older than 30 s is refused as stale. Each one runs
   through the same function the desktop UI calls, so the existing clamps still
   apply, and each writes an `events.log` line — `REMOTE <name> viewer=<id>
   ok|err=<code>`, or `REMOTE DENIED <name>` for anything off the list.
7. **The LAN path is untouched and stays read-only.** It gains no
   authentication and no write, so this decision never widens the
   unauthenticated surface. Closing that gap (LAN pairing) is filed in
   [`.plan/BACKLOG.md`](../../.plan/BACKLOG.md).

## Alternatives

| | A. Pairing code + device tokens (chosen) | B. Accounts, magic-link e-mail, JWTs | C. One long-lived shared passphrase |
|---|---|---|---|
| Summary | Desk registers with a license key; phone enters a short-lived code; both sides revocable per device | Vision E010-T01: log in on desktop and phone, tokens issued per login | One passphrase configured on the PC and typed on the phone, used as a bearer |
| Effort | M | L — e-mail delivery, user table, consent screens, data-subject flows | S |
| Risk | Low | Medium | High — no revocation without changing it everywhere; ends up in URLs and logs |
| Pros | No PII stored anywhere; per-device revoke; a QR makes pairing a ten-second flow on a kiosk browser with no real keyboard | Multi-desk, multi-user, billing-ready | Trivial to build |
| Cons | Entitlement still needs a license key, and something has to mint it | Stores e-mail addresses — the first PII in a privacy-first product — and blocks this epic on E010 | No identity, so no per-viewer routing and no audit line; brute-forceable if short |

**Chosen: A**, explicitly as a foundation B can sit on: an account, when it
arrives, is "the thing that owns license keys and lists desks". Nothing in A's
tables has to be thrown away for that. C was rejected on revocation alone —
ADR 020's shared secret is already the largest shared secret this app should
ever have, and it guards reads of a LAN-local page, not writes from the
internet.

## Consequences

- **This is the app's first auth surface, and it sets the pattern.** Any later
  cloud feature — history sync, hosted coaching — follows the same two rules:
  *a credential the relay only ever holds hashed, and a command the desk
  allowlisted*. A generic "apply this settings object" endpoint is out of
  bounds by this ADR, not by taste.
- **Calibration is deliberately not remotable.** It needs the person at the
  desk, so it stays local IPC; the allowlist makes that a fact rather than an
  omission.
- **The desk can turn itself off without the cloud.** `relay_disable` deletes
  the local credential even when the relay is unreachable; the failure is
  logged. A user must never need a working network to stop sharing.
- **The security review (E022-T13) is a named, unskippable task**, with a
  threat model recording an observation per checklist row: unauthenticated
  socket → `4401`; a token in a query string ignored; a revoked viewer
  disconnected within a second; brute-force lockout; `desk_token` absent from
  every file under `%APPDATA%\io.zntl.desk`; relay logs free of payload fields;
  a viewer of desk A unable to reach desk B.
- **`keyring` is a new dependency** and a new failure mode: no credential store
  means the relay client simply never starts, and Settings says `disabled`
  rather than pretending to connect.
