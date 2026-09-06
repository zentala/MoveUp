# Backlog — Desk App

## Planned Epics

- [x] **[E016 — Plan hygiene and AO readiness](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/PLAN.md)** — fix `.plan/STATE.md` self-contradiction, consolidate four backlog files into `.plan/BACKLOG.md`, close E011's ceremony, untrack `coverage/`+`test-performance-report/`, add `.giter.yaml`+root `justfile` for AO. Handoff: [HANDOFF.md](epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/HANDOFF.md). 8 points. Done via AO run E016-20260906-0348, promoted `0995f42` (2026-09-06). See [HISTORY.md](HISTORY.md#e016--plan-hygiene-and-ao-readiness-2026-09-06).
- [x] **[E015 — One truth for the sitting counter](epics/E015-2026-09-06-engine-single-truth/PLAN.md)** — popup reads the uncredited `current_session_secs`; delete it, one credited counter, PostureBalance + `break_credit` row, ADR 008 rev. Handoff: [HANDOFF.md](epics/E015-2026-09-06-engine-single-truth/HANDOFF.md). 21 points. Code-complete via AO, promoted `7a8bbf3`, tagged v0.6.0 (2026-09-06). T05's browser evidence gap stays open — see "Dev-mode remote display" section below. See [HISTORY.md](HISTORY.md#e015--one-truth-for-the-sitting-counter-2026-09-06-v060--code-complete-t05-open).
- [ ] **E014 HANDOFF.md has no `## AO` block** — waves 1-2 are unblocked but cannot be dispatched through AO until the block exists ([epics/E014-2026-09-05-supervised-release-rollback/HANDOFF.md](epics/E014-2026-09-05-supervised-release-rollback/HANDOFF.md)); write it in the E016 session using E015's block as the template. (Medium, 2)
- [ ] **[E014 — Supervised release rollback](epics/E014-2026-09-05-supervised-release-rollback/PLAN.md)** — handoff: [HANDOFF.md](epics/E014-2026-09-05-supervised-release-rollback/HANDOFF.md). Keep several installed builds with a `last-known-good` marker so a bad release can be rolled back, and let PM3 fill the gap when nothing holds the app. Waves 1-2 are unblocked; waves 3-4 wait on two PM3 backlog items (`int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md`, section "2026-09-05 — Nadzór z powrotem do poprzedniego builda").
- [ ] **[E013 — Signed Tauri release and PM3 deployment](epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md)** — build signed Windows release, deploy the installed SmartDesk executable under PM3, and expose the remote display at `moveup.internal`. Prepare implementation tasks and execute in a new session.
- [ ] **Code-signing provider — decision deliberately deferred** (2026-09-06,
  E017 "Outside AO" checklist, [HANDOFF.md](epics/E017-2026-09-06-release-readiness/HANDOFF.md#outside-ao)):
  the installer ships unsigned, so every download shows a Windows SmartScreen
  warning. Constraint from
  [E013 PLAN.md, Wave 2](epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md):
  no exportable private key in the repo, must support unattended CI signing.
  Two candidates surfaced and not chosen: a traditional EV/OV cert
  (DigiCert/GlobalSign, ~200-500 USD/yr per
  [`.claude/rules/installer.md`](../.claude/rules/installer.md)) or Azure
  Trusted Signing (~10 USD/mo + per-signing, key never leaves Azure — matches
  the "no exportable key" constraint directly). Paweł chose to leave this open
  rather than decide now (2026-09-06) — does not block E017 or any other
  epic, only blocks a SmartScreen-warning-free public release. (Importance:
  Medium, Points: 8 decision + 5 CI wiring, per E013 Wave 2 estimate)
- [ ] **[E012 — Analyst Dashboard](epics/E012-2026-05-16-analyst-dashboard/PLAN.md)** — separate Tauri window with Data Catalog (8 sources, schema + samples) and Explorer (5 charts over last 7 days). Mockup-first per ux-design-flow rule. Research: [reports/2026-05-16-data-sources.md](epics/E012-2026-05-16-analyst-dashboard/reports/2026-05-16-data-sources.md). Version bump to 0.5.0 in epic setup.

---

## Bugs — Fix Now

- [x] **CRITICAL: Seeding ignores daily reset** — fixed in commit `c1d286f`: `load_today_totals()` now filters by `daily_reset_after` timestamp persisted to tauri-plugin-store. Sessions started before the last daily reset are excluded from DB seeding.
- [x] **hourly_break_tracker not persisted** — fixed: `HourlyBreakTracker` fields (`hours_with_break`, `hours_active`, `current_away_secs`) now persisted in `PersistedSessionState` via tauri-plugin-store. Survives restarts within same day. Backward compatible via `#[serde(default)]`.
- [x] **Duplicate notifications on sit limit** — fixed: removed `notify: "popup"` from second escalation step in all profiles. Now only one toast fires at limit, visual-only escalation (blink+pulse) at +5 min.
- [x] **Notification spam when ignored** — fixed: escalating cooldown (0→5m→15m→30m→silence), configurable per profile via `snooze.notify_cooldowns_secs`. Max 4 reminders, then silence until position change.
- [x] **3 conflicting autostart registry entries** — removed `zntlDesk`, `Smart Desk`, `SmartDesk` from HKCU Run.
- [x] **`welcome.html` shows raw `\uXXXX` escapes instead of Polish diacritics**
  — the first-run welcome popup title renders literally as
  `Cześć! Jestem Twoim osobistym asystentem biurkowym.` instead of
  `Cześć! …` in the real rendered UI (verified via browser at
  `http://localhost:3390/welcome.html`, screenshot 2026-09-06). Every other
  diacritic in the same copy block (`długo`, `żwiga`, etc.) has the
  same problem. Likely a JSON/i18n string that was JS-escaped
  (`JSON.stringify`) somewhere upstream and never decoded before being put in
  JSX, or a build step mangling a UTF-8 source file. `src/components/WelcomePopup.tsx`
  (component `WelcomePopup`) and whatever supplies its copy strings.
  Found 2026-09-06, E017 `first-run-browser` evidence check
  (`.plan/epics/E017-2026-09-06-release-readiness/PLAN.md:202`).
  (Importance: High, Points: 2)
  Fixed 2026-09-06: `WelcomePopup.tsx`'s JSX text nodes carried literal
  `\uXXXX` byte sequences instead of decoded UTF-8 (only the curly-brace
  `{"\uXXXX"}` emoji were real JS string escapes; plain JSX text like
  `Cześć!` is never interpreted by JS, so it rendered as-is).
  Rewrote the file with real UTF-8 Polish characters; added a test asserting
  the rendered text contains `ś`/`ć` and no `\uXXXX` pattern
  (`src/components/WelcomePopup.test.tsx`).
- [x] **`remote_server.rs` frontend static path is CWD-relative and breaks
  outside one specific launch context** — `frontend_service()` in
  `src-tauri/src/remote_server.rs:205-209` defaults `DESK_REMOTE_DIST` to the
  literal string `"../../dist"`, resolved against the **process's current
  working directory**, not the exe's own location. Launching the freshly
  built `desk.exe` with CWD = repo root (e.g. from a script, a shortcut with
  no explicit "Start in" folder, or a scheduled task) makes the remote
  display (`http://localhost:3390/`) 404 on every route — confirmed
  2026-09-06 while trying to browser-verify E017's `first-run-browser`
  check: root gave `404` until relaunched with `DESK_REMOTE_DIST` pointed
  explicitly at the real `dist/` folder. Whether double-click-from-Explorer
  or the installed shortcut happens to set the right CWD is untested and
  should not be relied on — this needs an absolute path derived from
  `std::env::current_exe()` (or a Tauri resource dir), not a relative guess.
  (Importance: High, Points: 3)
  Fixed 2026-09-06: added `default_dist_path()` in `remote_server.rs`, a pure
  function deriving `<exe-parent>/dist` from `std::env::current_exe()`, used
  only when `DESK_REMOTE_DIST` is unset (override still works). Tests in
  `remote_server_tests.rs` assert the result is independent of
  `std::env::current_dir()`.
- [x] **`App.tsx` calls Tauri's `listen()` unconditionally — throws on every
  browser/remote-display load** — `src/App.tsx`'s
  `useEffect(() => { const unWidget = listen("desk:show-widget", ...); const
  unSettings = listen("desk:show-settings", ...); ... }, [])` is not gated by
  `isTauri`, so in remote-display (plain browser, no `window.__TAURI_INTERNALS__`)
  it throws `TypeError: Cannot read properties of undefined (reading
  'transformCallback')` on mount — confirmed via browser console at
  `http://localhost:3390/`, 2026-09-06 (E017 `first-run-browser` check).
  Same class of bug in `src/components/StepsWidget.tsx`: its Tauri `invoke`
  call in remote mode fails with `Cannot read properties of undefined
  (reading 'invoke')`, surfaced to the user as "Last refresh failed: TypeError:
  …" in the Steps KPI tile instead of a clean "not available in remote mode"
  state. Neither crash currently breaks the rest of the UI (the dashboard
  still renders), but both are unhandled exceptions on every phone/remote
  page load and should be guarded with `if (!isTauri) return;`.
  (Importance: Medium, Points: 3)
  Fixed 2026-09-06: `App.tsx`'s tray-command `useEffect` now returns early
  when `!isTauri` before calling `listen()`. `StepsWidget.tsx` gained its own
  `isTauri` check (same `window.__TAURI_INTERNALS__` pattern as
  `useDeskAuto.ts`), guards `invoke()` in both `loadCached` and `refresh`,
  and renders a "Steps … not available" badge in remote mode instead of
  attempting the call. Tests: `src/App.test.tsx` (new) asserts `listen()` is
  not called without `__TAURI_INTERNALS__`; `StepsWidget.test.tsx` gained a
  remote-mode describe block asserting the clean unavailable state and zero
  `invoke()` calls.
- [ ] **No `.gitattributes` — line endings depend on local `core.autocrlf`** — a
  fresh `git worktree add` checkout on this machine converted LF blobs to
  CRLF, which made 3 unrelated files (`.plan/.../HANDOFF.md`, its task file,
  `src-tauri/gen/schemas/*.json`) show as dirty with zero real diff lines,
  and an Agent Orchestrator run refused to merge a clean worker over it
  (`dirty_worktree`). Worked around locally with
  `git config --local core.autocrlf false`, verified clean on a scratch
  worktree — but that only protects worktrees created on THIS machine.
  Fix properly: add `.gitattributes` with `* text=auto` (or pin `eol=lf`
  for `.md`/`.json`/`.rs`/`.ts`), then `git add --renormalize .` once.
  Found 2026-09-06 running E015 through AO
  (`.plan/epics/E015-2026-09-06-engine-single-truth/JOURNAL.md`).
  (Importance: Low, Points: 2)

---

## Autostart — findings 2026-09-05

- [x] **Autostart dead since the registry cleanup** — the earlier entry "3 conflicting
  autostart registry entries — removed `zntlDesk`, `Smart Desk`, `SmartDesk` from HKCU Run"
  deleted all three and re-registered none. Nothing could restore them: `ensure_autostart`
  is a deliberate no-op in debug builds ([src-tauri/src/setup_helpers.rs:92](../src-tauri/src/setup_helpers.rs)),
  and no release build existed on this machine. Fixed 2026-09-05 by building and installing
  release 0.5.0; `HKCU\...\Run\MoveUp` now points at
  `%LOCALAPPDATA%\MoveUp\desk.exe --minimized`. (Importance High, 3 points)
- [x] **Self-heal read the wrong registry key** — `read_autostart_registry_path` hardcoded
  `SmartDesk` while the plugin registers under `productName` (`MoveUp`), so the E011-T01
  stale-path check compared nothing. The value also carries `--minimized`, which never
  equals a bare exe path. Fixed in commit `7305214`,
  [src-tauri/src/setup_helpers.rs:115](../src-tauri/src/setup_helpers.rs). (Importance High, 2 points)
- [x] **Release build was blocked twice** — npm/Cargo Tauri version mismatch (commit `86b2f4d`)
  and a parallel-test env race in [src-tauri/src/google_fit.rs:375](../src-tauri/src/google_fit.rs)
  (commit `9559dea`). (Importance Medium, 3 points)
- [x] **Decide the supervision model for the installed app** — decided 2026-09-05.
  The Windows `Run` key stays as the login path, because it starts the app even when
  the PM3 daemon is down and needs no infrastructure. PM3 does not replace it; PM3
  owns only gap-filling and rollback. Supervision splits in two: PM3 gets the generic
  loop (60-second poll, act only after two consecutive misses, stand down when another
  process holds the `io.zntl.desk` singleton, demote to an older candidate), MoveUp
  gets the app-specific half (a store of several builds, the `last-known-good` marker,
  the definition of a healthy build, retention). Written up as
  [E014](epics/E014-2026-09-05-supervised-release-rollback/PLAN.md). **MoveUp is waiting
  on two PM3 items** filed in `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` under
  "2026-09-05 — Nadzór z powrotem do poprzedniego builda": candidate lists with demotion
  (8 points) and singleton stand-down (5 points). E014 tasks T05, T06 and T07 cannot
  start until those land, and T10 additionally waits on PM3 item E000-A4 (`pm3d` has no
  ONLOGON task). Cross-repo handoff: `int://mATX.lan/C:/code/pm3-mcp/.plan/handoffs/HANDOFF-2026-09-05-moveup-supervision.md`.
  Revised 2026-09-05: Paweł decided PM3 must be the ONLY daemonizer, so the `Run` key is
  retired by E014-T10 rather than kept. (Importance Medium, 5 points)

## Dev-mode remote display — findings from E015-T05 (2026-09-06)

The `browser` agent tried to visually verify E015's fix (popup timer after a
2-min stand) via the Remote Display server and could not reach a rendered
UI at all in dev mode — two pre-existing gaps, neither caused by E015:

- [ ] **`vite.config.ts` has no dev proxy for `/display/*` → `localhost:3390`**
  — in a debug build, `remote_server.rs:72-80`'s fallback route
  (`dev_fallback()`) serves a static placeholder page literally reading
  "Vite proxy not yet implemented (T05)" instead of the real app, so
  `http://localhost:3390/display` never shows the actual UI in dev mode.
  Loading the real Vite dev server at `:1443` instead fails differently:
  `src/hooks/useRemoteDesk.ts:123` builds `ws://${window.location.host}/display/ws`,
  which becomes `ws://localhost:1443/display/ws` — nothing answers there,
  so the page is stuck on "Reconnecting...". Add the Vite proxy (or fix the
  WS URL construction to target `:3390` explicitly in dev) so remote-display
  QA doesn't require a full `pnpm tauri:build`. (Importance: Medium, Points: 3)
- [ ] **`src/App.tsx:93-107` calls Tauri `invoke`/`transformCallback`
  unconditionally**, not gated by `useDeskAuto`'s `isTauri` check — throws
  `Cannot read properties of undefined` on every page load when the app runs
  in a plain browser (confirmed on `:1443` during the above check). Gate
  those calls the same way `useDeskAuto` already gates the hook choice.
  (Importance: Low, Points: 1)
- [ ] **No session-level mock/fixture path for the popup UI** — `OVERLAY_DATA=mock`
  (`pnpm tauri:dev:mock`) only drives the disconnected native overlay bar
  (`overlay_renderer.rs`); the actual `SessionManager` behind the popup and
  remote display is always fed by the real serial sensor, so there is no way
  to force a controlled sit→stand→sit cycle without physical hardware or a
  production build + standing at the desk. This is why E015-T05's browser
  acceptance check (`.plan/epics/E015-2026-09-06-engine-single-truth/PLAN.md`
  evidence contract, `popup-visual`) could not be completed and has NO
  evidence record — the engine-level fix is proven by the `e015_` Rust
  scenario test (sit 30 min → stand 2 min → sit, asserts the DTO) and by the
  live `limit_used_secs` field seen in the real `/display/api` payload during
  this session, but nobody has watched the rendered `OneBarTimer` after a real
  stand since the fix landed. Add a `tests/emulator`-style `inject_reading`
  path reachable from a plain browser (or accept `pnpm tauri:build` + physical
  dogfooding as the only route) so this class of check doesn't depend on
  hardware timing. (Importance: Medium, Points: 5)

**Sposób ogarnięcia** (dla agenta, który to rozpisze na taski — nie
rozstrzygam tu, tylko daję kierunek): trzy niezależne naprawy, można robić
osobno lub razem.
1. **Vite dev proxy** — dopisać w `vite.config.ts` `server.proxy` dla
   `/display` → `http://localhost:3390` (REST + WS), tak jak każdy typowy
   Vite+backend setup. Naprawia stronę `:1443`, nie rusza Rusta.
2. **`dev_fallback()` w `remote_server.rs`** — w buildzie debug albo przekierować
   na `:1443` (302), albo serwować to samo co (1) z drugiej strony. Wybór
   między tym a (1) zależy, gdzie ma żyć prawda o porcie — jedno z nich
   wystarczy, nie oba na raz.
3. **Mock na poziomie silnika, nie tylko overlay-bara** — dodać ścieżkę typu
   `SESSION_MOCK=1`, która wstrzykuje syntetyczne odczyty do `SessionManager`
   przez ten sam mechanizm co `inject_reading` w `tests/emulator`, tylko
   dostępny z przeglądarki (np. debug-only Tauri command albo REST endpoint
   pod `/display`). To jedyna z trzech napraw, która realnie odblokowuje
   zautomatyzowaną weryfikację wizualną bez stania przy biurku 2 minuty za
   każdym razem.
Punkt 3 jest tym, co faktycznie rozwiązuje problem E015-T05; punkty 1-2 są
warunkiem wstępnym (bez proxy i tak nie ma czego oglądać w przeglądarce).

## UX Issues — High Priority

- [x] **Timeline readability** — fixed: implemented timeline skin system with 3 switchable themes (Semantic, Amber, Clinical). Each skin defines distinct `--tl-*` CSS vars. Dropdown in Settings → More → Timeline Theme. Default: Semantic (burgundy/green/blue/gray).

---

## Open Tasks from Previous Epics

### From E002 — Overlay Progress Bar
- [ ] E002-T09 — [Choose production render mode](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T09-choose-production-mode.md) (OPAQUE vs LAYERED) — decision made (OPAQUE=default), needs ADR
- [ ] E002-T11 — [Verify debug overlay info](epics/E002-2026-03-16-overlay-progress-bar/tasks/E002-T11-verify-debug-overlay.md) — display works, needs manual QA sign-off

### From E003 — Installer & Distribution
- [ ] E003-T07 — [GitHub Releases CI/CD](epics/E003-2026-03-16-installer-distribution/tasks/E003-T07-github-releases-automation.md) (requires code signing certificate) — superseded by E013 (see [E013/PLAN.md](epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md) status note).

### From E004 — Session Alerts & Snooze
- [ ] E004-T04 — [Integration test: full alert flow](epics/E004-2026-03-20-session-alerts/tasks/E004-T04-integration-test-alert-flow.md) — basic tests exist, time simulation missing
- [ ] T017 — Stages 3-5 implementation (no task file)
- [ ] T018 — Notification A/B testing (no task file) — profiles now enable this; see Profile System section
- [ ] T019 — Success notifications + gamification (no task file)

### From E006 — Session Bugs & Polish
- [ ] E006-T14 — Connection status UI cleanup — functional in debug section only
- [ ] E006-T15 — Tooltip + UI integration tests — tooltips done, e2e coverage basic

### From E007 — KPI Dashboard + Timer UX
- [ ] T047 — Visual verification (demo smoke test)
- [ ] T048 — Fix issues found during verification

---

## Tauri Plugins

- **`window-vibrancy`** (crate) — Windows Acrylic/Mica blur effect on floating window
  - requires `"transparent": true` in `tauri.conf.json` + `background: transparent` in CSS
- **`tauri-plugin-autostart`** — start app at system login

---

## App Icon — True Vector SVG

- `src-tauri/icons/icon-source.svg` is NOT a real vector — it's a PNG rasterized image wrapped in an SVG container (base64-encoded `data:image/png` inside `<image href=...>`). Colors cannot be changed via CSS/attributes.
- **To do:** Replace with a proper SVG drawn with `<path>` / `<rect>` elements so it can be recolored and scaled infinitely.
- Search terms: Noun Project `standing desk`, SVG Repo `adjustable desk`, Flaticon `sit stand`
- Or: design from scratch in Inkscape/Figma — a simple desk silhouette (horizontal top + two legs at different heights) works at 16px.
- Once real SVG exists: use `pnpm tauri icon source.png` to auto-generate all required sizes.

---

## Alert System — Future

- **Redesign alert popups — Tauri WebviewWindow instead of raw WinAPI** — current popups (`alert_popup_window.rs`) use raw WinAPI GDI, which looks like a 2003 Win32 dialog. Migrate to a Tauri WebviewWindow so we can style alerts with HTML/CSS using the instrument panel design system (--panel-*, --beam-*, --ink-*). This enables: dark themed popups matching the main window, animated transitions, rich content (progress bars, coach messages in popup), and the acrylic blur effect.

---

## Sensor & Readings

- **Extended height stabilization** — when desk is stationary for extended period (no significant movement), increase smoothing window 2x to eliminate residual jitter (e.g. 92→91→92→91 flickering). Current HeightStabilizer uses 10-sample window; for stationary desk, double to 20 samples or use exponential moving average with lower alpha.
- **Sensor diagnostics panel** — show raw vs smoothed readings, debounce state, threshold visualization
- [ ] **Migotanie połączenia (connect/lost w tej samej sekundzie) nie jest nigdzie sygnalizowane** — `src-tauri/src/serial.rs:161-200`: przy złym kablu urządzenie enumeruje się i natychmiast pada, co w `events.log` wygląda tak: `00:30:47 DEVICE connected COM3` / `00:30:47 DEVICE lost` / `00:30:50 DEVICE connected COM3` / `00:30:50 DEVICE lost`. Apka traktuje każdy cykl jak normalne podłączenie — brak licznika, brak progu, brak ostrzeżenia dla użytkownika, mimo że to jednoznaczny objaw problemu z zasilaniem/kablem (patrz CLAUDE.md → Hardware → Cable sensitivity). Fix: wykrywać N połączeń trwających krócej niż X sekund w oknie Y i emitować ostrzeżenie („niestabilne połączenie — spróbuj innego kabla / portu bez huba”) do tray + zakładki Debug. Znalezione 2026-09-06 przy diagnozie „sensor podłączony, apka nie wykrywa”. (Importance: Medium, Points: 3)
- [ ] **Brak portu COM jest nieodróżnialny od "sensor nie odpowiada"** — `src-tauri/src/serial.rs:161-200`: pętla skanująca loguje `No desk sensor found` tylko przez `info!` (stderr), nie do `events.log`, i nie rozróżnia dwóch zupełnie różnych stanów: (a) `available_ports()` zwróciło PUSTĄ listę (Windows w ogóle nie widzi urządzenia — kabel tylko do ładowania, martwy port, płytka bez zasilania), (b) porty są, ale żaden nie odpowiedział `DEVICE: zntl-desk-sensor v1` (zły firmware/baud/zajęty port). Użytkownik widzi w obu wypadkach to samo „brak sensora”. Fix: logować do `event_logger` liczbę znalezionych portów i ich nazwy (`DEVICE scan ports=0` / `DEVICE scan ports=2 [COM3, COM5] none matched`), pokazać to w zakładce Debug. Zgodne z regułą „cisza nigdy nie znaczy sukcesu” — asertuj LICZBĘ przeskanowanych rzeczy, nie sam brak wyniku. Znalezione 2026-09-05 przy diagnozie „sensor podłączony, apka nie wykrywa” (okazało się: zero portów COM w systemie). (Importance: Medium, Points: 2)

---

## Logging & Observability

- **SQLite time-series storage** — replace file-per-minute snapshots with queryable SQLite table. Enables: search across days, trend analysis (sitting % over weeks), anomaly detection (daily score dropping), dashboard visualization. Natural phase 2 after file-based logging proves useful. Effort: M, Priority: P3.
- Note: file-per-minute snapshots (`snapshot_logger.rs`) and event log (`event_logger.rs`) are already implemented. `height_readings` table exists in DB schema but is never populated — could be used for time-series.

---

## Max Continuous Computer Time Alert

Alert: max continuous work at computer. Standing ≠ break from screen.

- Separate timer "time at computer" (independent from sitting session timer)
- Thresholds: <45m green, 45-75m yellow, >75m red
- Alert on limit exceeded (default 75m, configurable)
- Reset: ≥5 min away from computer
- Note: `longest_computer_session_secs` and `continuous_computer_secs` fields already exist and work. This is about adding an alert when limit exceeded — the KPI already shows the value.

---

## Activity Tracking

- [x] **Activity status in UI** — done: StateIndicator shows "Active" / "Idle Xm Ys". Toggle: Settings → More → "Show activity status".

- [ ] **Split position_changes into two metrics: desk changes + posture changes** — currently `position_changes` counts both desk height changes (Sitting↔Standing) and Away resets (user left computer for 5+ min). These are different things:
  - **Desk changes** (`desk_position_changes`) — physical desk movement (sit↔stand)
  - **Posture changes** (`posture_changes`) — any change including leaving computer (desk changes + away returns)
  - Both matter for health: desk changes = ergonomic variety, posture changes = not being sedentary
  - KPI "Changes/h" should show posture_changes (broader metric)
  - New KPI badge or detail view could show desk_position_changes separately
  - Requires: split `position_changes` field into two, update KPI metric engine, update frontend
  - Priority: P3 — works fine as-is, split when adding detailed analytics

- **Cross-platform activity detection** — current implementation is Windows-only (`GetLastInputInfo`). Before release, need Linux/macOS support. Options: `rdev` crate (cross-platform input hooks), X11/Wayland idle APIs, macOS `CGEventSource`. Non-Windows currently returns `idle=0` (always active) — Away state will never trigger on Linux/macOS.

---

## Visualization & Charts

- **[E012-T09 — Analyst layout flip](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md)** — top-centred date header, bottom day-navigator. Sticky bottom nav.
- **[E012-T10 — Recharts donut KPIs](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md)** — replace KpiTrend + BreakCreditHistogram bars with donut/gauge cards. Numeric centres. New ADR 014.
- **[E012-T11 — Pulse on selectedDay change](epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T11-selected-day-pulse.md)** — CSS-only highlight animation on KPI cards when day changes.
- **[Spike — time-viz library evaluation](reports/2026-05-21-time-viz-libraries-spike.md)** — research charting + timeline + time-data libs (Recharts / ECharts / Visx / vis-timeline / crossfilter / Temporal / etc.). Output: recommendation matrix + migration cost for replacing custom SVG timeline.
- **[Spike — timeline visual upgrades](reports/2026-05-21-timeline-visual-upgrades-spike.md)** — HTML demo gallery with 5-8 visual treatments of our SVG strip (gradients, cardiogram, stream graph, mini-multiples, typographic header, hover cursor, heatmap calendar). User picks 1-2 to productise.

---

## Timeline Full Window — Expandable History View

- Click on timeline in popup → opens a dedicated window with full-day/multi-day timeline
- Scroll through days, zoom in/out, click sessions for details
- Shows: all states with durations, breaks, position changes, daily score
- Implementation: Tauri WebviewWindow (like welcome popup), separate route
- Priority: P3 — needs SQLite history infrastructure first

---

## KPI Time Range Selector

- Switch to view stats for: Today / 7 days / 30 days
- Requires: SQLite time-series storage
- Currently all KPIs are today-only (in-memory, reset at midnight)
- UI: small toggle/tabs above KPI strip ("Today | 7d | 30d")
- Priority: P3 — needs DB history infrastructure first

---

## PostureBalance Ratio Refinement

- [ ] **PostureBalance ratio uses mixed counters** — condition is `sitting_seconds_total >= 6h AND sitting_seconds > standing_seconds * 2`. But `sitting_seconds` is reduced by break credit while `sitting_seconds_total` is not. After a day of 8h sitting with breaks, `sitting_seconds_total = 28800` but `sitting_seconds` may be 0 — ratio never triggers despite genuinely sedentary day. Consider using `sitting_seconds_total` for both checks, or a separate daily-ratio metric. Track in real usage first. See [ADR 009](.arch/ADR/009-day-break-credit.md).

---

## Gamification & Scoring

- [x] **Notification flag persistence** — done: flags + credit-reduced `sitting_seconds` + `daily_score` persisted to tauri-plugin-store, survive restarts. Date-guarded (discards stale data after midnight).
- **Gamification techniques research** — deep research report listing 100+ gamification techniques (loss aversion, streaks, milestones, social proof, progression systems, comeback mechanics, etc.). Output: `.plan/reports/gamification-techniques.md` with categorized list, brief descriptions, and applicability to desk app. Reference material for future epic planning.
- **Scoring & metrics redesign** — umbrella task for iterating on daily_score formula, KPI thresholds, comeback mechanics (show user HOW to recover negative score), penalty/reward balance. Collect ideas here, plan as epic when ready.
- **Notification flag persistence** — `alert_fired`, `standing_target_reached_fired`, `notify_inactivity_fired` etc. reset on app restart causing notification spam. Persist in DB or derive from today's event log on startup. Priority: P2 (affects UX on every dev restart).

## Profile System

- [ ] **Profile editor UI** — visual form for editing profiles instead of raw JSON. Dropdown is done, but editing requires opening JSON in external editor. Build an in-app form with live preview.
- [ ] **Per-application profiles** — auto-switch profile when gaming (detect fullscreen app), on calls (detect audio), or in presentations. Needs activity/application detection.
- [ ] **Scheduled profiles** — time-based auto-switching (e.g., "aggressive" during work hours 9-17, "gentle" evenings). Cron-like schedule in profile config.
- [ ] **Profile analytics** — track which profile produces better ergonomic outcomes over time. Compare daily scores across profiles.

---

## Communication

- [ ] **Profile reload notification** — show a subtle toast/log when profile is hot-reloaded so user knows the edit was picked up. Currently silent.
- [ ] **Overlay neutral bar style** — the "neutral" overlay bar (sitting within limit) needs visual design. Currently uses dark panel color — may need a subtle gradient or pattern to be distinguishable from "no bar."
- [ ] **Per-profile cooldown tuning** — `snooze.notify_cooldowns_secs` is now configurable per profile, but built-in profiles all use the same default `[0, 300, 900, 1800]`. Tune: aggressive = shorter (e.g., `[0, 120, 300, 600]`), gentle = longer (e.g., `[0, 600, 1800, 3600]`), silent = empty `[]` (no notifications).

---

## Content / Articles

- [ ] **Article: Screen time & eye health** — blog post for landing page based on [screen time research report](reports/screen-time-eye-health-research.md). Topics: CVS statistics, 20-20-20 rule, break frequency research, how SmartDesk helps. Target: SEO for "screen break reminder" / "computer eye strain". Link to product page. Priority: P2 — write after computer time alert feature ships.

---

## Future Features

- **Notification A/B testing** — now enabled via communication profiles: switch between profiles to A/B test notification strategies (T018)
- **Success notifications + gamification** — streak tracking, milestone celebrations (T019)
- **Phone-as-hub** — old phone + BLE sensor, works without desktop app
- **Smartwatch integration** — proximity detection, walking state, HRV
- **Sleep API integration for Analyst timeline** — current "Away" overnight is honest but uninformed; pull from Google Fit / Health Connect / Apple Health / smartwatch to overlay actual sleep window on the Daily Timeline Strip. Replaces the cut "sleep band" heuristic (E012-T07 brainstorm 2026-05-18).
- Data export to CSV
- Height threshold calibration via UI
- App icon design: desk silhouette SVG → PNG/ICO assets

---

## Vision / Architecture (future)

- **T-ARCH-001** — Extended body positions: kneeling / leaning / sitting / standing
  - Same sensor, richer height-to-position mapping
  - `DeskPosition` enum extensible, per-position session stats + points
  - Max standing session: 90 min (configurable, option: 60 min) — symmetric to sit limit

---

## Removed (already implemented or superseded)

- ~~`tauri-plugin-notification`~~ — already integrated
- ~~`tauri-plugin-store`~~ — already integrated
- ~~Dynamic tray icon~~ — superseded by T016 (color dot approach)
- ~~Standing Mode Theme~~ — superseded by T032 (widget temperature system)
- ~~Bar flashing at limit~~ — implemented in T013 (alert Stage 1, variant 2 pulsing)
- ~~Red popup at limit~~ — implemented in T013+T014 (AlertPopup)
- ~~Snooze logic~~ — implemented in T015 (deescalating snooze)
- ~~Color dot on tray icon~~ — implemented as T016
- ~~E001-T09 position_changes DB~~ — implemented (db.rs schema + migrations)
- ~~E001-T11 Rail pulse animation~~ — implemented (HeightRail.tsx + CSS keyframes)
- ~~E001-T12 Yesterday delta arrow~~ — implemented (TodayStats.tsx ↑/↓ delta)
- ~~E004-T05 Dismiss without sensor~~ — implemented (alert_manager.rs snooze)
- ~~KPI Strip~~ — implemented in E007 (4 badges: standing%, changes/h, breaks, screen)
- ~~KPI "Session" rename~~ — renamed to "Screen" in E008-T05
- ~~State Machine Redesign (Away)~~ — implemented in E008-T02 (Away triggers on 60s idle)
- ~~NotificationService~~ — implemented in E008-T09 (centralized routing)
- ~~Coach Messages removal~~ — removed in E007-T11
- ~~Height stabilization (basic)~~ — implemented in E006-T10 (HeightStabilizer module)
- ~~Persistent KPI Strip~~ — implemented in E007-T08 (KpiStrip component)
- ~~HourlyBreakTracker wiring~~ — wired into session loop, metric uses real data
- ~~Standing % counts Away time~~ — fixed: standing_bout_started tracks actual standing only
- ~~AlertManager message strings in settings panel~~ — superseded by CommunicationPolicy profiles (messages live in communication profile JSON, editable per profile)
- ~~Notification strategy as pluggable system~~ — superseded by CommunicationPolicy + profiles (different backends, escalation patterns, thresholds are all configurable per profile)
- ~~Color inconsistency (green/gold)~~ — fixed: unified color dictionary, green/gold removed from color system

---

## Backend architecture review — 2026-09-06

Full findings, module map, persistence/config/event inventories, and a
recommended target structure: [reports/_review-2026-09-06/backend.md](reports/_review-2026-09-06/backend.md).
Not implemented (review task explicitly did not modify source). Top items,
low-confidence-to-fix-alone so filed here rather than auto-applied. **Planned
as [E019 — backend hardening](epics/E019-2026-09-06-backend-hardening/PLAN.md)
(2026-09-06, 28 points, 8 tasks)** — done via AO run `E019-20260906-0822`,
promoted `4649dc5` (2026-09-06). Items below closed by E019's tasks (T01,
T02, T05, T04, T03, and T07/T08 for the TrayController item) — the clippy
and popup-timer items are NOT E019's scope and stay open.

