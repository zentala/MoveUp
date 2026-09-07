/**
 * The `hello` → `welcome` handshake and everything that can go wrong in it.
 * Four shadow paths per `rules/testing.md`: happy, nil, empty, error.
 */
import { describe, expect, it } from "vitest";

import { CLOSE_CODES } from "@app/remote/protocol";

import { connect, helloPayload, join } from "../helpers";

describe("hello / welcome", () => {
  it("happy: a desk is welcomed and told it is online", async () => {
    const { welcome } = await join("desk", "handshake-happy");
    expect(welcome.payload).toMatchObject({
      role: "desk",
      desk_id: "handshake-happy",
      desk_online: true,
      viewer_count: 0,
    });
  });

  it("echoes the hello's id so a client can correlate the reply", async () => {
    const socket = await connect("handshake-echo");
    const id = socket.send("hello", helloPayload("desk", "handshake-echo"));
    await expect(socket.next()).resolves.toMatchObject({ id, type: "welcome" });
  });

  it("nil: a viewer that arrives before any desk gets a null snapshot, not a spinner", async () => {
    const { welcome } = await join("viewer", "handshake-nodesk");
    expect(welcome.payload).toMatchObject({
      desk_online: false,
      snapshot: null,
      snapshot_ts: null,
    });
  });

  it("empty: an empty token is rejected 4401", async () => {
    const socket = await connect("handshake-empty");
    socket.send("hello", { ...helloPayload("viewer", "handshake-empty"), token: "" });
    await expect(socket.closed()).resolves.toMatchObject({
      code: CLOSE_CODES.UNAUTHENTICATED,
    });
  });

  it("error: a token with the wrong role prefix is rejected 4401", async () => {
    const socket = await connect("handshake-wrongprefix");
    socket.send("hello", {
      ...helloPayload("viewer", "handshake-wrongprefix"),
      token: `mu_d_${"a".repeat(43)}`,
    });
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
    const { socket } = await join("viewer", "handshake-twice");
    socket.send("hello", helloPayload("viewer", "handshake-twice"));
    await expect(socket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "already_authenticated" },
    });
  });
});
