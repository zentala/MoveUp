#!/usr/bin/env node
// Verifies E016-T04: coverage/ and test-performance-report/ are untracked
// and gitignored.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const problems = [];

const tracked = execFileSync("git", ["ls-files"], { encoding: "utf8" })
  .split("\n")
  .filter((f) => f.startsWith("coverage/") || f.startsWith("test-performance-report/"));

if (tracked.length > 0) {
  problems.push(tracked.length + " tracked file(s) remain under coverage/ or test-performance-report/");
}

const gitignore = readFileSync(".gitignore", "utf8");
for (const dir of ["coverage/", "test-performance-report/"]) {
  if (!gitignore.includes(dir)) {
    problems.push(".gitignore is missing an entry for " + dir);
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: 0 tracked files under coverage/ or test-performance-report/, both gitignored");
process.exit(0);