- [x] **`commands.rs` uses plain `.unwrap()` on every mutex lock (14 sites)** — a panic
  elsewhere while holding `session`/`db`/`config`/`comm_policy` poisons the mutex and
  then every subsequent IPC call through `commands.rs` panics too, unlike
  `tray_controller.rs`/`tray_signal_exec.rs`/`remote_server.rs` which use
  `unwrap_or_else(|e| e.into_inner())` to survive a poisoned lock. Violates
  `rules/rust.md` ("No `unwrap()` in production code").
  [src-tauri/src/commands.rs:44,74,75,80,86,89,95,97,102,155,166,182,193,199,216](../src-tauri/src/commands.rs)
  (Importance High, 3 points) — closed by E019-T01 (poison-safe pattern + regression test).
- [x] **Four overlapping persistence mechanisms, no documented precedence** — SQLite
  `sessions` table, `tauri-plugin-store` (`persisted_session_state`), per-minute JSON
  snapshots, and `events.log` each independently reconstruct "today's totals" via three
  different code paths (`db_sessions::load_today_totals`,
  `session_persistence::PersistedSessionState::load`,
  `commands_analyst::collect_snapshots`). A discrepancy between them is currently
  undetectable at runtime. [src-tauri/src/commands.rs:60-82](../src-tauri/src/commands.rs),
  [src-tauri/src/commands_analyst.rs:129-159](../src-tauri/src/commands_analyst.rs)
  (Importance High, 8 points) — closed by E019-T02 (`today_totals.rs` + ADR 014).
