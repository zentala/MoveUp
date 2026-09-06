# Brief for planning: Cross-Device & Phone Relay (Pro Feature)

## TLDR

This is a **research brief**, not a plan — a fresh planning session (using
the `Plan` agent) reads this, then writes `PLAN.md` + `HANDOFF.md` +
`PRES.md` under a new epic `E022-2026-09-06-cross-device-phone-relay/`.
Research already happened (cheap model, file discovery); findings are
compiled below with file:line pointers.

**Decision from Paweł**: yes, plan this fully and ship it — "będziemy w
ogóle to wydawać, lecimy z tym. Rozpisz API-ki." He wants the actual APIs
specified, not just a concept.

## Ground truth — the vision document's own spec (read first)

`.plan/vision/2026-03-25-premium-tier-definition.md`, lines 130-139 —
**quoted verbatim, this is the closest thing to a spec that exists today**:

```
### Epic E014: Cross-Device & Phone Relay (Pro Feature #4)

Cloud relay for phone display (works outside LAN). Builds on E009 (web kiosk).

| ID | Task | Description | Effort |
|----|------|-------------|--------|
| T01 | WebSocket relay server | Cloudflare Durable Objects: relay desk events to phone | M |
| T02 | Desktop: cloud relay mode | Send events to cloud relay instead of (or in addition to) LAN | S |
| T03 | Phone: cloud connection | Connect to cloud relay when not on same LAN | S |
| T04 | Latency optimization | Minimize delay between desk event and phone display | S |
```

