/**
 * `POST /v1/desks/register` — the only route that turns a licence into a
 * credential. Four shadow paths, plus the per-IP throttle.
 */
import { describe, expect, it } from "vitest";

import { api, seedLicense, testDb } from "../helpers";

const register = (body: unknown, ip?: string) =>
  api("/v1/desks/register", { method: "POST", body, ip });

describe("register", () => {
  it("happy: a valid license yields a desk id, a token and the plan", async () => {
    const key = await seedLicense({ plan: "founder", maxDesks: 2, expiresAt: null });
    const response = await register({
      license_key: key,
      desk_name: "Studio PC",
      app_version: "0.7.0",
    });

    expect(response.status).toBe(201);
    const body = (await response.json()) as Record<string, unknown>;
    expect(body).toMatchObject({ plan: "founder", expires_at: null });
    expect(String(body.desk_token)).toMatch(/^mu_d_[A-Za-z0-9_-]{43}$/);
    expect(String(body.desk_id)).toHaveLength(36);

    const row = await testDb()
      .prepare("SELECT desk_name, app_version, last_seen FROM desks WHERE desk_id = ?")
      .bind(body.desk_id)
      .first<{ desk_name: string; app_version: string; last_seen: number | null }>();
    expect(row).toMatchObject({ desk_name: "Studio PC", app_version: "0.7.0", last_seen: null });
  });

  it("names a desk that did not name itself, rather than storing an empty string", async () => {
    const key = await seedLicense();
    const response = await register({ license_key: key });
    expect(response.status).toBe(201);
    const { desk_id: deskId } = (await response.json()) as { desk_id: string };
    const row = await testDb()
      .prepare("SELECT desk_name, app_version FROM desks WHERE desk_id = ?")
      .bind(deskId)
      .first<{ desk_name: string; app_version: string }>();
    expect(row).toMatchObject({ desk_name: "desk", app_version: "unknown" });
  });

  it("nil: no license_key is `invalid_license`, not a crash", async () => {
    const response = await register({ desk_name: "nameless" });
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "invalid_license" },
    });
  });

  it("nil: no body at all is `invalid_license`", async () => {
    const response = await api("/v1/desks/register", { method: "POST" });
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "invalid_license" },
    });
  });

  it("empty: an empty license_key is `invalid_license`", async () => {
    const response = await register({ license_key: "   " });
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "invalid_license" },
    });
  });

  it("error: an unknown license is refused", async () => {
    const response = await register({ license_key: "mu_lic_not-a-real-key" });
    expect(response.status).toBe(400);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "invalid_license" },
    });
  });

  it("error: an expired license is refused with 403", async () => {
    const key = await seedLicense({ expiresAt: Date.now() - 1_000 });
    const response = await register({ license_key: key });
    expect(response.status).toBe(403);
    await expect(response.json()).resolves.toMatchObject({
      error: { code: "invalid_license" },
    });
  });

  it("error: a license at its desk limit is `license_exhausted`", async () => {
    const key = await seedLicense({ maxDesks: 1 });
    expect((await register({ license_key: key })).status).toBe(201);

    const second = await register({ license_key: key });
    expect(second.status).toBe(409);
    await expect(second.json()).resolves.toMatchObject({
      error: { code: "license_exhausted" },
    });
  });

  it("error: the sixth registration from one address in a minute is throttled", async () => {
    const ip = `203.0.113.${Math.floor(Math.random() * 250) + 1}`;
    const key = await seedLicense({ maxDesks: 10 });

    for (let i = 0; i < 5; i += 1) {
      expect((await register({ license_key: key }, ip)).status).toBe(201);
    }
    const throttled = await register({ license_key: key }, ip);
    expect(throttled.status).toBe(429);
    await expect(throttled.json()).resolves.toMatchObject({ error: { code: "rate_limited" } });

    // The throttle is per address, not global.
    expect((await register({ license_key: key }, "198.51.100.7")).status).toBe(201);
  });

  it("bounds what it stores: an absurd desk_name is refused, not truncated silently", async () => {
    const key = await seedLicense();
    const response = await register({ license_key: key, desk_name: "x".repeat(500) });
    // Over-long optional fields fall back to the default rather than failing the
    // registration — but nothing over 64 characters reaches a row.
    expect(response.status).toBe(201);
    const { desk_id: deskId } = (await response.json()) as { desk_id: string };
    const row = await testDb()
      .prepare("SELECT desk_name FROM desks WHERE desk_id = ?")
      .bind(deskId)
      .first<{ desk_name: string }>();
    expect(row?.desk_name).toBe("desk");
  });
});
