#!/usr/bin/env node
// Verifies E016-T03: E011 closing ceremony recorded (HISTORY.md entries),
// E003-T07 marked superseded, E013 carries the wave-split note.
import { existsSync, readFileSync } from "node:fs";

const problems = [];
const historyPath = [".plan", "HISTORY.md"].join("/");
const backlogPath = [".plan", "BACKLOG.md"].join("/");
const e013PlanPath = [
  ".plan",
  "epics",
  "E013-2026-08-28-signed-tauri-pm3-deployment",
  "PLAN.md",
].join("/");

if (!existsSync(historyPath)) {
  problems.push(historyPath + " does not exist");
} else {
  const history = readFileSync(historyPath, "utf8");
  if (!/E011/.test(history)) problems.push(historyPath + " has no E011 entry");
  if (!/E012/.test(history)) problems.push(historyPath + " has no E012 entry");
}

if (!existsSync(backlogPath)) {
  problems.push(backlogPath + " does not exist");
} else {
  const backlog = readFileSync(backlogPath, "utf8");
  const e003Line = backlog
    .split("\n")
    .find((line) => line.includes("E003-T07"));
  if (!e003Line || !/superseded/i.test(e003Line)) {
    problems.push("E003-T07 line in " + backlogPath + " is not marked superseded");
  }
}

if (!existsSync(e013PlanPath)) {
  problems.push(e013PlanPath + " does not exist");
} else {
  const e013 = readFileSync(e013PlanPath, "utf8");
  if (!/superseded/i.test(e013)) {
    problems.push(e013PlanPath + " has no superseded status note");
  }
  if (!/E017/.test(e013) || !/E014/.test(e013)) {
    problems.push(e013PlanPath + " does not name both split targets (E017, E014)");
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: E011 ceremony recorded, E003-T07 and E013 supersession notes present");
process.exit(0);
