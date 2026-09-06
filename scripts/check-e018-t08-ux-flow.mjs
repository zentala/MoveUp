// scripts/check-e018-t08-ux-flow.mjs
//
// Guards E018-T08: `.arch/UX-FLOW.md` must describe the Steps widget (Google
// Fit) and the click-timeline-to-Analyst interaction. Both landed in the app
// months before the doc caught up; this check makes the drift loud.
import { readFileSync } from "node:fs";

const doc = readFileSync(".arch/UX-FLOW.md", "utf8").toLowerCase();
const problems = [];
if (!doc.includes("steps")) problems.push("no mention of the Steps widget");
if (!doc.includes("google fit")) problems.push("no mention of Google Fit");
if (!/timeline[^\n]{0,80}analyst|click[^\n]{0,80}timeline/.test(doc)) {
  problems.push("no mention of the click-timeline-to-Analyst interaction");
}
if (problems.length > 0) {
  console.error("FAIL:", problems.join("; "));
  process.exit(1);
}
console.log("OK: UX-FLOW.md mentions Steps, Google Fit, and timeline-to-Analyst");
