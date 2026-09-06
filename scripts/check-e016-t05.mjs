#!/usr/bin/env node
// Verifies E016-T05: AO preconditions — .giter.yaml and root justfile exist
// and are minimally well-formed.
import { existsSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const problems = [];

if (!existsSync(".giter.yaml")) {
  problems.push(".giter.yaml does not exist at repo root");
} else {
  const giter = readFileSync(".giter.yaml", "utf8");
  if (!/worktree\s*:/.test(giter)) problems.push(".giter.yaml has no worktree: key");
  if (!/copy\s*:/.test(giter)) problems.push(".giter.yaml has no worktree.copy key");
  if (!/guards\s*:/.test(giter)) problems.push(".giter.yaml has no worktree.guards key");
  if (!/node_modules/.test(giter)) problems.push(".giter.yaml guards do not cover node_modules");
  if (!/target/.test(giter)) problems.push(".giter.yaml guards do not cover src-tauri/target");
}

if (!existsSync("justfile") && !existsSync("Justfile")) {
  problems.push("no justfile at repo root");
} else {
  try {
    execFileSync("just", ["--list"], { stdio: "pipe" });
  } catch (err) {
    problems.push("just --list did not exit 0: " + err.message);
  }
  const jf = readFileSync(existsSync("justfile") ? "justfile" : "Justfile", "utf8");
  for (const target of ["setup", "dev", "build", "test", "check", "typecheck", "lint", "clean"]) {
    if (!new RegExp("^" + target + "\\s*:", "m").test(jf)) {
      problems.push("justfile is missing target: " + target);
    }
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: .giter.yaml and justfile present and well-formed");
process.exit(0);
