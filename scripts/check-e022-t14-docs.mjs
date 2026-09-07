#!/usr/bin/env node
// Verifies E022-T14: ADR 022 (relay on Cloudflare Durable Objects) and ADR 023
// (pairing code → device tokens) exist, are reachable from every index document,
// and still describe what the code does.
//
// The last part matters most. An ADR nothing links to is invisible, and an ADR
// that outlived its code is worse than none — so every doc claim below is
// anchored to a symbol or a number the implementation must still carry. Numbers
// in prose (5..240 minutes, 5-minute code, 4403) are the ones that rot silently,
// so they are checked against the source, not trusted.
//
// An empty run is a failure, not a pass.
//
// Modelled on check-e021-t09-docs.mjs.
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";

const ADR_RELAY = ".arch/ADR/022-relay-on-cloudflare-durable-objects.md";
const ADR_AUTH = ".arch/ADR/023-pairing-code-device-token-auth.md";
const ARCH = ".arch/ARCHITECTURE.md";
const CLAUDE = "CLAUDE.md";
const REMOTE_DOC = "docs/REMOTE_DISPLAY.md";
const PRIVACY = "docs/PRIVACY.md";
const PROJECT = "PROJECT.xml";
const DECISIONS = ".plan/decisions.jsonl";
const BACKLOG = ".plan/BACKLOG.md";
const BUSINESS = ".plan/BUSINESS_CONTEXT.md";

const SOURCES = {
  protocol: "src-tauri/src/remote_protocol.rs",
  client: "src-tauri/src/relay_client.rs",
  status: "src-tauri/src/relay_status.rs",
  auth: "src-tauri/src/relay_auth.rs",
  deskCommands: "src-tauri/src/relay_commands.rs",
  ipc: "src-tauri/src/commands_relay.rs",
  config: "src-tauri/src/config.rs",
  server: "src-tauri/src/remote_server.rs",
  setup: "src-tauri/src/setup_helpers.rs",
  tokens: "relay/src/auth/tokens.ts",
  room: "relay/src/room/desk-room.ts",
  pairing: "relay/src/room/pairing.ts",
  migration: "relay/migrations/0001_init.sql",
  transportRelay: "src/remote/transports/relay.ts",
  transportLan: "src/remote/transports/lan.ts",
  settings: "src/components/settings/RemoteSection.tsx",
  pairScreen: "src/remote/PairScreen.tsx",
};

const FIXTURE_DIR = "tests/fixtures/relay-protocol";

const problems = [];
let checks = 0;

function check(label, ok, detail) {
  checks += 1;
  if (!ok) problems.push(`${label}: ${detail}`);
}

function read(path) {
  if (!existsSync(path)) {
    problems.push(`${path} is missing`);
    checks += 1;
    return null;
  }
  return readFileSync(path, "utf8");
}

// --- 1. The ADRs themselves --------------------------------------------------
for (const [name, path] of [["ADR 022", ADR_RELAY], ["ADR 023", ADR_AUTH]]) {
  const adr = read(path);
  check(`${name} is non-empty`, adr !== null && statSync(path).size > 1000, `${path} is missing or shorter than 1000 bytes`);
  if (!adr) continue;
  check(`${name} is accepted`, /^- \*\*Status\*\*: accepted/m.test(adr), `${path} has no "Status: accepted" line`);
  check(`${name} names its epic`, /E022/.test(adr), `${path} does not name epic E022`);
  for (const section of ["Context", "Decision", "Alternatives", "Consequences"]) {
    check(`${name} has ## ${section}`, new RegExp(`^## ${section}`, "m").test(adr), `${path} has no "## ${section}" section`);
  }
}

const adrRelay = read(ADR_RELAY) ?? "";
const adrAuth = read(ADR_AUTH) ?? "";

