/**
 * relay-e2e.test.ts — smoke test for `scripts/relay-e2e.mjs` (E022-T12).
 *
 * The script's own job needs a running relay, so what is asserted here is the
 * part that must hold without one: it parses its arguments, and it refuses to
 * run when it has not been told where the relay is. A script that defaulted to
 * some URL would turn "aimed at nothing" into "aimed somewhere", which is the
 * exact failure the run is supposed to detect.
 *
 * Run: npx vitest run --config vitest.scripts.config.ts tests/scripts/relay-e2e.test.ts
 */
import { describe, it, expect } from "vitest";

// @ts-expect-error — a plain .mjs script, deliberately untyped and outside tsconfig's `src` include.
import { STEPS, USAGE, main, parseArgs, wsUrlFor } from "../../scripts/relay-e2e.mjs";

const ok = (argv: string[]) => {
  const parsed = parseArgs(argv);
  expect(parsed.ok, `expected ${argv.join(" ")} to parse: ${parsed.error}`).toBe(true);
  return parsed.options;
};

const rejected = (argv: string[]) => {
  const parsed = parseArgs(argv);
  expect(parsed.ok, `expected ${argv.join(" ")} to be refused`).toBeFalsy();
  return parsed;
};

const MINIMAL = ["--relay", "http://127.0.0.1:8787", "--license", "mu_lic_test"];

describe("relay-e2e argument parsing", () => {
  it("accepts a relay URL and a licence", () => {
    const options = ok(MINIMAL);
    expect(options.relayUrl).toBe("http://127.0.0.1:8787");
    expect(options.license).toBe("mu_lic_test");
    expect(options.deviceName).toBe("e2e-phone");
    expect(options.timeoutMs).toBeGreaterThan(0);
  });

  it("takes an optional device name and timeout", () => {
    const options = ok([...MINIMAL, "--device-name", "pixel", "--timeout", "2500"]);
    expect(options.deviceName).toBe("pixel");
    expect(options.timeoutMs).toBe(2500);
  });

  it("refuses to run without a relay URL", () => {
    expect(rejected(["--license", "mu_lic_test"]).error).toMatch(/--relay is required/);
  });

  it("refuses a relay URL that is not one, or not http", () => {
    expect(rejected(["--relay", "not a url", "--license", "k"]).error).toMatch(/not a URL/);
    expect(rejected(["--relay", "ws://host", "--license", "k"]).error).toMatch(/http or https/);
  });

  it("refuses to run without a licence", () => {
    expect(rejected(["--relay", "http://127.0.0.1:8787"]).error).toMatch(/--license is required/);
  });

  it("refuses a flag with no value, and an unknown flag", () => {
    expect(rejected(["--relay", "--license", "k"]).error).toMatch(/--relay needs a value/);
    expect(rejected([...MINIMAL, "--nope", "1"]).error).toMatch(/unknown argument/);
  });

  it("refuses a timeout that is not a positive integer", () => {
    expect(rejected([...MINIMAL, "--timeout", "0"]).error).toMatch(/positive integer/);
    expect(rejected([...MINIMAL, "--timeout", "soon"]).error).toMatch(/positive integer/);
  });

  it("reports --help as help, not as an error", () => {
    const parsed = parseArgs(["--help"]);
    expect(parsed.help).toBe(true);
    expect(parsed.error).toBeNull();
  });
});

describe("relay-e2e wiring", () => {
  it("upgrades the relay scheme for the socket URL", () => {
    expect(wsUrlFor("http://127.0.0.1:8787", "abc")).toBe("ws://127.0.0.1:8787/v1/desks/abc/ws");
    expect(wsUrlFor("https://relay.desk.zentala.io", "a b")).toBe(
      "wss://relay.desk.zentala.io/v1/desks/a%20b/ws",
    );
  });

  it("has steps to run — an empty plan is a failure, not a pass", () => {
    expect(STEPS.length).toBeGreaterThan(0);
    expect(STEPS.map((s: { name: string }) => s.name)).toContain("command-round-trip");
    for (const step of STEPS) expect(typeof step.run).toBe("function");
  });

  it("exits 2 and prints usage when told nothing", async () => {
    const lines: string[] = [];
    await expect(main([], (line: string) => lines.push(line))).resolves.toBe(2);
    expect(lines).toHaveLength(0);
    expect(USAGE).toMatch(/--relay <url>/);
  });

  it("exits 0 on --help without touching the network", async () => {
    const lines: string[] = [];
    await expect(main(["--help"], (line: string) => lines.push(line))).resolves.toBe(0);
    expect(lines.join("\n")).toMatch(/Usage: node scripts\/relay-e2e\.mjs/);
  });
});
