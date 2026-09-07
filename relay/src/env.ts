/**
 * Bindings the Worker reads. Mirrors `wrangler.toml` and the `miniflare`
 * block in `vitest.config.ts` — change all three together.
 */
export interface Env {
  DESK_ROOM: DurableObjectNamespace;
  /** One instance per rate-limit bucket; see `http/rate-limit.ts`. */
  RATE_LIMITER: DurableObjectNamespace;
  /** Absent until a `vite build` has produced `../dist`. */
  ASSETS?: Fetcher;
  /**
   * Credential store. Optional in the type on purpose: a deployment without it
   * must refuse every credential rather than fall back to a shape check, and
   * that refusal is only expressible if "absent" is a state the code can see.
   */
  DB?: D1Database;
  RELAY_VERSION: string;
  HELLO_TIMEOUT_MS?: string;
  IDLE_TIMEOUT_MS?: string;
  /** Lifetime of a pairing code. Protocol says 300 s. */
  PAIRING_TTL_MS?: string;
  /** How long pairing stays locked after too many wrong codes. 15 min. */
  PAIRING_LOCKOUT_MS?: string;
  /** Wrong codes tolerated before the lockout. 10. */
  PAIRING_MAX_ATTEMPTS?: string;
  /**
   * How often a room with a live desk socket re-checks its licence against D1.
   * A `hello`-time check alone lets a licence expire under an already-open
   * connection, since pings refresh `lastSeen` without ever re-authenticating.
   * Default 5 min.
   */
  LICENSE_RECHECK_MS?: string;
}

/** Reads a numeric var, falling back when it is unset or not a number. */
export function numVar(raw: string | undefined, fallback: number): number {
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}