check("ADR 022 cross-references ADR 023", /023-pairing-code-device-token-auth\.md/.test(adrRelay), "ADR 022 does not link ADR 023");
check("ADR 023 cross-references ADR 022", /022-relay-on-cloudflare-durable-objects\.md/.test(adrAuth), "ADR 023 does not link ADR 022");
check(
  "ADR 022 places itself against the LAN kiosk decision",
  /001-remote-display-web-kiosk\.md/.test(adrRelay),
  "ADR 022 does not reference ADR 001, the decision it extends",
);
check(
  "ADR 022 names the open-core tier line",
  /005-open-core-software-model\.md/.test(adrRelay),
  "ADR 022 does not reference ADR 005 — the relay is the paid half and should say so",
);
check(
  "ADR 023 names the shared-secret gate it narrows",
  /020-health-source-inlet\.md/.test(adrAuth),
  "ADR 023 does not reference ADR 020, the DESK_REMOTE_TOKEN gate it supersedes for the relay path",
);

// --- 2. Reachability: an unlinked ADR is an invisible ADR --------------------
const arch = read(ARCH);
const claude = read(CLAUDE);
const remoteDoc = read(REMOTE_DOC);
const privacy = read(PRIVACY);
const project = read(PROJECT);
const business = read(BUSINESS);

for (const [label, body, path] of [
  ["ARCHITECTURE.md", arch, ARCH],
  ["CLAUDE.md", claude, CLAUDE],
  ["BUSINESS_CONTEXT.md", business, BUSINESS],
  ["PRIVACY.md", privacy, PRIVACY],
  ["REMOTE_DISPLAY.md", remoteDoc, REMOTE_DOC],
]) {
  check(`${label} links ADR 022`, /022-relay-on-cloudflare-durable-objects\.md/.test(body ?? ""), `${path} does not link ${ADR_RELAY}`);
  check(`${label} links ADR 023`, /023-pairing-code-device-token-auth\.md/.test(body ?? ""), `${path} does not link ${ADR_AUTH}`);
}
check(
  "PROJECT.xml knows the relay exists",
  /relay_client\.rs/.test(project ?? "") && /remote_protocol\.rs/.test(project ?? ""),
  `${PROJECT} does not name relay_client.rs and remote_protocol.rs`,
);

// --- 3. The documents carry the sections they are supposed to ---------------
check(
  "ARCHITECTURE has the two-transport section",
  /^## Remote Display: two transports, one contract/m.test(arch ?? ""),
  `${ARCH} has no "## Remote Display: two transports, one contract" section`,
);
check(
  "ARCHITECTURE records both decisions in the key-decisions table",
  /E022, ADR 022/.test(arch ?? "") && /E022, ADR 023/.test(arch ?? ""),
  `${ARCH}'s Key Architectural Decisions table has no E022 rows`,
);
check(
  "CLAUDE.md's Remote Display section covers the relay, not just the LAN server",
  /^## Remote Display \(Phone Dashboard\)/m.test(claude ?? "") &&
    /relay_client\.rs/.test(claude ?? "") &&
    /pairing code/i.test(claude ?? ""),
  `${CLAUDE}'s Remote Display section still describes only the LAN server`,
);
check(
  "CLAUDE.md states the LAN path stays read-only",
  /read-only/i.test(claude ?? "") && /remote_lan_enabled/.test(claude ?? ""),
  `${CLAUDE} does not say the LAN path is read-only and gated by remote_lan_enabled`,
);

