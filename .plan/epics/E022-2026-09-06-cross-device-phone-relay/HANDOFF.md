---
formatVersion: 1
type: handoff
status: todo
---

# E022 Handoff — cross-device phone relay

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md) (the
protocol tables there are the spec — do not redesign them). 16 tasks, 111
points, 7 waves (W0–W6), three parallel chains: relay Worker (T02→T03→T04),
desktop Rust (T05→T06→T07), phone React (T08→T09→T10). Runs through the full
Agent Orchestrator (`## AO` below): W1 has four independent tasks, W2 and W3
three each — that is exactly the concurrency AO exists to protect. Bump the
minor version before T01 per `.claude/rules/versioning.md` (0.7.0 unless E021
already took it — then the next free minor). **T13, the security review, is
pipeline step 5 and has no skip path in this epic**; a commit that closes the
epic without `T13-security-review.json` as `current` is not a closed epic.

## Decisions already made (apply unless Paweł overrides in PLAN.md)

- D1 relay on Cloudflare Workers + Durable Objects + D1; D2 pairing code →
  device tokens, no accounts; D3 three commands only; D4 LAN stays, read-only,
  toggle, shared envelope; D5 `relay.desk.zentala.io`, constant
  `RELAY_DEFAULT_URL`, overridable in Settings. Full reasoning: PLAN.md
  §Decisions and §Alternatives.
- ADR numbers 020 (relay) and 021 (auth). If E021 claimed them first, T14
  takes the next free integers and fixes references — no renumbering fights.
- Tokens: `mu_d_`/`mu_v_` prefix + 43 base64url chars from 32 random bytes;
  SHA-256 at rest; constant-time compare; **never in a URL**.
- Secrets on the desktop go to the OS credential store via the `keyring`
  crate; `tauri-plugin-store` holds `desk_id`, `relay_url`, flags only.
- Envelope `{v, type, id, ts, payload}` on BOTH transports. `DisplayEvent`'s
  own `{event, payload}` shape is unchanged inside `event.payload`.
- Close codes 4400/4401/4402/4403/4409/4429/4503 as in PLAN.md — put them in
  one constants module per language (`remote_protocol.rs`,
  `src/remote/protocol.ts`), never as literals at call sites.

## Mental model

- **Where events come from today**: `ws_broadcaster.rs:58-68` — a
  `broadcast::Sender<String>` of already-serialized `DisplayEvent` JSON.
  Producers: `tray_controller.rs:93,124` (state change, snapshot tick),
  `setup_helpers.rs:242,249,256` (device connected/lost, daily reset),
  `remote_display_state.rs:74`. The relay client (T05) is **one more
  `subscribe()`** on that sender — no new producer, no new event type.
- **Where the LAN server lives**: `remote_server.rs` (`start`, `build_router`,
  `ws_handler`, `handle_ws_client`, `api_handler`), started from
  `setup_helpers.rs:213-230` (`setup_remote_display`) with the port from
  `DESK_REMOTE_PORT`. T11 wraps outgoing messages in the envelope inside
  `handle_ws_client` (both the broadcast forward at `:118-130` and the
  heartbeat at `:131-136`) and gates `setup_remote_display` on
  `AppConfig.remote_lan_enabled`. The receive arm `:137-142` stays
  `Some(Ok(_)) => {}` — add a `log::debug!` and a test, nothing else.
- **Shared state**: `commands.rs:24-38` `AppState` (`session`, `comm_policy`,
  `alert_popup`, `ws_tx`, `today_cache`, `config`). T06 adds
  `relay: Arc<Mutex<RelayStatus>>` (+ a handle to stop/restart the client).
  Lock order when a command touches two: `comm_policy` before `alert_popup`,
  `session` alone — follow whatever `tray_controller.rs` already does for the
  dismiss path.
