/**
 * licenses.ts — entitlement lookup.
 *
 * Every function here refuses rather than guesses. A relay that cannot check a
 * license must not behave like a relay whose licenses are all valid: `findLicense`
 * throws when there is no database to ask and returns `null` only when the ask
 * succeeded and found nothing.
 */
import type { Env } from "../env";

/** The fixed error-code list from PLAN.md §Protocol — REST, plus three the
 * router needs for states the protocol table does not cover: a route that is
 * not built yet, a viewer build that has not been produced, and a relay that
 * cannot reach its own store. */
export const ERROR_CODES = [
  "invalid_license",
  "license_exhausted",
  "bad_code",
  "code_expired",
  "pairing_locked",
  "viewer_limit",
  "unauthorized",
  "rate_limited",
  "not_found",
  "bad_request",
  "not_implemented",
  "assets_unavailable",
  "internal",
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

/** One row of the `licenses` table. */
export interface License {
  key_hash: string;
  plan: string;
  max_desks: number;
  max_viewers: number;
  expires_at: number | null;
  created_at: number;
}

/** SHA-256 of an arbitrary string, hex. What every `*_hash` column holds. */
export async function sha256Hex(value: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** The database, or an error naming what is missing. Never a silent no-op. */
export function db(env: Env): D1Database {
  if (!env.DB) throw new Error("relay: no D1 binding — credentials cannot be checked");
  return env.DB;
}

/**
 * Looks a license up by the hash of its key.
 *
 * Returns `null` for "no such license" and throws for "could not ask" — the
 * two must never collapse into one answer, or an unreachable database reads as
 * an invalid key and the user is told their license is bad.
 */
export async function findLicense(env: Env, keyHash: string): Promise<License | null> {
  const row = await db(env)
    .prepare(
      "SELECT key_hash, plan, max_desks, max_viewers, expires_at, created_at FROM licenses WHERE key_hash = ?",
    )
    .bind(keyHash)
    .first<License>();
  return row ?? null;
}

/** True when the license is still inside its term. `null` expiry never expires. */
export function isActive(license: License, now: number): boolean {
  return license.expires_at === null || license.expires_at > now;
}

/** How many desks this license has already registered. */
export async function countDesks(env: Env, keyHash: string): Promise<number> {
  const row = await db(env)
    .prepare("SELECT COUNT(*) AS n FROM desks WHERE license_key_hash = ?")
    .bind(keyHash)
    .first<{ n: number }>();
  return row?.n ?? 0;
}

/** How many viewers are paired to this desk and not revoked. */
export async function countActiveViewers(env: Env, deskId: string): Promise<number> {
  const row = await db(env)
    .prepare("SELECT COUNT(*) AS n FROM viewers WHERE desk_id = ? AND revoked_at IS NULL")
    .bind(deskId)
    .first<{ n: number }>();
  return row?.n ?? 0;
}
