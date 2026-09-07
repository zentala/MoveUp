/**
 * rest.ts — the six credential routes from PLAN.md §Protocol — REST.
 *
 * The table is the contract: a path that matches with the wrong verb answers
 * `405`, not `404`, so a client learns whether its URL or its method is wrong,
 * and `REST_ROUTES` is what the router test asserts against.
 */
import { deleteDesk, listViewers, register, revokeViewer } from "./desks";
import { issuePairing, pair } from "./pairing";
import { fail } from "./responses";
import type { Env } from "../env";

export interface RestRoute {
  method: string;
  /** URLPattern-style path with `:name` segments. */
  path: string;
  summary: string;
  handle: (request: Request, env: Env, params: Record<string, string>) => Promise<Response>;
}

export const REST_ROUTES: readonly RestRoute[] = [
  {
    method: "POST",
    path: "/v1/desks/register",
    summary: "license key -> desk_id + desk_token",
    handle: (request, env) => register(request, env),
  },
  {
    method: "POST",
    path: "/v1/desks/:deskId/pairings",
    summary: "desk asks for a pairing code",
    handle: (request, env, p) => issuePairing(request, env, p.deskId),
  },
  {
    method: "POST",
    path: "/v1/pair",
    summary: "phone redeems a pairing code -> viewer_token",
    handle: (request, env) => pair(request, env),
  },
  {
    method: "GET",
    path: "/v1/desks/:deskId/viewers",
    summary: "paired devices, for the Settings list",
    handle: (request, env, p) => listViewers(request, env, p.deskId),
  },
  {
    method: "DELETE",
    path: "/v1/desks/:deskId/viewers/:viewerId",
    summary: "revoke one viewer",
    handle: (request, env, p) => revokeViewer(request, env, p.deskId, p.viewerId),
  },
  {
    method: "DELETE",
    path: "/v1/desks/:deskId",
    summary: "disable the relay for this desk",
    handle: (request, env, p) => deleteDesk(request, env, p.deskId),
  },
];

/** Matches a path against a `:name` pattern, returning its parameters. */
export function matchPath(pattern: string, pathname: string): Record<string, string> | null {
  const want = pattern.split("/");
  const got = pathname.split("/");
  if (want.length !== got.length) return null;

  const params: Record<string, string> = {};
  for (let i = 0; i < want.length; i += 1) {
    if (want[i].startsWith(":")) {
      if (got[i] === "") return null;
      params[want[i].slice(1)] = decodeURIComponent(got[i]);
      continue;
    }
    if (want[i] !== got[i]) return null;
  }
  return params;
}

/**
 * Answers a declared REST route, or `null` when the path belongs to no route.
 *
 * An unexpected throw becomes `503 internal` rather than a runtime 500 with no
 * body: the two states a relay must never confuse are "refused" and "could not
 * ask", and `db()` throws precisely to keep them apart.
 */
export async function handleRest(
  request: Request,
  pathname: string,
  env: Env,
): Promise<Response | null> {
  let pathMatched = false;
  for (const route of REST_ROUTES) {
    const params = matchPath(route.path, pathname);
    if (params === null) continue;
    pathMatched = true;
    if (route.method !== request.method) continue;
    try {
      return await route.handle(request, env, params);
    } catch (cause) {
      console.error(`relay: ${route.method} ${route.path} failed`, cause);
      return fail("internal", "the relay could not complete this request", 503);
    }
  }
  return pathMatched ? new Response("method not allowed", { status: 405 }) : null;
}
