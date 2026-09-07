/**
 * Command routing through the real Durable Object (E022-T04).
 *
 * Every case here goes over real sockets with real credentials, because the
 * thing under test is precisely who receives which frame — a stubbed room
 * would only prove that the stub agrees with the test's idea of the room.
 */
import { describe, expect, it } from "vitest";

import { joinDesk, joinViewer, newDesk, pairViewer, type TestSocket } from "../helpers";

const ackAlert = { name: "ack_alert", args: {} };

/**
 * Asserts nothing arrives on a socket.
 *
 * Deliberately short and deliberately not `next()`: the helper's own timeout is
 * as long as vitest's, so waiting for it would fail the test with a timeout
 * instead of the assertion. Every call sits at the end of its test, so the
 * waiter left behind cannot steal a later frame.
 */
async function expectSilence(socket: TestSocket, what: string): Promise<void> {
  const silent = Symbol("silent");
  const outcome = await Promise.race([
    socket.next().then<unknown, unknown>(
      (frame) => frame,
      () => silent,
    ),
    new Promise<unknown>((resolve) => setTimeout(() => resolve(silent), 300)),
  ]);
  expect(outcome, `${what} received a frame it should not have`).toBe(silent);
}

describe("command → desk", () => {
  it("happy: the desk receives the command with the sender's viewer_id", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const viewer = await pairViewer(desk);
    const { socket: viewerSocket } = await joinViewer(desk, viewer);

    const id = viewerSocket.send("command", ackAlert);

    const forwarded = await deskSocket.next();
    expect(forwarded).toMatchObject({
      type: "command",
      id,
      payload: { name: "ack_alert", args: {}, viewer_id: viewer.viewerId },
    });
  });

  it("the forwarded envelope keeps the viewer's own ts, so staleness stays checkable", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    const sentAt = Date.now() - 45_000;
    viewerSocket.ws.send(
      JSON.stringify({ v: 1, type: "command", id: "stale-1", ts: sentAt, payload: ackAlert }),
    );

    const forwarded = await deskSocket.next();
    expect(forwarded.ts).toBe(sentAt);
  });

  it("a command is never fanned out to the other viewers", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: other } = await joinViewer(desk, await pairViewer(desk, "phone a"));
    const { socket: sender } = await joinViewer(desk, await pairViewer(desk, "phone b"));

    sender.send("command", ackAlert);
    await deskSocket.next();

    // `other` only ever saw the desk_status for phone b joining; nothing else
    // may be queued behind it.
    await expectSilence(other, "the bystanding viewer");
  });

  it("nil: with no desk connected the viewer is told desk_offline immediately", async () => {
    const desk = await newDesk();
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    const id = viewerSocket.send("command", ackAlert);

    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "command_result",
      payload: { command_id: id, ok: false, error: { code: "desk_offline" } },
    });
  });

  it("empty: set_limits with no arguments is refused and never forwarded", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    const id = viewerSocket.send("command", { name: "set_limits", args: {} });

    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "error",
      id,
      payload: { code: "bad_args" },
    });
    await expectSilence(deskSocket, "the desk");
  });

  it("error: an unknown command name is refused with unknown_command", async () => {
    const desk = await newDesk();
    await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    viewerSocket.send("command", { name: "rm_rf", args: {} });
    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "unknown_command" },
    });
  });

  it("error: an out-of-range argument is refused", async () => {
    const desk = await newDesk();
    await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    viewerSocket.send("command", { name: "set_limits", args: { sit_min: 4 } });
    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "bad_args" },
    });
  });

  it("error: a desk may not send commands", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);

    deskSocket.send("command", ackAlert);
    await expect(deskSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "forbidden" },
    });
  });
});

