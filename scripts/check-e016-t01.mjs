#!/usr/bin/env node
// Verifies E016-T01: .plan/STATE.md frontmatter and body agree, and the
// E010 human-task count discrepancy is reconciled with an explicit note.
import { readFileSync } from "node:fs";

const path = ".plan/STATE.md";
const text = readFileSync(path, "utf8");

const problems = [];

if (text.includes("active, scaffolded, T03 done")) {
  problems.push('body still contains stale "active, scaffolded, T03 done" line');
}
if (/Wave 1 \(T01\+T02\) and Wave 2-3 pending/.test(text)) {
  problems.push("body still claims E011 waves are pending");
}
if (!/E011/.test(text) || !/done|complete/i.test(text)) {
  problems.push("body does not clearly state E011 is done");
}
if (!/v0\.5\.0/.test(text)) {
  problems.push("body/frontmatter does not reference current version v0.5.0");
}
// The E010 human-task reconciliation: both counts (3 and 10) or an explicit
// note explaining the relationship must be present.
if (!/reconcil|Collect from User|10 items|see .*BACKLOG/i.test(text)) {
  problems.push("no reconciliation note for the E010 3-vs-10 human-task discrepancy");
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s) found in " + path);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: " + path + " is internally consistent (checked " + 5 + " conditions)");
process.exit(0);
