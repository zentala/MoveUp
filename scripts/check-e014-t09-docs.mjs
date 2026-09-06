// Asserts E014-T09's two new ADRs exist and are referenced from
// .arch/ARCHITECTURE.md, so the docs task can't silently skip the wiring.
import { existsSync, readFileSync } from "node:fs";

const ADRS = [
  ".arch/ADR/018-pm3-app-ownership-split.md",
  ".arch/ADR/019-release-store-layout.md",
];
const ARCH_DOC = ".arch/ARCHITECTURE.md";

let failed = false;
function check(name, ok, detail) {
  if (ok) {
    console.log(`ok - ${name}`);
  } else {
    failed = true;
    console.error(`FAIL - ${name}: ${detail}`);
  }
}

for (const adr of ADRS) {
  check(`${adr} exists`, existsSync(adr), `${adr} is missing`);
}

if (existsSync(ARCH_DOC)) {
  const doc = readFileSync(ARCH_DOC, "utf8");
  for (const adr of ADRS) {
    const num = adr.match(/ADR\/(\d+)/)[1];
    check(
      `${ARCH_DOC} references ADR ${num}`,
      doc.includes(num),
      `${ARCH_DOC} does not mention ADR ${num}`,
    );
  }
} else {
  check(`${ARCH_DOC} exists`, false, `${ARCH_DOC} is missing`);
}

process.exit(failed ? 1 : 0);
