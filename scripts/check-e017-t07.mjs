#!/usr/bin/env node
// Verifies E017-T07: .perf-baseline.json was refreshed by `pnpm test:perf`
// during E017 and carries a complete, sane record.
//
// A missing or unparsable field is reported as a problem, never rendered as 0 —
// "no measurement" must not look like "measured zero".
import { existsSync, readFileSync } from "node:fs";

const BASELINE = ".perf-baseline.json";
// The stale baseline this task replaced was stamped 2026-03-16. Anything at or
// before that date means the refresh did not happen.
const MIN_TIMESTAMP = Date.parse("2026-09-01T00:00:00Z");
// Guardrails copied from tests/perf/memory-profile.test.ts, which is what
// produces this file.
const MAX_PEAK_MB = 350;
const MAX_STABLE_MB = 220;

const problems = [];

if (!existsSync(BASELINE)) {
  problems.push(`${BASELINE} does not exist at repo root`);
} else {
  let data;
  try {
    data = JSON.parse(readFileSync(BASELINE, "utf8"));
  } catch (err) {
    problems.push(`${BASELINE} is not valid JSON: ${err.message}`);
  }

  if (data) {
    for (const field of ["timestamp", "peak_mb", "stable_mb", "trend", "samples_count"]) {
      if (data[field] === undefined || data[field] === null) {
        problems.push(`${BASELINE} is missing field: ${field}`);
      }
    }

    const ts = Date.parse(data.timestamp ?? "");
    if (Number.isNaN(ts)) {
      problems.push(`${BASELINE} timestamp is not a parsable date: ${data.timestamp}`);
    } else if (ts < MIN_TIMESTAMP) {
      problems.push(
        `${BASELINE} timestamp ${data.timestamp} predates E017 — baseline was not refreshed`,
      );
    } else if (ts > Date.now() + 5 * 60 * 1000) {
      problems.push(`${BASELINE} timestamp ${data.timestamp} is in the future`);
    }

    for (const field of ["peak_mb", "stable_mb"]) {
      const value = data[field];
      if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
        problems.push(`${BASELINE} ${field} is not a positive finite number: ${value}`);
      }
    }

    if (typeof data.peak_mb === "number" && data.peak_mb > MAX_PEAK_MB) {
      problems.push(`${BASELINE} peak_mb ${data.peak_mb} exceeds ${MAX_PEAK_MB} MB`);
    }
    if (typeof data.stable_mb === "number" && data.stable_mb > MAX_STABLE_MB) {
      problems.push(`${BASELINE} stable_mb ${data.stable_mb} exceeds ${MAX_STABLE_MB} MB`);
    }

    if (!["stable", "growing", "declining"].includes(data.trend)) {
      problems.push(`${BASELINE} trend is not one of stable/growing/declining: ${data.trend}`);
    }

    if (!Number.isInteger(data.samples_count) || data.samples_count <= 0) {
      problems.push(`${BASELINE} samples_count is not a positive integer: ${data.samples_count}`);
    }
  }
}

if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} problem(s)`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

const data = JSON.parse(readFileSync(BASELINE, "utf8"));
console.log(
  `PASS: ${BASELINE} refreshed ${data.timestamp} — peak ${data.peak_mb.toFixed(1)} MB, ` +
    `stable ${data.stable_mb.toFixed(1)} MB, ${data.samples_count} samples, trend ${data.trend}`,
);
process.exit(0);