// REMOTE_DISPLAY.md leads with pairing: the paid, private path is the shorter
// road, and a doc that opens with "run ipconfig" teaches the wrong one first.
const pairHeading = (remoteDoc ?? "").indexOf("## Pair a phone");
const lanHeading = (remoteDoc ?? "").indexOf("## On your own Wi-Fi");
check(
  "REMOTE_DISPLAY documents pairing",
  pairHeading !== -1,
  `${REMOTE_DOC} has no "## Pair a phone" section`,
);
check(
  "REMOTE_DISPLAY documents the LAN path",
  lanHeading !== -1,
  `${REMOTE_DOC} has no "## On your own Wi-Fi" section`,
);
check(
  "REMOTE_DISPLAY puts pairing before the LAN path",
  pairHeading !== -1 && lanHeading !== -1 && pairHeading < lanHeading,
  `${REMOTE_DOC} still leads with the LAN setup instead of pairing`,
);
check(
  "REMOTE_DISPLAY keeps the Fully Kiosk instructions",
  /Fully Kiosk/.test(remoteDoc ?? ""),
  `${REMOTE_DOC} lost the Fully Kiosk Browser setup`,
);
check(
  "REMOTE_DISPLAY still documents both LAN inlets",
  /POST \/display\/health/.test(remoteDoc ?? "") && /POST \/display\/voice/.test(remoteDoc ?? ""),
  `${REMOTE_DOC} lost the health or voice inlet contract`,
);
check(
  "PRIVACY has a relay section",
  /^## Pairing a phone through the relay/m.test(privacy ?? ""),
  `${PRIVACY} has no "## Pairing a phone through the relay" section`,
);
check(
  "PRIVACY says the relay keeps no history",
  /no history/i.test(privacy ?? "") && /relay\.desk\.zentala\.io/.test(privacy ?? ""),
  `${PRIVACY} does not name the relay host and state that it keeps no history`,
);
check(
  "PRIVACY no longer claims there is no MoveUp server",
  !/there is none/.test(privacy ?? ""),
  `${PRIVACY} still says MoveUp contacts no server of ours — the relay makes that false`,
);

// --- 4. Decisions D1..D5 are recorded ---------------------------------------
const decisions = read(DECISIONS);
if (decisions !== null) {
  const ids = decisions
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => {
      try {
        return JSON.parse(l).id;
      } catch {
        problems.push(`${DECISIONS} has a line that is not valid JSON`);
        return null;
      }
    });
  for (const n of [1, 2, 3, 4, 5]) {
    check(`E022-D${n} recorded`, ids.includes(`E022-D${n}`), `${DECISIONS} has no entry with id "E022-D${n}"`);
  }
}

// --- 5. The follow-up this epic deliberately did not do ---------------------
const backlog = read(BACKLOG);
check(
  "backlog files LAN pairing as the gap E022-D4 left open",
  /LAN pairing/i.test(backlog ?? "") && /Importance: Low, Points: 5/.test(backlog ?? ""),
  `${BACKLOG} has no "LAN pairing" entry with (Importance: Low, Points: 5)`,
);

// --- 6. Grounding: the doc claims must still be true of the code -------------
const src = {};
for (const [key, path] of Object.entries(SOURCES)) {
  src[key] = read(path);
}

