#!/usr/bin/env node
/**
 * mint-license.mjs — issue one licence key and insert its hash into D1.
 *
 * **T02 ships the refusal, T03 ships the insert.** The generator below is real
 * and the printed key is real; what is missing is the `wrangler d1 execute`
 * that stores `sha256(key)`. A key that was never stored entitles nobody, so
 * printing one and exiting 0 would hand out a credential that silently does
 * not work. It therefore exits 3 unless `--dry-run` says the caller only wants
 * to see the shape.
 */
import { randomBytes, createHash } from "node:crypto";

const PREFIX = "mu_lic_";

function mintKey() {
  return PREFIX + randomBytes(32).toString("base64url");
}

const sha256Hex = (s) => createHash("sha256").update(s).digest("hex");

const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const key = mintKey();

console.log(`key:  ${key}`);
console.log(`hash: ${sha256Hex(key)}`);

if (dryRun) {
  console.log("dry-run: nothing was written to D1.");
  process.exit(0);
}

console.error(
  [
    "",
    "NOT MINTED: this key was not stored, so it entitles nobody.",
    "The D1 insert lands in E022-T03 (relay/src/auth/licenses.ts + this script).",
    "Re-run with --dry-run if you only wanted to see the key format.",
  ].join("\n"),
);
process.exit(3);
