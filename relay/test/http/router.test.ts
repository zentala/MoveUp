/**
 * The router's surfaces: health, assets, the WebSocket upgrade, the REST table,
 * and the private room prefix the Worker must never forward.
 */
import { SELF } from "cloudflare:test";
import { describe, expect, it } from "vitest";

import { REST_ROUTES, matchPath } from "../../src/http/rest";
import { RPC } from "../../src/room/rpc";
import { url } from "../helpers";

describe("GET /healthz", () => {
  it("reports the running version so a deploy can be told from a redeploy", async () => {
    const response = await SELF.fetch(url("/healthz"));
    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toMatchObject({ ok: true, version: "test" });
  });
});

describe("REST routes", () => {
  it("declares every route from PLAN.md", () => {
    expect(REST_ROUTES.length).toBeGreaterThan(0);
    expect(REST_ROUTES.map((r) => `${r.method} ${r.path}`)).toEqual([
      "POST /v1/desks/register",
      "POST /v1/desks/:deskId/pairings",
      "POST /v1/pair",
      "GET /v1/desks/:deskId/viewers",
      "DELETE /v1/desks/:deskId/viewers/:viewerId",
      "DELETE /v1/desks/:deskId",
    ]);
  });

  const deskOnly = REST_ROUTES.filter((r) => r.path.includes(":deskId"));

  it.each(deskOnly)("$method $path refuses a caller with no token", async (route) => {
    const path = route.path.replace(":deskId", "d1").replace(":viewerId", "v1");
    const response = await SELF.fetch(url(path), { method: route.method });
    expect(response.status).toBe(401);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "unauthorized" } });
  });

  it("answers 405 when the path is right and the verb is wrong", async () => {
    const response = await SELF.fetch(url("/v1/desks/register"), { method: "GET" });
    expect(response.status).toBe(405);
  });

  it("answers 404 for a path in no route", async () => {
    const response = await SELF.fetch(url("/v1/nope"));
    expect(response.status).toBe(404);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "not_found" } });
  });
});

describe("the room's private RPC prefix", () => {
  it.each(Object.values(RPC))("%s is not reachable from outside the Worker", async (path) => {
    const response = await SELF.fetch(url(path), { method: "POST", body: "{}" });
    expect(response.status).toBe(404);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "not_found" } });
  });
});

describe("matchPath", () => {
  it("extracts parameters", () => {
    expect(matchPath("/v1/desks/:deskId/ws", "/v1/desks/abc/ws")).toEqual({ deskId: "abc" });
  });

  it("nil: refuses an empty parameter segment", () => {
    expect(matchPath("/v1/desks/:deskId/ws", "/v1/desks//ws")).toBeNull();
  });

  it("refuses a path of a different length", () => {
    expect(matchPath("/v1/desks/:deskId", "/v1/desks/abc/ws")).toBeNull();
  });

  it("decodes percent-encoded segments", () => {
    expect(matchPath("/v1/desks/:deskId", "/v1/desks/a%20b")).toEqual({ deskId: "a b" });
  });
});

describe("GET /app/*", () => {
  it("says the build is missing rather than pretending the page is", async () => {
    // No ASSETS binding in tests — `../dist` only exists after a `vite build`.
    const response = await SELF.fetch(url("/app/index.html"));
    expect(response.status).toBe(503);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "assets_unavailable" },
    });
  });
});

describe("GET /v1/desks/:deskId/ws", () => {
  it("refuses a plain GET without an upgrade header", async () => {
    const response = await SELF.fetch(url("/v1/desks/d1/ws"));
    expect(response.status).toBe(426);
  });

  it("refuses a non-GET verb", async () => {
    const response = await SELF.fetch(url("/v1/desks/d1/ws"), { method: "POST" });
    expect(response.status).toBe(405);
  });
});