- [x] **`TrayController` does more than "execute signals"** (contradicts `CLAUDE.md`'s
  routing claim) — it computes `PolicyInput`, standing-lap math, tooltip text, and WS
  broadcasting; `tray_signal_exec.rs` is the real pure executor. Either retitle the
  module boundary in `CLAUDE.md`/`.arch/ARCHITECTURE.md` or move the computation out of
  `tray_controller.rs`. [src-tauri/src/tray_controller.rs:53-186](../src-tauri/src/tray_controller.rs)
  (Importance Medium, 3 points) — closed by E019-T08 (corrected the `CLAUDE.md` claim) and E019-T07 (deduped remote-display-state derivation out of `tray_controller.rs`).
- [x] **`google_fit.rs` (473 lines) and `google_fit_service.rs` (328 lines) breach the
  250-line file cap** — unlike every other oversized cluster in this codebase, which
  already follows the `<module>_tests.rs`/`<module>_helpers.rs` split convention
  consistently. [src-tauri/src/google_fit.rs](../src-tauri/src/google_fit.rs),
  [src-tauri/src/google_fit_service.rs](../src-tauri/src/google_fit_service.rs)
  (Importance Medium, 3 points) — closed by E019-T05 (split into ≤250-line siblings).
- [x] **`EventLogger::new` panics on log-dir creation failure**; sibling `SnapshotLogger`
  only warns on the same failure class — inconsistent startup-failure philosophy for two
  structurally identical loggers, and `EventLogger::new` runs early in
  `perform_app_setup`, so a permissions issue there crashes the whole app.
  [src-tauri/src/event_logger.rs:25-35](../src-tauri/src/event_logger.rs) vs.
  [src-tauri/src/snapshot_logger.rs:39-43](../src-tauri/src/snapshot_logger.rs)
  (Importance Medium, 2 points) — closed by E019-T04 (warns instead of panicking, regression test).
