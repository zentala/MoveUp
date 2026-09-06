#!/usr/bin/env node
// Verifies E016-T02: root BACKLOG.md/TASKS.md/ORCHESTRATOR.md are gone,
// their unique content survives verbatim in the canonical backlog file,
// and README/CONTRIBUTING no longer link to the removed files.
import { existsSync, readFileSync } from "node:fs";

const problems = [];
const canonicalBacklogPath = [".plan", "BACKLOG.md"].join("/");

for (const removed of ["BACKLOG.md", "TASKS.md", "ORCHESTRATOR.md"]) {
  if (existsSync(removed)) {
    problems.push("root " + removed + " still exists");
  }
}

if (!existsSync(canonicalBacklogPath)) {
  problems.push(canonicalBacklogPath + " is missing");
} else {
  const merged = readFileSync(canonicalBacklogPath, "utf8");
  if (!/Collect from User/i.test(merged)) {
    problems.push('merged backlog is missing the "Collect from User" section from root BACKLOG.md');
  }
  if (!/Stripe account/i.test(merged)) {
    problems.push("merged backlog is missing the Stripe/Plausible/GSC human-task items");
  }
}

for (const doc of ["README.md", "CONTRIBUTING.md"]) {
  if (existsSync(doc)) {
    const body = readFileSync(doc, "utf8");
    if (/\]\(BACKLOG\.md\)|\]\(TASKS\.md\)|\]\(ORCHESTRATOR\.md\)/.test(body)) {
      problems.push(doc + " still links to a deleted root file");
    }
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: backlog consolidated into " + canonicalBacklogPath + ", no dangling links");
process.exit(0);
