#!/usr/bin/env node
/**
 * relay-e2e.mjs — drive one whole pairing + control round trip against a relay.
 *
 * This is the only check that crosses every seam at once: REST credentials, the
 * Durable Object's socket protocol, event fan-out, and viewer -> desk command
 * routing. Unit suites prove each half against the other half's schema; this
 * proves the halves actually talk.
 *
 * Local run (two terminals):
 *
 *   pnpm --dir relay exec wrangler dev
 *   node scripts/relay-e2e.mjs --relay http://127.0.0.1:8787 \
 *     --license "$(node relay/scripts/mint-license.mjs --local)"
 *
 * Against the deployed host (T16) the licence comes from `password-broker
 * inject`, never from a file and never from the shell history.
 *
 * Exit codes: 0 every step passed, 1 a step failed, 2 the arguments are wrong.
 * A run that executed zero steps exits 1 — an empty run must never read as a
 * pass (CLAUDE.md, "Cisza nigdy nie znaczy sukcesu").
 */
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const PROTOCOL_VERSION = 1;
export const DEFAULT_TIMEOUT_MS = 10_000;

export const USAGE = `Usage: node scripts/relay-e2e.mjs --relay <url> --license <key> [options]

Required:
  --relay <url>        base URL of the relay, e.g. http://127.0.0.1:8787
  --license <key>      licence key to register a desk with (mu_lic_...)

Options:
  --device-name <s>    name recorded for the paired viewer (default "e2e-phone")
  --timeout <ms>       per-step timeout in milliseconds (default ${DEFAULT_TIMEOUT_MS})
  --help               print this text`;

// ─── Arguments ──────────────────────────────────────────────────────────────

const VALUE_FLAGS = new Set(["--relay", "--license", "--device-name", "--timeout"]);

/**
 * Parses argv into options. Returns `{ok: true, options}` or `{ok: false, error}`
 * — it never exits, so the smoke test can assert the refusals.
 *
 * There is no default relay URL on purpose: defaulting one would let a run
 * aimed at nothing look like a run aimed somewhere.
 */
export function parseArgs(argv) {
  const raw = {};
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (flag === "--help" || flag === "-h") return { ok: false, error: null, help: true };
    if (!VALUE_FLAGS.has(flag)) return { ok: false, error: `unknown argument: ${flag}` };
    const value = argv[i + 1];
    if (value === undefined || value.startsWith("--")) {
      return { ok: false, error: `${flag} needs a value` };
    }
    raw[flag] = value;
    i += 1;
  }

  const relay = raw["--relay"];
  if (!relay) return { ok: false, error: "--relay is required (no default relay URL)" };
  let base;
  try {
    base = new URL(relay);
  } catch {
    return { ok: false, error: `--relay is not a URL: ${relay}` };
  }
  if (base.protocol !== "http:" && base.protocol !== "https:") {
    return { ok: false, error: `--relay must be http or https, got ${base.protocol}` };
  }

  const license = raw["--license"];
  if (!license) return { ok: false, error: "--license is required" };

  const timeoutMs = raw["--timeout"] === undefined ? DEFAULT_TIMEOUT_MS : Number(raw["--timeout"]);
  if (!Number.isInteger(timeoutMs) || timeoutMs <= 0) {
    return { ok: false, error: "--timeout must be a positive integer of milliseconds" };
  }

  return {
    ok: true,
    options: {
      relayUrl: base.origin,
      license,
      deviceName: raw["--device-name"] ?? "e2e-phone",
      timeoutMs,
    },
  };
}

/** The socket URL for a desk room. http upgrades to ws, https to wss. */
export function wsUrlFor(relayUrl, deskId) {
  const url = new URL(`/v1/desks/${encodeURIComponent(deskId)}/ws`, relayUrl);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  return url.toString();
}

// ─── HTTP + socket helpers ──────────────────────────────────────────────────

async function call(options, method, route, { token, body } = {}) {
  const headers = {};
  if (token) headers.Authorization = `Bearer ${token}`;
  if (body !== undefined) headers["Content-Type"] = "application/json";
  const response = await fetch(new URL(route, options.relayUrl), {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(options.timeoutMs),
  });
  const text = await response.text();
  let json = null;
  try {
    json = text.length > 0 ? JSON.parse(text) : null;
  } catch {
    json = null;
  }
  return { status: response.status, json, text };
}