- [x] **8 `desk:*` Tauri event names are stringly typed with no shared constants module**
  — each `emit`/`listen` site retypes the literal; a typo fails silently (listener never
  fires). `ws_broadcaster.rs`'s `DisplayEvent` enum re-encodes 5 of the 8 with a better,
  serde-tagged representation that isn't shared with the Tauri-event side. See the
  Event/IPC inventory in the linked report for the full 8×N call-site table.
  (Importance Low, 3 points) — closed by E019-T03 (`desk_events.rs`/`src/events.ts` named constants).
- [ ] **`cargo-clippy` is not installed for this toolchain** (`stable-x86_64-pc-windows-msvc`)
  — the architecture review could not get a clippy warning count; `rustup component add
  clippy` was intentionally not run by the reviewing agent (out of scope to modify the
  toolchain). (Importance Low, 1 point)
- [ ] **Popup's main timer/progress bar reads the wrong field and hard-resets on every
  break, contradicting ADR-008's proportional break credit** — `OneBarTimer.tsx:28`
  binds its big number and progress-bar fill to `current_session_secs`, which is
  unconditionally zeroed on every Standing/Walking/Away→Sitting transition
  (`src-tauri/src/session_breaks.rs:34,63,85`) with no credit ever applied to it. The
  correctly-credited counter (`sitting_seconds`/`limitUsedSecs`) already exists and is
  used correctly by the overlay bar and notification engine
  (`src-tauri/src/tray_controller.rs:61-62,124-128`) — only the popup widget is wired to
  the wrong one. `useDesk.ts:106,170` even names its state `sittingSeconds` while it
  actually holds `current_session_secs`. A test
  (`src-tauri/src/session_tests_timers_live.rs:160-164`) explicitly asserts the reset
  behavior as correct inside a scenario titled around partial credit — the wrong
  behavior is locked in by the test suite, not merely untested. Full trace, log
  evidence, and migration plan: [.plan/reports/_review-2026-09-06/engine.md](reports/_review-2026-09-06/engine.md).
  (Importance High, 3 points)

