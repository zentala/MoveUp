/**
 * The `hello` → `welcome` handshake and everything that can go wrong in it.
 * Four shadow paths per `rules/testing.md`: happy, nil, empty, error.
 *
 * Credentials here are minted by the real REST routes (see `helpers.ts`), so
 * these tests cover `verifyToken`'s D1 path as well as the handshake.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { connect, helloPayload, joinDesk, joinViewer, newDesk, pairViewer } from "../helpers";

describe("e023 frame byte limit", () => {
  const limit = 16 * 1024;

  it("oversize malformed text closes 1009 before JSON parsing or authentication", async () => {
    const socket = await connect("e023-oversize-pending");
    socket.raw("{".repeat(limit + 1));
    await expect(socket.closed()).resolves.toMatchObject({ code: 1009 });
  });

  it("counts UTF-8 bytes rather than characters", async () => {
    const { socket } = await joinDesk(await newDesk());
    const text = JSON.stringify({ v: 1, type: "ping", id: "utf8", ts: 0, payload: null,
      padding: "é".repeat(limit / 2) });
    expect(text.length).toBeLessThan(limit);
    socket.raw(text);
    await expect(socket.closed()).resolves.toMatchObject({ code: 1009 });
  });

  it.each(["ascii", "unicode"])("a valid %s frame at exactly 16 KiB is processed", async (kind) => {
    const { socket } = await joinDesk(await newDesk());
    const base = JSON.stringify({ v: 1, type: "ping", id: "boundary", ts: 0, payload: null,
      padding: kind === "unicode" ? "é".repeat(100) : "" });
    const raw = base + " ".repeat(limit - new TextEncoder().encode(base).byteLength);
    expect(new TextEncoder().encode(raw).byteLength).toBe(limit);
    socket.raw(raw);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id: "boundary" });
    socket.ws.close();
  });

  it.each([limit, limit + 1])("binary frame of %i bytes respects size before binary policy", async (bytes) => {
    const socket = await connect(`e023-binary-${bytes}`);
    socket.ws.send(new ArrayBuffer(bytes));
    await expect(socket.closed()).resolves.toMatchObject({
      code: bytes > limit ? 1009 : CLOSE_CODES.PROTOCOL_ERROR,
    });
  });

  it("empty text keeps the existing malformed-JSON close", async () => {
    const socket = await connect("e023-empty");
    socket.raw("");
    await expect(socket.closed()).resolves.toMatchObject({ code: CLOSE_CODES.PROTOCOL_ERROR });
  });
});

describe("hello / welcome", () => {
  it("happy: a desk is welcomed and told it is online", async () => {
    const desk = await newDesk();
    const { welcome } = await joinDesk(desk);
    expect(welcome.payload).toMatchObject({
      role: "desk",
      desk_id: desk.deskId,
      desk_online: true,
      viewer_count: 0,
    });
  });

  it("echoes the hello's id so a client can correlate the reply", async () => {
    const desk = await newDesk();
    const socket = await connect(desk.deskId);
    const id = socket.send("hello", helloPayload("desk", desk.deskId, desk.token));
    await expect(socket.next()).resolves.toMatchObject({ id, type: "welcome" });
  });

  it("nil: a viewer that arrives before the desk gets a null snapshot, not a spinner", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const { welcome } = await joinViewer(desk, viewer);
    expect(welcome.payload).toMatchObject({
      desk_online: false,
      snapshot: null,
      snapshot_ts: null,
    });
  });

  it("empty: an empty token is rejected 4401", async () => {
    const desk = await newDesk();
    const socket = await connect(desk.deskId);
    socket.send("hello", { ...helloPayload("viewer", desk.deskId, desk.token), token: "" });
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
    });
  });

  it("error: a token with the wrong role prefix is rejected 4401", async () => {
    const desk = await newDesk();
    const socket = await connect(desk.deskId);
    // The desk's own, valid token — offered for the viewer role.
    socket.send("hello", helloPayload("viewer", desk.deskId, desk.token));
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
    });
  });

  it("error: a well-formed token belonging to nobody is rejected 4401", async () => {
    const desk = await newDesk();
    const socket = await connect(desk.deskId);
    socket.send("hello", helloPayload("desk", desk.deskId, `mu_d_${"a".repeat(43)}`));
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
    });
  });

  it("error: any frame before hello closes the socket 4401", async () => {
    const socket = await connect("handshake-early");
    socket.send("ping", null);
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
    });
  });

  it("error: no hello within the timeout closes the socket 4401", async () => {
    // HELLO_TIMEOUT_MS is 250 in vitest.config.ts; the alarm does the closing.
    const socket = await connect("handshake-timeout");
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
      reason: "no hello in time",
    });
  });

  it("error: a second hello on the same socket is answered, not honoured", async () => {
    const desk = await newDesk();
    const viewer = await pairViewer(desk);
    const { socket } = await joinViewer(desk, viewer);
    socket.send("hello", helloPayload("viewer", desk.deskId, viewer.token));
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "already_authenticated" },
    });
  });
});
