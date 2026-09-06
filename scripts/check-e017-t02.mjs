#!/usr/bin/env node
// Verifies E017-T02: docs/USER_UPDATES.md describes the real manual update
// path (no auto-updater exists in code), carries MoveUp naming, and the
// updater follow-on is filed in .plan/BACKLOG.md.
import { existsSync, readFileSync } from "node:fs";

const problems = [];
const updatesPath = ["docs", "USER_UPDATES.md"].join("/");
const backlogPath = [".plan", "BACKLOG.md"].join("/");

if (!existsSync(updatesPath)) {
  problems.push(updatesPath + " is missing");
} else {
  const doc = readFileSync(updatesPath, "utf8");

  for (const stale of ["zntlDesk", "zntl-tray", "Smart Desk"]) {
    if (new RegExp(stale.replace(/[-\s]/g, "[-\\s]"), "i").test(doc)) {
      problems.push(updatesPath + " still mentions retired name " + stale);
    }
  }

  if (!/MoveUp/.test(doc)) {
    problems.push(updatesPath + " never names the product MoveUp");
  }
  if (!/github\.com\/zentala\/MoveUp\/releases/.test(doc)) {
    problems.push(updatesPath + " does not point at the MoveUp GitHub Releases page");
  }
  if (!/no automatic updater/i.test(doc)) {
    problems.push(updatesPath + ' does not state plainly that there is "no automatic updater"');
  }
  if (!/manual/i.test(doc)) {
    problems.push(updatesPath + " does not describe the update as a manual step");
  }
  if (/checks for updates \*\*every 24 hours\*\*|every 24 hours while the application is running/i.test(doc)) {
    problems.push(updatesPath + " still claims a 24-hour automatic update check");
  }
  if (/Toggle off \*\*Check for updates automatically\*\*/i.test(doc)) {
    problems.push(updatesPath + " still documents an auto-update setting that does not exist");
  }
}

if (!existsSync(backlogPath)) {
  problems.push(backlogPath + " is missing");
} else {
  const lines = readFileSync(backlogPath, "utf8").split(/\r?\n/);
  const blocks = [];
  lines.forEach((line, i) => {
    if (/^- \[ \]/.test(line) && /updater/i.test(line)) {
      blocks.push(lines.slice(i, i + 20).join("\n"));
    }
  });
  const filed = blocks.some(
    (block) =>
      /USER_UPDATES\.md/.test(block) &&
      /Points: 8/.test(block) &&
      /Importance: Medium/.test(block),
  );
  if (!filed) {
    problems.push(
      backlogPath +
        " has no open updater follow-on entry linking docs/USER_UPDATES.md with Importance: Medium, Points: 8",
    );
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: " + updatesPath + " documents the manual update path; updater follow-on filed in " + backlogPath);
process.exit(0);
