#!/usr/bin/env node
// Verifies E017-T08: every T01-T07 check still passes against the current tree,
// and decision D3 (licence = MIT) is recorded in .plan/decisions.jsonl.
//
// This is the epic's aggregate gate. It re-runs the sibling checks rather than
// trusting their earlier verdicts: a later task can regress an earlier one, and
// a check that is never run again reads as a pass it did not earn.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";

const TASKS = ["t01", "t02", "t03", "t04", "t05", "t06", "t07"];
const DECISIONS = ".plan/decisions.jsonl";

const problems = [];
let ran = 0;

for (const task of TASKS) {
  const script = `scripts/check-e017-${task}.mjs`;
  if (!existsSync(script)) {
    problems.push(`${script} is missing — E017-${task.toUpperCase()} has no verification`);
    continue;
  }
  const result = spawnSync(process.execPath, [script], { encoding: "utf8" });
  ran += 1;
  if (result.error) {
    problems.push(`${script} could not be run: ${result.error.message}`);
    continue;
  }
  if (result.status !== 0) {
    const detail = (result.stderr || result.stdout || "").trim().split("\n").slice(0, 6).join(" | ");
    problems.push(`${script} failed (exit ${result.status}): ${detail}`);
  }
}

if (ran !== TASKS.length) {
  problems.push(`only ${ran} of ${TASKS.length} task checks ran — a missing check is not a pass`);
}

if (!existsSync(DECISIONS)) {
  problems.push(`${DECISIONS} is missing`);
} else {
  const lines = readFileSync(DECISIONS, "utf8").split("\n").filter((l) => l.trim().length > 0);
  const records = [];
  for (const [index, line] of lines.entries()) {
    try {
      records.push(JSON.parse(line));
    } catch (err) {
      problems.push(`${DECISIONS} line ${index + 1} is not valid JSON: ${err.message}`);
    }
  }
  const d3 = records.find((r) => r.id === "E017-D3");
  if (!d3) {
    problems.push("decision E017-D3 (licence = MIT) is not recorded in " + DECISIONS);
  } else {
    if (!/\bMIT\b/.test(d3.decision ?? "")) {
      problems.push("E017-D3 decision text does not name the MIT licence");
    }
    if (!d3.rationale) {
      problems.push("E017-D3 has no rationale");
    }
    if (d3.epic !== "E017") {
      problems.push(`E017-D3 epic is ${JSON.stringify(d3.epic)}, expected "E017"`);
    }
  }
}

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} problem(s)`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log(`PASS: ${ran}/${TASKS.length} E017 task checks pass, decision E017-D3 (MIT) recorded`);
process.exit(0);
