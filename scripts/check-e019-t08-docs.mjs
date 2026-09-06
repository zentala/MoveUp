#!/usr/bin/env node
// Verifies E019-T08: CLAUDE.md no longer claims TrayController holds no decision
// logic, and the replacement text matches what the code actually does.
//
// The old sentence ("TrayController only executes signals — no decision logic")
// was false: tray_controller.rs computes the policy inputs (elapsed seconds,
// standing laps, tooltip, sensor connectivity) and decides when to dismiss the
// alert popup. tray_signal_exec.rs is the part that only executes.
import { existsSync, readFileSync } from "node:fs";

const DOC = "CLAUDE.md";
const CONTROLLER = "src-tauri/src/tray_controller.rs";

const problems = [];
let checks = 0;

function check(label, ok, detail) {
  checks += 1;
  if (!ok) problems.push(`${label}: ${detail}`);
}

if (!existsSync(DOC)) {
  console.error(`FAIL: ${DOC} is missing`);
  process.exit(1);
}
const doc = readFileSync(DOC, "utf8");

check(
  "stale claim removed",
  !/TrayController only executes signals/.test(doc),
  `${DOC} still says "TrayController only executes signals — no decision logic"`,
);
check(
  "executor named",
  /tray_signal_exec\.rs`? is the pure executor/.test(doc),
  `${DOC} does not name tray_signal_exec.rs as the pure executor`,
);
check(
  "controller role stated",
  /TrayController`? computes the per-tick inputs/.test(doc),
  `${DOC} does not say TrayController computes the per-tick policy inputs`,
);

// Ground the doc against the code: if these disappear, the new sentence would
// itself become stale and this check must fail rather than pass quietly.
if (!existsSync(CONTROLLER)) {
  problems.push(`${CONTROLLER} is missing — cannot ground the doc claim in code`);
  checks += 1;
} else {
  const src = readFileSync(CONTROLLER, "utf8");
  check(
    "controller still computes inputs",
    /fn compute_standing_lap/.test(src),
    `${CONTROLLER} no longer defines compute_standing_lap`,
  );
  check(
    "controller still delegates to the executor",
    /tray_signal_exec::execute_tray/.test(src),
    `${CONTROLLER} no longer calls tray_signal_exec::execute_tray`,
  );
}

if (checks === 0) {
  console.error("FAIL: no checks ran — an empty run is not a pass");
  process.exit(1);
}

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} of ${checks} check(s) failed`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log(`PASS: ${checks}/${checks} checks — CLAUDE.md's TrayController description matches the code`);
process.exit(0);
