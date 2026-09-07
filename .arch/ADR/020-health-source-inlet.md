# ADR 020: Health data arrives through a source-agnostic inlet, not a vendor client

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E021 (smartwatch integration + voice dictation) — tasks T02, T03, T04
- **Related**: [ADR 001](001-remote-display-web-kiosk.md),
  [ADR 012](012-google-fit-integration.md) (partially superseded — see below),
  [ADR 017](017-ts-rs-for-rust-ts-codegen.md),
  [ADR 021](021-voice-in-on-phone-watch-as-glance.md),
  [E021](../../.plan/epics/E021-2026-09-06-smartwatch-integration/PLAN.md)

## Context

Until this epic the app had exactly one health input, and its shape was the
vendor's shape. `GoogleFitService` owned the cache, the dedup and the day
window; `commands_google_fit.rs` handed the frontend a `StepsView`;
`StepsWidget.tsx` named the vendor in its own filename. Three problems came
out of that, and they are different problems:

- **Google Fit's REST API has an announced end of life (late 2026).** ADR 012
  saw this coming and said the fix would be "local to ~3 files". That claim
  was never tested, and it was optimistic: the vendor's name was in the IPC
  command names, the DTO names and the component name, so a replacement was
  a rename across the whole path, not a config change.
- **The phone already holds better data than the desktop can reach.** The
  watch's step and heart-rate readings land in Health Connect on the phone,
  which is an on-device API with no cloud read — a Windows process cannot
  pull it. Whatever gets that data to the desk has to *push*, and pushing
  needs an endpoint the app owns.
- **The remote display was read-only.** `remote_server.rs` served two GET
  routes on the LAN with no authentication, which was correct while nothing
  could write. The moment anything writes app state, reading and writing stop
  being the same privilege.

## Decision

Health data enters through a **trait, an aggregator and an authenticated LAN
inlet** — three pieces, each with one job.

1. **`trait HealthSource`** (`health_source.rs`) — `id()`, `async view()`,
   `async refresh()`. `view()` is cache-only, `refresh()` may go to the
   network. Google Fit implements it; so does the push source. No consumer
   names either.
2. **`HealthAggregator`** merges every registered source's `HealthView` by
   the freshest `fetched_at_ms` and passes errors through, with
   `AuthRevoked` outranking `Transient`. A healthy source never hides
   another source's error — a silent merge would make a broken Fit token
   indistinguishable from a working one.
3. **`POST /display/health`** (`remote_routes_health.rs`) — the LAN inlet.
   Body capped at 1 KiB, schema
   `{steps_today, heart_rate_bpm?, hrv_rmssd_ms?, source_id(1..64), measured_at_ms}`,
   feeding a `PushHealthSource` registered in the aggregator. A pushed
   snapshot is **withdrawn after one hour**: the source stays configured but
   reports no snapshot, so the app falls back to another source instead of
   presenting an hour-old step count as current.
4. **`fn require_token`** (`remote_auth.rs`) guards every write route with a
   constant-time compare against `DESK_REMOTE_TOKEN`. **An unset token
   answers `503`, not `200`** — unconfigured means closed. Reads stay open on
   the LAN; ADR 001's "any browser on your network" contract is unchanged for
   the routes that only read.
5. The frontend follows: `HealthWidget` (was `StepsWidget`), `useHealth`,
   `get_health_today` / `refresh_health_now`. `commands_google_fit.rs` is
   deleted. Google Fit is now one row in a table rather than the table.

### What this supersedes in ADR 012

ADR 012's *Decision* stands: the Fit client still uses OAuth2 offline access,
per-call token refresh, source auto-discovery and the two-way error
classification. What is replaced is its **Deprecation-risk mitigation**
paragraph — the claim that swapping the backend is "local to ~3 files"
behind `GoogleFitClient` + `StepsView`. The seam is now the `HealthSource`
trait and the push contract, and the answer to Fit's shutdown is a source
that stops being registered, not a rewrite of the path above it.

## Alternatives

- **Keep pulling, swap the vendor when Fit dies (Strava / Garmin / Fitbit).**
  Cheapest today, and it is what ADR 012 assumed. Rejected because every
  cloud aggregator is the same bet re-placed: each needs its own OAuth dance,
  each can be retired, and none of them holds the watch data the phone
  already has. The pull path is kept — it is now one implementation of the
  trait, not the architecture.
- **An Android companion app talking Health Connect directly to a private
  cloud endpoint.** This is the right long-term shape (and is filed as a
  candidate epic), but it is weeks of Android work and a hosting decision
  before a single step count moves. Deferred by making the *interface* — this
  push contract — the deliverable instead. A companion app becomes one more
  `source_id`.
- **An unauthenticated push endpoint.** Simplest, and consistent with the
  existing open GET routes. Rejected: anything on the LAN could then write a
  number the user reads as their own health data. The asymmetry (open reads,
  authenticated writes) is the point.
- **A token in `AppConfig` instead of `.env`.** Rejected — `AppConfig` is
  written by the settings UI and read into a ts-rs-exported DTO, so the token
  would cross the IPC boundary and land in the frontend. `AppConfig` carries
  only the derived `remote_token_set` boolean; the value stays in `.env`.
- **Merging sources by a fixed priority list instead of by freshness.**
  Rejected: priority encodes an assumption about which device the user is
  carrying right now, which changes hourly. Freshness measures it.

## Consequences

- **Positive.** Fit's shutdown is a source that stops registering. The phone,
  a `curl` from a script, or a future companion app are all the same kind of
  thing to the app. The widget and the phone snapshot never learn which
  source produced a number.
- **Positive.** `/display/api` and the WS snapshot both carry `health`, so
  the phone shows steps for the first time — previously the widget bailed out
  whenever it was not running under Tauri.
- **Cost.** The remote server now has a privilege boundary to maintain.
  `require_token` guards writes today; every future write route must go
  through it, and nothing in the compiler enforces that. The 7 tests in
  `remote_auth_tests.rs` (including the 256-length case a naive `u8`
  accumulator gets wrong) are the guard against a lazy re-implementation.
- **Cost.** The user has to set `DESK_REMOTE_TOKEN` before anything can push.
  A missing token is a `503` with the reason documented in
  `docs/REMOTE_DISPLAY.md`, not a silent no-op.
- **Operational.** The one-hour staleness window is a guess, not a
  measurement. It is a constant (`STALE_AFTER_MS`) precisely so the first
  real complaint moves one line.
- **Testing.** `remote_routes_health_tests.rs` drives the real seam — axum
  route → `PushHealthSource` → `HealthAggregator` → `/display/api` — rather
  than stubbing either side of it, per the repo's rule against mocking both
  ends of an internal seam.

## Related

- Source: `src-tauri/src/health_source.rs`, `health_models.rs`,
  `remote_auth.rs`, `remote_routes_health.rs`, `google_fit_service.rs`,
  `commands_health.rs`, `src/hooks/useHealth.ts`,
  `src/components/HealthWidget.tsx`.
- User docs: [`docs/REMOTE_DISPLAY.md`](../../docs/REMOTE_DISPLAY.md)
  § Health Push Inlet.
- Developer contract: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
  § Health sources and the LAN inlet.
- Architecture: [`ARCHITECTURE.md`](ARCHITECTURE.md) § Health Sources and the
  LAN Inlet.