## Full review 2026-09-06 — frontend and release findings

Synthesis and roadmap: [reports/2026-09-06-pelny-przeglad-architektury-i-release.md](reports/2026-09-06-pelny-przeglad-architektury-i-release.md).
Engine and backend items were filed above by their reviewers. Frontend and release items:

- [ ] **`pnpm lint` calls an eslint that is not installed** — no devDependency, no config ([package.json:23](../package.json)). Found by frontend review. (High, 2) → E018
- [ ] **Coverage gate 80/80/75 never invoked by any script; real coverage 76.5/68.1/66.0** — [vite.config.ts:22-31](../vite.config.ts). (High, 2) → E018
- [ ] **`useDesk.ts` and `useRemoteDesk.ts` duplicate one state machine (~500 lines); `useDesk` at 0.78 % coverage** — [src/hooks/useDesk.ts:37-247](../src/hooks/useDesk.ts), [src/hooks/useRemoteDesk.ts:43-252](../src/hooks/useRemoteDesk.ts). (High, 8) → E018
- [ ] **Rust DTOs hand-copied into TS with no codegen or drift test** — [src/types.ts](../src/types.ts), `SettingsTypes.ts`. (Medium, 5) → E015 adds the drift test, E018 the codegen
- [ ] **5 dead pre-OneBar components still built and tested** (`AppProgressBar`, `HeightRail`, `SessionProgress`, `TodayStats`, `TransitionBanner`) plus dead `src/overlay/main.tsx` + `overlay.html` entry. (Medium, 3) → E018
- [ ] **UX-FLOW.md missing Steps/Google Fit widget and timeline→Analyst click** — [.arch/UX-FLOW.md](../.arch/UX-FLOW.md). (Medium, 2) → E018
- [ ] **6 local `formatXxx` helpers duplicate `src/utils/format.ts`**. (Low, 1) → E018
- [ ] **"Smart Desk" branding in share-card copy** — [src/components/ShareStats.tsx](../src/components/ShareStats.tsx). (Low, 1) → E017
- [ ] **No `justfile`**; `.claude/rules/overlay.md` documents `tauri-dev.sh --force` that was replaced by `tauri-dev.ps1 -Force`. (Low, 1) → E018
- [ ] **User docs describe zntlDesk: name, `AppData\Local\zntlDesk`, repo `zentala/zntl-tray`** — `docs/README.md`, `USER_INSTALL.md`, `USER_SUPPORT.md`, `PRIVACY.md`. (High, 3) → E017
- [ ] **`docs/USER_UPDATES.md` documents a 24h auto-updater that is not wired in code** (no updater plugin in `src-tauri/tauri.conf.json`). (High, 2) → E017
- [ ] **No LICENSE file, no `license` field, despite open-core (ADR 005)**. (High, 2) → E017, decision D3
- [ ] **`PRIVACY.md` claims no network egress; Google Fit sends to Google** — [src-tauri/src/google_fit.rs](../src-tauri/src/google_fit.rs). (High, 2) → E017
- [ ] **`.github/workflows/test.yml` targets defunct `apps/desk/` path, runs on every push**. (High, 3) → E017
- [ ] **`firmware/` ships a bare `.ino`, no README, no flashing steps, no cable warning** (see CLAUDE.md Hardware). (High, 3) → E017
- [ ] **`coverage/` and `test-performance-report/` are tracked in git** (`git ls-files coverage | wc -l`). (Medium, 1) → E016
- [ ] **Cargo package metadata still says "zntl Desk"** — [src-tauri/Cargo.toml](../src-tauri/Cargo.toml). (Medium, 1) → E017
- [ ] **`installer.md` size target 60-70 MB vs observed ~4-6 MB; `.perf-baseline.json`/`.build-sizes.json` stale**. (Low, 1) → E017
- [x] **Four backlog-like files (root `BACKLOG.md`, `TASKS.md`, `ORCHESTRATOR.md`, this file); E010 human-task count contradicts (3 vs 10)** — consolidated 2026-09-06 by E016-T02: the three root files are deleted and every section unique to root `BACKLOG.md` is merged verbatim below under "Merged from root `BACKLOG.md`". E010's count is E016-T01's job ([STATE.md](STATE.md)). (High, 2) → E016
- [ ] **E011 close-out ceremony not done; E003-T07 not marked superseded by E013** — [epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md:44-51](epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md). (High, 3) → E016

