/**
 * tokens.ts — the seam between the room and the credential store.
 *
 * **T02 ships the hook, not the authentication.** `verifyToken` currently does
 * a shape check only; T03 replaces its body with a D1 lookup (`sha256(token)`
 * compared in constant time) without changing this signature or the room's
 * call site. What T02 does own, and what is tested, is the *failure* path: a
 * missing, malformed or wrong-role token closes the socket 4401 today, so T03
 * inherits a working rejection rather than having to invent one.
 *
 * The relay is therefore NOT deployable before T03 — see `relay/README.md`.
 */
import type { Env } from "../env";
import type { Role } from "@app/generated/Role";

/** Prefix per role. Also what secret scanners and the emission guard match on. */
export const TOKEN_PREFIX: Record<Role, string> = {
  desk: "mu_d_",
  viewer: "mu_v_",
};

/** 32 random bytes, base64url — 43 characters after the prefix. */
export const TOKEN_BODY_LENGTH = 43;

/** Reasons a token is refused, each mapping to one close code in the room. */
export type AuthFailure = "unauthorized" | "unentitled" | "revoked";

export type AuthResult =
  | { ok: true; role: Role; viewerId: string | null }
  | { ok: false; reason: AuthFailure };

/** True when the string has the shape a minted token has. Not authentication. */
export function looksLikeToken(role: Role, token: string): boolean {
  const prefix = TOKEN_PREFIX[role];
  if (!token.startsWith(prefix)) return false;
  const body = token.slice(prefix.length);
  return body.length >= 8 && /^[A-Za-z0-9_-]+$/.test(body);
}

/**
 * SHA-256 of a token, hex. What T03 stores in D1 — the plaintext never lands
 * in a row, a log line or a URL.
 */
export async function hashToken(token: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(token));
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/**
 * Constant-time comparison of two hex digests. Length is public information
 * (both are SHA-256), so an early length exit leaks nothing.
 */
export function timingSafeEqualHex(a: string, b: string): boolean {
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i += 1) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

/**
 * Decides whether a `hello` may open the socket.
 *
 * @param env    Worker bindings — T03 reads `env.DB` here.
 * @param deskId Room the socket is connecting to.
 * @param role   Role claimed by the `hello` payload.
 * @param token  Bearer token from the `hello` payload; never from the URL.
 */
export async function verifyToken(
  env: Env,
  deskId: string,
  role: Role,
  token: string,
): Promise<AuthResult> {
  if (!deskId || !token || !looksLikeToken(role, token)) {
    return { ok: false, reason: "unauthorized" };
  }
  if (env.DB) {
    // T03 owns this branch. Refusing outright is deliberate: a half-written
    // credential check that answers "ok" is worse than one that answers
    // nothing, and a deployment with a database bound but no lookup must not
    // silently fall through to the shape check below.
    return { ok: false, reason: "unauthorized" };
  }
  // Shape-only acceptance. A viewer has no identity yet — T03 derives
  // `viewerId` from the row the token hash matches.
  return { ok: true, role, viewerId: null };
}
