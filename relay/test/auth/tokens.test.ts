/**
 * The `verifyToken` seam. T03 replaces the body; these tests pin the contract
 * it has to keep — above all that a token which cannot be checked is refused.
 */
import { describe, expect, it } from "vitest";

import type { Env } from "../../src/env";
import {
  TOKEN_PREFIX,
  hashToken,
  looksLikeToken,
  timingSafeEqualHex,
  verifyToken,
} from "../../src/auth/tokens";

const env = {} as Env;
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

describe("verifyToken", () => {
  it("happy: a shape-valid token opens the socket while T03 is pending", async () => {
    await expect(verifyToken(env, "desk-1", "desk", deskToken)).resolves.toEqual({
      ok: true,
      role: "desk",
      viewerId: null,
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

  it("refuses everything once a database is bound but unread", async () => {
    // The shape check must not be reachable in a deployment that has D1: a
    // relay with credentials it never consults would accept any well-formed
    // string as a credential.
    const withDb = { DB: {} } as unknown as Env;
    await expect(verifyToken(withDb, "desk-1", "desk", deskToken)).resolves.toEqual({
      ok: false,
      reason: "unauthorized",
    });
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