---

## Follow-ons filed by E017

- [ ] **Build the auto-updater that `USER_UPDATES.md` no longer promises** —
  E017-T02 rewrote [docs/USER_UPDATES.md:1](../docs/USER_UPDATES.md) to describe the
  real manual path (download the installer from GitHub Releases, run it over the
  existing install) because no `tauri-plugin-updater` is wired anywhere:
  [src-tauri/tauri.conf.json:1](../src-tauri/tauri.conf.json),
  [src-tauri/Cargo.toml:1](../src-tauri/Cargo.toml) and
  [package.json:1](../package.json) all lack it. Implementing it means adding
  `tauri-plugin-updater`, an `updater` block with a public key and the GitHub
  Releases endpoint, a signed `latest.json` per release in
  [.github/workflows/release-baseline.yml:1](../.github/workflows/release-baseline.yml),
  and an in-app update prompt. **Blocked on code signing** (E013 Wave 2, no
  provider chosen) — an unsigned auto-update would install unverified binaries.
  Rewrite `USER_UPDATES.md` back once it ships. (Importance: Medium, Points: 8)

## Merged from root `BACKLOG.md` (2026-09-06, E016-T02)

Root `BACKLOG.md`, `TASKS.md` and `ORCHESTRATOR.md` were deleted; this file is the
single backlog. Every section below is copied byte-for-byte from root `BACKLOG.md`,
original headings kept so provenance is traceable. Sections whose content already
existed here (Tauri Plugins, App Icon, Alert System, Max Continuous Computer Time
Alert, Activity Tracking, Logging & Observability, Timeline Full Window, KPI Time
Range Selector, Future Features, Removed) were not copied twice.

