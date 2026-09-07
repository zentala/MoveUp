/**
 * Ping/pong, the idle close, and the frames the room refuses to accept.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { connect, join, helloPayload } from "../helpers";

describe("ping / pong", () => {
  it("happy: a ping is answered with a pong carrying the same id", async () => {
    const { socket } = await join("viewer", "live-ping");
    const id = socket.send("ping", null);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id, payload: null });
  });

  it("a pong from the client is accepted silently and keeps the socket alive", async () => {
    const { socket } = await join("viewer", "live-pong");
    socket.send("pong", null);
    // Nothing comes back, but the socket must still be usable.
    const id = socket.send("ping", null);
    await expect(socket.next()).resolves.toMatchObject({ type: "pong", id });
  });

  it("error: a silent socket is closed once the idle timeout passes", async () => {
    // IDLE_TIMEOUT_MS is 700 in vitest.config.ts.
    const { socket } = await join("viewer", "live-idle");
    await expect(socket.closed()).resolves.toMatchObject({ code: 1001, reason: "idle" });
  });
});

describe("malformed input", () => {
  it("error: a frame that is not JSON closes the socket 4400", async () => {
    const { socket } = await join("viewer", "bad-json");
    socket.raw("{not json");
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.PROTOCOL_ERROR,
    });
  });

  it("error: an unsupported protocol version closes the socket 4400", async () => {
    const socket = await connect("bad-version");
    socket.raw(
      JSON.stringify({ v: 2, type: "hello", id: "x", ts: Date.now(), payload: helloPayload("desk") }),
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
    const { socket } = await join("viewer", "unknown-type");
    const id = socket.send("telepathy", { thought: "stand up" });
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      id,
      payload: { code: "unknown_type" },
    });
  });

  it("a known type with a bad payload is answered, not fatal", async () => {
    const { socket } = await join("viewer", "bad-payload");
    const id = socket.send("command", { name: "", args: {} });
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      id,
      payload: { code: "bad_payload" },
    });
  });
});