Note the epic number "E014" in this old vision doc collides with the real,
already-shipped `E014-2026-09-05-supervised-release-rollback` in
`.plan/epics/` — that numbering is stale (the vision doc predates this
repo's real epic sequence). Use `E022` for the real epic; do not reuse
E014.

## What exists today — read these files

| File | Lines | What's there |
|---|---|---|
| `CLAUDE.md` | "Remote Display (Phone Dashboard)" section | High-level description of today's feature |
| `docs/REMOTE_DISPLAY.md` | 1-100 | User setup docs: LAN IP + port 3390, Fully Kiosk Browser on Android |
| `src-tauri/src/remote_server.rs` | 1-198 | axum HTTP+WS server, binds `0.0.0.0:3390`, **no authentication anywhere**, max 10 clients |
| `src-tauri/src/ws_broadcaster.rs` | 1-100 | `DisplayEvent` enum — **one-way only** (desktop → phone): `StateChanged`, `DeviceConnected`, `DailyReset`, `Heartbeat` |
| `src/hooks/useRemoteDesk.ts` | 1-157 (esp. 68-120 WS connect, 143-145) | WS client with REST fallback + exponential backoff. Lines 143-145: `calibrate`/`setSitLimit`/`setStandLimit` are **no-ops today** — the phone side cannot control anything |
| `src/components/ConnectionOverlay.tsx` | 1-68 | Connection/sensor status banner shown in remote mode |
| `.arch/ADR/001-remote-display-web-kiosk.md` | 1-54 | The original decision: embedded HTTP+WS, LAN-only, "Tauri Mobile planned as Phase 2" (never built) |

## Gap between today and a real Pro feature — established by research

| Gap | Why it blocks calling this "Pro" | Where |
|---|---|---|
| **LAN-only** | Users want the dashboard away from home too — this is the vision doc's entire T01-T03 | No cloud path exists at all today |
| **Zero authentication** | Anyone on the same LAN (roommate, guest, office wifi) sees your ergonomics data, no login | `remote_server.rs` has no auth check anywhere |
| **One-way only, no control** | Today's remote view is read-only; a paid feature should let you change limits/profile from the phone | `ws_broadcaster.rs` only emits; `handle_ws_client`'s receive loop ignores incoming messages (`Some(Ok(_)) => {}`); `useRemoteDesk.ts:143-145` control functions are no-ops |
| **No cross-device state sync** | If you change a limit on desktop, there's no cloud record of it for the phone to read when reconnecting from elsewhere | No cloud DB for user config exists |
| **Browser-kiosk-only client** | Today's client is "point Fully Kiosk Browser at an IP" — a Pro feature likely wants a cleaner connection story (QR pairing? account-based?) | `docs/REMOTE_DISPLAY.md` |

Related but NOT this epic's scope — three **dev-mode** gaps already filed
in `.plan/BACKLOG.md` (search "Dev-mode remote display", ~lines 150-208):
missing Vite proxy for `/display`, an unguarded Tauri `invoke()` in
`App.tsx`, and no session-level mock for the popup UI. Those block
**local development/testing** of remote display, not the cloud-relay
feature itself — do not fold them into this epic; leave them where they
are (tied to E015-T05).

## What this brief does NOT resolve — the planning session must decide

Real trade-offs for the plan's "Alternatives considered" section:

- **Relay infrastructure**: the vision doc names Cloudflare Durable
  Objects specifically. Confirm or replace this choice — check
  `knowdlege/my-severs.md` for Paweł's stated hosting preferences
  (self-host vs Cloudflare) before assuming the vision doc's pick is still
  right eleven months later. State the recommendation either way.
- **Authentication model**: pairing code (QR from desktop, scanned by
  phone) vs. account/login vs. a long-lived device token. Pick one, with
  the reasoning — this app has no existing account system, so whichever
  is picked is also the app's *first* auth surface, which is a bigger
  architectural decision than it looks.
- **Two-way control scope**: does "control" mean just "acknowledge a
  popup remotely" (small) or "change ergonomic profile/limits from phone"
  (bigger, touches `config.rs`/`session_manager.rs` from a new remote
  write path with its own security implications)? Name a concrete, scoped
  v1.
- **LAN mode stays or gets replaced?** The existing LAN-only path (free,
  no cloud dependency, no auth needed since it's already local-network-
  trusted) could remain the free-tier behavior, with cloud relay as the
  Pro-tier addition on top — decide and state this explicitly, since it
  affects whether `remote_server.rs` gets modified in place or a new
  parallel path is added.

## Constraints from CLAUDE.md / repo conventions to honor in the plan

- File ≤ 250 lines, function ≤ 50 lines.
- Any new external dependency or hosting choice → new ADR in `.arch/ADR/`
  (check current highest number first).
- **Security review is not optional here**: this epic adds remote write
  access and a new auth surface to an app that has none today. The plan's
  own review step (pipeline step 5, `skill review-loop`) must not be
  skipped for this epic regardless of point count.
- Pricing/tier claims must read from `.plan/vision/config/pricing.json`,
  never be hardcoded from the old vision doc's prose.
- Evidence contract required (`rules/evidence.md`); since this has a UI
  and a security surface, both a `browser` agent visual pass AND explicit
  manual security checks (e.g. "unauthenticated request to the relay is
  rejected") belong in the acceptance criteria.

## Deliverables — where to save them

Create `.plan/epics/E022-2026-09-06-cross-device-phone-relay/` with:
- `PLAN.md` — full plan (TLDR, Decisions and ADRs, Alternatives considered
  with the relay/auth/control-scope/LAN-mode questions above resolved into
  a recommendation, Architecture impact section — this WILL trigger a new
  ADR given the new supervision boundary between local app and cloud relay
  — Test strategy, Acceptance criteria, Evidence contract). Rough API
  surface to specify concretely, per Paweł's explicit request: the relay
  server's message schema (auth handshake, event envelope), the desktop's
  outbound connection contract, the phone client's connection/pairing
  flow.
- `HANDOFF.md` — task breakdown with points, `agent:` per task, and a
  `## AO` YAML block (see `~/.claude/skills/ao/SKILL.md` §6 for the shape;
  §0 for whether this epic's size/wave-shape warrants full AO or direct
  dispatch).
- `PRES.md` — board-style presentation per `rules/workflows.md` "PRES.md"
  section, since Paweł approves from this, not from `PLAN.md`.

This session plans only — do not implement. Close by pointing Paweł at
`PRES.md` for approval.