Note: relative links inside the copied sections were written from the repo root, so
they read as `.plan/...` rather than as paths relative to this file.

## Dokumentacja

- **Overlay Progress Bar Architecture Document** — pełny opis systemu paska na górze ekranu

---

## Sensor & Readings

- **Height stabilization algorithms** — moving average, 1cm rounding, trend locking (scheduled: T037)
- **Sensor diagnostics panel** — show raw vs smoothed readings, debounce state, threshold visualization
  - Helps debug standing detection issues like T036

---

## Persistent KPI Strip

Always-visible KPI strip on every view — fixed position, same place. Shows 4 daily ergonomics metrics with color coding.

### KPIs (all RELATIVE, not raw counts):

| KPI | Format | Green | Yellow | Red |
|-----|--------|-------|--------|-----|
| **Standing %** | `12%` | ≥15% | 10-15% | <10% |
| **Position changes/h** | `1.2/h` | ≥1.0 | 0.5-1.0 | <0.5 |
| **Hourly breaks** | `5/7h` | all hours covered | 1-2 missed | ≥3 missed |
| **Longest session** | `47m` | <45m | 45-75m | >75m |

- Position change count (raw number) = secondary/small, shown alongside changes/h
- No raw counts without context — everything relative to time worked

### Design rules:

- Widoczne na KAŻDYM widoku, stałe miejsce (np. dolny pasek, header strip)
- Kompaktowe — 4 liczby + kolory, zero tekstu opisowego
- Kolor komunikuje stan (zielony/żółty/czerwony) — nie trzeba czytać

### Position change definition:

- 5+ minut stania LUB 5+ minut away = 1 zmiana pozycji
- Standing i away są wymienne jako "przerwa"
- Cel: ~1 zmiana/godzinę

### Hourly break definition:

- Binarne sprawdzenie per godzina: "Czy była ≥5 min przerwa od ekranu?"
- KPI = ile godzin miało przerwę / ile godzin pracowaliśmy
- Jedna 30-min przerwa w jednej godzinie = 1/1 dla tej godziny, NIE nadrabia za inne
- Away = odpoczynek dla oczu (nawet stojąc, oczy pracują)

---

## Coach Messages → Progress Bar

Zamienić tekstowe coach messages ("Half limit used", "On track", etc.) w OneBarCoach na wizualne elementy. Istniejący 4px progress bar w OneBarTimer działa dobrze — coach message pod nim jest niepotrzebny/nieczytelny.

- Progress bar już istnieje i jest dynamiczny (OneBarTimer.tsx) — reużyć/rozszerzyć
- Usunąć lub uprościć coach text messages — bar sam komunikuje stan
- Ewentualnie: coach message jako tooltip, nie stały tekst

---

## KPI "Session" Label — Confusing, Rename Required

- KPI badge labeled "Session" actually means "longest continuous screen time without 5+ min away"
- User interprets "Session" as sitting session or app uptime — both wrong
- **Rename to**: "Screen" or "Screen time" or "At desk" — something that communicates "continuous time at computer"
- **Possible bug**: Away detection doesn't work (Away state unreachable) → screen time counter NEVER resets → always red after 75 min. Fix Away state first (see "State Machine Redesign" below), then verify this KPI resets properly.
- **Not covered by tests**: no test verifying that 5+ min away resets the longest_session counter in practice (only unit tests on counter logic, but Away is never triggered by sensor)

---

## Notification Centralization (pre-requisite for alert escalation)

> Before implementing full alert escalation flow (T017 Stages 3-5), centralize all notification sources.

- **NotificationService** — single Rust module that routes ALL notifications through one backend
  - Currently two parallel systems: native toasts (`tauri-plugin-notification`) scattered across `serial_periodic.rs` + custom WinAPI popup (`alert_popup_window.rs`)
  - 6 toast conditions spread across `serial_periodic.rs` and `session_breaks.rs` with no central coordination
  - `notification_backend` field exists in `config.rs` (`"toast" | "popup" | "both"`) but is **dead code** — never read
  - **Goal:** one `NotificationService` that:
    1. Collects all notification intents (inactivity, posture balance, praise, alerts, escalation)
    2. Routes through selected backend: native toast OR custom popup OR both
    3. Manages once-per-day / once-per-session gates centrally (not scattered flags)
    4. Makes T018 (A/B testing) trivial — just flip `notification_backend` in config
  - **Blocks:** T017 (Stages 3-5), T018 (A/B testing)
  - **Custom popup redesign** — migrate from raw WinAPI GDI (Win32 2003 look) to Tauri WebviewWindow (HTML/CSS, dark theme, acrylic blur). Already in backlog above under "Alert System — Future".

---

## Away Detection — Runtime Diagnostic (2026-03-25)

> State machine logic FIXED (commit c305f08): `!active → Away` regardless of desk height.
> Unit tests pass (340/340), including Standing→Away regression test.
> BUT: user reports Standing + inactive does NOT transition to Away at runtime.

**Status:** Diagnostic logging added to `serial_periodic.rs`. Next occurrence, check:
- Event log: `STATE Standing→Away idle=XXs` — if missing, `is_active()` never returned false
- Debug log: `Standing idle diagnostic: idle=XXs` — shows Windows API idle value

**Open questions:**
1. Does something on Windows reset `GetLastInputInfo` when overlay bar is in standing (gold) mode?
2. Is there a race condition where UI snapshot reads old state before transition?
3. Could the floating window (Tauri webview) or overlay (WinAPI) generate synthetic input?

**Next steps:**
- Run app with `RUST_LOG=desk_lib=debug`, reproduce, check logs
- If `is_active()` always returns true during standing: investigate WinAPI interactions
- Add integration test: `inject_reading(1200, false)` from frontend → verify UI shows Away

---

## DB Persistence in Dev Mode (2026-03-25)

> Database keeps getting reset during development. User needs persistent data even in dev mode.

**Current behavior:**
- DB path: `{AppData}/io.zntl.desk/desk.db`
- Lazy-initialized via `ensure_initialized()` — only opens when first IPC command fires
- `load_today_totals()` overwrites in-memory counters with DB values on init
- In dev mode (`pnpm tauri:dev`), rebuilds may change identifier → different `app_data_dir` → lost DB

**Proposed fixes:**
1. **Eager DB init** — open DB in `setup()`, not lazy on first IPC call. Prevents lost sessions before frontend loads.
2. **Log DB path at startup** — print `info!("DB: {}", db_path)` so user can verify path stays consistent.
3. **DB backup on startup** — copy `desk.db` → `desk.db.bak` before opening, protect against corruption.
4. **Pin `identifier` in dev mode** — ensure `tauri.conf.json` identifier doesn't change between builds.

**Priority:** HIGH — data loss during development is unacceptable when testing break patterns

---

## Remote Display — Phone as Desk Dashboard

**Epic E009** — full spec at `.plan/epics/E009-2026-03-24-remote-display/PLAN.md`

Phase 1 (E009): Web kiosk — PC serves React+WS to phone browser. 7 tasks, ~16h.
Phase 2 (future): Tauri Mobile native Android app.
Phase 3 (future): Standalone — sensor communicates wirelessly (BLE/WiFi) with phone, no PC needed.

See `.plan/vision/2026-03-15-desk-app-vision.md` → "Remote Display" section for full vision.

---

## Motivation Analytics & Adaptive Coaching (ongoing process)

- **Progressive break credit curve** — replace step function (<5m=0, 5-9m=-20m, ≥10m=reset) with smooth curve where every minute of break gives increasing credit. Short breaks (1-4 min) should give *some* reward. See memory: `project_progressive_break_credit.md`.
- **`/ergo-review` skill** — agent reads minute snapshots + event log, analyzes sitting/standing patterns, discusses UX effectiveness with user, proposes parameter tweaks. Created as `.claude/skills/ergo-review/`.
- **Notification outcome tracking** — log whether a notification led to action within 5 min (standing/away). Currently we fire notifications but don't track if they worked. Needed for measuring motivation effectiveness.
- **Configurable break credit parameters** — move hardcoded `BREAK_SHORT_SECS`, `BREAK_LONG_SECS`, `SHORT_BREAK_CREDIT_SECS` to `AppConfig` so they can be tuned without code changes.
- **Adaptive motivation engine (long-term)** — A/B test different notification strategies, learn what works for this user, optimize automatically. Needs: notification outcomes, sufficient history, parameter framework.

