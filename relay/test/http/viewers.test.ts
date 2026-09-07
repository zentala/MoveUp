/**
 * The three desk-authenticated routes: listing paired devices, revoking one,
 * and disabling the relay for a desk entirely.
 *
 * Revocation has two halves — the row and the open socket — and both are
 * asserted here, because a revoke that only updates D1 leaves a phone watching
 * a live stream it is no longer entitled to.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { api, joinDesk, joinViewer, newDesk, pairViewer, testDb } from "../helpers";

describe("GET /v1/desks/{id}/viewers", () => {
  it("empty: a desk with no paired devices answers an empty list, not an error", async () => {
    const desk = await newDesk();
    const response = await api(`/v1/desks/${desk.deskId}/viewers`, { token: desk.token });
    expect(response.status).toBe(200);
    await expect(response.json()).resolves.toEqual([]);
  });

  it("happy: paired devices are listed with their online state", async () => {
    const desk = await newDesk();
    const offline = await pairViewer(desk, "Old tablet");
    const online = await pairViewer(desk, "Pixel 8");
    await joinViewer(desk, online);

    const response = await api(`/v1/desks/${desk.deskId}/viewers`, { token: desk.token });
    expect(response.status).toBe(200);
    const rows = (await response.json()) as Array<Record<string, unknown>>;

    expect(rows).toHaveLength(2);
    expect(rows.find((r) => r.viewer_id === online.viewerId)).toMatchObject({
      device_name: "Pixel 8",
      online: true,
    });
    expect(rows.find((r) => r.viewer_id === offline.viewerId)).toMatchObject({
      device_name: "Old tablet",
      online: false,
    });
    expect(rows.every((r) => typeof r.paired_at === "number")).toBe(true);
  });

  it("never leaks a credential into the list", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const response = await api(`/v1/desks/${desk.deskId}/viewers`, { token: desk.token });
    const text = await response.text();
    expect(text).not.toContain(viewer.token);
    expect(text).not.toContain("token_hash");
  });

  it("error: another desk's token cannot read this desk's devices", async () => {
    const desk = await newDesk();
    await pairViewer(desk);
    const other = await newDesk();
    const response = await api(`/v1/desks/${desk.deskId}/viewers`, { token: other.token });
    expect(response.status).toBe(401);
  });
});

describe("DELETE /v1/desks/{id}/viewers/{viewerId}", () => {
  it("happy: the row is revoked and the open socket is closed 4403", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const { socket } = await joinViewer(desk, viewer);

    const response = await api(`/v1/desks/${desk.deskId}/viewers/${viewer.viewerId}`, {
      method: "DELETE",
      token: desk.token,
    });
    expect(response.status).toBe(204);
    await expect(socket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.REVOKED });

    const row = await testDb()
      .prepare("SELECT revoked_at FROM viewers WHERE viewer_id = ?")
      .bind(viewer.viewerId)
      .first<{ revoked_at: number | null }>();
    expect(row?.revoked_at).toBeGreaterThan(0);
  });

  it("a revoked device may not reconnect", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    await api(`/v1/desks/${desk.deskId}/viewers/${viewer.viewerId}`, {
      method: "DELETE",
      token: desk.token,
    });

    const { connect, helloPayload } = await import("../helpers");
    const socket = await connect(desk.deskId);
    socket.send("hello", helloPayload("viewer", desk.deskId, viewer.token));
    await expect(socket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.REVOKED });
  });

  it("it drops out of the Settings list", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    await api(`/v1/desks/${desk.deskId}/viewers/${viewer.viewerId}`, {
      method: "DELETE",
      token: desk.token,
    });

    const response = await api(`/v1/desks/${desk.deskId}/viewers`, { token: desk.token });
    await expect(response.json()).resolves.toEqual([]);
  });

  it("nil: revoking a device that is not paired here is 404, not a silent 204", async () => {
    const desk = await newDesk();
    const response = await api(`/v1/desks/${desk.deskId}/viewers/${crypto.randomUUID()}`, {
      method: "DELETE",
      token: desk.token,
    });
    expect(response.status).toBe(404);
  });

  it("error: revoking the same device twice is 404 the second time", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const path = `/v1/desks/${desk.deskId}/viewers/${viewer.viewerId}`;
    expect((await api(path, { method: "DELETE", token: desk.token })).status).toBe(204);
    expect((await api(path, { method: "DELETE", token: desk.token })).status).toBe(404);
  });
});

describe("DELETE /v1/desks/{id}", () => {
  it("happy: every credential for the desk stops working and every socket closes", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, viewer);

    const response = await api(`/v1/desks/${desk.deskId}`, {
      method: "DELETE",
      token: desk.token,
    });
    expect(response.status).toBe(204);

    for (const socket of [deskSocket, viewerSocket]) {
      await expect(socket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.REVOKED });
    }

    const rows = await testDb()
      .prepare("SELECT COUNT(*) AS n FROM viewers WHERE desk_id = ?")
      .bind(desk.deskId)
      .first<{ n: number }>();
    expect(rows?.n).toBe(0);

    // The desk's own token is now worthless, so the route refuses a second call.
    expect(
      (await api(`/v1/desks/${desk.deskId}`, { method: "DELETE", token: desk.token })).status,
    ).toBe(401);
  });

  it("frees the license slot, so the same machine can register again", async () => {
    const desk = await newDesk({ maxDesks: 1 });
    await api(`/v1/desks/${desk.deskId}`, { method: "DELETE", token: desk.token });

    const again = await api("/v1/desks/register", {
      method: "POST",
      body: { license_key: desk.licenseKey, desk_name: "Studio PC" },
    });
    expect(again.status).toBe(201);
  });

  it("error: a desk cannot disable another desk", async () => {
    const desk = await newDesk();
    const other = await newDesk();
    const response = await api(`/v1/desks/${desk.deskId}`, {
      method: "DELETE",
      token: other.token,
    });
    expect(response.status).toBe(401);
  });
});