- **Existing write paths the commands reuse** (T07):
  `commands_config.rs:78-102` (`set_session_limit`, `set_stand_limit` →
  `SessionManager::set_limit_minutes` / `set_stand_limit_minutes`, clamps
  inside), `commands_profiles.rs:161-206` (`switch_communication_profile`,
  `switch_ergonomic_profile` — these take `AppHandle`; extract the body into
  `pub(crate) fn switch_*_by_name(app_data_dir, policy, name)` so the relay
  path can call it without an `AppHandle` at hand), `alert_popup.rs:74`
  (`AlertPopup::dismiss`) + `communication_policy.rs:105`
  (`CommunicationPolicy::dismiss`) — `ack_alert` = both, in that order, the
  way the tray handles a user dismiss (`alert_popup.rs:88-90`
  `take_user_dismissed` is the tray's poll; set the same flag so snooze logic
  sees a remote ack exactly like a click).
- **Config**: `config.rs:30-56` `AppConfig` with `#[serde(default)]` per
  field and `clamped()`. Add `relay_enabled: bool` (default false),
  `relay_url: String` (default `RELAY_DEFAULT_URL`), `remote_lan_enabled:
  bool` (default true). `save_settings` (`commands_config.rs:21-49`) already
  persists the whole struct — the Settings UI goes through it; the relay
  client re-reads the flag on save (T06 restarts the client on change).
- **Frontend hook shape**: `useRemoteDesk.ts` owns a `WebSocket` directly
  (`:68-121`) and a REST fallback (`:125-132`); it dispatches into
  `deskReducer` (`applySnapshot`, `applyStateChanged`). T08 moves the socket
  into `src/remote/transports/lan.ts` and adds `relay.ts`; the hook keeps its
  `UseDeskResult` contract (`useDeskTypes.ts`) and adds
  `capabilities: { control: boolean }` + `deskOnline: boolean` to the result
  (also add to `useDesk.ts` as constants `true`/`true` so the type stays one).
  Selection lives in `useDeskAuto.ts:19-24` today for Tauri-vs-browser; keep
  that, and put LAN-vs-relay inside `useRemoteDesk`.
  Tests to update: `useRemoteDesk.test.ts`, `useRemoteDeskExtra.test.ts`,
  `remoteDesk.test-helpers.ts` (fake WebSocket lives here — extend, do not
  fork).
- **Routes**: `App.tsx:30-50` switches on `window.location.hash`
  (`#/analyst`, dev-only `#/mockup*`). T09 adds `#/pair` the same way.
  `isTauri` gating at `App.tsx:17-20,55-68` — the pair screen and controls
  are browser-only; the Settings section is Tauri-only.
- **Codegen**: ADR 017 — Rust DTOs carry `#[derive(ts_rs::TS)]` and export
  into `src/generated/`; `src/types.ts` re-exports. Envelope + command types
  from `remote_protocol.rs` go through the same path so the viewer and the
  relay import them from one place (`src/remote/protocol.ts` wraps them in
  zod for runtime validation; `relay/tsconfig.json` aliases `@app/*` →
  `../src/*`).
- **The relay directory** does not exist. Layout T02 creates:
  `relay/{package.json, pnpm-lock.yaml, wrangler.toml, tsconfig.json,
  vitest.config.ts, migrations/0001_init.sql, src/index.ts (router),
  src/room/desk-room.ts, src/room/messages.ts, src/auth/tokens.ts,
  src/auth/licenses.ts, src/http/*.ts, scripts/mint-license.mjs,
  test/{room,auth,http,commands}/*.test.ts}`. Static viewer assets:
  `wrangler.toml [assets] directory = "../dist"` served under `/app`.
  Standalone pnpm project — **do not** add `packages:` to the root
  `pnpm-workspace.yaml` (`.giter.yaml` guards only root `node_modules`; T12
  adds `relay/node_modules` to `worktree.guards`).
- **Fixtures** (`tests/fixtures/relay-protocol/*.json`): one file per
  message type and per close/error case, named `<type>[-<case>].json`. Both
  test suites glob the directory and **assert the count is > 0** (an empty
  glob is a failure, never a pass — CLAUDE.md "Cisza nigdy nie znaczy
  sukcesu").
- **Existing dev-mode gap you will hit**: in a debug build
  `remote_server.rs:163-172` serves a placeholder for `/display`, and the
  Vite dev server has no proxy — `.plan/BACKLOG.md` "Dev-mode remote
  display". T12 adds the Vite proxy (`/display` → `http://127.0.0.1:3390`,
  `ws: true`) and closes that one backlog entry; leave the other two.
