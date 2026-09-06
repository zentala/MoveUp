#!/usr/bin/env node
// Verifies E017-T04: MIT LICENSE at repo root, `license` declared in
// package.json and src-tauri/Cargo.toml, and a Cargo description naming MoveUp.
import { existsSync, readFileSync } from "node:fs";

const problems = [];

if (!existsSync("LICENSE")) {
  problems.push("LICENSE is missing at the repo root");
} else {
  const license = readFileSync("LICENSE", "utf8");
  const required = [
    "MIT License",
    "Paweł Żentała",
    "Permission is hereby granted, free of charge",
    'THE SOFTWARE IS PROVIDED "AS IS"',
  ];
  for (const phrase of required) {
    if (!license.includes(phrase)) {
      problems.push("LICENSE does not contain: " + phrase);
    }
  }
}

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
if (pkg.license !== "MIT") {
  problems.push('package.json "license" is ' + JSON.stringify(pkg.license) + ', expected "MIT"');
}

const cargo = readFileSync("src-tauri/Cargo.toml", "utf8");
const pkgSection = cargo.split(/^\[/m)[1] ?? "";
if (!/^license\s*=\s*"MIT"$/m.test(pkgSection)) {
  problems.push('src-tauri/Cargo.toml [package] is missing license = "MIT"');
}

const descMatch = pkgSection.match(/^description\s*=\s*"(.*)"$/m);
if (!descMatch) {
  problems.push("src-tauri/Cargo.toml [package] has no description");
} else {
  const description = descMatch[1];
  if (!description.includes("MoveUp")) {
    problems.push("Cargo description does not name MoveUp: " + description);
  }
  if (/zntl/i.test(description)) {
    problems.push("Cargo description still carries the legacy zntl name: " + description);
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log("PASS: MIT LICENSE present, license declared in package.json and Cargo.toml, description names MoveUp");
process.exit(0);
