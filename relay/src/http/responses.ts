/**
 * responses.ts — one shape for every JSON answer the Worker gives.
 *
 * Errors are `{ error: { code, message } }` with `code` drawn from the fixed
 * list in `auth/licenses.ts`, so a client can branch on a value instead of on
 * prose (PLAN.md §Protocol — REST).
 */
import type { ErrorCode } from "../auth/licenses";

const JSON_HEADERS = { "content-type": "application/json; charset=utf-8" };

export function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), { status, headers: JSON_HEADERS });
}

export function fail(code: ErrorCode, message: string, status: number): Response {
  return json({ error: { code, message } }, status);
}
