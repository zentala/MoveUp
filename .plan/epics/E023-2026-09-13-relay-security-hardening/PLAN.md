---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 13
agent: ts-dev
wave: 6
parallel: [E024]
depends-on: [E022]
blocked-by: ""
---

# E023 — Relay security hardening

Source: the six deferred findings of the E022 security review,
[`../E022-2026-09-06-cross-device-phone-relay/reports/2026-09-07-security-review.md`](../E022-2026-09-06-cross-device-phone-relay/reports/2026-09-07-security-review.md)
(Medium #3, #4; Low #5-#8). Moved here from `.plan/BACKLOG.md` §"E022 security
review — deferred findings" on 2026-09-13. Handoff: [`HANDOFF.md`](HANDOFF.md).
Board deck (Polish): [`PRES.md`](PRES.md).

## TLDR

The E022 relay shipped code-complete with six known security gaps that did not
block a deploy. The relay is still not deployed (E022-D6), so this is the
cheapest moment to close them: no live users, no migration. Six small fixes,
one per finding, each proven by a test that fails on today's code, then one
verify + security re-review pass. 7 tasks, 13 points, subagents (≤13 rule).
Direction: the Pro relay is the paid surface of the open-core model
(ADR 005) — it must not ship with known holes.

## Problem

| # | Finding | Where | Severity |
|---|---|---|---|
| F1 | Desk accepts a `ws://` relay URL and sends `desk_token` over plain TCP | `src-tauri/src/relay_client.rs:82-90` (`ws_url`) | Medium |
| F2 | TOCTOU on `max_viewers`: two concurrent `POST /v1/pair` pass the count check, cap exceeded by one | `relay/src/http/pairing.ts:61-65` | Medium |
| F3 | Licence key input is plain text (shoulder-surfing, browser autofill history) | `src/components/settings/RemoteSection.tsx:52-60` | Low |
| F4 | `clientIp` falls back to `X-Forwarded-For` (client-controlled) and then to one shared `"unknown"` bucket | `relay/src/http/rate-limit.ts:53-54` | Low |
| F5 | No frame-size limit on room WebSocket messages before `JSON.parse` (REST caps at 2 KiB) | `relay/src/room/desk-room.ts:165`, `relay/src/room/messages.ts:92` | Low |
| F6 | `register` distinguishes "no such licence key" (400) from "expired" (403) | `relay/src/http/desks.ts:62-66` | Low |

## Decisions and ADRs

- [ADR 022](../../../.arch/ADR/022-relay-on-cloudflare-durable-objects.md) /
  E022-D1: relay is a Worker + one Durable Object per desk + D1. F2 and F5 stay
  inside that shape; no new infrastructure.
- [ADR 023](../../../.arch/ADR/023-pairing-code-device-token-auth.md) /
  E022-D2: the relay is untrusted, tokens never in a URL, deliberate silence on
  auth failures. F6 aligns `register` with that silence.
- E022-D6: relay not deployed. This epic deploys nothing either.
- New decisions (record in `.plan/decisions.jsonl` during T07):
  - **D1** `ws://` is allowed only for a loopback host (`localhost`,
    `127.0.0.1`, `[::1]`) — that is how `just relay-dev` runs; any other host
    must be `wss://`/`https://`. No separate "dev mode" flag.
  - **D2** F2 is closed with a single conditional D1 statement
    (`INSERT … SELECT … WHERE (SELECT COUNT(*) …) < ?`) as the final authority;
    the existing pre-check stays so a desk at its cap does not burn a code.
  - **D3** F4: only `CF-Connecting-IP` is trusted. Missing header → `500
    relay_misconfigured`, never a shared bucket.
  - **D4** F5: room frames are capped at `MAX_FRAME_BYTES = 16 KiB`; oversize
    closes the socket with code `1009`.
  - **D5** F6: both branches return `403 invalid_license` with one message.

Architecture impact: none — no new component, data flow or integration; five
functions change behaviour at existing boundaries.

## Approaches considered

| | A — minimum: fix each finding in place | B — target: move all admission (pair, register, rate limit, frame size) into the Durable Object | C — put Cloudflare WAF / rate-limit rules in front |
|---|---|---|---|
| Summary | Six local changes, each at the line the review named | Serialize every admission decision in the room object | Offload F4/F5 to platform config |
| Effort / Risk | S / L | L / M | M / M |
| Plus | Smallest diff, every fix has its own failing test | One place for every limit | No code for rate limits |
| Minus | Admission logic stays spread over 3 files | Rewrites working, tested E022 code for 2 Low findings | Config outside the repo, untestable in `relay-test`, needs a deployed zone (E022-D6 says no) |
| Reuses | existing `relay/test` harness, `helpers.ts` | DO RPC | nothing |

**Recommendation: A.** B was weighed honestly: it is the better shape if the
relay grows more limits, but today only F2 is a real race, and a conditional
SQL insert closes it atomically without moving code. C is ruled out by E022-D6
(no deployed zone to configure).

## Scope

In: F1-F6, their tests, the security review report status update, BACKLOG
pointer. Out: deploying the relay (E022 T16), the E022 evidence-record backlog
item, any new command or auth feature.

## Acceptance criteria

1. `ws_url`/relay start refuses a `ws://` or `http://` base whose host is not
   loopback; Settings shows the refusal as `last_error`. Loopback `ws://` still
   works.
2. Two concurrent `POST /v1/pair` against a licence with `max_viewers = 1` and
   one free slot produce exactly one `201` and one `409 viewer_limit`; the
   `viewers` table holds exactly one active row.
3. The licence key input has `type="password"` and `autoComplete="off"`.
4. A request without `CF-Connecting-IP` gets `500 relay_misconfigured`; an
   `X-Forwarded-For` header alone is ignored.
5. A room frame over 16 KiB closes the socket with `1009` and is never parsed;
   a frame at exactly 16 KiB is processed.
6. `register` with an unknown key and with an expired key return identical
   status and body.
7. `just relay-test`, `cargo test --lib`, `pnpm test:unit` green; agent
   `security-reviewer` re-reads F1-F6 and confirms each closed.

## Test strategy

| Crit. | Kind | Assertion that fails today | File |
|---|---|---|---|
| 1 | Rust unit | `relay_base_allowed("ws://relay.example")` is `false` (today every URL passes); `ws://127.0.0.1:8787` is `true`; `""` (nil/empty) falls back to `RELAY_DEFAULT_URL` | `src-tauri/src/relay_client_tests.rs` (`e023_` prefix) |
| 2 | Relay integration (miniflare) | `Promise.all` of two pairs → statuses `[201, 409]` sorted; today `[201, 201]` | `relay/test/http/pair.test.ts` |
| 3 | TS unit | `getByLabelText("Licence key")` has `type="password"` | `src/components/settings/RemoteSection.test.tsx` |
| 4 | Relay integration | header absent → 500; only `X-Forwarded-For` → 500; present → normal | `relay/test/http/router.test.ts` (helpers get `ip: null`) |
| 5 | Relay integration | 16 KiB + 1 byte frame → close `1009`; exactly 16 KiB valid frame → handled; empty frame → existing `bad_frame` error | `relay/test/room/handshake.test.ts` |
| 6 | Relay integration | unknown vs expired → deep-equal responses | `relay/test/http/register.test.ts` |

Paths covered per data path: happy (valid input), nil (missing header / empty
URL), empty (empty frame), error (oversize, race loser). Rule
`rules/testing.md` applies: each new test is run once against reverted code and
must fail.

## Constraints

- Every relay change keeps `just relay-test` green; no `wrangler deploy`.
- Files ≤ 250 lines, functions ≤ 50 lines.
- The desk-side check (F1) lives in Rust, not in the React form — the form is
  not the only writer of `relay_url` (store file, IPC).
