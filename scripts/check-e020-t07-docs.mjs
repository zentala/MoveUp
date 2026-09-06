#!/usr/bin/env node
// Verifies E020-T07: ADR 015 (pure ergo engine) exists, is non-empty, is
// reachable from both index documents, and describes what the code does.
//
// The last part matters most. An ADR that nothing links to is invisible, and an
// ADR that outlived its code is worse than none — so every doc claim below is
// grounded in a symbol the engine must still define. An empty run is a failure,
// not a pass.
import { existsSync, readFileSync, statSync } from "node:fs";

const ADR = ".arch/ADR/015-pure-ergo-engine.md";
const ARCH = ".arch/ARCHITECTURE.md";
const CLAUDE = "CLAUDE.md";
const DECISIONS = ".plan/decisions.jsonl";

const SOURCES = {
  manager: "src-tauri/src/session_manager.rs",
  reading: "src-tauri/src/session_reading.rs",
  daily: "src-tauri/src/session_daily.rs",
  breaks: "src-tauri/src/session_breaks.rs",
  persistence: "src-tauri/src/session_persistence.rs",
  tray: "src-tauri/src/tray_controller.rs",
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

// --- 1. The ADR itself -------------------------------------------------------
const adr = read(ADR);
check("ADR is non-empty", adr !== null && statSync(ADR).size > 1000, `${ADR} is missing or shorter than 1000 bytes`);
if (adr) {
  check("ADR is accepted", /^- \*\*Status\*\*: accepted/m.test(adr), `${ADR} has no "Status: accepted" line`);
  check("ADR names its epic", /E020/.test(adr), `${ADR} does not name epic E020`);
  for (const section of ["Context", "Decision", "Alternatives", "Consequences"]) {
    check(`ADR has ## ${section}`, new RegExp(`^## ${section}`, "m").test(adr), `${ADR} has no "## ${section}" section`);
  }
}

// --- 2. Reachability: an unlinked ADR is an invisible ADR ---------------------
const arch = read(ARCH);
const claude = read(CLAUDE);
check(
  "ARCHITECTURE.md links the ADR",
  arch !== null && /ADR\/015-pure-ergo-engine\.md/.test(arch),
  `${ARCH} does not link ${ADR}`,
);
check(
  "CLAUDE.md links the ADR",
  claude !== null && /015-pure-ergo-engine\.md/.test(claude),
  `${CLAUDE} does not link ${ADR}`,
);
check(
  "ADR 008 cross-references the ADR",
  /015-pure-ergo-engine\.md/.test(read(".arch/ADR/008-proportional-break-credit.md") ?? ""),
  "ADR 008 has no cross-reference to ADR 015",
);
check(
  "ADR 009 cross-references the ADR",
  /015-pure-ergo-engine\.md/.test(read(".arch/ADR/009-day-break-credit.md") ?? ""),
  "ADR 009 has no cross-reference to ADR 015",
);

// --- 3. Decision D3 is recorded ---------------------------------------------
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
  check("D3 recorded", ids.includes("E020-D3"), `${DECISIONS} has no entry with id "E020-D3"`);
}

// --- 4. Grounding: the doc claims must still be true of the code -------------
const src = {};
for (const [key, path] of Object.entries(SOURCES)) {
  src[key] = read(path);
}

check(
  "clock is injectable",
  /fn on_reading_at\(/.test(src.reading ?? "") && /fn check_daily_reset_at\(/.test(src.daily ?? ""),
  "on_reading_at / check_daily_reset_at no longer exist — the ADR's clock-injection claim is stale",
);
check(
  "limits live on the manager, not in state",
  /pub limits: Limits/.test(src.manager ?? "") && /fn set_limits\(/.test(src.manager ?? ""),
  "SessionManager no longer holds `limits` with a set_limits() — the ADR's config claim is stale",
);
check(
  "engine builds the policy input",
  /fn policy_input\(/.test(src.manager ?? ""),
  "SessionManager::policy_input no longer exists — the ADR's derived-values claim is stale",
);
check(
  "tray does not re-derive the standing lap",
  !/fn compute_standing_lap/.test(src.tray ?? ""),
  "tray_controller.rs defines compute_standing_lap again — the ADR's tray claim is stale",
);
check(
  "persistence is versioned",
  /struct PersistedEngineState/.test(src.persistence ?? "") && /schema_version/.test(src.persistence ?? ""),
  "PersistedEngineState / schema_version are gone — the ADR's persistence claim is stale",
);
check(
  "day break credit is its own function",
  /fn apply_day_break_reset\(/.test(src.breaks ?? ""),
  "apply_day_break_reset no longer exists — ADR 009's cross-reference is stale",
);
for (const prefix of ["[break:credit]", "[break:day]", "[break:hourly]"]) {
  check(
    `break log prefix ${prefix} documented`,
    (src.breaks ?? "").includes(prefix),
    `session_breaks.rs's module doc comment no longer names ${prefix}`,
  );
}

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
console.log(`PASS: ${checks}/${checks} checks — ADR 015 exists, is linked, and matches the code`);
process.exit(0);
