/**
 * Ping/pong, the idle close, and the frames the room refuses to accept.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { connect, helloPayload, joinViewer, newDesk, pairViewer, type Desk, type Viewer } from "../helpers";

/** A desk with one paired phone; only the phone's socket is used below. */
async function phone(): Promise<{ desk: Desk; viewer: Viewer }> {
  const desk = await newDesk();
  return { desk, viewer: await pairViewer(desk) };
}

describe("ping / pong", () => {
  it("happy: a ping is answered with a pong carrying the same id", async () => {
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    const id = socket.send("ping", null);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id, payload: null });
  });

  it("a pong from the client is accepted silently and keeps the socket alive", async () => {
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    socket.send("pong", null);
    // Nothing comes back, but the socket must still be usable.
    const id = socket.send("ping", null);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id });
  });

  it("error: a silent socket is closed once the idle timeout passes", async () => {
    // IDLE_TIMEOUT_MS is 700 in vitest.config.ts.
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    await expect(socket.closed()).resolves.toMatchObject({ code: 1001, reason: "idle" });
  });
});

describe("malformed input", () => {
  it("error: a frame that is not JSON closes the socket 4400", async () => {
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    socket.raw("{not json");
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.PROTOCOL_ERROR,
    });
  });

  it("error: an unsupported protocol version closes the socket 4400", async () => {
    const desk = await newDesk();
    const socket = await connect(desk.deskId);
    socket.raw(
      JSON.stringify({
        v: 2,
        type: "hello",
        id: "x",
        ts: Date.now(),
        payload: helloPayload("desk", desk.deskId, desk.token),
      }),
    );
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.PROTOCOL_ERROR,
    });
  });

  it("error: a frame that is not an envelope closes the socket 4400", async () => {
    const socket = await connect("bad-envelope");
    socket.raw(JSON.stringify({ hello: "there" }));
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.PROTOCOL_ERROR,
    });
  });

  it("an unknown type is answered, never fatal — a newer peer stays connected", async () => {
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    const id = socket.send("telepathy", { thought: "stand up" });
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      id,
      payload: { code: "unknown_type" },
    });
  });

  it("a known type with a bad payload is answered, not fatal", async () => {
    const { desk, viewer } = await phone();
    const { socket } = await joinViewer(desk, viewer);
    const id = socket.send("command", { name: "", args: {} });
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      id,
      payload: { code: "bad_payload" },
    });
  });
});
