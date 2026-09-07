/**
 * tokens.ts — minting and checking the two device credentials (E022-T03).
 *
 * A token is 32 random bytes, base64url, behind a per-role prefix. Only its
 * SHA-256 is stored, and a stored hash is never used as a SQL selector: rows are
 * fetched by `desk_id` and their hashes compared in constant time, so the
 * database never learns which candidate matched by index probe order.
 *
 * `verifyToken` refuses when it cannot check — an unbound database means every
 * credential is unauthorized, never that every credential is fine.
 */
import type { Role } from "@app/generated/Role";

import type { Env } from "../env";
import { db, findLicense, isActive, sha256Hex } from "./licenses";

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

/** One row of the `desks` table, as the credential path reads it. */
export interface DeskRow {
  desk_id: string;
  license_key_hash: string;
  token_hash: string;
  desk_name: string;
}

/** True when the string has the shape a minted token has. Not authentication. */
export function looksLikeToken(role: Role, token: string): boolean {
  const prefix = TOKEN_PREFIX[role];
  if (!token.startsWith(prefix)) return false;
  const body = token.slice(prefix.length);
  return body.length >= 8 && /^[A-Za-z0-9_-]+$/.test(body);
}

/** Base64url without padding — the alphabet `looksLikeToken` accepts. */
function base64url(bytes: Uint8Array): string {
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** A fresh credential for `role`. The plaintext is returned exactly once. */
export function mintToken(role: Role): string {
  return TOKEN_PREFIX[role] + base64url(crypto.getRandomValues(new Uint8Array(32)));
}

/**
 * SHA-256 of a token, hex. What is stored in D1 — the plaintext never lands in
 * a row, a log line or a URL.
 */
export const hashToken = sha256Hex;

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

/** Reads one desk row, or `null` when there is no such desk. */
export async function findDesk(env: Env, deskId: string): Promise<DeskRow | null> {
  const row = await db(env)
    .prepare("SELECT desk_id, license_key_hash, token_hash, desk_name FROM desks WHERE desk_id = ?")
    .bind(deskId)
    .first<DeskRow>();
  return row ?? null;
}

interface ViewerCredential {
  viewer_id: string;
  token_hash: string;
  revoked_at: number | null;
}

/**
 * Finds the viewer whose stored hash equals `hash`.
 *
 * The loop does not break on a match: every candidate is compared, so the time
 * taken does not depend on the position of the matching row.
 */
function matchViewer(rows: ViewerCredential[], hash: string): ViewerCredential | null {
  let found: ViewerCredential | null = null;
  for (const row of rows) {
    if (timingSafeEqualHex(row.token_hash, hash)) found = row;
  }
  return found;
}

/** Records that a credential was used, so the Settings list can show it. */
async function touch(env: Env, table: "desks" | "viewers", id: string): Promise<void> {
  const column = table === "desks" ? "desk_id" : "viewer_id";
  await db(env)
    .prepare(`UPDATE ${table} SET last_seen = ? WHERE ${column} = ?`)
    .bind(Date.now(), id)
    .run();
}

/**
 * Decides whether a `hello` may open the socket.
 *
 * @param env    Worker bindings; the credential store is read here.
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
  // A relay that cannot consult its credentials must not accept any: a
  // well-formed string is not a credential.
  if (!env.DB) return { ok: false, reason: "unauthorized" };

  const desk = await findDesk(env, deskId);
  if (!desk) return { ok: false, reason: "unauthorized" };

  const license = await findLicense(env, desk.license_key_hash);
  if (!license || !isActive(license, Date.now())) return { ok: false, reason: "unentitled" };

  const hash = await hashToken(token);

  if (role === "desk") {
    if (!timingSafeEqualHex(desk.token_hash, hash)) return { ok: false, reason: "unauthorized" };
    await touch(env, "desks", deskId);
    return { ok: true, role, viewerId: null };
  }

  const rows = await db(env)
    .prepare("SELECT viewer_id, token_hash, revoked_at FROM viewers WHERE desk_id = ?")
    .bind(deskId)
    .all<ViewerCredential>();
  const viewer = matchViewer(rows.results ?? [], hash);
  if (!viewer) return { ok: false, reason: "unauthorized" };
  if (viewer.revoked_at !== null) return { ok: false, reason: "revoked" };
  await touch(env, "viewers", viewer.viewer_id);
  return { ok: true, role, viewerId: viewer.viewer_id };
}