/** One end of the relay socket: says hello, then queues what arrives. */
class Peer {
  constructor(role, options) {
    this.role = role;
    this.options = options;
    this.inbox = [];
    this.waiters = [];
  }

  async open(deskId, token) {
    this.ws = new WebSocket(wsUrlFor(this.options.relayUrl, deskId));
    this.ws.addEventListener("message", (e) => this.#deliver(e.data));
    await new Promise((resolve, reject) => {
      this.ws.addEventListener("open", resolve, { once: true });
      this.ws.addEventListener("error", () => reject(new Error(`${this.role}: socket failed`)), {
        once: true,
      });
      AbortSignal.timeout(this.options.timeoutMs).addEventListener("abort", () =>
        reject(new Error(`${this.role}: socket did not open in ${this.options.timeoutMs}ms`)),
      );
    });
    const client = { app: "relay-e2e", version: "1" };
    return this.send("hello", { role: this.role, desk_id: deskId, token, client });
  }

  send(type, payload, id = crypto.randomUUID()) {
    this.ws.send(JSON.stringify({ v: PROTOCOL_VERSION, type, id, ts: Date.now(), payload }));
    return id;
  }

  #deliver(raw) {
    const message = JSON.parse(String(raw));
    const at = this.waiters.findIndex((w) => w.match(message));
    if (at === -1) {
      this.inbox.push(message);
      return;
    }
    const [waiter] = this.waiters.splice(at, 1);
    waiter.resolve(message);
  }

  /** The next message of `type`, whether it already arrived or is still coming. */
  expect(type, id) {
    const match = (m) => m.type === type && (id === undefined || m.id === id || m.payload?.command_id === id);
    const at = this.inbox.findIndex(match);
    if (at !== -1) return Promise.resolve(this.inbox.splice(at, 1)[0]);
    return new Promise((resolve, reject) => {
      const waiter = { match, resolve };
      this.waiters.push(waiter);
      AbortSignal.timeout(this.options.timeoutMs).addEventListener("abort", () => {
        const still = this.waiters.indexOf(waiter);
        if (still === -1) return;
        this.waiters.splice(still, 1);
        reject(new Error(`${this.role}: no ${type} within ${this.options.timeoutMs}ms`));
      });
    });
  }

  close() {
    try {
      this.ws?.close();
    } catch {
      // Already gone — nothing to undo.
    }
  }
}

// ─── The run ────────────────────────────────────────────────────────────────

/**
 * Every step in order. Each one returns the facts the later steps need, so the
 * list doubles as the dependency chain — and the step names are what the run
 * prints, so a failure names the seam rather than a stack frame.
 */
