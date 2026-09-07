#!/usr/bin/env node
// Verifies E021-T09: ADR 020 (health source inlet) and ADR 021 (voice in on the
// phone, watch as glance) exist, are reachable from every index document, and
// still describe what the code does.
//
// The last part matters most. An ADR that nothing links to is invisible, and an
// ADR that outlived its code is worse than none — so every doc claim below is
// grounded in a symbol the implementation must still define. An empty run is a
// failure, not a pass.
//
// Modelled on check-e020-t07-docs.mjs.
import { existsSync, readFileSync, statSync } from "node:fs";

const ADR_HEALTH = ".arch/ADR/020-health-source-inlet.md";
const ADR_VOICE = ".arch/ADR/021-voice-in-on-phone-watch-as-glance.md";
const ADR_FIT = ".arch/ADR/012-google-fit-integration.md";
const ARCH = ".arch/ARCHITECTURE.md";
const UX = ".arch/UX-FLOW.md";
const CLAUDE = "CLAUDE.md";
const CONTRIBUTING = "CONTRIBUTING.md";
const REMOTE_DOC = "docs/REMOTE_DISPLAY.md";
const PROJECT = "PROJECT.xml";
const DECISIONS = ".plan/decisions.jsonl";
const BACKLOG = ".plan/BACKLOG.md";

const SOURCES = {
  healthSource: "src-tauri/src/health_source.rs",
  healthModels: "src-tauri/src/health_models.rs",
  remoteAuth: "src-tauri/src/remote_auth.rs",
  routesHealth: "src-tauri/src/remote_routes_health.rs",
  routesVoice: "src-tauri/src/remote_routes_voice.rs",
  intent: "src-tauri/src/voice_intent.rs",
  voiceAi: "src-tauri/src/voice_ai.rs",
  webhook: "src-tauri/src/notify_webhook.rs",
  voiceNotes: "src-tauri/src/db_voice_notes.rs",
  commandsHealth: "src-tauri/src/commands_health.rs",
  server: "src-tauri/src/remote_server.rs",
  widget: "src/components/HealthWidget.tsx",
  capture: "src/components/VoiceCapture.tsx",
};

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
for (const [name, path] of [["ADR 020", ADR_HEALTH], ["ADR 021", ADR_VOICE]]) {
  const adr = read(path);
  check(`${name} is non-empty`, adr !== null && statSync(path).size > 1000, `${path} is missing or shorter than 1000 bytes`);
  if (!adr) continue;
  check(`${name} is accepted`, /^- \*\*Status\*\*: accepted/m.test(adr), `${path} has no "Status: accepted" line`);
  check(`${name} names its epic`, /E021/.test(adr), `${path} does not name epic E021`);
  for (const section of ["Context", "Decision", "Alternatives", "Consequences"]) {
    check(`${name} has ## ${section}`, new RegExp(`^## ${section}`, "m").test(adr), `${path} has no "## ${section}" section`);
  }
}

// --- 2. ADR 012 records that it was partly superseded ------------------------
const fit = read(ADR_FIT);
check(
  "ADR 012 marks itself partially superseded",
  /partially superseded/i.test(fit ?? "") && /020-health-source-inlet\.md/.test(fit ?? ""),
  `${ADR_FIT} does not say it is partially superseded by ADR 020`,
);

// --- 3. Reachability: an unlinked ADR is an invisible ADR --------------------
const arch = read(ARCH);
const ux = read(UX);
const claude = read(CLAUDE);
const contributing = read(CONTRIBUTING);
const project = read(PROJECT);
const remoteDoc = read(REMOTE_DOC);

for (const [label, body, path] of [
  ["ARCHITECTURE.md", arch, ARCH],
  ["CLAUDE.md", claude, CLAUDE],
  ["CONTRIBUTING.md", contributing, CONTRIBUTING],
]) {
  check(`${label} links ADR 020`, /020-health-source-inlet\.md/.test(body ?? ""), `${path} does not link ${ADR_HEALTH}`);
  check(`${label} links ADR 021`, /021-voice-in-on-phone-watch-as-glance\.md/.test(body ?? ""), `${path} does not link ${ADR_VOICE}`);
}
check("UX-FLOW.md links ADR 021", /021-voice-in-on-phone-watch-as-glance\.md/.test(ux ?? ""), `${UX} does not link ${ADR_VOICE}`);
check("UX-FLOW.md links ADR 020", /020-health-source-inlet\.md/.test(ux ?? ""), `${UX} does not link ${ADR_HEALTH}`);
check("PROJECT.xml names both ADRs", /020-health-source-inlet/.test(project ?? "") && /021-voice-in-on-phone/.test(project ?? ""), `${PROJECT} does not name both new ADRs`);
check("ADR 021 cross-references ADR 020", /020-health-source-inlet\.md/.test(read(ADR_VOICE) ?? ""), "ADR 021 has no cross-reference to ADR 020");
check("ADR 020 cross-references ADR 012", /012-google-fit-integration\.md/.test(read(ADR_HEALTH) ?? ""), "ADR 020 has no cross-reference to ADR 012");

