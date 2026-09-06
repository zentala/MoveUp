#!/usr/bin/env node
// Verifies E015-T04: the docs describe the same single model as the code —
// no `current_session_secs` left in UX-FLOW, ADR 008 carries the 3.0 default,
// CLAUDE.md and ARCHITECTURE.md name the credited counter, and the two
// decisions (D1, D2) are recorded in .plan/decisions.jsonl.
import { existsSync, readFileSync } from "node:fs";

const problems = [];

const read = (path) => {
  if (!existsSync(path)) {
    problems.push(path + " does not exist");
    return null;
  }
  return readFileSync(path, "utf8");
};

const uxFlowPath = [".arch", "UX-FLOW.md"].join("/");
const adrPath = [".arch", "ADR", "008-proportional-break-credit.md"].join("/");
const archPath = [".arch", "ARCHITECTURE.md"].join("/");
const claudePath = "CLAUDE.md";
const decisionsPath = [".plan", "decisions.jsonl"].join("/");

const uxFlow = read(uxFlowPath);
if (uxFlow !== null) {
  if (uxFlow.includes("current_session_secs")) {
    problems.push(uxFlowPath + " still mentions current_session_secs");
  }
  if (!uxFlow.includes("limit_used_secs")) {
    problems.push(uxFlowPath + " never names the credited counter limit_used_secs");
  }
  if (/Subtract 1200s/.test(uxFlow) || /BREAK_SHORT_SECS/.test(uxFlow)) {
    problems.push(uxFlowPath + " still describes the pre-ADR-008 three-tier credit");
  }
}

const adr = read(adrPath);
if (adr !== null) {
  if (!adr.includes("3.0")) {
    problems.push(adrPath + " does not state the 3.0 multiplier (decision D1)");
  }
  if (!adr.includes("limit_used_secs")) {
    problems.push(adrPath + " does not state the one-credited-counter rule (decision D2)");
  }
}

const arch = read(archPath);
if (arch !== null && !arch.includes("limit_used_secs")) {
  problems.push(archPath + " session section does not name limit_used_secs");
}

const claude = read(claudePath);
if (claude !== null) {
  if (claude.includes("current_session_secs")) {
    problems.push(claudePath + " still mentions current_session_secs");
  }
  if (!claude.includes("limitUsedSecs")) {
    problems.push(claudePath + " Session Logic does not name the credited counter");
  }
}

const decisions = read(decisionsPath);
if (decisions !== null) {
  const lines = decisions.split("\n").filter((line) => line.trim() !== "");
  if (lines.length === 0) problems.push(decisionsPath + " is empty");
  const ids = [];
  for (const [index, line] of lines.entries()) {
    try {
      const entry = JSON.parse(line);
      if (!entry.decision || !entry.rationale) {
        problems.push(decisionsPath + " line " + (index + 1) + " lacks decision/rationale");
      }
      if (entry.id) ids.push(entry.id);
    } catch {
      problems.push(decisionsPath + " line " + (index + 1) + " is not valid JSON");
    }
  }
  for (const required of ["E015-D1", "E015-D2"]) {
    if (!ids.includes(required)) {
      problems.push(decisionsPath + " has no " + required + " entry");
    }
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: ADR 008, UX-FLOW, ARCHITECTURE, CLAUDE.md and decisions.jsonl agree on one credited counter");
process.exit(0);
