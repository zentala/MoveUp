---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 18
agent: ts-dev
wave: 6
parallel: [E023]
depends-on: []
blocked-by: ""
---

# E024 — Sensor connection diagnostics

Source: two `.plan/BACKLOG.md` §"Sensor & Readings" findings (2026-09-05 and
2026-09-06, both from the "sensor plugged in, app does not see it" diagnosis),
moved here 2026-09-13. Hardware background: `CLAUDE.md` → Hardware → "Cable
sensitivity". Handoff: [`HANDOFF.md`](HANDOFF.md). Board deck (Polish):
[`PRES.md`](PRES.md).

## TLDR

When the sensor is not working, the app shows one state — "no sensor" — for
three different causes: Windows sees no COM port at all (cable/power), ports
exist but none answers as the sensor (firmware/busy port), or the device
connects and drops within a second in a loop (bad cable, brown-out). The user
cannot tell which, and the fix for each is different. This epic makes the
scan loop testable through a serial backend seam, adds an emulator backend,
and then teaches the app to name each cause in `events.log`, the Debug tab and
the tray tooltip. Paweł's condition: both diagnostics are covered by
**integration tests on the emulator**. 6 tasks, 18 points → AO (>13).
Direction: MoveUp is a hardware product (ADR 004 dev kit) — a kit whose
failure mode is silent is a support ticket per buyer.

## Problem

- **F1 — no port vs no match are indistinguishable.**
  `src-tauri/src/serial.rs:164-205`: `available_ports()` is read with
  `unwrap_or_default()` (an enumeration error looks like zero ports), the
  "none found" branch logs only via `info!` to stderr, never to `events.log`,
  and emits `desk:device-missing` with an empty payload. On 2026-09-05 the real
  cause was zero COM ports in the system; nothing in the app said so.
- **F2 — connection flapping is silent.** `serial.rs:172-201` treats every
  connect/lost cycle as normal. On 2026-09-06 `events.log` showed
  `DEVICE connected COM3` / `DEVICE lost` pairs in the same second, repeatedly,
  with Windows plug/unplug chimes — a textbook cable/power fault — and the app
  never warned.
- **Why it has no test today.** The loop calls `serialport` and
  `probe_port` (`serial_parser.rs:27,55`) directly and needs an `AppHandle`;
  `tests/integration/device-reconnect.test.ts` only checks that commands do
  not throw, and `tests/emulator/DeskDeviceEmulator.ts` formats strings that
  nothing consumes.

## Decisions and ADRs

- [ADR 015](../../../.arch/ADR/015-pure-ergo-engine.md): the caller owns the
  clock. The new diagnostics module takes `now` as an argument, same as the
  engine; it never reads the clock itself. Diagnostics do not touch
  `session_*.rs`.
- CLAUDE.md "Cisza nigdy nie znaczy sukcesu": assert the count of scanned
  ports; an enumeration error is its own state, not zero.