- **What not to touch**: `session_*.rs` engine files (E015/E020 just
  finished there — nothing in this epic needs the engine), `overlay_*.rs`,
  `google_fit*.rs`, `db*.rs`, `tests/e2e/**`, pricing prose (read
  `.plan/vision/config/pricing.json` if a doc needs a price).

## Tasks

- [ ] **T01** (5, ts-dev) — Protocol contract. `src-tauri/src/remote_protocol.rs`
  (+ `remote_protocol_tests.rs`, `mod` lines in `lib.rs`): `Envelope<T>`,
  `Hello`, `Welcome`, `Command`, `CommandResult`, `DeskStatus`, `ErrorBody`,
  close-code consts, `COMMAND_ALLOWLIST` with arg bounds as data; `ts_rs`
  derives → `src/generated/`; `src/remote/protocol.ts` (zod) +
  `protocol.test.ts`; `tests/fixtures/relay-protocol/*.json` (≥ 12 files:
  hello-desk, hello-viewer, welcome-online, welcome-offline, event-snapshot,
  event-state-changed, desk-status, command-ack, command-set-limits,
  command-switch-profile, command-result-ok, command-result-err, ping, error).
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_protocol`
  and `npx vitest run --config vite.config.ts src/remote/protocol.test.ts`.
- [ ] **T02** (13, ts-dev) — Relay Worker core. Create `relay/` per Mental
  model; `DeskRoom` DO using the WebSocket Hibernation API (`acceptWebSocket`,
  `webSocketMessage`, `webSocketClose`, tags `desk`/`viewer:<id>`), `hello`
  timeout via DO alarm, `welcome`, `event` fan-out, `desk_status`, last
  snapshot in `this.snapshot`, ping/pong + 60 s idle close, all close codes,
  `/healthz`, `/app/*` assets, router stubs for the T03 REST paths returning
  501. Auth in this task is a `verifyToken` hook that T03 fills — but the
  4401 path must already exist and be tested. Verify:
  `pnpm --dir relay install --frozen-lockfile && pnpm --dir relay exec vitest run test/room`.
- [ ] **T03** (13, ts-dev) — Relay auth + REST. `migrations/0001_init.sql`
  (`licenses(key_hash PK, plan, max_desks, max_viewers, expires_at,
  created_at)`, `desks(desk_id PK, license_key_hash, token_hash, desk_name,
  app_version, created_at, last_seen)`, `viewers(viewer_id PK, desk_id,
  token_hash, device_name, paired_at, last_seen, revoked_at)`), the six REST
  routes from PLAN.md, pairing codes in the DO (`this.pairing = {code_hash,
  expires_at, attempts}`), rate limits (per-IP via a small DO or the
  `RateLimiter` binding — pick one, document in `relay/README.md`), lockout,
  `scripts/mint-license.mjs` (prints one key, inserts its hash via `wrangler
  d1 execute`). Verify:
  `pnpm --dir relay exec vitest run test/auth test/http`.
- [ ] **T04** (5, ts-dev) — Relay command routing. `command` from a viewer:
  validate name/args with the shared zod schema, attach `viewer_id`, forward
  to the desk socket; `desk_offline` immediate result when none; route
  `command_result` by `command_id` to the originating viewer only (keep a
  bounded `Map<command_id, viewer_tag>` with TTL 30 s); 10/min/viewer.
  Verify: `pnpm --dir relay exec vitest run test/commands`.
- [ ] **T05** (8, ts-dev) — Desktop relay client. `src-tauri/src/relay_client.rs`
  (+ `relay_client_tests.rs`, `relay_status.rs`): tokio task with
  `tokio-tungstenite` (`rustls-tls-webpki-roots`), `hello` → `welcome` →
  snapshot → forward `ws_tx` subscription; ping 25 s / 60 s timeout; backoff
  1→60 s ±20 % jitter (injectable sleeper for tests); close-code → status
  mapping with **no reconnect** on 4402/4403/4409; a `ClientHandle {
  stop(), restart() }`. Tests use an in-process tokio-tungstenite server on
  port 0. Add deps in `Cargo.toml`. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_client relay_status`.
- [ ] **T06** (8, ts-dev) — Desktop credentials + pairing + IPC.
  `relay_auth.rs` (`keyring` with the `mock` feature under `cfg(test)`;
  `register`, `start_pairing`, `list_viewers`, `revoke_viewer`,
  `disable_relay` over `reqwest`), `commands_relay.rs` (Tauri commands:
  `relay_register {license_key}`, `relay_start_pairing` → `{code, expires_at,
  qr_payload}`, `relay_list_viewers`, `relay_revoke_viewer {viewer_id}`,
  `relay_disable`, `get_relay_status`), `AppConfig` fields, `AppState.relay`,
  client start/restart on config save, registration in `lib.rs` handler
  list. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_auth commands_relay config`.
- [ ] **T07** (5, ts-dev) — Desktop command execution. `relay_commands.rs`
  (+ tests): `execute(cmd, &AppState-like struct) -> CommandResult`;
  allowlist + bounds from `remote_protocol.rs`; stale `ts` > 30 s rejected;
  extract `switch_*_by_name` from `commands_profiles.rs`; `ack_alert` per
  Mental model; `events.log` lines `REMOTE <name> viewer=<id> ok|err=<code>`
  and `REMOTE DENIED <name>`. Wire into `relay_client.rs`'s receive arm.
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_commands commands_profiles`.
- [ ] **T08** (8, ts-dev) — Phone transport abstraction. `src/remote/transports/
  {types.ts, lan.ts, relay.ts, index.ts}` (`Transport { connect, close,
  sendCommand?, capabilities, onMessage, onStatus }`), envelope parse via
  `protocol.ts`, stored record helpers `src/remote/storage.ts`
  (`moveup.relay.v1`, try/catch), `useRemoteDesk.ts` consumes a transport and
  exposes `capabilities` + `deskOnline`, `ConnectionOverlay.tsx` fourth state
  (`data-testid="conn-overlay-desk-offline"`), `useDesk.ts` gets the two new
  constant fields, `useDeskTypes.ts` updated. Verify:
  `npx vitest run --config vite.config.ts src/remote src/hooks/useRemoteDesk src/components/ConnectionOverlay`.
- [ ] **T09** (8, ts-dev) — Phone pairing + controls UI. `src/remote/PairScreen.tsx`
  (+ test), `src/remote/RemoteControls.tsx` (+ test), `#/pair` route in
  `App.tsx`, `RemoteControls` mounted in the remote layout only when
  `capabilities.control`; styles in `src/remote/remote.css`; scenarios for
  the mockup gallery in `src/test/scenarios.ts` (`pair-empty`,
  `pair-deeplink`, `pair-error-locked`, `controls-pending`). Verify:
  `npx vitest run --config vite.config.ts src/remote/PairScreen src/remote/RemoteControls`.
- [ ] **T10** (8, ts-dev) — Desktop Settings → Remote section.
  `src/components/settings/RemoteSection.tsx` (+ test, ≤ 100 lines per
  component — split `PairedDevicesList.tsx`, `PairingCodeCard.tsx`), `qrcode`
  dependency, wired into the existing settings panel tabs, scenarios
  `relay-disabled`, `relay-online-2-viewers`, `relay-unentitled`,
  `relay-pairing-code-shown` in `src/test/scenarios.ts` **before** the
  component (ux-design-flow.md — show the mockup, get "ok", then build).
  Verify: `npx vitest run --config vite.config.ts src/components/settings/RemoteSection src/components/settings/PairedDevicesList src/components/settings/PairingCodeCard`.
- [ ] **T11** (3, ts-dev) — LAN path on the shared envelope. `remote_server.rs`
  envelope wrap + `remote_lan_enabled` gate in `setup_helpers.rs` +
  read-only test; `docs/REMOTE_DISPLAY.md` LAN section mentions the toggle
  (T14 rewrites the rest). Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_server`.
- [ ] **T12** (5, ts-dev) — Wiring + local e2e. `justfile` recipes
  `relay-dev`, `relay-test`, `relay-deploy`; `.giter.yaml` guard;
  `vite.config.ts` `/display` proxy (closes the first "Dev-mode remote
  display" backlog entry — mark it `[x]` with this task id);
  `scripts/relay-e2e.mjs` (+ `tests/scripts/relay-e2e.test.ts` smoke that
  the script parses args and refuses to run without a relay URL);
  `PROJECT.xml` new files/commands/test layer. Verify:
  `node scripts/relay-e2e.mjs --relay http://127.0.0.1:8787 --license $(node relay/scripts/mint-license.mjs --local)`
  — from the AO manifest use the smoke test instead:
  `npx vitest run --config vitest.scripts.config.ts tests/scripts/relay-e2e.test.ts`.
- [ ] **T13** (5, main) — **Security review (pipeline step 5, mandatory).**
  Write `reports/threat-model.md` (assets, entry points, trust boundaries,
  the checklist from PLAN.md §Test strategy T13 with a column for the
  recorded observation). Dispatch agent `security-reviewer` over `relay/src`,
  `src-tauri/src/relay_*.rs`, `commands_relay.rs`, `remote_server.rs`,
  `src/remote/**`; run skill `review-loop`; fix confirmed findings here (or
  file them with the disputed list after 3 rounds); `review-log record
  review --status clean|issues --findings N`. Manual — skipped in AO.
- [ ] **T14** (3, main) — Docs + ADRs. `.arch/ADR/022-relay-on-cloudflare-durable-objects.md`,
  `.arch/ADR/023-pairing-code-device-token-auth.md`, `.arch/ARCHITECTURE.md`
  section, `CLAUDE.md` Remote Display section rewrite, `docs/REMOTE_DISPLAY.md`
  rewrite (pairing first, LAN second, Fully Kiosk notes kept),
  `docs/PRIVACY.md` relay paragraph, `.plan/decisions.jsonl` D1–D5,
  `.plan/BACKLOG.md` entry "LAN pairing" (Importance Low, 5), `.plan/BUSINESS_CONTEXT.md`
  link to ADR 022/023, `scripts/check-e022-t14-docs.mjs`. Verify:
  `node scripts/check-e022-t14-docs.mjs`.
- [ ] **T15** (2, verify) — Verify + browser pass. Agent `verify` over all
  ten acceptance criteria; ONE `browser` dispatch (three pages per PLAN.md
  evidence row `browser-visual`); write every evidence record as `current`
  via `bin/verify-evidence`. Manual — skipped in AO.
- [ ] **T16** (2, main) — Deploy. `consent-broker consent -Reason "deploy
  MoveUp relay to relay.desk.zentala.io" -Action "wrangler deploy + d1
  migrate + mint 1 Founder license"`; on ALLOW: `just relay-deploy`, DNS
  record via the Cloudflare account, `/healthz`, e2e script against the live
  host with the license injected by `password-broker`. Manual — skipped in
  AO.

## Done means

All ten acceptance criteria in PLAN.md hold; all 16 evidence records are
`current` (T13's included — no skip); version bumped and tagged;
`.plan/HISTORY.md` entry; `STATE.md` updated; the epic's `IMPRO.md` triaged;
`epics/INDEX.md` row set to `done` in the closing commit.

## AO

Decision: **full AO run** (skill `ao` §0). The epic is 111 points, and waves
W1 (4 tasks), W2 (3) and W3 (3) fan out across three chains with disjoint
write sets — the per-task isolation buys real protection here. Skip T13,
T15, T16 (`--skip E022-T13,E022-T15,E022-T16`) — their verification is
manual by nature. Integration-surface files (§3a) are pre-granted in every
write set below: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`,
`src-tauri/Cargo.lock`, `src-tauri/src/commands.rs`, `package.json`,
`pnpm-lock.yaml`, `src/types.ts`, `src/generated/**`, `PROJECT.xml`.

```yaml
project: MoveUp
epic: E022
base_ref: main
tasks:
  - id: E022-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/remote_protocol.rs", "src-tauri/src/remote_protocol_tests.rs", "src-tauri/src/lib.rs", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src/remote/protocol.ts", "src/remote/protocol.test.ts", "src/remote/index.ts", "src/generated/**", "src/types.ts", "tests/fixtures/relay-protocol/**", "package.json", "pnpm-lock.yaml", "PROJECT.xml"]
    claims: ["src-tauri/src/remote_protocol.rs", "src-tauri/src/remote_protocol_tests.rs", "src/remote/protocol.ts", "src/remote/protocol.test.ts", "tests/fixtures/relay-protocol/**"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_protocol && npx vitest run --config vite.config.ts src/remote/protocol.test.ts"
    budget_minutes: 60
  - id: E022-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T01"]
    write_set: ["relay/**", "src/remote/protocol.ts", "PROJECT.xml"]
    claims: ["relay/**"]
    verification: "pnpm --dir relay install --frozen-lockfile && pnpm --dir relay exec vitest run test/room"
    budget_minutes: 120
  - id: E022-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T02"]
    write_set: ["relay/**", "PROJECT.xml"]
    claims: ["relay/migrations/**", "relay/src/auth/**", "relay/src/http/**", "relay/src/index.ts", "relay/scripts/**", "relay/test/auth/**", "relay/test/http/**", "relay/README.md"]
    verification: "pnpm --dir relay exec vitest run test/auth test/http"
    budget_minutes: 120
  - id: E022-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T03"]
    write_set: ["relay/**"]
    claims: ["relay/src/room/**", "relay/test/commands/**"]
    verification: "pnpm --dir relay exec vitest run test/commands"
    budget_minutes: 60
  - id: E022-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T01"]
    write_set: ["src-tauri/src/relay_client.rs", "src-tauri/src/relay_client_tests.rs", "src-tauri/src/relay_status.rs", "src-tauri/src/lib.rs", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/src/commands.rs", "PROJECT.xml"]
    claims: ["src-tauri/src/relay_client.rs", "src-tauri/src/relay_client_tests.rs", "src-tauri/src/relay_status.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_client relay_status"
    budget_minutes: 90
  - id: E022-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T05"]
    write_set: ["src-tauri/src/relay_auth.rs", "src-tauri/src/relay_auth_tests.rs", "src-tauri/src/commands_relay.rs", "src-tauri/src/commands_relay_tests.rs", "src-tauri/src/config.rs", "src-tauri/src/config_tests.rs", "src-tauri/src/commands.rs", "src-tauri/src/commands_config.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/lib.rs", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/src/relay_client.rs", "src-tauri/src/relay_status.rs", "src/components/settings/SettingsTypes.ts", "src/generated/**", "src/types.ts", "PROJECT.xml"]
    claims: ["src-tauri/src/relay_auth.rs", "src-tauri/src/relay_auth_tests.rs", "src-tauri/src/commands_relay.rs", "src-tauri/src/commands_relay_tests.rs", "src-tauri/src/config.rs", "src-tauri/src/config_tests.rs", "src-tauri/src/commands.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_auth commands_relay config"
    budget_minutes: 90
  - id: E022-T07
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T06"]
    write_set: ["src-tauri/src/relay_commands.rs", "src-tauri/src/relay_commands_tests.rs", "src-tauri/src/relay_client.rs", "src-tauri/src/relay_client_tests.rs", "src-tauri/src/commands_profiles.rs", "src-tauri/src/commands_profiles_tests.rs", "src-tauri/src/event_logger.rs", "src-tauri/src/lib.rs", "src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "PROJECT.xml"]
    claims: ["src-tauri/src/relay_commands.rs", "src-tauri/src/relay_commands_tests.rs", "src-tauri/src/commands_profiles.rs", "src-tauri/src/commands_profiles_tests.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_commands commands_profiles"
    budget_minutes: 60
  - id: E022-T08
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T01"]
    write_set: ["src/remote/**", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/useRemoteDeskExtra.test.ts", "src/hooks/remoteDesk.test-helpers.ts", "src/hooks/useDesk.ts", "src/hooks/useDesk.test.ts", "src/hooks/useDeskTypes.ts", "src/hooks/useDeskAuto.ts", "src/hooks/useWidgetData.ts", "src/components/ConnectionOverlay.tsx", "src/components/ConnectionOverlay.test.tsx", "src/types.ts", "package.json", "pnpm-lock.yaml", "PROJECT.xml"]
    claims: ["src/remote/transports/**", "src/remote/storage.ts", "src/remote/storage.test.ts", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/useRemoteDeskExtra.test.ts", "src/hooks/remoteDesk.test-helpers.ts", "src/hooks/useDeskTypes.ts", "src/components/ConnectionOverlay.tsx", "src/components/ConnectionOverlay.test.tsx"]
    verification: "npx vitest run --config vite.config.ts src/remote src/hooks/useRemoteDesk src/components/ConnectionOverlay"
    budget_minutes: 90
  - id: E022-T09
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T08"]
    write_set: ["src/remote/**", "src/App.tsx", "src/App.test.tsx", "src/test/scenarios.ts", "src/widgets/**", "package.json", "pnpm-lock.yaml", "PROJECT.xml"]
    claims: ["src/remote/PairScreen.tsx", "src/remote/PairScreen.test.tsx", "src/remote/RemoteControls.tsx", "src/remote/RemoteControls.test.tsx", "src/remote/remote.css", "src/App.tsx"]
    verification: "npx vitest run --config vite.config.ts src/remote/PairScreen src/remote/RemoteControls"
    budget_minutes: 90
  - id: E022-T10
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T06", "E022-T09"]
    write_set: ["src/components/settings/**", "src/components/SettingsPanel.tsx", "src/components/SettingsPanel.test.tsx", "src/test/scenarios.ts", "package.json", "pnpm-lock.yaml", "src/types.ts", "PROJECT.xml"]
    claims: ["src/components/settings/RemoteSection.tsx", "src/components/settings/RemoteSection.test.tsx", "src/components/settings/PairedDevicesList.tsx", "src/components/settings/PairingCodeCard.tsx", "src/components/SettingsPanel.tsx"]
    verification: "npx vitest run --config vite.config.ts src/components/settings/RemoteSection src/components/settings/PairedDevicesList src/components/settings/PairingCodeCard"
    budget_minutes: 90
  - id: E022-T11
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T01"]
    write_set: ["src-tauri/src/remote_server.rs", "src-tauri/src/remote_server_tests.rs", "src-tauri/src/setup_helpers.rs", "docs/REMOTE_DISPLAY.md", "src-tauri/src/lib.rs", "PROJECT.xml"]
    claims: ["src-tauri/src/remote_server.rs", "src-tauri/src/remote_server_tests.rs", "docs/REMOTE_DISPLAY.md"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_server"
    budget_minutes: 45
  - id: E022-T12
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E022-T04", "E022-T07", "E022-T10", "E022-T11"]
    write_set: ["justfile", ".giter.yaml", "vite.config.ts", "vitest.scripts.config.ts", "scripts/relay-e2e.mjs", "tests/scripts/relay-e2e.test.ts", "PROJECT.xml", ".plan/BACKLOG.md", "package.json", "pnpm-lock.yaml"]
    claims: ["justfile", ".giter.yaml", "vite.config.ts", "vitest.scripts.config.ts", "scripts/relay-e2e.mjs", "tests/scripts/relay-e2e.test.ts", "PROJECT.xml"]
    verification: "npx vitest run --config vitest.scripts.config.ts tests/scripts/relay-e2e.test.ts"
    budget_minutes: 60
  - id: E022-T14
    repo: MoveUp
    executor: main
    depends_on: ["E022-T12"]
    write_set: [".arch/ADR/022-relay-on-cloudflare-durable-objects.md", ".arch/ADR/023-pairing-code-device-token-auth.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", "docs/REMOTE_DISPLAY.md", "docs/PRIVACY.md", ".plan/decisions.jsonl", ".plan/BACKLOG.md", ".plan/BUSINESS_CONTEXT.md", "scripts/check-e022-t14-docs.mjs"]
    claims: [".arch/ADR/022-relay-on-cloudflare-durable-objects.md", ".arch/ADR/023-pairing-code-device-token-auth.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", "docs/REMOTE_DISPLAY.md", "docs/PRIVACY.md", ".plan/decisions.jsonl", "scripts/check-e022-t14-docs.mjs"]
    verification: "node scripts/check-e022-t14-docs.mjs"
    budget_minutes: 45
```

Outside AO, in this order after `ao promote`: T13 (security review — before
anything is deployed), T15 (verify + browser), T16 (deploy behind
`consent-broker`). `just check` on the integration branch before promote,
as §3 of the AO skill says.
