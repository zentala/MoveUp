#!/usr/bin/env node
// Verifies E017-T05: .github/workflows/test.yml runs from the repo root.
// Every path the workflow names must exist in this checkout, the dead
// `apps/desk` prefix must be gone, and release-baseline.yml must be intact.
import { existsSync, readFileSync, statSync } from "node:fs";

const WORKFLOW = ".github/workflows/test.yml";
const BASELINE = ".github/workflows/release-baseline.yml";

const problems = [];
let checked = 0;

const fail = (msg) => problems.push(msg);

if (!existsSync(WORKFLOW)) {
  fail(`${WORKFLOW} does not exist`);
} else {
  const yml = readFileSync(WORKFLOW, "utf8");

  // 1. The dead path prefix is gone everywhere, not just in working-directory.
  const stale = yml.match(/apps\/desk/g) ?? [];
  checked += 1;
  if (stale.length > 0) {
    fail(`${WORKFLOW} still names apps/desk ${stale.length} time(s)`);
  }

  // 2. Every working-directory: resolves to a real directory.
  const workdirs = [...yml.matchAll(/^\s*working-directory:\s*(.+?)\s*$/gm)].map(
    (m) => m[1].replace(/^["']|["']$/g, ""),
  );
  for (const dir of workdirs) {
    checked += 1;
    if (!existsSync(dir) || !statSync(dir).isDirectory()) {
      fail(`working-directory: ${dir} is not a directory in this repo`);
    }
  }

  // 3. cache-dependency-path points at a real lockfile.
  const caches = [...yml.matchAll(/^\s*cache-dependency-path:\s*(.+?)\s*$/gm)].map(
    (m) => m[1].replace(/^["']|["']$/g, ""),
  );
  if (caches.length === 0) fail(`${WORKFLOW} declares no cache-dependency-path`);
  for (const p of caches) {
    checked += 1;
    if (!existsSync(p)) fail(`cache-dependency-path: ${p} does not exist`);
  }

  // 4. The integration-ready job's `test -f` assertions name real files.
  const probes = [...yml.matchAll(/^\s*test -f\s+(\S+)\s*$/gm)].map((m) => m[1]);
  if (probes.length !== 3) {
    fail(`expected 3 'test -f' integration probes, found ${probes.length}`);
  }
  for (const p of probes) {
    checked += 1;
    if (!existsSync(p)) fail(`integration probe file ${p} does not exist`);
  }

  // 5. Any step invoking cargo against the crate must run inside src-tauri.
  //    `cargo install` is the exception — it is location-independent.
  const steps = yml.split(/^ {6}- /m).slice(1);
  let cargoSteps = 0;
  for (const step of steps) {
    const cargoUse = [...step.matchAll(/\bcargo\s+([a-z-]+)/g)]
      .map((m) => m[1])
      .filter((sub) => sub !== "install");
    if (cargoUse.length === 0) continue;
    cargoSteps += 1;
    checked += 1;
    if (!/working-directory:\s*src-tauri\b/.test(step)) {
      fail(`step running 'cargo ${cargoUse[0]}' does not set working-directory: src-tauri`);
    }
  }
  checked += 1;
  if (cargoSteps < 3) {
    fail(`expected at least 3 cargo steps (audit, bench, mutants), found ${cargoSteps}`);
  }

  // 6. pnpm must be able to read lockfileVersion 9.
  const pnpmVersion = yml.match(/pnpm\/action-setup@v\d+[\s\S]{0,120}?version:\s*["']?(\d+)/);
  checked += 1;
  if (!pnpmVersion) {
    fail("could not read the pnpm version pinned in the Setup pnpm step");
  } else if (Number(pnpmVersion[1]) < 9) {
    fail(`pnpm ${pnpmVersion[1]} cannot read pnpm-lock.yaml lockfileVersion 9`);
  }

  // 7. actions/upload-artifact v3 is retired and fails the job outright.
  checked += 1;
  if (/upload-artifact@v3/.test(yml)) {
    fail("actions/upload-artifact@v3 is retired and fails on GitHub-hosted runners");
  }
}

// 8. release-baseline.yml stays the working release job — T05 must not touch it.
checked += 1;
if (!existsSync(BASELINE)) {
  fail(`${BASELINE} is missing — T05 must leave it in place`);
} else {
  const baseline = readFileSync(BASELINE, "utf8");
  for (const marker of ["working-directory: MoveUp", "pnpm exec tauri build", "upload-artifact@v4"]) {
    checked += 1;
    if (!baseline.includes(marker)) fail(`${BASELINE} no longer contains "${marker}"`);
  }
}

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} problem(s) across ${checked} check(s)`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

if (checked < 15) {
  console.error(`FAIL: only ${checked} checks ran — the workflow was not parsed as expected`);
  process.exit(1);
}

console.log(`PASS: ${checked} checks — test.yml runs from the repo root, release-baseline.yml intact`);
process.exit(0);
