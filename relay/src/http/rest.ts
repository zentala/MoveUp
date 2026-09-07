/**
 * rest.ts — the six credential routes from PLAN.md §Protocol — REST.
 *
 * **Bodies are T03's.** They exist here as 501 stubs on purpose: the desktop
 * client (T05/T06) is built in the same wave and needs the paths to resolve to
 * a documented answer rather than a 404 that reads like a typo in its own URL.
 * `501` also keeps the deploy gate honest — a relay deployed before T03 fails
 * every pairing loudly.
 */
import { notImplemented } from "./responses";

export interface RestRoute {
  method: string;
  /** URLPattern-style path with `:name` segments. */
  path: string;
  task: string;
  summary: string;
}

export const REST_ROUTES: readonly RestRoute[] = [
  {
    method: "POST",
    path: "/v1/desks/register",
    task: "E022-T03",
    summary: "license key -> desk_id + desk_token",
  },
  {
    method: "POST",
    path: "/v1/desks/:deskId/pairings",
    task: "E022-T03",
    summary: "desk asks for a pairing code",
  },
  {
    method: "POST",
    path: "/v1/pair",
    task: "E022-T03",
    summary: "phone redeems a pairing code -> viewer_token",
  },
  {
    method: "GET",
    path: "/v1/desks/:deskId/viewers",
    task: "E022-T03",
    summary: "paired devices, for the Settings list",
  },
  {
    method: "DELETE",
    path: "/v1/desks/:deskId/viewers/:viewerId",
    task: "E022-T03",
    summary: "revoke one viewer",
  },
  {
    method: "DELETE",
    path: "/v1/desks/:deskId",
    task: "E022-T03",
    summary: "disable the relay for this desk",
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
 * A path that matches but with the wrong method answers 405 rather than 404 —
 * the difference tells a client whether its URL or its verb is wrong.
 */
export function handleRest(request: Request, pathname: string): Response | null {
  let pathMatched = false;
  for (const route of REST_ROUTES) {
    if (matchPath(route.path, pathname) === null) continue;
    pathMatched = true;
    if (route.method === request.method) return notImplemented(route.task);
  }
  return pathMatched ? new Response("method not allowed", { status: 405 }) : null;
}
