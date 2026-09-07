/**
 * index.ts — the Worker's router (E022-T02).
 *
 * Four surfaces: the WebSocket upgrade that reaches a `DeskRoom`, the health
 * probe the deploy gate reads, the viewer build under `/app`, and the REST
 * credential routes that T03 fills in.
 */
import type { Env } from "./env";
import { serveApp, APP_PREFIX } from "./http/assets";
import { healthz } from "./http/health";
import { handleRest, matchPath } from "./http/rest";
import { fail } from "./http/responses";

export { DeskRoom } from "./room/desk-room";

const WS_PATH = "/v1/desks/:deskId/ws";

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const { pathname } = url;

    if (pathname === "/healthz") return healthz(env);

    if (pathname === APP_PREFIX || pathname.startsWith(`${APP_PREFIX}/`)) {
      return serveApp(request, env);
    }

    const ws = matchPath(WS_PATH, pathname);
    if (ws !== null) {
      if (request.method !== "GET") return new Response("method not allowed", { status: 405 });
      // The room is addressed by `desk_id` alone. Authentication happens inside,
      // over the socket, in `hello` — the token is never a URL component.
      const id = env.DESK_ROOM.idFromName(ws.deskId);
      return env.DESK_ROOM.get(id).fetch(request);
    }

    const rest = handleRest(request, pathname);
    if (rest !== null) return rest;

    return fail("not_found", `no route for ${request.method} ${pathname}`, 404);
  },
} satisfies ExportedHandler<Env>;
