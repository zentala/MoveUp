/**
 * `verifyToken` against a real D1 database with the shipped migrations applied.
 *
 * The credentials are minted by the real REST routes, so nothing here stubs the
 * seam between the credential store and the socket authenticator — the two
 * halves that would otherwise agree only with each other.
 */
import { describe, expect, it } from "vitest";

import type { Env } from "../../src/env";
import {
  TOKEN_PREFIX,
  hashToken,
  looksLikeToken,
  mintToken,
  timingSafeEqualHex,
  verifyToken,
} from "../../src/auth/tokens";
import { api, env, newDesk, pairViewer, testDb } from "../helpers";

const deskToken = `${TOKEN_PREFIX.desk}${"a".repeat(43)}`;
const viewerToken = `${TOKEN_PREFIX.viewer}${"b".repeat(43)}`;

describe("looksLikeToken", () => {
  it("happy: accepts a minted-shaped token for its own role", () => {
    expect(looksLikeToken("desk", deskToken)).toBe(true);
    expect(looksLikeToken("viewer", viewerToken)).toBe(true);
  });

  it("error: rejects a token minted for the other role", () => {
    expect(looksLikeToken("viewer", deskToken)).toBe(false);
    expect(looksLikeToken("desk", viewerToken)).toBe(false);
  });

  it("empty: rejects the empty string and a bare prefix", () => {
    expect(looksLikeToken("desk", "")).toBe(false);
    expect(looksLikeToken("desk", TOKEN_PREFIX.desk)).toBe(false);
  });

  it("error: rejects characters outside base64url", () => {
    expect(looksLikeToken("desk", `${TOKEN_PREFIX.desk}abc def!`)).toBe(false);
  });
});

describe("mintToken", () => {
  it("produces the documented shape: prefix plus 43 base64url characters", () => {
    for (const role of ["desk", "viewer"] as const) {
      const token = mintToken(role);
      expect(token.startsWith(TOKEN_PREFIX[role])).toBe(true);
      expect(token.slice(TOKEN_PREFIX[role].length)).toMatch(/^[A-Za-z0-9_-]{43}$/);
      expect(looksLikeToken(role, token)).toBe(true);
    }
  });

  it("does not repeat itself", () => {
    const minted = new Set(Array.from({ length: 50 }, () => mintToken("viewer")));
    expect(minted.size).toBe(50);
  });
});

describe("verifyToken", () => {
  it("happy: a registered desk's own token opens the socket", async () => {
    const desk = await newDesk();
    await expect(verifyToken(env, desk.deskId, "desk", desk.token)).resolves.toEqual({
      ok: true,
      role: "desk",
      viewerId: null,
    });
  });

  it("happy: a paired viewer is identified by its viewer_id", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    await expect(verifyToken(env, desk.deskId, "viewer", viewer.token)).resolves.toEqual({
      ok: true,
      role: "viewer",
      viewerId: viewer.viewerId,
    });
  });

  it("records last_seen, so the column has a writer and not only a reader", async () => {
    const desk = await newDesk();
    const before = await testDb()
      .prepare("SELECT last_seen FROM desks WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ last_seen: number | null }>();
    expect(before?.last_seen).toBeNull();

    await verifyToken(env, desk.deskId, "desk", desk.token);

    const after = await testDb()
      .prepare("SELECT last_seen FROM desks WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ last_seen: number | null }>();
    expect(after?.last_seen).toBeGreaterThan(0);
  });

  it("error: another desk's token does not open this desk's room", async () => {
    const mine = await newDesk();
    const theirs = await newDesk();
    await expect(verifyToken(env, mine.deskId, "desk", theirs.token)).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
  });

  it("error: a revoked viewer is refused with `revoked`, not `unauthorized`", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const response = await api(`/v1/desks/${desk.deskId}/viewers/${viewer.viewerId}`, {
      method: "DELETE",
      token: desk.token,
    });
    expect(response.status).toBe(204);

    await expect(verifyToken(env, desk.deskId, "viewer", viewer.token)).resolves.toEqual({
      ok: false,
      reason: "revoked",
    });
  });

  it("error: an expired license makes its desk `unentitled`", async () => {
    const desk = await newDesk();
    await testDb()
      .prepare("UPDATE licenses SET expires_at = ? WHERE key_hash = (SELECT license_key_hash FROM desks WHERE desk_id = ?)")
      .bind(Date.now() - 1_000, desk.deskId)
      .run();

    await expect(verifyToken(env, desk.deskId, "desk", desk.token)).resolves.toEqual({
      ok: false,
      reason: "unentitled",
    });
  });

  it("nil: an empty desk id is unauthorized", async () => {
    await expect(verifyToken(env, "", "desk", deskToken)).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
  });

  it("empty: an empty token is unauthorized", async () => {
    await expect(verifyToken(env, "desk-1", "desk", "")).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
  });

  it("error: a malformed token is unauthorized", async () => {
    await expect(verifyToken(env, "desk-1", "viewer", "hunter2")).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
  });

  it("error: no credential store means no credentials, never all of them", async () => {
    const desk = await newDesk();
    const blind = { ...env, DB: undefined } as Env;
    await expect(verifyToken(blind, desk.deskId, "desk", desk.token)).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
  });
});

describe("what the database holds", () => {
  it("stores hashes only — no plaintext token appears in any row", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);

    const tables = ["licenses", "desks", "viewers"];
    let scanned = 0;
    for (const table of tables) {
      const rows = await testDb().prepare(`SELECT * FROM ${table}`).all();
      const dump = JSON.stringify(rows.results ?? []);
      scanned += (rows.results ?? []).length;
      for (const secret of [desk.token, viewer.token, desk.licenseKey]) {
        expect(dump).not.toContain(secret);
        // Also not the body without the prefix, in case something stripped it.
        expect(dump).not.toContain(secret.slice(5));
      }
    }
    // An empty database would pass every assertion above without proving
    // anything: assert the rows were actually there to be scanned.
    expect(scanned).toBeGreaterThanOrEqual(3);
  });

  it("stores the hash the authenticator computes", async () => {
    const desk = await newDesk();
    const row = await testDb()
      .prepare("SELECT token_hash FROM desks WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ token_hash: string }>();
    expect(row?.token_hash).toBe(await hashToken(desk.token));
  });
});

describe("hashToken", () => {
  it("is the SHA-256 hex of the token and never contains it", async () => {
    const hash = await hashToken(deskToken);
    expect(hash).toMatch(/^[0-9a-f]{64}$/);
    expect(hash).not.toContain(deskToken.slice(TOKEN_PREFIX.desk.length));
  });

  it("is stable and distinguishes two tokens", async () => {
    expect(await hashToken(deskToken)).toBe(await hashToken(deskToken));
    expect(await hashToken(deskToken)).not.toBe(await hashToken(viewerToken));
  });
});

describe("timingSafeEqualHex", () => {
  it("happy: equal digests compare equal", async () => {
    const hash = await hashToken(viewerToken);
    expect(timingSafeEqualHex(hash, hash)).toBe(true);
  });

  it("error: a one-character difference compares unequal", async () => {
    const hash = await hashToken(viewerToken);
    const altered = `${hash.slice(0, -1)}${hash.endsWith("0") ? "1" : "0"}`;
    expect(timingSafeEqualHex(hash, altered)).toBe(false);
  });

  it("nil: different lengths compare unequal", () => {
    expect(timingSafeEqualHex("abc", "abcd")).toBe(false);
  });
});
