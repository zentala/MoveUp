/**
 * Entitlement lookup, and the one seam nothing else covers: the licence
 * minting script writes rows this Worker reads, but it runs in Node against
 * `wrangler d1 execute` and can never share code with the Worker. The drift
 * test below reads the script's own SQL and compares its column list with the
 * table the migration creates, so a column renamed on one side fails here
 * rather than at the customer's first pairing.
 */
import { describe, expect, it } from "vitest";

import {
  countActiveViewers,
  countDesks,
  findLicense,
  isActive,
  sha256Hex,
  type License,
} from "../../src/auth/licenses";
import type { Env } from "../../src/env";
import { env, newDesk, pairViewer, seedLicense, testDb } from "../helpers";

const sources = import.meta.glob("../../scripts/*.mjs", {
  query: "?raw",
  eager: true,
  import: "default",
}) as Record<string, string>;

describe("mint-license.mjs", () => {
  const entry = Object.entries(sources).find(([path]) => path.endsWith("mint-license.mjs"));

  it("is readable — an empty glob is a broken test, not a passing one", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(0);
    expect(entry).toBeDefined();
  });

  it("inserts exactly the columns the licenses table declares", async () => {
    const source = entry?.[1] ?? "";
    const insert = /INSERT INTO licenses\s*\(([^)]+)\)/i.exec(source);
    expect(insert, "the script must contain an INSERT INTO licenses (...)").not.toBeNull();

    const named = (insert?.[1] ?? "")
      .split(",")
      .map((c) => c.trim())
      .sort();
    const columns = await testDb().prepare("PRAGMA table_info(licenses)").all<{ name: string }>();
    const declared = (columns.results ?? []).map((c) => c.name).sort();

    expect(declared.length).toBeGreaterThan(0);
    expect(named).toEqual(declared);
  });

  it("refuses to print a key it did not store", () => {
    const source = entry?.[1] ?? "";
    // A key that was never inserted entitles nobody; printing one and exiting 0
    // would hand out a credential that silently does not work.
    expect(source).toMatch(/process\.exit\([1-9]/);
  });
});

describe("findLicense", () => {
  it("happy: finds a seeded license by the hash of its key", async () => {
    const key = await seedLicense({ plan: "founder", maxDesks: 3, maxViewers: 7 });
    const license = await findLicense(env, await sha256Hex(key));
    expect(license).toMatchObject({ plan: "founder", max_desks: 3, max_viewers: 7 });
  });

  it("nil: an unknown key is `null` — found nothing, not could-not-ask", async () => {
    await expect(findLicense(env, await sha256Hex("mu_lic_nope"))).resolves.toBeNull();
  });

  it("error: no database throws rather than answering `null`", async () => {
    const blind = { ...env, DB: undefined } as Env;
    await expect(findLicense(blind, "deadbeef")).rejects.toThrow(/no D1 binding/);
  });
});

describe("isActive", () => {
  const base: License = {
    key_hash: "x",
    plan: "founder",
    max_desks: 1,
    max_viewers: 5,
    expires_at: null,
    created_at: 0,
  };

  it("a null expiry never expires", () => {
    expect(isActive(base, 8.64e15)).toBe(true);
  });

  it("an expiry in the future is active, in the past is not", () => {
    expect(isActive({ ...base, expires_at: 2_000 }, 1_000)).toBe(true);
    expect(isActive({ ...base, expires_at: 1_000 }, 2_000)).toBe(false);
  });

  it("the moment of expiry is not active", () => {
    expect(isActive({ ...base, expires_at: 1_000 }, 1_000)).toBe(false);
  });
});

describe("counting", () => {
  it("counts the desks a license has registered", async () => {
    const key = await seedLicense({ maxDesks: 2 });
    const hash = await sha256Hex(key);
    expect(await countDesks(env, hash)).toBe(0);

    const { registerDesk } = await import("../helpers");
    await registerDesk(key);
    expect(await countDesks(env, hash)).toBe(1);
  });

  it("empty: a desk with no viewers counts zero, and a revoked one stops counting", async () => {
    const desk = await newDesk();
    expect(await countActiveViewers(env, desk.deskId)).toBe(0);

    const viewer = await pairViewer(desk);
    expect(await countActiveViewers(env, desk.deskId)).toBe(1);

    await testDb()
      .prepare("UPDATE viewers SET revoked_at = ? WHERE viewer_id = ?")
      .bind(Date.now(), viewer.viewerId)
      .run();
    expect(await countActiveViewers(env, desk.deskId)).toBe(0);
  });
});
