#!/usr/bin/env node
// Verifies E017-T01: the five user-facing docs carry MoveUp naming, the real
// install/data paths, and the verbatim cable-sensitivity warning.
import { readFileSync } from "node:fs";

const DOCS = [
  "docs/README.md",
  "docs/USER_INSTALL.md",
  "docs/USER_SUPPORT.md",
  "docs/REMOTE_DISPLAY.md",
  "docs/OPTIMIZATION_GUIDE.md",
];

// Retired identity that must not survive anywhere in the five docs.
const FORBIDDEN = [
  [/zntldesk/i, "retired product name zntlDesk"],
  [/zntl[- ]tray/i, "retired repo name zntl-tray"],
  [/zntl desk/i, 'retired product name "zntl Desk"'],
  [/smart ?desk/i, "retired product name SmartDesk"],
  [/AppData\\Local\\zntl/i, "retired data path AppData\\Local\\zntlDesk"],
];

// Per-file facts that must be present.
const REQUIRED = {
  "docs/README.md": [
    [/\bMoveUp\b/, "the product name MoveUp"],
    [/github\.com\/zentala\/MoveUp/, "the current repo URL"],
    [/io\.zntl\.desk/, "the real app-data directory io.zntl.desk"],
    [/no auto-?updater/i, "that there is no auto-updater"],
    [/REMOTE_DISPLAY\.md/, "a link to the remote-display guide"],
  ],
  "docs/USER_INSTALL.md": [
    [/\bMoveUp\b/, "the product name MoveUp"],
    [/github\.com\/zentala\/MoveUp/, "the current repo URL"],
    [/%LOCALAPPDATA%\\MoveUp\\/, "the install directory %LOCALAPPDATA%\\MoveUp\\"],
    [/%APPDATA%\\io\.zntl\.desk\\/, "the data directory %APPDATA%\\io.zntl.desk\\"],
  ],
  "docs/USER_SUPPORT.md": [
    [/\bMoveUp\b/, "the product name MoveUp"],
    [/github\.com\/zentala\/MoveUp/, "the current repo URL"],
    [/%APPDATA%\\io\.zntl\.desk\\/, "the data directory %APPDATA%\\io.zntl.desk\\"],
    [/events\.log/, "the events log file"],
    [/desk\.db/, "the SQLite database file name"],
    [/no auto-?updater/i, "that there is no auto-updater"],
  ],
  "docs/REMOTE_DISPLAY.md": [[/\bMoveUp\b/, "the product name MoveUp"]],
  "docs/OPTIMIZATION_GUIDE.md": [[/\bMoveUp\b/, "the product name MoveUp"]],
};

// Claims that are false for this build and must not reappear.
const STALE_CLAIMS = [
  [/Automatic every 24 hours/i, "the 24-hour auto-update claim"],
  [
    /Check for updates automatically/i,
    "the non-existent auto-update setting",
  ],
  [/nothing leaves your computer/i, "the blanket no-network claim"],
];

// The cable warning must be copied, not paraphrased. These three sentences are
// the load-bearing parts of the CLAUDE.md paragraph.
const CABLE_SENTENCES = [
  "most USB-C cables do NOT work with this board",
  "Charge-only cables (VBUS+GND, no D+/D-)",
  "5.1k CC resistors",
  "Voltage drop on thin (28 AWG) or long cables",
  "HKLM\\HARDWARE\\DEVICEMAP\\SERIALCOMM",
];
const CABLE_DOCS = ["docs/USER_INSTALL.md", "docs/USER_SUPPORT.md"];

const problems = [];

for (const path of DOCS) {
  const text = readFileSync(path, "utf8");

  for (const [re, label] of FORBIDDEN) {
    if (re.test(text)) problems.push(`${path}: still contains ${label}`);
  }
  for (const [re, label] of REQUIRED[path]) {
    if (!re.test(text)) problems.push(`${path}: does not mention ${label}`);
  }
  for (const [re, label] of STALE_CLAIMS) {
    if (re.test(text)) problems.push(`${path}: still contains ${label}`);
  }
}

for (const path of CABLE_DOCS) {
  const text = readFileSync(path, "utf8");
  for (const sentence of CABLE_SENTENCES) {
    if (!text.includes(sentence)) {
      problems.push(`${path}: cable warning is missing "${sentence}"`);
    }
  }
}

console.log(`Checked ${DOCS.length} documents.`);

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} problem(s) found`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: all five docs use MoveUp naming and real paths");
process.exit(0);
