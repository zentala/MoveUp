---
formatVersion: 1
type: handoff
status: todo
---

# E021 Handoff — smartwatch integration + voice dictation (phone bridge)

## TLDR

The implementing session reads only this file plus [`PLAN.md`](PLAN.md).
Ten tasks, 51 points, six waves (W0-W5). **Run through the full Agent
Orchestrator** — three waves have 2-3 independent tasks with disjoint
files (skill `ao` §0: over 13 points *and* real fan-out). Before the first
run: bump the version to `0.7.0` in `package.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` per
`.claude/rules/versioning.md` (`chore(desk): bump version to 0.7.0 for
E021`, annotated tag `v0.7.0`), and commit this HANDOFF. Branch
`feat/E021-smartwatch-voice`, worktrees via `wt-add`.

## Decisions already made (apply unless Paweł overrides in PRES §6)

- **D1** phone is the bridge, watch is a glance surface (custom firmware
  rejected). **D2** `HealthSource` trait + LAN push inlet (ADR 020).
  **D3** voice intents never touch `session_*.rs` — they go through
  `EventLogger`, `db_voice_notes`, and `CommunicationPolicy`'s *existing*
  snooze fields (`communication_policy.rs:34-35,71`). **D4** everything
  local ships open; the AI reply is BYOK OpenRouter; hosted AI stays Pro.
  **D5** keyboard dictation is the primary voice path, `SpeechRecognition`
  is progressive enhancement. **D6** Google Fit stays with a dated hint.
- ADR numbers **020** and **021**; if taken by the time T09 runs, take the
  next free integers and fix cross-references — do not block on it.
