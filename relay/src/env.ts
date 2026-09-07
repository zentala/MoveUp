/**
 * Bindings the Worker reads. Mirrors `wrangler.toml` and the `miniflare`
 * block in `vitest.config.ts` — change all three together.
 */
export interface Env {
  DESK_ROOM: DurableObjectNamespace;
  /** Absent until a `vite build` has produced `../dist`. */
  ASSETS?: Fetcher;
  /** Absent until T03 creates the D1 database. */
  DB?: D1Database;
  RELAY_VERSION: string;
  HELLO_TIMEOUT_MS?: string;
  IDLE_TIMEOUT_MS?: string;
}

/** Reads a numeric var, falling back when it is unset or not a number. */
export function numVar(raw: string | undefined, fallback: number): number {
  const parsed = Number(raw);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}
