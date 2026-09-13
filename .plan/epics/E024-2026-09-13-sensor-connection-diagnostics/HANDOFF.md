---
formatVersion: 1
type: handoff
status: todo
---

# E024 Handoff — sensor connection diagnostics

## TLDR

Read only this file and [`PLAN.md`](PLAN.md). 18 points → run through AO
(`ao plan`, then `ao run`; skill `ao`). Strictly sequential: the seam first,
then the emulator IPC, then each diagnostic with its own tests, then docs and
verify. Bump version to **0.8.0** before T01 if E023 already took 0.7.0,
otherwise 0.7.0 (`.claude/rules/versioning.md`).

## Mental model

- Scan loop: `scan_and_connect` (`src-tauri/src/serial.rs:144-210`) — a
  thread that every 10 s lists ports, probes each with `probe_port`
  (`serial_parser.rs:55`), and on success runs `reader_loop`
  (`serial.rs:70-132`) until the port dies, then logs `DEVICE lost`.
- Events: `desk:device-connected` (`serial.rs:176`), `desk:device-lost`
  (`:105`), `desk:device-missing` (`:204`, empty payload today). Rust listener
  in `setup_helpers.rs:185`; TS consumers `src/hooks/deskReducer.ts`,
  `useDesk.ts`, `useRemoteDesk.ts`. `communication_policy.rs:61` already greys
  the tray when `sensor_connected` is false — do not duplicate that; the
  unstable warning is a tooltip line on top of it.
- `EventLogger::log` (`event_logger.rs:45`) writes `HH:MM:SS <text>`; tests
  need a capturing sink — give the loop a `&dyn Fn(&str)` or a small
  `EventSink` trait rather than a real file.
- Gating pattern to copy: `inject_reading` is `#[cfg(any(test,
  debug_assertions))]` (`commands.rs:217`) and registered only in the debug
  handler list (`lib.rs:220` vs `:266`).
- TS emulator today (`tests/emulator/DeskDeviceEmulator.ts`,
  `scenarios.ts`) only formats strings. T02 gives it `SerialScenario` names
  that map 1:1 to the Rust emulator scripts.
- Do not touch: `session_*.rs`, `reader_loop` parsing logic, firmware,
  `relay_*` (E023 runs in parallel there).

## Tasks

