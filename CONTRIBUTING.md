# Contributing to MoveUp

This guide is for developers and coding agents. It is not required to evaluate or
use a future released MoveUp installer.

## Development model

MoveUp is one Tauri desktop process: the tray app owns the sensor, local SQLite
data and the embedded browser dashboard. Its remote dashboard is served by that
same process over HTTP and WebSocket; it is not a separate product server.

- **Ordinary development:** run `pnpm tauri:dev` (or its `:live`, `:mock` or
  `:demo` variant). `scripts/tauri-dev.ps1` avoids colliding with an existing
  development instance.
- **Local always-on use:** PM3 may supervise a release MoveUp binary with
  `--minimized`, then `idomains` may expose its chosen dashboard port as a local
  `*.internal` name. PM3 is a workstation convenience, not a runtime dependency
  of MoveUp and not something shipped to end users.
- **Distribution:** ship a signed Windows installer containing the release
  binary. Do not ship Rust, Cargo, PM3 or a developer helper as part of the
  user-facing product.

## Health sources and the LAN inlet

If you are adding a device, an app or a script that feeds health data into
MoveUp, you do **not** write a client inside MoveUp. You implement one of two
things.

**Pushing from outside the process** — the normal case, and the one an Android
companion app or a phone automation should use. Post to `POST /display/health`
with the `X-Desk-Token` header and the schema documented in
[`docs/REMOTE_DISPLAY.md`](docs/REMOTE_DISPLAY.md). Pick your own
`source_id`; the app merges your reading with every other source by
`measured_at_ms` freshness and never tells the UI which source won. Your
snapshot is withdrawn after an hour, so keep pushing.

**A new in-process source** — for something MoveUp must *pull*. Implement
`trait HealthSource` (`health_source.rs`): `id()`, `view()` (cache-only, no
network) and `refresh()` (may go to the network), then register it in the
`HealthAggregator`. Two rules the aggregator relies on: return a `HealthView`
with an `error_kind` rather than swallowing failures, and make outbound
endpoints injectable the way `google_fit.rs` does (`Endpoints::at_base`) so
the source is testable against `wiremock` rather than the live API.

Do not add a vendor's name to a command, a DTO or a component. That is what
[ADR 020](.arch/ADR/020-health-source-inlet.md) exists to prevent — it is
exactly why Google Fit's shutdown was a rewrite before this epic.

### The write gate

Every route on `:3390` that changes app state goes through
`remote_auth.rs::require_token`. There is no second gate and there should not
be one. Three behaviours are contractual, and the tests in
`remote_auth_tests.rs` enforce them:

- an **unset** `DESK_REMOTE_TOKEN` answers `503`, never `200` — unconfigured
  means closed, and a missing secret must never read as a working endpoint;
- a **wrong** token answers `401`, so the two failures stay distinguishable;
- the compare is constant-time, including the length-multiple-of-256 case a
  naive `u8` accumulator gets wrong.

Cap the body before parsing, and answer `422` for a body that parses but
carries an unusable value — a malformed push must never look like an empty
one.

## Voice dictation is an event path

`POST /display/voice` records and acknowledges. It does not steer the app.

The session engine is a pure core ([ADR 015](.arch/ADR/015-pure-ergo-engine.md)),
and a voice route is an adapter, so nothing on this path may call into
`session_*.rs` or mutate `SessionState`. A parsed `Intent` becomes an
`events.log` line, a `voice_notes` row and an acknowledgement. `Snooze` is the
**only** intent with a side effect, and it sets `CommunicationPolicy`'s
existing snooze fields — the same ones the alert popup's snooze button sets.
A spoken "I'm walking" is deliberately not allowed to set `DeskState`; making
it do so is a new engine feature with its own ADR, not a line in this handler.
Reasoning: [ADR 021](.arch/ADR/021-voice-in-on-phone-watch-as-glance.md).

Adding a phrase means adding a row to the `RULES` table in `voice_intent.rs`
and a case to the acceptance table in `voice_intent_tests.rs`. `RULE_COUNT` is
asserted, so a pattern that fails to compile cannot silently shrink the
parser. Intent parsing stays offline and deterministic — no API key, no
network. The LLM is used only for the reply, where being wrong is cosmetic.

Every leg of the pipeline is independently optional (no database, no logger,
no AI key, no webhook). Keep it that way: a new leg degrades the
acknowledgement, it never fails the request.

### Secrets

`DESK_REMOTE_TOKEN`, `OPENROUTER_API_KEY` and `DESK_NOTIFY_WEBHOOK_URL` live
in `apps/desk/.env` and are read the way `Credentials::from_env` reads them.
They never enter `AppConfig`, which is ts-rs exported and therefore crosses to
the frontend — `AppConfig` carries the derived `remote_token_set` boolean, not
the token. Never log a secret, and keep sanitized error strings on the path to
the UI while full errors go to `log::warn!`.

## Prerequisites

- Windows with the Rust MSVC toolchain and Node.js/pnpm;
- optional: the VL53L1X desk controller for live sensor work.

Install dependencies and run checks from the repository root:

```powershell
pnpm install
pnpm test:all
pnpm tauri:build
```

## Windows Application Control

Cargo generates small unsigned executable build scripts for dependencies during
a Rust build. Some Windows Application Control (WDAC/App Control) policies block
those scripts with `os error 4551`. That is a developer-machine policy issue,
not something a MoveUp user should encounter.

Do not disable App Control globally and do not add a broad antivirus exclusion.
If your managed development machine blocks Cargo, request a narrowly scoped
developer policy from its administrator for the approved Rust toolchain and this
repository's generated build artifacts. Record the policy owner and review it when
the build toolchain changes. A signed release installer is the correct answer for
end-user machines; it is not a substitute for a controlled developer policy.

## Release baseline (CI)

The `Release baseline` workflow is manual and accepts only a full 40-character
commit SHA. It checks out that exact revision, runs the frontend and Rust test
gates, builds the NSIS installer, and publishes a seven-day unsigned diagnostic
artifact with `manifest.json`. The manifest records the source revision,
installer filename, byte size, and SHA-256 hash.

This baseline is not deployable output. Do not install it on a Smart App Control
machine or use it as a signing workaround. The next release stage must use an
approved managed signing provider, verify Authenticode trust and timestamping,
and only then publish a signed installer.

## Before planning changes

Load the global `how-we-build` skill before writing a plan or making a design
decision. It contains the ecosystem conventions, including the distinction between
this file and README.