check(
  "the envelope is defined once, in Rust",
  /pub struct Envelope<T>/.test(src.protocol ?? "") && /pub const PROTOCOL_VERSION: u32 = 1/.test(src.protocol ?? ""),
  "Envelope<T> / PROTOCOL_VERSION are gone — every doc describing the v1 envelope is stale",
);
check(
  "the close codes the docs quote are the ones the code defines",
  /CLOSE_UNAUTHENTICATED: u16 = 4401/.test(src.protocol ?? "") &&
    /CLOSE_UNENTITLED: u16 = 4402/.test(src.protocol ?? "") &&
    /CLOSE_REVOKED: u16 = 4403/.test(src.protocol ?? "") &&
    /CLOSE_REPLACED: u16 = 4409/.test(src.protocol ?? ""),
  "the 4401/4402/4403/4409 constants changed — the docs quote close codes the code does not send",
);
check(
  "the command allowlist still holds exactly the three documented names",
  /COMMAND_ALLOWLIST/.test(src.protocol ?? "") &&
    /name: "ack_alert"/.test(src.protocol ?? "") &&
    /name: "set_limits"/.test(src.protocol ?? "") &&
    /name: "switch_profile"/.test(src.protocol ?? ""),
  "COMMAND_ALLOWLIST no longer lists ack_alert/set_limits/switch_profile — ADR 023 and the user docs are stale",
);
check(
  "the limit bounds the docs print are the bounds the code enforces",
  /name: "sit_min",\s*min: 5,\s*max: 240/.test(src.protocol ?? "") &&
    /name: "stand_min",\s*min: 1,\s*max: 120/.test(src.protocol ?? ""),
  "the sit 5..240 / stand 1..120 bounds changed — docs/REMOTE_DISPLAY.md prints numbers the allowlist does not use",
);
check(
  "the fixtures that pin the contract exist and are not an empty set",
  existsSync(FIXTURE_DIR) && readdirSync(FIXTURE_DIR).filter((f) => f.endsWith(".json")).length > 0,
  `${FIXTURE_DIR} is missing or holds no .json fixture — an empty contract is not a contract`,
);
check(
  "the desktop connects outbound as one more broadcast subscriber",
  /ws_url\(/.test(src.client ?? "") && /PING_EVERY: Duration = Duration::from_secs\(25\)/.test(src.client ?? ""),
  "relay_client.rs no longer exposes ws_url / the 25 s ping — the outbound-only contract in the docs is stale",
);
check(
  "the terminal close codes really are terminal",
  /pub fn should_reconnect/.test(src.status ?? "") && /pub enum RelayState/.test(src.status ?? ""),
  "relay_status.rs lost should_reconnect / RelayState — the 'no silent retry on a dead credential' claim is stale",
);
check(
  "Settings can show every state the docs list",
  ["Disabled", "Connecting", "Online", "Unentitled", "Revoked", "Replaced", "Error"].every((s) =>
    new RegExp(`\\b${s}\\b`).test(src.status ?? ""),
  ),
  "RelayState no longer carries all seven documented states — the Settings table in REMOTE_DISPLAY.md is stale",
);
check(
  "the desk token goes to the OS credential store",
  /keyring::Entry/.test(src.auth ?? ""),
  "relay_auth.rs no longer uses keyring — ADR 023's 'secrets never in tauri-plugin-store' claim is stale",
);
check(
  "the relay host is a constant with an override",
  /RELAY_DEFAULT_URL: &str = "https:\/\/relay\.desk\.zentala\.io"/.test(src.auth ?? "") &&
    /pub fn effective_url/.test(src.auth ?? ""),
  "RELAY_DEFAULT_URL / effective_url changed — E022-D5 and every doc naming the host are stale",
);
check(
  "the three config fields the docs name exist",
  /pub relay_enabled: bool/.test(src.config ?? "") &&
    /pub relay_url: String/.test(src.config ?? "") &&
    /pub remote_lan_enabled: bool/.test(src.config ?? ""),
  "config.rs lost relay_enabled / relay_url / remote_lan_enabled — the Settings docs describe fields that do not exist",
);
check(
  "the relay is opt-in and the LAN default stays on",
  /relay_enabled: bool_false\(\)/.test(src.config ?? "") && /remote_lan_enabled: bool_true\(\)/.test(src.config ?? ""),
  "the defaults flipped — PRIVACY.md promises the relay is off by default and the LAN display keeps working",
);
check(
  "the LAN server wraps its events in the shared envelope",
  /remote_protocol::\{Envelope/.test(src.server ?? "") && /LAN_ENABLED_KEY/.test(src.server ?? ""),
  "remote_server.rs no longer uses the shared envelope or the LAN toggle key — 'one client, two transports' is stale",
);
check(
  "the LAN gate is applied where the server starts",
  /remote_lan_enabled/.test(src.setup ?? ""),
  "setup_helpers.rs no longer mentions remote_lan_enabled — the documented toggle may not gate anything",
);
check(
  "every remote command is executed through one checked entry point",
  /pub fn execute\(/.test(src.deskCommands ?? ""),
  "relay_commands::execute is gone — ADR 023's second-check-on-the-desk claim is stale",
);
check(
  "commands are logged the way the docs promise",
  /REMOTE DENIED/.test(src.deskCommands ?? "") && /viewer=/.test(src.deskCommands ?? ""),
  "the REMOTE / REMOTE DENIED events.log lines changed — the audit-trail claim is stale",
);
check(
  "the IPC surface the Settings docs describe exists",
  ["relay_register", "relay_start_pairing", "relay_list_viewers", "relay_revoke_viewer", "relay_disable", "get_relay_status"].every(
    (c) => new RegExp(`pub (async )?fn ${c}\\b`).test(src.ipc ?? ""),
  ),
  "commands_relay.rs no longer exposes all six documented commands — Settings → Remote access is documented on symbols that are gone",
);
check(
  "tokens carry the documented prefixes and are stored hashed",
  /desk: "mu_d_"/.test(src.tokens ?? "") &&
    /viewer: "mu_v_"/.test(src.tokens ?? "") &&
    /timingSafeEqualHex/.test(src.tokens ?? ""),
  "the mu_d_/mu_v_ prefixes or the constant-time compare are gone — ADR 023's token contract is stale",
);
check(
  "pairing codes are short-lived, single-desk and rate-limited as documented",
  /CODE_LENGTH = 8/.test(src.pairing ?? "") &&
    /DEFAULT_TTL_MS = 300_000/.test(src.pairing ?? "") &&
    /DEFAULT_LOCKOUT_MS = 900_000/.test(src.pairing ?? "") &&
    /DEFAULT_MAX_ATTEMPTS = 10/.test(src.pairing ?? ""),
  "the 8-char / 5-minute / 10-attempt / 15-minute pairing numbers changed — the user docs print numbers the relay does not use",
);
check(
  "the pairing alphabet excludes the ambiguous characters the docs promise",
  /CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"/.test(src.pairing ?? ""),
  "the pairing alphabet changed — REMOTE_DISPLAY.md tells the user there is no I, O, 0 or 1",
);
check(
  "the room is a Durable Object using the hibernation API",
  /export class DeskRoom/.test(src.room ?? "") && /acceptWebSocket/.test(src.room ?? ""),
  "DeskRoom / acceptWebSocket are gone — ADR 022's whole decision is stale",
);
check(
  "D1 holds exactly the three documented tables",
  ["licenses", "desks", "viewers"].every((t) => new RegExp(`CREATE TABLE IF NOT EXISTS ${t}\\b`).test(src.migration ?? "")),
  "the D1 migration no longer creates licenses/desks/viewers — the privacy claim about what the relay stores is stale",
);
check(
  "D1 stores hashes, not plaintext tokens",
  /token_hash/.test(src.migration ?? "") && !/\btoken TEXT\b/.test(src.migration ?? ""),
  "the schema has a plaintext token column — PRIVACY.md and ADR 023 both promise hashes only",
);
check(
  "the two transports differ exactly in control capability",
  /capabilities: TransportCapabilities = \{ control: true \}/.test(src.transportRelay ?? "") &&
    /control: false/.test(src.transportLan ?? ""),
  "the relay/LAN capability split changed — 'LAN is view-only, relay can control' is stale",
);
check(
  "the desktop pairing UI exists with the wording the docs quote",
  /Pair a phone/.test(src.settings ?? "") && /Paired phones/.test(src.settings ?? ""),
  "RemoteSection.tsx no longer shows 'Pair a phone' / 'Paired phones' — REMOTE_DISPLAY.md quotes buttons that are gone",
);
check(
  "the phone pairing screen exists",
  /Pair with your desk/.test(src.pairScreen ?? ""),
  "PairScreen.tsx changed its heading — the #/pair flow documented for the phone is stale",
);

// --- report ------------------------------------------------------------------
if (checks === 0) {
  console.error("FAIL: no checks ran — an empty run is not a pass");
  process.exit(1);
}
if (problems.length > 0) {
  console.error(`FAIL: ${problems.length} of ${checks} check(s) failed`);
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}
console.log(`PASS: ${checks}/${checks} checks — ADRs 022 and 023 exist, are linked, and match the code`);
process.exit(0);