- [ ] **T01** (5, ts-dev, Rust) — `serial_backend.rs` (`SerialBackend`,
  `LineSource`, `ProbeOutcome`, `SystemSerialBackend` = moved code),
  `serial_emulator.rs` (`EmulatorSerialBackend` with scripts: `happy`,
  `no_ports`, `no_match`, `enumeration_failed`, `flapping`), scan loop takes
  `Arc<dyn SerialBackend>` + clock + event sink; one iteration extracted as
  `scan_once(...)` so tests step it without sleeping. Tests `e024_seam_*`:
  happy script → `DEVICE connected COM3` then readings reach the session
  (reuse existing `serial_periodic` test helpers). No diagnostics yet.
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_`.
  [task](tasks/T01-serial-backend-seam-and-emulator.md)
- [ ] **T02** (3, ts-dev) — `emulate_serial_scenario(name)` debug-only IPC
  that restarts the scan loop on `EmulatorSerialBackend`; TS
  `SerialScenario` + `emulateSerial()` helper in `tests/emulator/`; rewrite
  the hollow `desk:device-connected` test in
  `tests/integration/device-reconnect.test.ts:40-56` to use scenario `happy`
  and assert the port name. `cfg` test proves the command is absent from the
  release handler list.
  Verify: `cargo test … -- e024_`; integration part in "Outside AO".
  [task](tasks/T02-emulator-ipc-and-ts-scenarios.md)
- [ ] **T03** (3, ts-dev) — F1: `ScanOutcome`, change-only logging in
  `serial_diagnostics.rs`, `desk:device-missing` payload (Rust + TS types +
  reducer), Debug tab row "scan". Tests: unit transitions; Rust emulator
  integration for `no_ports` / `no_match` / `enumeration_failed` (criteria
  1-3); `tests/integration/device-diagnostics.test.ts` cases for the three
  reasons. Verify: `cargo test … -- e024_`,
  `npx vitest run --config vite.config.ts src/hooks src/components/settings`.
  [task](tasks/T03-no-port-vs-no-match.md)
- [ ] **T04** (5, ts-dev) — F2: flap detector in `serial_diagnostics.rs`
  (D3 constants), `desk:device-unstable` / `desk:device-stable`, tray tooltip
  line, Debug tab row, `sensor-unstable` mockup scenario. Tests: unit (3 short
  / 2 short / spread over 61 s / 5-min clears), Rust emulator `flapping`
  script, TS integration `flapping` case (criterion 4). Verify: same as T03.
  [task](tasks/T04-flapping-detection.md)
- [ ] **T05** (1, main) — docs: `CLAUDE.md` Hardware diagnosis order cites the
  new log lines; `.claude/rules/logging.md` event list; `.arch/UX-FLOW.md`
  tray tooltip; `.arch/ARCHITECTURE.md` serial section; D1-D5 into
  `.plan/decisions.jsonl`; `PROJECT.xml` new files.
  Verify: `grep` each new log line in `CLAUDE.md` and `logging.md` (0 hits = fail).
  [task](tasks/T05-docs.md)
- [ ] **T06** (1, verify) — `just check`; running debug app +
  `pnpm test:integration` (device-reconnect + device-diagnostics green, test
  count > 0); `cargo build --release` has no `emulate_serial_scenario`
  symbol; per-criterion verdict. Set E024 `done` in `epics/INDEX.md`.
  [task](tasks/T06-verify.md)

## Waves

Fala 1: T01 → Fala 2: T02 → Fala 3: T03 → Fala 4: T04 → Fala 5: T05 → Fala 6: T06.
Sequential because T01-T04 all edit `serial.rs` / `serial_diagnostics.rs`
and each test layer builds on the previous seam. E023 may run at the same time:
no shared files.

## AO

```yaml
project: MoveUp
epic: E024
base_ref: main
tasks:
  - id: E024-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/serial.rs", "src-tauri/src/serial_parser.rs", "creates:src-tauri/src/serial_backend.rs", "creates:src-tauri/src/serial_emulator.rs", "creates:src-tauri/src/serial_scan_emulator_tests.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/serial_periodic_tests.rs", "src-tauri/src/commands.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_"
    budget_minutes: 90
  - id: E024-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E024-T01"]
    write_set: ["src-tauri/src/commands.rs", "creates:src-tauri/src/commands_serial_emulator.rs", "src-tauri/src/serial_emulator.rs", "src-tauri/src/serial_scan_emulator_tests.rs", "src-tauri/src/lib.rs", "tests/emulator/**", "tests/integration/device-reconnect.test.ts"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_"
    budget_minutes: 60
  - id: E024-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E024-T02"]
    write_set: ["src-tauri/src/serial.rs", "creates:src-tauri/src/serial_diagnostics.rs", "creates:src-tauri/src/serial_diagnostics_tests.rs", "src-tauri/src/serial_scan_emulator_tests.rs", "src-tauri/src/desk_events.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/lib.rs", "src/events.ts", "src/types.ts", "src/hooks/**", "src/components/settings/DebugSection.tsx", "src/components/settings/DebugSection.test.tsx", "creates:tests/integration/device-diagnostics.test.ts", "tests/emulator/**"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_ && npx vitest run --config vite.config.ts src/hooks src/components/settings"
    budget_minutes: 90
  - id: E024-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E024-T03"]
    write_set: ["src-tauri/src/serial.rs", "src-tauri/src/serial_diagnostics.rs", "src-tauri/src/serial_diagnostics_tests.rs", "src-tauri/src/serial_scan_emulator_tests.rs", "src-tauri/src/serial_emulator.rs", "src-tauri/src/desk_events.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/setup_helpers.rs", "src/events.ts", "src/types.ts", "src/hooks/**", "src/components/settings/DebugSection.tsx", "src/components/settings/DebugSection.test.tsx", "src/test/scenarios.ts", "tests/integration/device-diagnostics.test.ts", "tests/emulator/**"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_ && npx vitest run --config vite.config.ts src/hooks src/components/settings"
    budget_minutes: 90
  - id: E024-T05
    repo: MoveUp
    executor: main
    depends_on: ["E024-T04"]
    write_set: ["CLAUDE.md", ".claude/rules/logging.md", ".arch/UX-FLOW.md", ".arch/ARCHITECTURE.md", ".plan/decisions.jsonl", "PROJECT.xml"]
    verification: "grep -q 'DEVICE scan ports=' CLAUDE.md && grep -q 'DEVICE unstable' .claude/rules/logging.md"
    budget_minutes: 30
```

## Outside AO

- T02-T04 TS integration tests need the running debug app: start it through
  PM3 with `--owner worktree:<path>` (not a bare `pnpm tauri:dev &`), then
  `pnpm test:integration`. Runs in T06 on merged `main`.
- T04 mockup: `sensor-unstable` scenario on `/#/mockup`; screenshot pair via
  skill `before-after` goes into the epic `reports/`.
- Version bump + tag: main loop, before `ao run`.

## Done means

All six acceptance criteria in PLAN.md hold, `pnpm test:integration` ran
against the emulator with a non-zero test count, every new test was seen
failing on reverted code, INDEX row `done`, `.plan/HISTORY.md` entry written.