---

## Fullscreen Debug Dashboard (2026-03-26)

Dedicated fullscreen window for debugging and verifying app behavior. NOT the popup — a separate Tauri WebviewWindow.

### Must show:
1. **Timeline visualization** — full-day timeline bar (like popup but bigger), color-coded by state (green=standing, red=sitting, gray=away, gold=break credit applied)
2. **Event log overlay** — event log entries mapped to timeline positions. Each STATE/CREDIT/ALERT/NOTIF event visible as markers on the timeline
3. **Counter dashboard** — all live counters with their data source explained:
   - `sitting_seconds` — "Total sitting today (committed + live elapsed)"
   - `standing_seconds` — "Total standing today (committed + live bout)"
   - `break_seconds` — "Current break duration (from break_started)"
   - `position_changes` — "Sit↔Stand transitions only"
   - `daily_score` — "Points formula: -0.5/min sit, +1.0/min stand, +5.0/lap"
   - `continuous_computer_secs` — "Time at keyboard without 5min break"
4. **Raw state dump** — all SessionState fields, updated live (1s polling)
5. **DB session history** — list of CompletedSession rows from SQLite for today, with started_at, ended_at, state, duration_secs

### Purpose:
When user sees timeline not matching reality, they can open this view and immediately compare: "timeline shows X, but event log says Y, and DB has Z." No more guessing.

### Implementation:
- New Tauri WebviewWindow (like welcome popup pattern)
- Route: `/#/debug-dashboard`
- Read-only — no mutations
- IPC: `get_dashboard_state` (existing) + new `get_event_log` + `get_db_sessions_today`

---

## 🔴 Collect from User (zentala) — blocking launch

These items require human action. Everything else is blocked until these are done.

- [ ] **Real usage screenshots** — app popup showing real KPIs (not mock data)
- [ ] **Sensor photo** — VL53L1X mounted under real desk, visible cable
- [ ] **Desk setup photo** — full desk with sensor visible, monitor, keyboard
- [ ] **15-second hero GIF** — screen recording: overlay bar going green→red, popup open
- [ ] **2-3 marketing videos** — (1) "How I track sitting" 2-3min, (2) short-form 1min, (3) maker build 5-10min
- [ ] **Real usage stats** — export 30 days of data: standing %, position changes/day, longest session. Replace all [PLACEHOLDER] markers in posts, emails, blog, social proof
- [ ] **OG cover image** — 1200×630px for social sharing (Figma/Canva: app screenshot + sensor + headline)
- [ ] **Stripe account** — create Stripe account, generate 3 Payment Links (Basic €49, Pro €79, Founder's €149), replace placeholder URLs in Pricing.tsx
- [ ] **Plausible account** — sign up at plausible.io, configure desk.zentala.io domain
- [ ] **Google Search Console** — verify desk.zentala.io, submit sitemap

---

## Landing Page — Post-Launch

- **Cloudflare Worker for waitlist** — real endpoint to store emails (CF Worker + D1). Currently forms submit to placeholder URL
- **Live pre-order counter** — Stripe webhook → CF Worker → KV → landing page fetches live count. Currently static JSON
- **Share as image** — html2canvas or Rust screenshot API for better viral sharing (currently text-only)
- **Telemetry CF Worker** — endpoint at telemetry.desk.zentala.io/api/report to receive opt-in daily aggregates
- **Blog RSS feed** — Astro has built-in plugin, helps SEO and HN/Reddit readers

---

## i18n — Landing Page Translations (post-validation)

Priority order based on global market size and standing desk adoption:
1. 🇬🇧 English (done — primary)
2. 🇵🇱 Polish (personal — zentala is Polish)
3. 🇨🇳 Chinese (Simplified) — huge market, low English proficiency
4. 🇧🇷🇵🇹 Portuguese — Brazil + Portugal
5. 🇪🇸 Spanish — Latin America + Spain
6. 🇩🇪 German — strong standing desk market
7. 🇫🇷 French — France + francophone Africa
8. 🇰🇷 Korean — tech-savvy market, low English
9. 🇯🇵 Japanese — similar to Korean, secondary priority
10. 🇸🇪🇳🇴🇩🇰 Scandinavian — high desk adoption but speak English well (lowest priority)

**Not doing**: Dutch, Finnish — too small, English proficiency too high.
**When**: after 100 pre-orders validated demand. i18n is post-product-market-fit.

---

## ESLint baseline downgrades (E018-T04, 2026-09-06)

E018-T04 installed ESLint 10 + `typescript-eslint` +
`eslint-plugin-react-hooks` v7 ([`eslint.config.mjs`](../eslint.config.mjs)).
Six rules are pinned to `"warn"` in the config's "baseline downgrades" block
because they flag pre-existing code in files that task could not touch. They
are warnings, not disables — `pnpm lint` still prints all 27 of them. Tighten
each back to `"error"` once its sites are fixed.

- [ ] **`react-hooks/rules-of-hooks` → error** — 8 sites, 2 real patterns:
  `src/App.tsx:53` (hooks called after `import.meta.env.DEV` hash-route early
  returns at `src/App.tsx:31-52`) and `src/hooks/useDeskAuto.ts:21,23`
  (`useDesk()` vs `useRemoteDesk()` chosen by a module-level `isTauri`
  constant). The `useDeskAuto` case is stable in practice but the App.tsx one
  is not — the hash can change without a reload. Fix: hoist the mockup routes
  above `App`, and make `useDeskAuto` call both hooks or split the component.
  (Importance: Medium, Points: 3)
- [ ] **`react-hooks/set-state-in-effect` → error** — 8 sites:
  `src/analyst/charts/TimelineDetail.tsx:118`,
  `src/analyst/hooks/useDataCatalog.ts:43`,
  `src/analyst/hooks/useRangeQuery.ts:54`,
  `src/analyst/hooks/useTimelineNav.ts:83`,
  `src/components/ConnectionOverlay.tsx:35`,
  `src/components/StepsWidget.tsx:108`,
  `src/components/settings/ProfileSelector.tsx:69`, `src/hooks/useTimer.ts:21`.
  React-Compiler-era rule; each needs a per-site judgement call, not a sweep.
  (Importance: Low, Points: 5)
- [ ] **`react-hooks/immutability` → error** — `src/components/AutostartToggle.tsx:15`,
  `src/widgets/one-bar/OneBarTimeline.tsx:129` (running offset accumulated by
  reassignment inside `.map()`). (Importance: Low, Points: 2)
- [ ] **`react-hooks/purity` → error** — `src/components/StepsWidget.tsx:167`
  calls `Date.now()` during render to compute staleness. (Importance: Low, Points: 2)
- [ ] **`no-useless-assignment` → error** — `src/analyst/CatalogTab.tsx:111`.
  (Importance: Low, Points: 1)
- [ ] **`prefer-const` → error** — `src/hooks/useTimerAnimations.test.ts:24`.
  Auto-fixable with `npx eslint src/ --fix`. (Importance: Low, Points: 1)

Also left as warnings by the plugins' own defaults (not downgraded here):
`react-hooks/exhaustive-deps` (`src/analyst/charts/TimelineDetail.tsx:141`,
`src/hooks/useExponentialPoll.ts:92`) and `react-refresh/only-export-components`
(3 sites). `src/hooks/useExponentialPoll.ts:55` carries a now-unused
`eslint-disable` directive that can be deleted.