- T01 is a spike whose verification is a filled-in table; it is
  `--skip`ped in the AO manifest and done by the operator session before
  `ao run` (it needs Paweł's phone in hand for rows (b) and (c)).

## Mental model

- **Today's health path** (T02 wraps it): `lib.rs:185` manages
  `GoogleFitState = Arc<GoogleFitService>` from `google_fit_service.rs:87`
  (`from_env`); `view()` (`:112`) is cache-only, `refresh()` (`:179`) is
  the network call with in-flight dedup; IPC in `commands_google_fit.rs`
  (`get_steps_today`, `refresh_steps_now`, registered at `lib.rs:221-222`
  **and** `:258-259` — the two invoke lists are duplicated on purpose and
  a test guards their equality, `lib.rs:282-284`; edit both). DTOs in
  `google_fit_models.rs:71-101` (`StepsSnapshot`, `StepsView`). HTTP in
  `google_fit_client.rs` (`fetch_steps`, `list_step_sources`,
  `rank_step_sources`); endpoints overridable via `Endpoints::at_base`
  (`google_fit.rs:57`) for wiremock. Keep that override pattern for every
  new outbound client (T07, T08).
- **Today's remote server** (T03/T06 extend it): `remote_server.rs:70-82`
  `build_router` — two GET routes + a debug fallback; `RemoteState`
  (`:22-33`) carries `ws_tx`, `session`, `comm_policy`, `today_cache`.
  `RemoteDisplayState` (`ws_broadcaster.rs:17-21`) is the snapshot sent
  on connect and every ~1 s; add `health: HealthView` there (T03) so the
  phone gets it for free. `DisplayEvent` (`:30-54`) is the tagged enum —
  add `VoiceAck { transcript, intent, reply }` (T06); `desk_events`'s
  tests assert wire names match constants, so add the constant there too.
  Tests use axum `oneshot` — copy the shape from `remote_server_tests.rs`.
- **Frontend transports**: `useDeskAuto.ts` picks `useDesk` (Tauri) or
  `useRemoteDesk` (WS + `/display/api` polling fallback,
  `useRemoteDesk.ts:65-70`). `StepsWidget.tsx:44,66` bails out when not
  Tauri — that is why the phone never shows steps. T04's `useHealth.ts`
  must read the reducer's snapshot in remote mode and `invoke` in Tauri
  mode; the widget is slotted into `KpiStrip` via `children`
  (`OneBarWidget.tsx:69-71`).
- **Config**: `AppConfig` lives in `config.rs` (ts-rs exported → `src/generated/AppConfig.ts`,
  E018-D3); new fields `notify_webhook_url: Option<String>`,
  `voice_ai_model: Option<String>`, `remote_token_set: bool` (derived,
  never the token itself). Secrets stay in `.env` (`DESK_REMOTE_TOKEN`,
  `OPENROUTER_API_KEY`, `DESK_NOTIFY_WEBHOOK_URL` may also be `.env`) —
  read the way `Credentials::from_env` does; never log them.
- **Event log**: `event_logger.rs` writes `HH:MM:SS TYPE details` lines
  to `logs/YYYY-MM-DD/events.log` (`.claude/rules/logging.md`); add
  `VOICE <intent> <transcript-truncated-80>` (T06). Log *before* the DB
  write.
- **DB**: `db.rs` + `db_sessions.rs`/`db_queries.rs` — follow the same
  migration style for `voice_notes`; tests on a real temp SQLite like
  `db_tests_roundtrip.rs`. Catalog: `commands_catalog_sources.rs` lists
  data sources for the Analyst Catalog tab — add `voice_notes` there
  (T06) so the source is discoverable.
- **Codegen**: every new `Serialize` DTO gets `#[derive(ts_rs::TS)]
  #[ts(export)]`; run the export test so `src/generated/*.ts` updates
  (E018 learned this costs an 18-file write_set widening when forgotten —
  `src/generated/**` is in every Rust task's write_set below).
- **Files that wire any feature in** (AO §3a, in every write_set, never
  in claims): `src-tauri/src/lib.rs`, `src/generated/**`, `PROJECT.xml`,
  `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `package.json`,
  `pnpm-lock.yaml`.
- **Do not touch**: `src-tauri/src/session_*.rs` (D3, acceptance
  criterion 8), `overlay_renderer.rs`, `tray_controller.rs` except the
  one call site where the sit-limit toast fires (T07 adds the webhook
  call next to it — find it with `grep -n "notify" tray_controller.rs
  communication_policy.rs`, it is the `NotifySignal` consumer).

## Tasks

- [ ] **T01** (2, main, wave 0, outside AO) — spike on real hardware.
  Write `reports/2026-09-06-spike-results.md` in this epic folder with a
  yes/no table: Fit `refresh_steps_now` → 200 today?; HR data type
  present in `dataSources`?; Mi Fitness listed under Health Connect on
  the phone?; `SpeechRecognition` starts on `http://<PC-IP>:3390` in
  Chrome Android? in Fully Kiosk?; keyboard-mic dictation into a textarea
  works? Add `scripts/check-e021-t01-spike.mjs` (fails when the file or
  any row is missing/blank — an empty table is not a pass). Tests: the
  script itself. Verify: `node scripts/check-e021-t01-spike.mjs`.
- [ ] **T02** (8, ts-dev Rust, wave 1) — `health_source.rs` (trait
  `HealthSource { fn id(&self) -> &str; async fn view(&self) -> HealthView; async fn refresh(&self) -> HealthView }`,
  `HealthAggregator` merging by `fetched_at_ms`, error passthrough),
  `health_models.rs` (`HealthSnapshot`, `HealthView`, `HealthErrorKind`
  — reuse `ErrorKind`'s two variants), `GoogleFitService` implements the
  trait, HR via `com.google.heart_rate.bpm` aggregate (`Option`, skip
  when T01 said no), `commands_health.rs` (`get_health_today`,
  `refresh_health_now`), delete `commands_google_fit.rs`, both invoke
  lists in `lib.rs`. Tests: `health_source_tests.rs`, HR case in
  `google_fit_http_tests.rs`, four paths per PLAN. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- health_source google_fit`.
- [ ] **T05** (5, ts-dev React, wave 1) — `src/components/VoiceCapture.tsx`
  (+ `voice-capture.css`), rendered on `/display` only (not the desktop
  popup); textarea + Send; mic button gated on `SpeechRecognition`
  presence and a `navigator.permissions.query({name:"microphone"})`
  result ≠ `denied` (wrap in try/catch — the query throws on some
  browsers); token sheet storing `desk_token` in `localStorage`
  (try/catch); `POST /display/voice` with `X-Desk-Token`; renders the
  `VoiceAck` from the WS stream (subscribe via a small
  `onVoiceAck` callback added to `useRemoteDesk`'s options — keep
  `UseDeskResult` unchanged). Scenario `voiceCapture*` in
  `src/test/scenarios-other.ts`; mockup gallery entry. Tests:
  `VoiceCapture.test.tsx` (four paths + double-click guard). Verify:
  `npx vitest run --config vite.config.ts src/components/VoiceCapture.test.tsx`.
- [ ] **T07** (5, ts-dev Rust, wave 1) — `notify_webhook.rs`:
  `WebhookNotifier::from_env_or_config`, `send(&Notification) -> ()`
  spawned on tokio (never awaited by callers), 5 s timeout, one retry on
  5xx/timeout, JSON body `{title, message, priority, tags}` (ntfy accepts
  this on `POST <base>/<topic>` with `Content-Type: application/json`;
  document that). Call it where the sit-limit toast fires. Settings →
  More toggle + URL field bound to `AppConfig.notify_webhook_url`. Tests:
  `notify_webhook_tests.rs` (wiremock, four paths). Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- notify_webhook`.
- [ ] **T03** (8, ts-dev Rust, wave 2, after T02) — `remote_auth.rs`
  (`require_token(headers, expected) -> Result<(), StatusCode>`,
  constant-time compare via `subtle` or a manual XOR fold — add the crate
  only if `subtle` is already transitive), `remote_routes_health.rs`
  (`POST /display/health`, body cap 1 KiB, schema
  `{steps_today: u32, heart_rate_bpm?: u16, hrv_rmssd_ms?: f32, source_id: String(1..64), measured_at_ms: i64}`
  → `PushHealthSource` registered in the aggregator; stale after 1 h),
  mount in `build_router`, `RemoteDisplayState.health`, `/display/api`
  includes it, `docs/REMOTE_DISPLAY.md` gets the `curl` example and the
  token setup. Tests: `remote_routes_health_tests.rs`, `remote_auth_tests.rs`
  (real seam: route → source → aggregator). Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_routes_health remote_auth`.
- [ ] **T08** (5, ts-dev Rust, wave 2) — `voice_ai.rs`:
  `VoiceAi::from_env` (`OPENROUTER_API_KEY`; endpoint overridable for
  tests), `reply(transcript, &SessionStateDto) -> Result<Option<String>, VoiceAiError>`,
  fixed system prompt (≤60-word reply, Polish or English matching the
  transcript, ergonomics coaching only), model from
  `AppConfig.voice_ai_model` else the cheap default named in
  `~/.claude/skills/openrouter/SKILL.md`, 8 s timeout. Not wired to any
  route yet (T06 does). Tests: `voice_ai_tests.rs` (wiremock, five
  cases). Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_ai`.
- [ ] **T04** (5, ts-dev React, wave 3, after T03) — rename
  `StepsWidget.tsx` → `HealthWidget.tsx` (+ test), `src/hooks/useHealth.ts`
  (Tauri: `invoke("get_health_today"/"refresh_health_now")` with the
  existing `useExponentialPoll` ladder; remote: read `health` from the
  reducer snapshot, no polling), HR badge when `heart_rate_bpm` present,
  source label from `source_id`, "Google Fit ends late 2026" tooltip when
  `source_id == "google_fit"` is the only source; `deskReducer` snapshot
  action carries `health`. Tests: `HealthWidget.test.tsx`,
  `useHealth.test.ts`, `useRemoteDesk.test.ts` extended. Verify:
  `npx vitest run --config vite.config.ts src/components/HealthWidget.test.tsx src/hooks/useHealth.test.ts src/hooks/useRemoteDesk.test.ts`.
- [ ] **T06** (8, ts-dev Rust, wave 3, after T03 and T08) —
  `voice_intent.rs` (`parse(&str) -> Intent` with
  `Snooze(u16)`/`Note`/`WalkStart`/`WalkEnd`, table-driven regexes,
  Polish + English), `remote_routes_voice.rs` (`POST /display/voice`,
  body cap 4 KiB, `{transcript: String(1..2000), lang?: String, captured_at_ms: i64}`,
  token required; pipeline: `EventLogger` `VOICE` line → `db_voice_notes`
  insert → intent side effect (`Snooze` → `CommunicationPolicy` snooze
  fields; `Walk*`/`Note` → none) → `voice_ai::reply` when configured →
  `VoiceAck` broadcast → webhook push via T07), `db_voice_notes.rs`
  (table + `list_voice_notes(day)` IPC in `commands_health.rs`, catalog
  entry in `commands_catalog_sources.rs`). Tests: `voice_intent_tests.rs`
  (12-phrase table), `remote_routes_voice_tests.rs` (real seam to the
  parser and a real temp SQLite), `db_voice_notes_tests.rs`. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_intent remote_routes_voice db_voice_notes`.
- [ ] **T09** (3, main, wave 4) — `.arch/ADR/020-health-source-inlet.md`,
  `.arch/ADR/021-voice-in-on-phone-watch-as-glance.md` (both with
  Context/Decision/Alternatives/Consequences and `Status: accepted`),
  supersede note in ADR 012, `.arch/ARCHITECTURE.md` remote-server +
  health-inlet sections, `.arch/UX-FLOW.md` rows, `CLAUDE.md` "Google Fit
  Integration" → "Health sources" (keys, push contract, voice, webhook,
  BYOK), `CONTRIBUTING.md` developer contract, `docs/REMOTE_DISPLAY.md`
  voice setup, `PROJECT.xml`, `.plan/decisions.jsonl` E021-D1..D6,
  `.plan/BACKLOG.md` entries: candidate E022 (Android Health Connect
  companion, interface = this push contract) and "create
  `.plan/epics/INDEX.md`" (the guard `epic-index-guard.mjs` cannot
  enforce status here until it exists). `scripts/check-e021-t09-docs.mjs`
  modelled on `check-e020-t07-docs.mjs`: grounds every doc claim in a
  symbol (`trait HealthSource`, `fn require_token`, `enum Intent`,
  `struct WebhookNotifier`, `fn reply`), counts checks, fails on zero.
  Verify: `node scripts/check-e021-t09-docs.mjs`.
- [ ] **T10** (2, verify + browser, wave 5) — `just check`; write all
  evidence records; then the `browser` agent: `/display` at 360×780 DPR 3
  after `curl -X POST -H "X-Desk-Token: …" http://127.0.0.1:3390/display/health -d '{"steps_today":1234,"source_id":"curl","measured_at_ms":…}'`
  shows `1 234` with source `curl`; typing "drzemka 5" and Send shows an
  ack; console clean. Mic-button path is **manual** on Paweł's phone —
  record the observation (or "not observed") in
  `evidence/.local/T10-mic-manual.md`; never upgrade it to PASS from the
  headless run. Verify: `just check`.

## Outside AO

- T01 (needs the phone in hand; `--skip E021-T01` in the manifest).
- T10's phone-mic observation and the watch-mirroring hop of acceptance
  criterion 6 — Paweł observes both; the record says "manual".

## Done means

All nine acceptance criteria in PLAN.md hold, eleven evidence records
`current`, `.plan/HISTORY.md` entry, `STATE.md` updated, `IMPRO.md`
triaged, version `0.7.0` tagged, and PRES §status updated honestly.

## AO

```yaml
project: MoveUp
epic: E021
base_ref: main
tasks:
  - id: E021-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/health_source.rs", "src-tauri/src/health_source_tests.rs", "src-tauri/src/health_models.rs", "src-tauri/src/google_fit.rs", "src-tauri/src/google_fit_client.rs", "src-tauri/src/google_fit_models.rs", "src-tauri/src/google_fit_service.rs", "src-tauri/src/google_fit_service_tests.rs", "src-tauri/src/google_fit_http_tests.rs", "src-tauri/src/google_fit_tests.rs", "src-tauri/src/commands_health.rs", "src-tauri/src/commands_google_fit.rs", "src-tauri/src/lib.rs", "src/generated/**", "PROJECT.xml", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]
    claims: ["src-tauri/src/health_source.rs", "src-tauri/src/health_source_tests.rs", "src-tauri/src/health_models.rs", "src-tauri/src/google_fit_client.rs", "src-tauri/src/google_fit_models.rs", "src-tauri/src/google_fit_service.rs", "src-tauri/src/google_fit_service_tests.rs", "src-tauri/src/google_fit_http_tests.rs", "src-tauri/src/commands_health.rs", "src-tauri/src/commands_google_fit.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- health_source google_fit"
    budget_minutes: 90
  - id: E021-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src/components/VoiceCapture.tsx", "src/components/VoiceCapture.test.tsx", "src/components/voice-capture.css", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/remoteDesk.test-helpers.ts", "src/App.tsx", "src/test/scenarios-other.ts", "src/test/scenarios.ts", "src/test/scenario-helpers.ts", "src/mockup/**", "PROJECT.xml", "package.json", "pnpm-lock.yaml"]
    claims: ["src/components/VoiceCapture.tsx", "src/components/VoiceCapture.test.tsx", "src/components/voice-capture.css", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/test/scenarios-other.ts"]
    verification: "npx vitest run --config vite.config.ts src/components/VoiceCapture.test.tsx"
    budget_minutes: 60
  - id: E021-T07
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/notify_webhook.rs", "src-tauri/src/notify_webhook_tests.rs", "src-tauri/src/config.rs", "src-tauri/src/config_tests.rs", "src-tauri/src/commands_config.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/tray_controller_tests.rs", "src/components/settings/**", "src/components/SettingsPanel.tsx", "src/components/SettingsPanel.test.tsx", "src-tauri/src/lib.rs", "src/generated/**", "PROJECT.xml", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]
    claims: ["src-tauri/src/notify_webhook.rs", "src-tauri/src/notify_webhook_tests.rs", "src-tauri/src/config.rs", "src-tauri/src/config_tests.rs", "src-tauri/src/tray_controller.rs", "src/components/settings/**"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- notify_webhook"
    budget_minutes: 60
  - id: E021-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E021-T02"]
    write_set: ["src-tauri/src/remote_auth.rs", "src-tauri/src/remote_auth_tests.rs", "src-tauri/src/remote_routes_health.rs", "src-tauri/src/remote_routes_health_tests.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/remote_server_tests.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/remote_display_state.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/health_source.rs", "src-tauri/src/health_source_tests.rs", "src-tauri/src/health_models.rs", "docs/REMOTE_DISPLAY.md", "src-tauri/src/lib.rs", "src/generated/**", "PROJECT.xml", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]
    claims: ["src-tauri/src/remote_auth.rs", "src-tauri/src/remote_auth_tests.rs", "src-tauri/src/remote_routes_health.rs", "src-tauri/src/remote_routes_health_tests.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/remote_server_tests.rs", "src-tauri/src/ws_broadcaster.rs", "docs/REMOTE_DISPLAY.md"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_routes_health remote_auth"
    budget_minutes: 90
  - id: E021-T08
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/voice_ai.rs", "src-tauri/src/voice_ai_tests.rs", "src-tauri/src/config.rs", "src-tauri/src/config_tests.rs", "src-tauri/src/lib.rs", "src/components/settings/SettingsTypes.ts", "src/generated/**", "PROJECT.xml", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]
    claims: ["src-tauri/src/voice_ai.rs", "src-tauri/src/voice_ai_tests.rs", "src-tauri/src/config_tests.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_ai"
    budget_minutes: 60
  - id: E021-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E021-T03"]
    write_set: ["src/components/HealthWidget.tsx", "src/components/HealthWidget.test.tsx", "src/components/StepsWidget.tsx", "src/components/StepsWidget.test.tsx", "src/hooks/useHealth.ts", "src/hooks/useHealth.test.ts", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/hooks/useDeskTypes.ts", "src/widgets/OneBarWidget.tsx", "src/widgets/OneBarWidget.test.tsx", "src/widgets/one-bar/one-bar.css", "src/types.ts", "PROJECT.xml"]
    claims: ["src/components/HealthWidget.tsx", "src/components/HealthWidget.test.tsx", "src/components/StepsWidget.tsx", "src/components/StepsWidget.test.tsx", "src/hooks/useHealth.ts", "src/hooks/useHealth.test.ts", "src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/widgets/OneBarWidget.tsx", "src/widgets/OneBarWidget.test.tsx"]
    verification: "npx vitest run --config vite.config.ts src/components/HealthWidget.test.tsx src/hooks/useHealth.test.ts src/hooks/useRemoteDesk.test.ts"
    budget_minutes: 60
  - id: E021-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E021-T03", "E021-T08", "E021-T07"]
    write_set: ["src-tauri/src/voice_intent.rs", "src-tauri/src/voice_intent_tests.rs", "src-tauri/src/remote_routes_voice.rs", "src-tauri/src/remote_routes_voice_tests.rs", "src-tauri/src/db_voice_notes.rs", "src-tauri/src/db_voice_notes_tests.rs", "src-tauri/src/db.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/desk_events.rs", "src-tauri/src/event_logger.rs", "src-tauri/src/communication_policy.rs", "src-tauri/src/communication_policy_tests.rs", "src-tauri/src/commands_health.rs", "src-tauri/src/commands_catalog.rs", "src-tauri/src/commands_catalog_sources.rs", "src-tauri/src/commands_catalog_tests.rs", "src-tauri/src/lib.rs", "src/events.ts", "src/generated/**", "PROJECT.xml", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock"]
    claims: ["src-tauri/src/voice_intent.rs", "src-tauri/src/voice_intent_tests.rs", "src-tauri/src/remote_routes_voice.rs", "src-tauri/src/remote_routes_voice_tests.rs", "src-tauri/src/db_voice_notes.rs", "src-tauri/src/db_voice_notes_tests.rs", "src-tauri/src/db.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/desk_events.rs", "src-tauri/src/event_logger.rs", "src-tauri/src/communication_policy.rs", "src-tauri/src/commands_catalog_sources.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_intent remote_routes_voice db_voice_notes"
    budget_minutes: 90
  - id: E021-T09
    repo: MoveUp
    executor: main
    depends_on: ["E021-T04", "E021-T06"]
    write_set: [".arch/ADR/020-health-source-inlet.md", ".arch/ADR/021-voice-in-on-phone-watch-as-glance.md", ".arch/ADR/012-google-fit-integration.md", ".arch/ARCHITECTURE.md", ".arch/UX-FLOW.md", "CLAUDE.md", "CONTRIBUTING.md", "docs/REMOTE_DISPLAY.md", "PROJECT.xml", ".plan/decisions.jsonl", ".plan/BACKLOG.md", "scripts/check-e021-t09-docs.mjs"]
    claims: [".arch/ADR/020-health-source-inlet.md", ".arch/ADR/021-voice-in-on-phone-watch-as-glance.md", ".arch/ADR/012-google-fit-integration.md", ".arch/ARCHITECTURE.md", ".arch/UX-FLOW.md", "CLAUDE.md", "CONTRIBUTING.md", "docs/REMOTE_DISPLAY.md", ".plan/decisions.jsonl", ".plan/BACKLOG.md", "scripts/check-e021-t09-docs.mjs"]
    verification: "node scripts/check-e021-t09-docs.mjs"
    budget_minutes: 45
  - id: E021-T10
    repo: MoveUp
    executor: verify
    depends_on: ["E021-T09"]
    write_set: [".plan/epics/E021-2026-09-06-smartwatch-integration/evidence/records/**"]
    claims: [".plan/epics/E021-2026-09-06-smartwatch-integration/evidence/records/**"]
    verification: "just check"
    budget_minutes: 45
```

T01 is deliberately absent from the block (manual spike, `--skip`); T10's
browser half runs after `ao promote`, by the operator session, per
`ao` §6 "verify in the browser once per epic".
