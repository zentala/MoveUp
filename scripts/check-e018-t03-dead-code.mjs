/**
 * check-e018-t03-dead-code.mjs — asserts the E018-T03 dead paths are gone.
 *
 * Exits non-zero (and lists them) if any deleted path is still present, so a
 * partial deletion can never read as a pass.
 */
import { existsSync, readFileSync } from "node:fs";

const deletedPaths = [
  "src/components/AppProgressBar.tsx",
  "src/components/HeightRail.tsx",
  "src/components/SessionProgress.tsx",
  "src/components/TodayStats.tsx",
  "src/components/TransitionBanner.tsx",
  "src/overlay/main.tsx",
  "overlay.html",
];

const stillPresent = deletedPaths.filter((p) => existsSync(p));
if (stillPresent.length > 0) {
  console.error(`FAIL: ${stillPresent.length} dead path(s) still exist:`, stillPresent);
  process.exit(1);
}

const viteConfig = readFileSync("vite.config.ts", "utf8");
if (viteConfig.includes("overlay.html")) {
  console.error("FAIL: vite.config.ts still references overlay.html");
  process.exit(1);
}

console.log(`OK: confirmed ${deletedPaths.length} dead paths absent, vite.config.ts clean`);