export const STEPS = [
  {
    name: "register",
    async run(ctx) {
      const body = { license_key: ctx.options.license, desk_name: "e2e-desk", app_version: "e2e" };
      const { status, json } = await call(ctx.options, "POST", "/v1/desks/register", { body });
      assert.equal(status, 201, `register answered ${status}`);
      assert.ok(json.desk_id && json.desk_token, "register returned no credential");
      ctx.deskId = json.desk_id;
      ctx.deskToken = json.desk_token;
      return `desk ${ctx.deskId.slice(0, 8)} on plan ${json.plan}`;
    },
  },
  {
    name: "desk-connect",
    async run(ctx) {
      ctx.desk = new Peer("desk", ctx.options);
      const id = await ctx.desk.open(ctx.deskId, ctx.deskToken);
      const welcome = await ctx.desk.expect("welcome", id);
      assert.equal(welcome.payload.role, "desk");
      return "desk socket authenticated";
    },
  },
  {
    name: "pairing-code",
    async run(ctx) {
      const route = `/v1/desks/${ctx.deskId}/pairings`;
      const { status, json } = await call(ctx.options, "POST", route, { token: ctx.deskToken });
      assert.equal(status, 201, `pairings answered ${status}`);
      assert.ok(json.code, "no pairing code issued");
      ctx.code = json.code;
      return `code issued, valid for ${Math.round((json.expires_at - Date.now()) / 1000)}s`;
    },
  },
  {
    name: "pair",
    async run(ctx) {
      const body = { desk_id: ctx.deskId, code: ctx.code, device_name: ctx.options.deviceName };
      const { status, json } = await call(ctx.options, "POST", "/v1/pair", { body });
      assert.equal(status, 201, `pair answered ${status}`);
      ctx.viewerId = json.viewer_id;
      ctx.viewerToken = json.viewer_token;
      return `viewer ${ctx.viewerId.slice(0, 8)} paired`;
    },
  },
  {
    name: "viewer-connect",
    async run(ctx) {
      ctx.viewer = new Peer("viewer", ctx.options);
      const id = await ctx.viewer.open(ctx.deskId, ctx.viewerToken);
      const welcome = await ctx.viewer.expect("welcome", id);
      assert.equal(welcome.payload.desk_online, true, "desk is connected but welcome says offline");
      return `welcome: ${welcome.payload.viewer_count} viewer(s), desk online`;
    },
  },
  {
    name: "event-fanout",
    async run(ctx) {
      const marker = crypto.randomUUID();
      ctx.desk.send("event", { event: "desk:e2e", payload: { marker } });
      const seen = await ctx.viewer.expect("event");
      assert.equal(seen.payload.payload.marker, marker, "the viewer got a different event");
      return "desk event reached the viewer verbatim";
    },
  },
  {
    name: "command-round-trip",
    async run(ctx) {
      const id = ctx.viewer.send("command", { name: "set_limits", args: { sit_min: 45 } });
      const forwarded = await ctx.desk.expect("command", id);
      assert.equal(forwarded.payload.viewer_id, ctx.viewerId, "relay stamped the wrong viewer");
      ctx.desk.send("command_result", { command_id: id, ok: true, error: null });
      const result = await ctx.viewer.expect("command_result", id);
      assert.equal(result.payload.ok, true, "the result came back not-ok");
      return "viewer -> desk -> viewer, one reply to one phone";
    },
  },
  {
    name: "viewers-list",
    async run(ctx) {
      const route = `/v1/desks/${ctx.deskId}/viewers`;
      const { status, json } = await call(ctx.options, "GET", route, { token: ctx.deskToken });
      assert.equal(status, 200, `viewers answered ${status}`);
      const row = json.find((v) => v.viewer_id === ctx.viewerId);
      assert.ok(row, "the paired viewer is missing from the list");
      assert.equal(row.online, true, "the connected viewer is listed as offline");
      return `${json.length} paired device(s), the e2e one online`;
    },
  },
  {
    name: "revoke",
    async run(ctx) {
      const route = `/v1/desks/${ctx.deskId}/viewers/${ctx.viewerId}`;
      const { status } = await call(ctx.options, "DELETE", route, { token: ctx.deskToken });
      assert.equal(status, 204, `revoke answered ${status}`);
      return "viewer revoked";
    },
  },
  {
    name: "disable",
    async run(ctx) {
      const route = `/v1/desks/${ctx.deskId}`;
      const { status } = await call(ctx.options, "DELETE", route, { token: ctx.deskToken });
      assert.equal(status, 204, `disable answered ${status}`);
      return "desk removed, room closed";
    },
  },
];

/** Runs every step, printing one line each. Resolves to the process exit code. */
export async function run(options, log = console.log) {
  const ctx = { options };
  let passed = 0;
  try {
    for (const step of STEPS) {
      const detail = await step.run(ctx);
      passed += 1;
      log(`  ok   ${step.name.padEnd(20)} ${detail}`);
    }
  } catch (cause) {
    log(`  FAIL ${STEPS[passed].name.padEnd(20)} ${cause.message}`);
    return 1;
  } finally {
    ctx.desk?.close();
    ctx.viewer?.close();
  }

  // Zero executed steps is a failure, whatever the loop's exit code says.
  if (passed !== STEPS.length) {
    log(`relay-e2e: ${passed}/${STEPS.length} steps ran — that is not a pass`);
    return 1;
  }
  log(`relay-e2e: ${passed}/${STEPS.length} steps passed against ${options.relayUrl}`);
  return 0;
}

export async function main(argv = process.argv.slice(2), log = console.log) {
  const parsed = parseArgs(argv);
  if (parsed.help) {
    log(USAGE);
    return 0;
  }
  if (!parsed.ok) {
    console.error(`relay-e2e: ${parsed.error}\n\n${USAGE}`);
    return 2;
  }
  return run(parsed.options, log);
}

const invokedDirectly =
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url));

if (invokedDirectly) process.exit(await main());