describe("command_result → the one viewer that asked", () => {
  it("happy: the result reaches the originating viewer only", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: bystander } = await joinViewer(desk, await pairViewer(desk, "phone a"));
    const { socket: sender } = await joinViewer(desk, await pairViewer(desk, "phone b"));

    const id = sender.send("command", ackAlert);
    await deskSocket.next();
    deskSocket.send("command_result", { command_id: id, ok: true, error: null });

    await expect(sender.next()).resolves.toMatchObject({
      type: "command_result",
      payload: { command_id: id, ok: true },
    });
    await expectSilence(bystander, "the bystanding viewer");
  });

  it("a failed result is routed the same way, error body intact", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: sender } = await joinViewer(desk, await pairViewer(desk));

    const id = sender.send("command", { name: "switch_profile", args: { kind: "ergonomic", name: "strict" } });
    await deskSocket.next();
    deskSocket.send("command_result", {
      command_id: id,
      ok: false,
      error: { code: "no_such_profile", message: "strict is not installed" },
    });

    await expect(sender.next()).resolves.toMatchObject({
      payload: { ok: false, error: { code: "no_such_profile" } },
    });
  });

  it("error: a result for an id nobody is waiting on is answered, not dropped", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);

    deskSocket.send("command_result", { command_id: "never-sent", ok: true, error: null });
    await expect(deskSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "unknown_command_id" },
    });
  });

  it("error: the same result twice — the second finds the entry consumed", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: sender } = await joinViewer(desk, await pairViewer(desk));

    const id = sender.send("command", ackAlert);
    await deskSocket.next();
    deskSocket.send("command_result", { command_id: id, ok: true, error: null });
    await sender.next();

    deskSocket.send("command_result", { command_id: id, ok: true, error: null });
    await expect(deskSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "unknown_command_id" },
    });
  });

  it("error: the desk answers after the viewer left — the desk is told", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: sender } = await joinViewer(desk, await pairViewer(desk));

    const id = sender.send("command", ackAlert);
    await deskSocket.next();
    sender.ws.close(1000, "bye");
    await sender.closed();
    // `closed()` resolves on the client side; the room's own close handler runs
    // just after it, and there is no frame to wait on for that.
    await new Promise((resolve) => setTimeout(resolve, 100));
    deskSocket.send("command_result", { command_id: id, ok: true, error: null });

    await expect(deskSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "viewer_gone" },
    });
  });

  it("error: a viewer may not send a command_result", async () => {
    const desk = await newDesk();
    await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    viewerSocket.send("command_result", { command_id: "x", ok: true, error: null });
    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "forbidden" },
    });
  });
});

describe("rate limit", () => {
  it("error: the 11th command in a minute is refused, the first ten are not", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    for (let i = 0; i < 10; i += 1) {
      viewerSocket.send("command", ackAlert);
      const forwarded = await deskSocket.next();
      expect(forwarded.type).toBe("command");
    }

    viewerSocket.send("command", ackAlert);
    await expect(viewerSocket.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "rate_limited" },
    });
    await expectSilence(deskSocket, "the desk");
  });

  it("the window is per viewer, not per room", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: noisy } = await joinViewer(desk, await pairViewer(desk, "phone a"));
    const { socket: quiet } = await joinViewer(desk, await pairViewer(desk, "phone b"));

    for (let i = 0; i < 10; i += 1) {
      noisy.send("command", ackAlert);
      await deskSocket.next();
    }
    noisy.send("command", ackAlert);
    await expect(noisy.next()).resolves.toMatchObject({ payload: { code: "rate_limited" } });

    quiet.send("command", ackAlert);
    await expect(deskSocket.next()).resolves.toMatchObject({ type: "command" });
  });

  it("a refused command is not counted against the window", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewerSocket } = await joinViewer(desk, await pairViewer(desk));

    // Ten refusals cost ten tokens — the throttle is checked before the
    // allowlist on purpose, so a viewer cannot spam invalid names for free.
    for (let i = 0; i < 10; i += 1) {
      viewerSocket.send("command", { name: "rm_rf", args: {} });
      await expect(viewerSocket.next()).resolves.toMatchObject({
        payload: { code: "unknown_command" },
      });
    }

    viewerSocket.send("command", ackAlert);
    await expect(viewerSocket.next()).resolves.toMatchObject({
      payload: { code: "rate_limited" },
    });
    await expectSilence(deskSocket, "the desk");
  });
});
