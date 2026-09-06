---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 51
agent: ts-dev
wave: 5
parallel: []
depends-on: [E020]
blocked-by: ""
---

# E021 — Smartwatch integration + voice dictation (phone bridge)

Source brief: [`../../reports/2026-09-06-brief-smartwatch-integration.md`](../../reports/2026-09-06-brief-smartwatch-integration.md).
Decision that opened this epic: [`../../reports/2026-09-06-decyzje-do-planowania.md`](../../reports/2026-09-06-decyzje-do-planowania.md)
§Pytanie 2, option 2a — Paweł: "rozplanuj pełny 2a", "będę to wydawał".
Board deck (Polish): [`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

The brief's two hard facts hold: (1) Paweł's Xiaomi Watch 4 runs a closed
RTOS with no app API, and (2) no official watch SDK on any platform gives a
third-party app the microphone. Planning added a third fact the brief did
not have: **Google Fit's REST API — today's only step source — is being shut
down at the end of 2026** (no exact date published as of 2026-08; new sign-ups
blocked since 2024-05). So "steps are already solved with zero code" is true
for roughly four more months. This epic therefore builds the piece that
survives: a **source-agnostic health inlet** in the app (`HealthSource`
trait, Google Fit behind it, plus a token-guarded LAN push endpoint any
phone-side bridge can feed), **voice-in on the phone** through the existing
`/display` page (keyboard dictation always works; Web Speech API when the
browser allows it), a local **intent parser** that turns a transcript into
snooze / note / walk events, an **outbound notification webhook**
(ntfy-compatible) so replies reach the wrist by the phone-mirroring the
watch already does, and an **opt-in AI reply** (BYOK OpenRouter). The
official-SDK Android side (Health Connect companion) is named as the
target architecture and deferred to a follow-up epic, because it is a
different toolchain and this epic's push contract is exactly what it
needs. 10 tasks, 51 points, 5 waves with real fan-out — full AO.

## Problem

- **The only health source is on a deprecation clock.** ADR 012 chose Google
  Fit REST because it is the one Google source a Windows desktop app can
  read; it also noted "Google has signaled Fit API deprecation". Web
  research 2026-09-06 (see TLDR): end-of-service late 2026, replacement is
  Health Connect, which is **on-device Android only** — unreachable from
  Windows without a phone-side bridge. `google_fit_service.rs` is
  encapsulated (ADR 012 said "swapping the backend is local to ~3 files"),
  but there is no seam to swap *into*: `StepsView`/`StepsSnapshot`
  (`google_fit_models.rs:71-101`) are Google-shaped, `lib.rs:185` manages a
  concrete `GoogleFitService`, and `StepsWidget.tsx:44,66` returns nothing
  in remote (browser) mode because it only knows Tauri `invoke`.
- **No inbound channel exists at all.** `remote_server.rs:70-82` exposes
  two GET routes (`/display/ws`, `/display/api`); nothing accepts a POST.
  Every vision doc since March lists "smartwatch steers the AI agent by
  voice" (`2026-03-21-product-vision-coach-and-business.md:98`) and "Body —
  smartwatch (movement, HRV)" (`:66`), and there is no code path for
  either.
- **The watch cannot host anything.** `homelab/ref/wearables.md:10-17`:
  Watch 4 = HyperOS/NuttX, mirrors phone notifications, no third-party
  apps. The same file already prescribes the architecture this plan
  adopts: "voice capture goes via the phone; watch = glance surface".
- **Voice from the wrist is impossible under any official SDK** — Apple
  HealthKit, Wear OS Health Services and Garmin Connect IQ all lack a mic
  API for third parties (brief, "Hard facts" §2). Buying a Wear OS watch
  would not change this.
- **The tier boundary is undefined for this feature.** Vision docs place
  "smartwatch integration" in Pro (`premium-tier-definition.md:27`) and
  in Founder's Edition contents (`config/pricing.json:39`), while ADR 005
  keeps the desktop app open. Nothing says which *part* of a LAN-local
  feature is the paid part.

## Decisions and ADRs

Read before planning: `.plan/decisions.jsonl` (E015-D1/D2/D4, E017-D3,
E018-D3, E020-D3 — none constrain this epic beyond E018-D3, which means
every new Rust DTO here must be `ts-rs` exported so `src/generated/` mirrors
it), ADR 001 (remote display = embedded HTTP+WS — this epic extends that
server, it does not add a second one), ADR 005 (open core), ADR 012
(Google Fit — superseded in part by ADR 020 below), ADR 015 (pure engine:
**voice intents must not mutate the session engine**; they are recorded as
events and signals, see D3), ADR 017 (ts-rs codegen), ADR 018 (PM3 owns the
app process; nothing here changes that).

- **D1 — Phone is the bridge, watch is a glance surface.** Voice capture
  and health-data relay both run on the phone; the watch shows replies via
  notification mirroring. Rejects custom firmware (Alternative C). Source:
  brief §"What this brief does NOT resolve", `wearables.md:14-15`.
- **D2 — Source-agnostic health inlet.** A `HealthSource` trait with two
  implementations in this epic (Google Fit pull; LAN push) and a documented
  push contract, so Google Fit's shutdown is a config change, not a rewrite.
  → **ADR 020** `.arch/ADR/020-health-source-inlet.md` (new; partially
  supersedes ADR 012 — ADR 012's "Decision" stays valid for the Fit client,
  its "Mitigation" paragraph is replaced by the inlet).
- **D3 — Voice intents are events, not engine inputs.** A recognised intent
  (snooze, note, walk) is written to `events.log` + a `voice_notes` table
  and surfaced as a signal; it never calls `on_reading_at` or changes
  `SessionState`. ADR 015 forbids adapters from carrying policy; a manual
  "I am walking" override is a future engine feature with its own ADR, not
  a side effect of this one. → **ADR 021**
  `.arch/ADR/021-voice-in-on-phone-watch-as-glance.md` (new; records D1
  and D3 and the rejected alternatives).
- **D4 — Tier line: local ships open, hosted stays Pro.** The LAN push
  endpoint, voice capture, intent parser and webhook ship in the open app.
  The AI reply is **BYOK** (user's own OpenRouter key in `.env`; absence =
  feature off, never an error). Hosted AI coaching and cloud relay remain
  the Pro/Founder boundary exactly as ADR 005 draws it. `source: agent`,
  recorded in `decisions.jsonl`; Paweł may overturn it in PRES §6 without
  changing any task except T08's flag name.
- **D5 — Web Speech API is an enhancement, not the primary path.**
  `SpeechRecognition` needs a secure context in Chrome; `/display` is
  served over plain `http://<LAN-IP>:3390`. Primary voice path = a text
  field the user dictates into with the keyboard's mic (Gboard/Samsung —
  official OS dictation, works on any origin, the same "no-build path" the
  `notify` skill's wave-1 chose). `SpeechRecognition` is used when
  `window.SpeechRecognition` exists **and** `navigator.permissions` reports
  mic usable; otherwise the button is hidden, never broken. T01 measures
  this on Paweł's actual phone before T05 is written.
- **D6 — Google Fit stays until it dies, with a dated warning.** The Fit
  source is not removed; the widget shows a "source ends late 2026" hint
  when it is the only source. Removal is a one-line config change once the
  push source is live.
- Next ADR numbers: `.arch/ADR/` currently ends at 019 → this epic claims
  **020** and **021**. If another epic claims them first, T09 renumbers.

How-we-build: `~/code/harness/how-we-build/patterns.md` and
`conventions.md` were read. Neither pattern (controlled mutation broker;
canonical templates with generated views) applies to a device-data inlet;
the README/CONTRIBUTING split convention applies to T09's user docs
(`docs/REMOTE_DISPLAY.md` stays user-facing, developer contract goes to
`CONTRIBUTING.md`). `~/.claude/how-we-build/` does not exist — not found,
proceeded with the harness copy.

## Alternatives

| | A. Phone bridge, watch as glance (**this plan**) | B. Minimum — Fit-only aggregation, defer voice | C. Custom firmware on Watch 4 | D. Target — official Android companion (Health Connect + SpeechRecognizer) |
|---|---|---|---|---|
| Summary | Extend the existing `:3390` server with a push inlet and a voice inlet; voice captured on the phone's browser; replies via webhook → phone → mirrored to watch | Wrap `GoogleFitService` in a `HealthSource` trait, add HR, stop there; voice-in goes to a later epic "once a Wear OS device is owned" | Flash unofficial software on the Watch 4 to get a mic | Kotlin/Tauri-mobile app on the phone reading Health Connect (official Google SDK, where Mi Fitness syncs the Watch 4) and using Android `SpeechRecognizer`; pushes to MoveUp over LAN |
| Effort | L (51) | S (13) | XL, unbounded | XL (Android toolchain, Play signing, two codebases) |
| Risk | M | L | H | H |
| Pros | Zero new hardware; works with the watch Paweł owns; every piece is an official/standard API (HTTP, Web Speech, OS dictation, ntfy); survives Fit shutdown; the push contract is the exact input D needs later | Cheapest; ships HR | Only path to a true wrist mic | Cleanest "official SDK" story; reads HR/HRV/sleep directly; works offline from Google |
| Cons | Voice comes from the phone, not the wrist; Web Speech may be blocked on http (mitigated by D5); one more device in the loop | Leaves the app with no inlet when Fit dies — the "wait for Wear OS" premise is false anyway, Wear OS has no mic SDK either; Paweł explicitly asked for the full 2a | Contradicts "official SDK, out of the box"; no vendor surface to test against; bricking risk; nothing to ship | A different repo/toolchain; 10-20 pts of Android plumbing before the first byte of value; still needs A's push endpoint to talk to the desktop |
| Reuses | `remote_server.rs`, `ws_broadcaster.rs`, `useRemoteDesk.ts`, `/display` React UI, `GoogleFitService`, `EventLogger`, `db.rs`, `openrouter` skill defaults, ntfy at `ntfy.internal` | `GoogleFitService` | nothing | A's push contract, `/display` UI as WebView |

**Recommendation: A, with D named as the follow-up.** B is rejected
because its premise ("defer until Wear OS") is falsified by the brief's own
fact 2 and because it leaves the app source-less at Fit shutdown. C is
**explicitly rejected** — it contradicts Paweł's stated requirement and
has no test surface. D is the right *eventual* home for the Android half,
but it cannot deliver anything until A's inlet exists, and it belongs in a
repo with an Android toolchain; it is filed as a candidate E022 in
`BACKLOG.md` by T09, with this epic's push contract as its interface.

## Scope

**In**: `src-tauri/src/health_source.rs` (new trait + aggregator),
`health_models.rs` (new DTOs, ts-rs), `google_fit_service.rs` (implements
the trait; HR fetch added), `google_fit_client.rs` (HR aggregate call),
`google_fit_models.rs`, `commands_google_fit.rs` → `commands_health.rs`,
`remote_server.rs` (route mounting only), `remote_routes_health.rs` (new),
`remote_routes_voice.rs` (new), `remote_auth.rs` (new, shared-secret
check), `ws_broadcaster.rs` (`health` in `RemoteDisplayState`, new
`VoiceAck` event), `voice_intent.rs` (new), `db_voice_notes.rs` (new),
`event_logger.rs` (`VOICE` line), `notify_webhook.rs` (new), `voice_ai.rs`
(new), `config.rs`/`AppConfig` (webhook URL, remote token), `lib.rs`
(mods, manage, invoke lists), `src/components/HealthWidget.tsx` (renamed
from `StepsWidget.tsx`), `src/components/VoiceCapture.tsx` (new),
`src/hooks/useHealth.ts` (new, Tauri + remote), `useRemoteDesk.ts`
(health in snapshot), `src/test/scenarios*.ts` (voice + health
scenarios), `src/generated/` (codegen output), `.arch/ADR/020`, `021`,
`ADR/012` (partial supersede note), `.arch/ARCHITECTURE.md`,
`.arch/UX-FLOW.md`, `CLAUDE.md`, `CONTRIBUTING.md`,
`docs/REMOTE_DISPLAY.md`, `.plan/decisions.jsonl`, `PROJECT.xml`,
`scripts/check-e021-t09-docs.mjs`, `scripts/check-e021-t01-spike.mjs`.

**Out**: any Android/Kotlin/Tauri-mobile code (candidate E022); custom
watch firmware (rejected); cloud relay, accounts, hosted AI (Pro, E010
cloud backend); a manual "walking" override inside the session engine (own
ADR later); Wear OS / Apple Watch / Garmin native apps; TLS on `:3390`
(the http-origin limitation is handled by D5, not by certificates);
Analyst-window charts for HR/voice notes (data lands in the DB and the
Catalog tab lists the new source via the existing catalog mechanism — a
chart is a later E012-style task); code signing (deferred, BACKLOG).

## Architecture impact

```
phone browser /display ──POST /display/voice──▶ remote_routes_voice.rs ─▶ voice_intent.rs ─▶ db_voice_notes + events.log
                       ◀──WS VoiceAck / webhook push (ntfy) ◀── notify_webhook.rs ◀──┘ (+ voice_ai.rs reply, BYOK)
phone bridge (any)  ───POST /display/health──▶ remote_routes_health.rs ─▶ PushHealthSource ─┐
Google Fit REST     ───pull every 5 min──────▶ GoogleFitService        ─▶ HealthAggregator ─┴─▶ HealthView (IPC + WS snapshot)
```

- `.arch/ARCHITECTURE.md`: remote-server section gains the two POST routes
  and the shared-secret rule; a new "Health inlet" section replaces the
  Google-Fit-specific paragraph.
- ADR 012 gets a "Superseded in part by ADR 020" line (status stays
  `accepted` for the Fit client itself).
- `CLAUDE.md` "Google Fit Integration" section becomes "Health sources"
  with the push contract and the new `.env` keys (`DESK_REMOTE_TOKEN`,
  `DESK_NOTIFY_WEBHOOK_URL`, `OPENROUTER_API_KEY`).
- `.arch/UX-FLOW.md`: new rows for the voice capture panel and the HR badge
  (rule `ux-flow-sync.md`).
- `PROJECT.xml`: new files and the two new IPC commands (rule "update
  before every commit").

## Security notes (system boundary)

`/display` today is unauthenticated read-only on the LAN (max 10 WS
clients). The two POST routes are the first *writes* — they are guarded by
a shared secret (`DESK_REMOTE_TOKEN`, header `X-Desk-Token`, constant-time
compare) and refuse with 401 when the token is unset or wrong; when unset,
the server logs once at startup that push/voice are disabled. Bodies are
size-capped (voice 4 KiB, health 1 KiB) and schema-validated; a transcript
is stored verbatim but never interpolated into a shell, SQL (parameterised
via `tauri-plugin-sql`) or a log format string. The webhook URL is
outbound-only, user-configured, and never receives the token.

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Spike, on real hardware: (a) `refresh_steps_now` still returns 200 today and whether `com.google.heart_rate.bpm` for the Watch 4 appears in `dataSources` (Mi Fitness sync scope); (b) does Mi Fitness write to Health Connect on Paweł's phone (Settings → Health Connect → app permissions), for E022's feasibility; (c) on the phone's Chrome and Fully Kiosk: does `SpeechRecognition` start on `http://<PC-IP>:3390`, and does keyboard-mic dictation into a textarea work. Output: `reports/2026-09-06-spike-results.md` with a yes/no table, feeding D5 and T02's HR field | 2 | main | 0 |
| T02 | `HealthSource` trait + `HealthSnapshot { steps_today, heart_rate_bpm: Option<u16>, hrv_rmssd_ms: Option<f32>, source_id: String, fetched_at_ms }` + `HealthView` (ts-rs); `GoogleFitService` implements it (HR via a second aggregate call when T01 found the data type, else `None`); `HealthAggregator` merges N sources by freshness; `commands_health.rs` replaces `commands_google_fit.rs` (`get_health_today`, `refresh_health_now`); `lib.rs` manages the aggregator | 8 | ts-dev (Rust) | 1 |
| T05 | `VoiceCapture.tsx` on `/display`: textarea + Send (primary), mic button shown only when `SpeechRecognition` is available and the mic permission query does not say `denied`; live interim transcript; posts `{ transcript, lang, captured_at_ms }` to `POST /display/voice` with the token from `localStorage` (entered once via a small settings sheet); shows the `VoiceAck` reply; scenario in `src/test/scenarios-other.ts`; mockup route entry | 5 | ts-dev (React) | 1 |
| T07 | `notify_webhook.rs`: outbound `POST` of `{ title, message, priority, tags }` to a configured URL (ntfy topic URL is the reference target, plain JSON body so any webhook works); called from the existing sit-limit toast path and by T06 for voice acks; 5 s timeout, one retry, never blocks the caller; `AppConfig.notify_webhook_url` + Settings → More toggle; wiremock tests | 5 | ts-dev (Rust) | 1 |
| T03 | `remote_auth.rs` (token check) + `remote_routes_health.rs`: `POST /display/health` with `{ steps_today, heart_rate_bpm?, hrv_rmssd_ms?, source_id, measured_at_ms }` → `PushHealthSource` (in-memory, last-write-wins per `source_id`, stale after 1 h); `remote_server.rs::build_router` mounts it; `RemoteDisplayState.health: HealthView` so `/display/api` and the WS snapshot carry it; `docs/REMOTE_DISPLAY.md` gets a `curl` example | 8 | ts-dev (Rust) | 2 |
| T08 | `voice_ai.rs`: `reply(transcript, &SessionStateDto, cfg) -> Result<Option<String>>` calling OpenRouter chat completions (model from `AppConfig.voice_ai_model`, default per `openrouter` skill's cheap-tier pick) with a fixed ≤60-word coaching system prompt; `None` when no key; 8 s timeout; wiremock tests for happy / no key / 401 / 5xx / malformed | 5 | ts-dev (Rust) | 2 |
| T04 | Frontend: `StepsWidget.tsx` → `HealthWidget.tsx` (steps + optional HR badge, source label, "Fit ends late 2026" hint when Fit is the only source), `useHealth.ts` choosing Tauri `invoke` or the WS snapshot so the badge finally renders on the phone; tests for all five render states × two transports | 5 | ts-dev (React) | 3 |
| T06 | `remote_routes_voice.rs` + `voice_intent.rs` (rules: `snooze (\d+)`, "drzemka N", "note:/notatka:", "spacer/walk" start/end, else `Note`) + `db_voice_notes.rs` (table `voice_notes(id, captured_at, transcript, intent, reply)`) + `EventLogger` `VOICE` line + `SnoozeSignal` into `CommunicationPolicy`'s existing snooze path + `VoiceAck` WS event + webhook push via T07 + AI reply via T08 (when configured); IPC `list_voice_notes(day)`; catalog entry for the Analyst Catalog tab | 8 | ts-dev (Rust) | 3 |
| T09 | Docs: ADR 020, ADR 021, ADR 012 supersede note, `ARCHITECTURE.md`, `UX-FLOW.md`, `CLAUDE.md` section rewrite, `CONTRIBUTING.md` push/voice contract, `docs/REMOTE_DISPLAY.md` voice setup, `PROJECT.xml`, `decisions.jsonl` D1-D6, BACKLOG entry for candidate E022 (Android Health Connect companion) and for `epics/INDEX.md` creation; `scripts/check-e021-t09-docs.mjs` grounds every claim in a symbol | 3 | main | 4 |
| T10 | Verify: `just check` green; all evidence records `current`; `browser` agent drives `/display` at a phone viewport (360×780, DPR 3): health badge renders from a pushed value, voice panel accepts typed text and shows the ack; the `SpeechRecognition` mic path is **manual** (Paweł's phone, recorded in `evidence/.local/`) because headless Chrome cannot grant a mic | 2 | verify | 5 |

Wave points: W0=2, W1=18 (T02 ∥ T05 ∥ T07 — disjoint: Rust health files /
React display files / `notify_webhook.rs`), W2=13 (T03 ∥ T08 — T03 owns
`remote_server.rs`, T08 is a standalone module), W3=13 (T04 ∥ T06 — T04 is
TSX only, T06 is Rust + mounts its route; both depend on T03),
W4=3, W5=2. Every wave ≤ 40; three waves have real fan-out → AO earns its
isolation (skill `ao` §0).

## Test strategy

Four shadow paths per feature (`rules/testing.md`); Rust tests in sibling
`<module>_tests.rs`, TS co-located; external HTTP via `wiremock` (Rust) and
`msw`-free `fetch` mocks (TS). The real-seam rule: at least one test runs
the real `PushHealthSource` through the real `HealthAggregator` and the real
`remote_routes_voice` handler through the real `voice_intent` parser — no
stubbing both sides.

- **T02 (`HealthSource` + aggregator)** — happy: Fit source returns
  `steps=4321, hr=Some(72)` → `HealthView` carries both with
  `source_id="google_fit"`. nil: no source configured → `configured=false`,
  `snapshot=None` (fails today: type does not exist). empty: aggregate
  response with zero buckets → `steps_today=0`, `hr=None`, not an error
  (existing `total_steps_returns_zero_for_empty_response` extended). error:
  Fit 401 → `error_kind=auth_revoked` preserved through the aggregator;
  two sources where one errors → view shows the healthy one and the error
  of the other. File: `health_source_tests.rs`, `google_fit_http_tests.rs`
  (HR case added).
- **T03 (push route)** — happy: `POST /display/health` with a valid token
  and body → 204, next `/display/api` shows the pushed steps with
  `source_id` from the body (**real seam**: route → `PushHealthSource` →
  `HealthAggregator`). nil: no `X-Desk-Token` header → 401, state
  untouched. empty: `{}` body → 400 with a field list, state untouched.
  error: token configured wrong / body 2 KiB → 401 / 413. Staleness: a
  push 61 min old loses to a Fit snapshot 5 min old. File:
  `remote_routes_health_tests.rs` (axum `oneshot`, pattern from
  `remote_server_tests.rs`).
- **T05 (`VoiceCapture`)** — happy: type text, click Send → `fetch` called
  with token header and body, ack rendered. nil: `window.SpeechRecognition`
  undefined → no mic button in the DOM (assert absence, not "renders").
  empty: Send with whitespace-only text → disabled, no request. error:
  fetch 401 → "token rejected" hint with a link to the settings sheet;
  fetch network error → retry affordance, transcript preserved. File:
  `VoiceCapture.test.tsx`. Flake note: timers mocked, no real
  `SpeechRecognition`.
- **T06 (voice route + intent)** — happy: "drzemka 10" → `Snooze(10)` →
  `CommunicationPolicy` snooze path invoked (**real seam**) → `VOICE` line
  in `events.log` → row in `voice_notes` → `VoiceAck` broadcast. nil:
  missing `transcript` field → 400. empty: `""` → 400, nothing logged.
  error: DB write fails → 500 **and** the `VOICE` event line is still
  written first (log before store, so a lost note is visible). Parser
  table test: 12 Polish + English phrasings incl. "snooze 5 min", "notatka:
  ból pleców", "idę na spacer", "wróciłem" → expected intents; unknown →
  `Note`. AI branch: with `voice_ai` returning `Some` the ack carries it;
  with `None` the ack is the deterministic template. Files:
  `voice_intent_tests.rs`, `remote_routes_voice_tests.rs`,
  `db_voice_notes_tests.rs` (real SQLite in a temp dir, per
  `rules/testing.md` "state store on a real instance").
- **T07 (webhook)** — happy: wiremock receives one POST with the JSON
  body. nil: URL unset → no request, no log noise beyond one startup line.
  empty: empty message → not sent (assert zero requests). error: 500 →
  exactly one retry then give up; timeout → caller returns within 6 s.
  File: `notify_webhook_tests.rs`.
- **T08 (`voice_ai`)** — happy: wiremock returns a completion → `Some(text)`
  trimmed to 60 words. nil: no key → `None` without any request (assert
  zero requests). empty: completion with empty content → `None`. error:
  401 → `Err(AuthRevoked)`-class, 5xx → transient, malformed JSON → error,
  none of which panic. File: `voice_ai_tests.rs`.
- **T04 (`HealthWidget`/`useHealth`)** — five render states (unconfigured /
  loading / fresh / stale / auth_revoked / transient) × two transports
  (Tauri `invoke` mocked; remote = reducer snapshot with `health`). The
  remote case **fails today** (`StepsWidget.tsx:66` returns early when not
  Tauri). File: `HealthWidget.test.tsx`, `useHealth.test.ts`,
  `useRemoteDesk.test.ts` (snapshot with `health`).

Three frames: *Friday 2 a.m.* — `remote_routes_health_tests` +
`remote_routes_voice_tests` are the two that let this ship, because they
run the real handlers end to end. *Hostile QA* — double-click Send (in-
flight guard, one request), 4 KiB+1 transcript (413), token with trailing
newline (rejected, no partial match), two pushes from two `source_id`s in
the same second (both kept). *Chaos* — Fit dies mid-poll while a push is
fresh (view shows push, Fit error in `error_message`), webhook target down
during a sit-limit alert (toast still fires, webhook logged once).
Pyramid: ~40 unit, ~8 integration (axum oneshot, real SQLite), 1 browser
pass. Flaky by nature and marked: none — every network call is wiremocked.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| spike-results | manual | `node scripts/check-e021-t01-spike.mjs` | exits 0 only when `reports/2026-09-06-spike-results.md` exists and every row of its yes/no table is filled | `evidence/records/T01-spike-results.json` |
| health-source | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- health_source google_fit` | pass | `evidence/records/T02-health-source.json` |
| push-route | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_routes_health remote_auth` | pass | `evidence/records/T03-push-route.json` |
| health-widget | test | `npx vitest run --config vite.config.ts src/components/HealthWidget.test.tsx src/hooks/useHealth.test.ts src/hooks/useRemoteDesk.test.ts` | pass | `evidence/records/T04-health-widget.json` |
| voice-capture | test | `npx vitest run --config vite.config.ts src/components/VoiceCapture.test.tsx` | pass | `evidence/records/T05-voice-capture.json` |
| voice-intent | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_intent remote_routes_voice db_voice_notes` | pass | `evidence/records/T06-voice-intent.json` |
| webhook | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- notify_webhook` | pass | `evidence/records/T07-webhook.json` |
| voice-ai | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_ai` | pass | `evidence/records/T08-voice-ai.json` |
| docs-consistency | manual | `node scripts/check-e021-t09-docs.mjs` | exits 0; ADR 020/021 exist, are linked from `ARCHITECTURE.md` + `CLAUDE.md`, D1-D6 in `decisions.jsonl`, every symbol the ADRs cite still exists | `evidence/records/T09-docs-consistency.json` |
| full-suite | test | `just check` | exit 0 | `evidence/records/T10-full-suite.json` |
| display-visual | visual | `browser` agent, `/display` at 360×780 DPR 3, after one `curl` push and one typed voice note | health badge shows the pushed number; ack text visible; console clean | `evidence/records/T10-display-visual.json` (screenshots in `evidence/.local/`) |

Records follow `rules/evidence.md` schema 1; raw output under
`evidence/.local/` (gitignored).

## Acceptance criteria

1. With `GOOGLE_*` unset and one `POST /display/health` received, the
   desktop popup and the phone `/display` both show that step count and
   its `source_id` — no Google credentials involved anywhere.
2. With Google Fit configured and no push, behaviour is unchanged from
   v0.6.0 except the added HR badge (when the data type exists) and the
   "ends late 2026" hint.
3. Both POST routes return 401 without the token and change no state.
4. Typing "drzemka 10" on the phone snoozes the next sit-limit reminder by
   10 minutes through the existing `CommunicationPolicy` snooze path, and
   the note appears in `events.log`, in `voice_notes`, and as a `VoiceAck`
   on the phone within 2 s.
5. On a phone whose browser lacks or blocks `SpeechRecognition`, the voice
   panel still works via keyboard dictation and shows no broken control.
6. With `DESK_NOTIFY_WEBHOOK_URL` pointing at an ntfy topic the phone
   subscribes to, a sit-limit alert and a voice ack each arrive as one
   phone notification (and therefore on the Watch 4 by mirroring — this
   last hop is observed manually by Paweł, stated as such in evidence).
7. With `OPENROUTER_API_KEY` unset, no request leaves the app and the ack is
   the deterministic template; with it set, the ack carries a ≤60-word
   reply.
8. `SessionState` and every `session_*.rs` file are untouched by this epic
   (`git diff --stat main -- src-tauri/src/session_*.rs` is empty) — D3.
9. `just check` is green; all eleven evidence records are `current`.
