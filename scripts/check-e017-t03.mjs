#!/usr/bin/env node
// Verifies E017-T03: docs/PRIVACY.md uses MoveUp naming and discloses the
// Google Fit data flow instead of claiming that nothing ever leaves the machine.
import { readFileSync } from "node:fs";

const path = "docs/PRIVACY.md";
const text = readFileSync(path, "utf8");

const problems = [];

// --- 1. Retired product naming must be gone -------------------------------
const forbidden = [
  [/zntldesk/i, "retired product name zntlDesk"],
  [/zntl[- ]tray/i, "retired repo name zntl-tray"],
  [/zntl desk/i, 'retired product name "zntl Desk"'],
  [/smart ?desk/i, "retired product name SmartDesk"],
  [/AppData\\Local\\zntl/i, "retired data path AppData\\Local\\zntlDesk"],
];
for (const [re, label] of forbidden) {
  if (re.test(text)) problems.push(`still contains ${label}`);
}

// --- 2. Current identity must be present ----------------------------------
const required = [
  [/\bMoveUp\b/, "product name MoveUp"],
  [/github\.com\/zentala\/MoveUp/, "current repo URL github.com/zentala/MoveUp"],
  [/io\.zntl\.desk/, "real app-data directory io.zntl.desk"],
];
for (const [re, label] of required) {
  if (!re.test(text)) problems.push(`does not mention ${label}`);
}

// --- 3. The blanket "nothing leaves this machine" claim must be gone ------
const blanket = [
  [/send any data to servers/i, '"Send any data to servers" blanket claim'],
  [
    /No information leaves your machine/i,
    '"No information leaves your machine" blanket claim',
  ],
  [
    /No external network calls \(except GitHub updates\)/i,
    "stale third-party-library network claim",
  ],
];
for (const [re, label] of blanket) {
  if (re.test(text)) problems.push(`still contains the ${label}`);
}

// --- 4. Google Fit disclosure ---------------------------------------------
const disclosure = [
  [/^#+ .*Google Fit/im, "a Google Fit section heading"],
  [/googleapis\.com/, "the Google API endpoints it contacts"],
  [
    /fitness\.activity\.read/,
    "the OAuth scope (fitness.activity.read) it requests",
  ],
  [/opt[- ]in/i, "that the integration is opt-in"],
  [/GOOGLE_CLIENT_ID/, "the credentials that enable it"],
  [/turn it off|revoke/i, "how to turn it off"],
];
for (const [re, label] of disclosure) {
  if (!re.test(text)) problems.push(`Google Fit disclosure lacks ${label}`);
}

// --- 5. No claim of an auto-updater that does not exist -------------------
if (/Check for updates automatically/i.test(text)) {
  problems.push("still describes an automatic update check that does not exist");
}
if (!/no auto-?updater/i.test(text)) {
  problems.push("does not state that there is no auto-updater");
}

// --- 6. Other real network surfaces are disclosed -------------------------
if (!/3390/.test(text)) {
  problems.push("does not disclose the remote-display server on port 3390");
}
if (!/telemetry/i.test(text)) {
  problems.push("does not describe the telemetry opt-in toggle");
}

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} problem(s) found in ${path}`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log(`PASS: ${path} is MoveUp-named and discloses the Google Fit flow`);
process.exit(0);
