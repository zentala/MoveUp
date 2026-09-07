/**
 * licenses.ts — entitlement lookup.
 *
 * **Owned by T03.** T02 fixes only the vocabulary the rest of the relay speaks:
 * the error codes a REST caller sees and the shape a license row has, so the
 * room and the router can be written against names that will not move.
 *
 * Every function here refuses rather than guesses. A relay that cannot check a
 * license must not behave like a relay whose licenses are all valid.
 */
import type { Env } from "../env";

/** The fixed error-code list from PLAN.md §Protocol — REST. */
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
  "not_implemented",
  "assets_unavailable",
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

/** One row of the `licenses` table T03 creates. */
export interface License {
  key_hash: string;
  plan: string;
  max_desks: number;
  max_viewers: number;
  expires_at: number | null;
  created_at: number;
}

/** How many viewers a desk may pair while its license is unknown. */
export const DEFAULT_MAX_VIEWERS = 5;

/**
 * Looks a license up by the hash of its key.
 *
 * Returns `null` for "no such license" and throws for "could not ask" — the
 * two must never collapse into one answer, or an unreachable database reads as
 * an invalid key and the user is told their license is bad.
 */
export async function findLicense(env: Env, keyHash: string): Promise<License | null> {
  if (!env.DB) throw new Error("licenses: no D1 binding (E022-T03 not wired)");
  const row = await env.DB.prepare(
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