- New decisions (record in `.plan/decisions.jsonl` in T05):
  - **D1** Serial I/O goes behind `trait SerialBackend { list_ports() ->
    Result<Vec<String>, String>; probe(port) -> ProbeOutcome; open(port) ->
    Result<Box<dyn LineSource>, String> }`. Production = `SystemSerialBackend`
    (today's code moved, not rewritten); tests and debug builds may use
    `EmulatorSerialBackend`.
  - **D2** Scan outcome is an enum `ScanOutcome { EnumerationFailed(err),
    NoPorts, NoMatch { ports }, Found { port } }`. Logged to `events.log` only
    when the outcome **changes** (the loop rescans every 10 s — one line per
    change, not 8 640 a day).
  - **D3** Flapping = **3 connections shorter than 5 s within 60 s**. Warning
    clears after a connection lasts 5 min. Constants in one place, not in the
    ergonomic profile (hardware fact, not a user preference).
  - **D4** `desk:device-missing` gains a payload `{ reason: "enumeration_failed"
    | "no_ports" | "no_match", ports: string[] }`; new event
    `desk:device-unstable { short_connections, window_secs }` and
    `desk:device-stable`. TS consumers (`deskReducer.ts`, `useDesk.ts`,
    `useRemoteDesk.ts`) updated in the same task as the Rust emitter.
  - **D5** Emulator-backed IPC command `emulate_serial_scenario` exists only
    under `#[cfg(any(test, debug_assertions))]`, like `inject_reading`
    (`commands.rs:217`).

Architecture impact: new modules `serial_backend.rs` (seam),
`serial_emulator.rs` (test/debug backend), `serial_diagnostics.rs` (pure state
machine); changed data flow: scan loop → diagnostics → `events.log` + events +
tray. `.arch/ARCHITECTURE.md` serial section updated in T05. No ADR: a test
seam inside one module is not a system-shape change; D1-D5 go to
`decisions.jsonl`.

## Approaches considered

| | A — minimum: log lines + counters inline in `serial.rs`, Rust unit tests on a helper | B — target: backend seam + emulator + pure diagnostics module, integration tests drive the real scan loop | C — virtual COM ports (com0com) driven by the TS emulator |
|---|---|---|---|
| Effort / Risk | S / M | M / L | L / H |
| Plus | Fewest files | The real loop is tested end to end; emulator reusable for every future serial bug; loop stops needing hardware to test | Tests the real `serialport` crate |
| Minus | Does **not** meet Paweł's condition — no integration test on an emulator; loop stays untestable | More files (3 new modules) | Needs a signed kernel driver installed on every dev machine and CI; cannot emulate "zero ports" or brown-out timing |
| Reuses | — | `probe_port`, `parse_distance`, `EventLogger`, `inject_reading` gating pattern, `tests/emulator/` | `DeskDeviceEmulator.ts` |

**Recommendation: B.** A is cheaper but fails the stated requirement. C tests
one layer deeper but cannot produce the two faults this epic is about (no port,
power flap) and adds a driver dependency. The LLM reflex toward A is named and
rejected: the missing seam is the reason both bugs lived silently.

## Scope

In: F1, F2, the seam, the emulator (Rust backend + TS scenario helpers), the
integration tests, tray tooltip + Debug tab rows, docs. Out: firmware changes,
height stabilization, sensor diagnostics panel (separate backlog items),
replacing the other hollow tests in `device-reconnect.test.ts` beyond the one
T02 makes real.

## Acceptance criteria

1. With the emulator reporting zero ports, `events.log` gets exactly one
   `DEVICE scan ports=0` line across ≥3 rescans, and `desk:device-missing`
   carries `reason: "no_ports"`.
2. With ports `[COM3, COM5]` and no device, the log line is
   `DEVICE scan ports=2 [COM3, COM5] none matched`, reason `no_match`.
3. When enumeration returns an error, the log line is
   `DEVICE scan enumeration_failed <err>` and reason `enumeration_failed` —
   never `ports=0`.
4. Three connections each shorter than 5 s within 60 s produce one
   `DEVICE unstable short_connections=3 window=60s` line, a
   `desk:device-unstable` event, a tray tooltip line "Unstable sensor
   connection — try another cable or a port without a hub", and a Debug tab
   row. Two short connections produce none. A 5-minute connection clears it
   (`DEVICE stable`, `desk:device-stable`).
5. The real scan loop, not a helper, runs in those tests: Rust integration
   tests drive it through `EmulatorSerialBackend` with an injected clock, and
   `pnpm test:integration` drives the running debug app through
   `emulate_serial_scenario`.
6. Production behaviour with a working sensor is unchanged: existing
   `serial_*` tests and `cargo test --lib` stay green.

## Test strategy

| Crit. | Kind | Assertion that fails today | File |
|---|---|---|---|
| 1-3 | Rust integration on emulator (scan loop + fake clock + capturing logger) | log contains `DEVICE scan ports=0` once after 3 scans (today: nothing in `events.log`) | `src-tauri/src/serial_scan_emulator_tests.rs` (`e024_` prefix) |
| 1-3 | Rust unit | `ScanOutcome` transitions: same outcome twice → no line; change → one line | `src-tauri/src/serial_diagnostics_tests.rs` |
| 4 | Rust unit (pure state machine) | 3 short in 60 s → `Unstable`; 2 short → nothing; 3 short spread over 61 s → nothing; 5 min connection → `Stable` | `serial_diagnostics_tests.rs` |
| 4 | Rust integration on emulator | emulator script connect→drop after 200 ms ×3 → unstable line + event | `serial_scan_emulator_tests.rs` |
| 1-4 | TS integration on running app | `emulate_serial_scenario("no_ports")` → `desk:device-missing` payload `reason: "no_ports"`; `"flapping"` → `desk:device-unstable` within 5 s | `tests/integration/device-diagnostics.test.ts` |
| 4 | TS unit | reducer stores `sensorDiagnostic`; Debug tab renders the row | `src/hooks/deskReducer.test.ts`, `DebugSection.test.tsx` |
| 6 | Regression | existing suites | `just check` |

Four shadow paths: happy (device found), nil (enumeration error), empty (zero
ports), error (ports but probe fails / connection drops). Each new test is run
once against reverted code and must fail (`rules/testing.md`). The emulator
must not be stubbed on both sides of a seam: the scan loop under test is the
production function; only the backend is swapped.

## Constraints

- `serial.rs` is 228 lines; the seam must bring it down, not push it past 250.
- `SystemSerialBackend` is a move of today's code — probe timeouts, baud and
  PING protocol unchanged.
- The emulator backend and `emulate_serial_scenario` never compile into a
  release build (`cargo build --release` must not contain the symbol; T02
  asserts with a `cfg` test).
- UI text change (tray tooltip) follows `.claude/rules/ux-design-flow.md`:
  scenario `sensor-unstable` added to `src/test/scenarios.ts` and shown on
  `/#/mockup` in T04.
