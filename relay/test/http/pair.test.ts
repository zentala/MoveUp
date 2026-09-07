/**
 * `POST /v1/desks/{id}/pairings` and `POST /v1/pair` — the code round trip.
 *
 * The vitest bindings shorten the code's life to 1 s and the lockout to 2 s
 * (`vitest.config.ts`), which is what makes the expiry and lockout paths
 * assertable without a fake clock inside workerd.
 */
import { describe, expect, it } from "vitest";

import { api, newDesk, pairViewer, testDb, type Desk } from "../helpers";

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const issue = (desk: Desk) =>
  api(`/v1/desks/${desk.deskId}/pairings`, { method: "POST", token: desk.token, body: {} });

const redeem = (deskId: string, code: string, ip?: string) =>
  api("/v1/pair", { method: "POST", body: { desk_id: deskId, code, device_name: "Pixel" }, ip });

async function codeFor(desk: Desk): Promise<string> {
  const response = await issue(desk);
  expect(response.status).toBe(201);
  return ((await response.json()) as { code: string }).code;
}

describe("issuing a code", () => {
  it("happy: eight readable characters and an expiry in the future", async () => {
    const desk = await newDesk();
    const response = await issue(desk);
    expect(response.status).toBe(201);
    const body = (await response.json()) as { code: string; expires_at: number };
    expect(body.code).toMatch(/^[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{8}$/);
    expect(body.expires_at).toBeGreaterThan(Date.now());
  });

  it("error: only the desk itself may ask", async () => {
    const desk = await newDesk();
    const other = await newDesk();
    const response = await api(`/v1/desks/${desk.deskId}/pairings`, {
      method: "POST",
      token: other.token,
      body: {},
    });
    expect(response.status).toBe(401);
  });

  it("a new code replaces the old one — at most one is outstanding", async () => {
    const desk = await newDesk();
    const first = await codeFor(desk);
    const second = await codeFor(desk);
    expect(first).not.toBe(second);

    expect((await redeem(desk.deskId, first)).status).toBe(400);
    expect((await redeem(desk.deskId, second)).status).toBe(201);
  });

  it("the code is never written to the database", async () => {
    const desk = await newDesk();
    const code = await codeFor(desk);
    for (const table of ["licenses", "desks", "viewers"]) {
      const rows = await testDb().prepare(`SELECT * FROM ${table}`).all();
      expect(JSON.stringify(rows.results ?? [])).not.toContain(code);
    }
  });
});

describe("redeeming a code", () => {
  it("happy: a phone gets a viewer id, a token and the desk's name", async () => {
    const desk = await newDesk();
    const response = await api("/v1/pair", {
      method: "POST",
      body: { desk_id: desk.deskId, code: await codeFor(desk), device_name: "Pixel 8" },
    });

    expect(response.status).toBe(201);
    const body = (await response.json()) as Record<string, unknown>;
    expect(body).toMatchObject({ desk_name: "Test desk" });
    expect(String(body.viewer_token)).toMatch(/^mu_v_[A-Za-z0-9_-]{43}$/);

    const row = await testDb()
      .prepare("SELECT device_name, revoked_at FROM viewers WHERE viewer_id = ?")
      .bind(body.viewer_id)
      .first<{ device_name: string; revoked_at: number | null }>();
    expect(row).toMatchObject({ device_name: "Pixel 8", revoked_at: null });
  });

  it("accepts a code the way a person types it: lower case, spaced", async () => {
    const desk = await newDesk();
    const code = await codeFor(desk);
    const typed = `${code.slice(0, 4).toLowerCase()} ${code.slice(4).toLowerCase()}`;
    expect((await redeem(desk.deskId, typed)).status).toBe(201);
  });

  it("empty: an empty code is `bad_code` and does not spend an attempt", async () => {
    const desk = await newDesk();
    const code = await codeFor(desk);

    for (let i = 0; i < 12; i += 1) {
      const response = await redeem(desk.deskId, "");
      expect(response.status).toBe(400);
      await expect(response.json()).resolves.toMatchObject({ error: { code: "bad_code" } });
    }
    // Twelve empty submissions did not lock the desk out; the real code works.
    expect((await redeem(desk.deskId, code)).status).toBe(201);
  });

  it("nil: an unknown desk is `not_found`, and says nothing about codes", async () => {
    const response = await redeem(crypto.randomUUID(), "ABCD2345");
    expect(response.status).toBe(404);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "not_found" } });
  });

  it("error: a wrong code is `bad_code`", async () => {
    const desk = await newDesk();
    await codeFor(desk);
    const response = await redeem(desk.deskId, "ZZZZZZZZ");
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "bad_code" } });
  });

  it("error: a code past its ttl is `code_expired`, not `bad_code`", async () => {
    const desk = await newDesk();
    const code = await codeFor(desk);
    await sleep(1_100); // PAIRING_TTL_MS is 1000 in the test bindings.

    const response = await redeem(desk.deskId, code);
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "code_expired" } });
  });

  it("error: ten wrong codes lock pairing, and the lock lifts by itself", async () => {
    const desk = await newDesk();
    await codeFor(desk);

    for (let attempt = 1; attempt < 10; attempt += 1) {
      expect((await redeem(desk.deskId, "ZZZZZZZZ")).status).toBe(400);
    }
    const locked = await redeem(desk.deskId, "ZZZZZZZZ");
    expect(locked.status).toBe(429);
    await expect(locked.json()).resolves.toMatchObject({ error: { code: "pairing_locked" } });

    // Even a fresh, correct code is refused while the lock holds.
    const fresh = await codeFor(desk);
    expect((await redeem(desk.deskId, fresh)).status).toBe(429);

    await sleep(2_100); // PAIRING_LOCKOUT_MS is 2000 in the test bindings.
    expect((await redeem(desk.deskId, await codeFor(desk))).status).toBe(201);
  });

  it("error: a desk at its viewer limit answers `viewer_limit`", async () => {
    const desk = await newDesk({ maxViewers: 1 });
    await pairViewer(desk);

    const response = await redeem(desk.deskId, await codeFor(desk));
    expect(response.status).toBe(409);
    await expect(response.json()).resolves.toMatchObject({ error: { code: "viewer_limit" } });

    // Revoking a device frees the slot, and the same code still works: the
    // limit check runs before the code is spent.
    const paired = await testDb()
      .prepare("SELECT viewer_id FROM viewers WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ viewer_id: string }>();
    await api(`/v1/desks/${desk.deskId}/viewers/${paired?.viewer_id}`, {
      method: "DELETE",
      token: desk.token,
    });
    expect((await redeem(desk.deskId, await codeFor(desk))).status).toBe(201);
  });

  it("hostile QA: the same code submitted twice at once pairs exactly one device", async () => {
    const desk = await newDesk({ maxViewers: 5 });
    const code = await codeFor(desk);

    const results = await Promise.all([
      redeem(desk.deskId, code),
      redeem(desk.deskId, code),
      redeem(desk.deskId, code),
    ]);
    const created = results.filter((r) => r.status === 201);
    expect(created).toHaveLength(1);

    const rows = await testDb()
      .prepare("SELECT COUNT(*) AS n FROM viewers WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ n: number }>();
    expect(rows?.n).toBe(1);
  });

  it("error: the sixth pairing attempt from one address in a minute is throttled", async () => {
    const desk = await newDesk({ maxViewers: 10 });
    const ip = `192.0.2.${Math.floor(Math.random() * 250) + 1}`;

    for (let i = 0; i < 5; i += 1) {
      expect((await redeem(desk.deskId, await codeFor(desk), ip)).status).toBe(201);
    }
    const throttled = await redeem(desk.deskId, await codeFor(desk), ip);
    expect(throttled.status).toBe(429);
    await expect(throttled.json()).resolves.toMatchObject({ error: { code: "rate_limited" } });
  });
});
