#!/usr/bin/env node
/**
 * mint-license.mjs — issue one licence key and store its hash in D1.
 *
 * `stdout` carries the key and nothing else, so the caller can use it directly:
 *
 *   node scripts/mint-license.mjs --local --max-viewers 5
 *   node scripts/relay-e2e.mjs --license "$(node scripts/mint-license.mjs --local)"
 *
 * Everything else — the hash, the SQL, warnings — goes to `stderr`.
 *
 * The key is printed **after** the insert succeeds. A key that was never stored
 * entitles nobody, so printing one and exiting 0 would hand out a credential
 * that silently does not work.
 *
 * Usage:
 *   --local           against the local `wrangler dev` D1 (default)
 *   --remote          against the deployed database (T16)
 *   --dry-run         print the SQL to stderr, write nothing, print no key
 *   --plan <slug>     default `founder`
 *   --max-desks <n>   default 1
 *   --max-viewers <n> default 5
 *   --days <n>        expire after n days; omitted means never
 */
import { createHash, randomBytes } from "node:crypto";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const RELAY_ROOT = path.resolve(HERE, "..");
const DATABASE = "moveup-relay";
const PREFIX = "mu_lic_";

const sha256Hex = (s) => createHash("sha256").update(s).digest("hex");
const die = (message) => {
  console.error(`mint-license: ${message}`);
  process.exit(2);
};

// ─── Arguments ──────────────────────────────────────────────────────────────

const argv = process.argv.slice(2);
const has = (flag) => argv.includes(flag);
function option(flag, fallback) {
  const at = argv.indexOf(flag);
  return at === -1 ? fallback : argv[at + 1];
}

function positiveInt(raw, what) {
  const value = Number(raw);
  if (!Number.isInteger(value) || value <= 0) die(`${what} must be a positive integer`);
  return value;
}

const plan = String(option("--plan", "founder"));
// The values below are interpolated into SQL, so each one is validated rather
// than escaped: a slug and three integers cannot carry a quote.
if (!/^[a-z0-9_-]{1,32}$/.test(plan)) die("--plan must be a lower-case slug");
const maxDesks = positiveInt(option("--max-desks", "1"), "--max-desks");
const maxViewers = positiveInt(option("--max-viewers", "5"), "--max-viewers");
const days = has("--days") ? positiveInt(option("--days"), "--days") : null;

const now = Date.now();
const expiresAt = days === null ? "NULL" : String(now + days * 86_400_000);

const key = PREFIX + randomBytes(32).toString("base64url");
const sql =
  "INSERT INTO licenses (key_hash, plan, max_desks, max_viewers, expires_at, created_at) VALUES " +
  `('${sha256Hex(key)}', '${plan}', ${maxDesks}, ${maxViewers}, ${expiresAt}, ${now});`;

console.error(`plan=${plan} max_desks=${maxDesks} max_viewers=${maxViewers} expires_at=${expiresAt}`);

if (has("--dry-run")) {
  console.error(sql);
  console.error("dry-run: nothing was written, and no key was printed.");
  process.exit(0);
}

// ─── Insert ─────────────────────────────────────────────────────────────────

// Resolved through the package manifest rather than the bin subpath: wrangler
// declares `exports`, so `require.resolve("wrangler/bin/wrangler.js")` is a
// package-path error under pnpm's strict layout, not a missing install.
const require = createRequire(path.join(RELAY_ROOT, "package.json"));
let wranglerBin;
try {
  const manifestPath = require.resolve("wrangler/package.json");
  const { bin } = require(manifestPath);
  const entry = typeof bin === "string" ? bin : bin?.wrangler;
  if (!entry) die("wrangler declares no bin entry");
  wranglerBin = path.resolve(path.dirname(manifestPath), entry);
} catch (cause) {
  die(`wrangler is not installed — run \`pnpm install\` in relay/ (${cause.message})`);
}

const target = has("--remote") ? "--remote" : "--local";
const result = spawnSync(
  process.execPath,
  [wranglerBin, "d1", "execute", DATABASE, target, "--command", sql],
  // wrangler's own chatter is captured and re-emitted on stderr: this script's
  // stdout must carry the key and nothing else, or `$(mint-license --local)`
  // feeds a banner into `--license`.
  { cwd: RELAY_ROOT, stdio: ["ignore", "pipe", "inherit"], encoding: "utf8" },
);

if (result.error) die(`could not run wrangler: ${result.error.message}`);
if (result.stdout) console.error(result.stdout.trimEnd());
if (result.status !== 0) {
  console.error(`\nNOT MINTED: wrangler exited ${result.status}; no key is printed.`);
  process.exit(3);
}

// Only now — the row exists, so the key means something.
process.stdout.write(`${key}\n`);