// --- 4. Decisions D1..D6 are recorded ---------------------------------------
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
  for (const n of [1, 2, 3, 4, 5, 6]) {
    check(`E021-D${n} recorded`, ids.includes(`E021-D${n}`), `${DECISIONS} has no entry with id "E021-D${n}"`);
  }
}

// --- 5. Backlog carries the two follow-ups T09 owes -------------------------
const backlog = read(BACKLOG);
check(
  "backlog files the Health Connect companion candidate",
  /Health Connect companion/i.test(backlog ?? ""),
  `${BACKLOG} has no Android Health Connect companion candidate epic`,
);
check(
  "backlog files creating epics/INDEX.md",
  /epics\/INDEX\.md/.test(backlog ?? ""),
  `${BACKLOG} has no entry for creating .plan/epics/INDEX.md`,
);

// --- 6. Grounding: the doc claims must still be true of the code -------------
const src = {};
for (const [key, path] of Object.entries(SOURCES)) {
  src[key] = read(path);
}

check(
  "the health seam is a trait",
  /pub trait HealthSource/.test(src.healthSource ?? ""),
  "trait HealthSource no longer exists — ADR 020's whole decision is stale",
);
check(
  "the trait still has the three documented methods",
  /fn id\(&self\)/.test(src.healthSource ?? "") &&
    /async fn view\(&self\)/.test(src.healthSource ?? "") &&
    /async fn refresh\(&self\)/.test(src.healthSource ?? ""),
  "HealthSource no longer declares id/view/refresh — the docs describe a different trait",
);
check(
  "sources are merged by an aggregator",
  /pub struct HealthAggregator/.test(src.healthSource ?? "") && /fn merge\(/.test(src.healthSource ?? ""),
  "HealthAggregator / merge are gone — the 'merged by freshness' claim is stale",
);
check(
  "health DTOs are the source-agnostic ones",
  /pub struct HealthSnapshot/.test(src.healthModels ?? "") && /pub struct HealthView/.test(src.healthModels ?? ""),
  "HealthSnapshot / HealthView no longer exist — the docs name DTOs the code does not have",
);
check(
  "writes are gated by one function",
  /pub fn require_token\(/.test(src.remoteAuth ?? ""),
  "remote_auth::require_token no longer exists — ADR 020's write-gate claim is stale",
);
check(
  "the gate compares in constant time",
  /pub fn constant_time_eq\(/.test(src.remoteAuth ?? ""),
  "constant_time_eq is gone — the docs promise a constant-time compare the code no longer does",
);
check(
  "the token comes from the documented env var",
  /"DESK_REMOTE_TOKEN"/.test(src.remoteAuth ?? ""),
  "remote_auth.rs no longer names DESK_REMOTE_TOKEN — every doc naming that key is stale",
);
check(
  "the push inlet exists with the documented caps",
  /pub struct PushHealthSource/.test(src.routesHealth ?? "") &&
    /MAX_BODY_BYTES: usize = 1024/.test(src.routesHealth ?? "") &&
    /STALE_AFTER_MS/.test(src.routesHealth ?? ""),
  "PushHealthSource / the 1 KiB cap / STALE_AFTER_MS changed — the documented push contract is stale",
);
check(
  "the voice inlet exists with the documented caps",
  /MAX_BODY_BYTES: usize = 4096/.test(src.routesVoice ?? "") &&
    /MAX_TRANSCRIPT_CHARS: usize = 2000/.test(src.routesVoice ?? ""),
  "the voice route's 4 KiB / 2000-char limits changed — docs/REMOTE_DISPLAY.md is stale",
);
check(
  "both inlets are mounted on the one server",
  /remote_routes_health::routes\(\)/.test(src.server ?? "") && /remote_routes_voice::routes\(\)/.test(src.server ?? ""),
  "build_router no longer merges both inlet routers — the 'one server, four routes' table is stale",
);
check(
  "intent is a closed enum with the four documented variants",
  /pub enum Intent/.test(src.intent ?? "") &&
    /Snooze\(u16\)/.test(src.intent ?? "") &&
    /WalkStart/.test(src.intent ?? "") &&
    /WalkEnd/.test(src.intent ?? ""),
  "enum Intent no longer has Snooze(u16)/Note/WalkStart/WalkEnd — the UX table is stale",
);
check(
  "snooze minutes are clamped to the documented range",
  /MIN_SNOOZE_MINS: u16 = 1/.test(src.intent ?? "") &&
    /MAX_SNOOZE_MINS: u16 = 180/.test(src.intent ?? "") &&
    /DEFAULT_SNOOZE_MINS: u16 = 5/.test(src.intent ?? ""),
  "the 1..180 clamp / 5-minute default changed — the docs quote numbers the parser does not use",
);
check(
  "the VOICE log line is truncated to the documented width",
  /LOG_TRANSCRIPT_CHARS: usize = 80/.test(src.intent ?? ""),
  "LOG_TRANSCRIPT_CHARS is no longer 80 — the events.log format documented in UX-FLOW is stale",
);
check(
  "the voice pipeline still writes a note row",
  /pub fn insert_voice_note\(/.test(src.voiceNotes ?? "") && /pub fn list_voice_notes\(/.test(src.voiceNotes ?? ""),
  "db_voice_notes no longer inserts/lists — the 'every sentence lands in three places' claim is stale",
);
check(
  "nothing on the voice path touches the session engine",
  !/session_manager|session_reading|SessionState\b/.test(src.routesVoice ?? ""),
  "remote_routes_voice.rs now references the session engine — ADR 021 / E021-D3 say it must not",
);
check(
  "the AI reply is a BYOK OpenRouter call",
  /pub async fn reply\(/.test(src.voiceAi ?? "") && /"OPENROUTER_API_KEY"/.test(src.voiceAi ?? ""),
  "voice_ai::reply / the OPENROUTER_API_KEY env name changed — the BYOK claim (E021-D4) is stale",
);
check(
  "the outbound push is a webhook notifier",
  /pub struct WebhookNotifier/.test(src.webhook ?? "") && /fn from_env_or_config\(/.test(src.webhook ?? ""),
  "WebhookNotifier / from_env_or_config are gone — the phone/watch notification path documented in UX-FLOW §9 is stale",
);
check(
  "health IPC uses the source-agnostic command names",
  /pub async fn get_health_today\(/.test(src.commandsHealth ?? "") &&
    /pub async fn refresh_health_now\(/.test(src.commandsHealth ?? ""),
  "get_health_today / refresh_health_now are gone — every doc naming them is stale",
);
check(
  "the vendor-named IPC module is really gone",
  !existsSync("src-tauri/src/commands_google_fit.rs"),
  "commands_google_fit.rs is back — ADR 020 claims it was deleted",
);
check(
  "the widget is named for the metric, not the vendor",
  existsSync(SOURCES.widget) && !existsSync("src/components/StepsWidget.tsx"),
  "HealthWidget.tsx is missing or StepsWidget.tsx is back — UX-FLOW §2.9's rename claim is stale",
);
check(
  "the widget carries the dated Google Fit warning",
  /Google Fit ends late 2026/.test(src.widget ?? ""),
  "the 'Google Fit ends late 2026' hint is gone from HealthWidget — E021-D6 is stale",
);
check(
  "voice capture gates the mic on both conditions",
  /SpeechRecognition/.test(src.capture ?? "") && /permissions/.test(src.capture ?? ""),
  "VoiceCapture no longer feature-detects SpeechRecognition and the mic permission — E021-D5 is stale",
);

// --- 7. The user docs actually carry the contracts ---------------------------
check(
  "REMOTE_DISPLAY documents the health inlet",
  /POST \/display\/health/.test(remoteDoc ?? ""),
  `${REMOTE_DOC} does not document POST /display/health`,
);
check(
  "REMOTE_DISPLAY documents the voice inlet",
  /POST \/display\/voice/.test(remoteDoc ?? ""),
  `${REMOTE_DOC} does not document POST /display/voice`,
);
check(
  "REMOTE_DISPLAY says an unset token means closed",
  /503/.test(remoteDoc ?? "") && /DESK_REMOTE_TOKEN/.test(remoteDoc ?? ""),
  `${REMOTE_DOC} does not explain that an unset DESK_REMOTE_TOKEN answers 503`,
);
check(
  "CLAUDE.md replaced the vendor-named section",
  /^## Health sources/m.test(claude ?? "") && !/^## Google Fit Integration/m.test(claude ?? ""),
  `${CLAUDE} still leads with "Google Fit Integration" instead of "Health sources"`,
);
check(
  "UX-FLOW has the voice section",
  /^## 12\. Voice Dictation/m.test(ux ?? ""),
  `${UX} has no "## 12. Voice Dictation" section`,
);
check(
  "ARCHITECTURE has the inlet sections",
  /^## Health Sources and the LAN Inlet/m.test(arch ?? "") && /^## Voice Inlet/m.test(arch ?? ""),
  `${ARCH} is missing the health-inlet or voice-inlet section`,
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
console.log(`PASS: ${checks}/${checks} checks — ADRs 020 and 021 exist, are linked, and match the code`);
process.exit(0);
