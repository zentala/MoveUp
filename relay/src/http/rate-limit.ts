/**
 * rate-limit.ts — per-IP throttling for the unauthenticated REST routes.
 *
 * **Why a Durable Object and not the `RateLimiter` binding.** The platform
 * binding is cheaper, but its counters are per-colo and it cannot be exercised
 * in `vitest-pool-workers`: a test would have to assert that a limit exists
 * rather than that it holds. Pairing brute-force protection is the one thing in
 * this relay that must be provably tested, so the limiter is a Durable Object
 * addressed by bucket name — one instance per `<route>:<ip>` — whose window is
 * plain in-memory state. Documented in `relay/README.md`.
 *
 * The window is memory-only, so an eviction forgives a caller's history. That
 * is deliberate for a throttle (as opposed to the pairing lockout, which the
 * room holds for the whole of its life): the cost of a forgiven minute is one
 * extra burst, and the alternative is a storage write per request.
 */
import type { Env } from "../env";

export const MINUTE_MS = 60_000;

/** 5 registrations and 5 pairing attempts a minute per IP (PLAN.md §REST). */
export const REGISTER_LIMIT = 5;
export const PAIR_LIMIT = 5;

export interface RateVerdict {
  allowed: boolean;
  /** Milliseconds until the caller may retry. Zero when allowed. */
  retryAfterMs: number;
}

/**
 * Takes one token from the bucket named `key`.
 *
 * Throws when the binding is missing — a relay that cannot throttle must fail
 * loudly rather than serve an unthrottled pairing endpoint.
 */
export async function take(
  env: Env,
  key: string,
  limit: number,
  windowMs = MINUTE_MS,
): Promise<RateVerdict> {
  if (!env.RATE_LIMITER) throw new Error("relay: no RATE_LIMITER binding");
  const stub = env.RATE_LIMITER.get(env.RATE_LIMITER.idFromName(key));
  const response = await stub.fetch("https://rate-limiter.internal/take", {
    method: "POST",
    body: JSON.stringify({ limit, windowMs }),
  });
  return (await response.json()) as RateVerdict;
}

/** The client address a bucket is keyed by. Unknown callers share one bucket. */
export const clientIp = (request: Request): string =>
  request.headers.get("CF-Connecting-IP") ?? request.headers.get("X-Forwarded-For") ?? "unknown";

/**
 * One bucket. Addressed by name, so the object *is* the key and its whole state
 * is the list of hits still inside the window.
 */
export class RateLimiter implements DurableObject {
  private hits: number[] = [];

  async fetch(request: Request): Promise<Response> {
    const { limit, windowMs } = (await request.json()) as { limit: number; windowMs: number };
    const now = Date.now();
    this.hits = this.hits.filter((at) => now - at < windowMs);

    if (this.hits.length >= limit) {
      const oldest = this.hits[0];
      const verdict: RateVerdict = { allowed: false, retryAfterMs: windowMs - (now - oldest) };
      return Response.json(verdict);
    }

    this.hits.push(now);
    return Response.json({ allowed: true, retryAfterMs: 0 } satisfies RateVerdict);
  }
}
