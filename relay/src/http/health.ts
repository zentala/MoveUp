/**
 * health.ts — `GET /healthz`, the deploy gate (T16).
 *
 * It reports the version the Worker was built with, so "the deploy succeeded"
 * and "the deploy replaced what was running" are two different observations.
 */
import type { Env } from "../env";
import { json } from "./responses";

export function healthz(env: Env): Response {
  return json({ ok: true, version: env.RELAY_VERSION ?? "unknown" });
}
