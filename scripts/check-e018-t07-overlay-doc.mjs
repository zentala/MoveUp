import { readFileSync } from "node:fs";

const doc = readFileSync(".claude/rules/overlay.md", "utf8");
const problems = [];
if (doc.includes("tauri-dev.sh")) problems.push("stale 'tauri-dev.sh' reference still present");
if (!doc.includes("tauri-dev.ps1 -Force")) problems.push("missing corrected 'tauri-dev.ps1 -Force' invocation");
if (problems.length > 0) {
  console.error("FAIL:", problems.join("; "));
  process.exit(1);
}
console.log("OK: overlay.md launcher references are correct");
