/**
 * Event fan-out, the cached snapshot, and `desk_status`.
 */
import { describe, expect, it } from "vitest";

import { connect, helloPayload, joinDesk, joinViewer, newDesk, pairViewer, snapshotEvent } from "../helpers";

describe("event fan-out", () => {
  it("happy: a desk event reaches every viewer verbatim", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: a } = await joinViewer(desk, await pairViewer(desk, "phone a"));
    const { socket: b } = await joinViewer(desk, await pairViewer(desk, "phone b"));

    const event = { event: "desk:state-changed", payload: { state: "Standing" } };
    const id = deskSocket.send("event", event);

    for (const viewer of [a, b]) {
      const frame = await viewer.next();
      expect(frame).toMatchObject({ type: "event", id, payload: event });
    }
  });

  it("empty: a desk event with no viewers is cached, not an error", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    deskSocket.send("event", snapshotEvent(900));

    const { welcome } = await joinViewer(desk, await pairViewer(desk));
    expect(welcome.payload).toMatchObject({
      desk_online: true,
      snapshot: snapshotEvent(900),
    });
    expect((welcome.payload as { snapshot_ts: number }).snapshot_ts).toBeGreaterThan(0);
  });

  it("caches only snapshots — a delta would lie to a late viewer", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    deskSocket.send("event", snapshotEvent(100));
    deskSocket.send("event", { event: "desk:state-changed", payload: { state: "Standing" } });

    const { welcome } = await joinViewer(desk, await pairViewer(desk));
    expect(welcome.payload).toMatchObject({ snapshot: snapshotEvent(100) });
  });

  it("the newest snapshot wins", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    deskSocket.send("event", snapshotEvent(100));
    deskSocket.send("event", snapshotEvent(200));

    const { welcome } = await joinViewer(desk, await pairViewer(desk));
    expect(welcome.payload).toMatchObject({ snapshot: snapshotEvent(200) });
  });

  it("error: a viewer may not publish events", async () => {
    const desk = await newDesk();
    const { socket: viewer } = await joinViewer(desk, await pairViewer(desk));
    viewer.send("event", snapshotEvent(1));
    await expect(viewer.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "forbidden" },
    });
  });
});

describe("desk_status", () => {
  it("viewers are told when a desk arrives", async () => {
    const desk = await newDesk();
    const { socket: viewer } = await joinViewer(desk, await pairViewer(desk));
    await joinDesk(desk);

    const frame = await viewer.next();
    expect(frame.type).toBe("desk_status");
    expect(frame.payload).toMatchObject({ online: true });
  });

  it("viewers are told when the desk drops", async () => {
    const desk = await newDesk();
    const { socket: deskSocket } = await joinDesk(desk);
    const { socket: viewer } = await joinViewer(desk, await pairViewer(desk));

    deskSocket.ws.close(1000, "bye");
    const frame = await viewer.next();
    expect(frame.type).toBe("desk_status");
    expect(frame.payload).toMatchObject({ online: false });
  });

  it("a viewer joining an occupied room sees the right viewer_count", async () => {
    const desk = await newDesk();
    await joinDesk(desk);
    await joinViewer(desk, await pairViewer(desk, "phone a"));
    const { welcome } = await joinViewer(desk, await pairViewer(desk, "phone b"));
    // The joining socket is authenticated before `welcome` is built, so it
    // counts itself: one earlier viewer plus this one.
    expect(welcome.payload).toMatchObject({ viewer_count: 2, desk_online: true });
  });

  it("a second desk connection replaces the first with 4409", async () => {
    const desk = await newDesk();
    const { socket: first } = await joinDesk(desk);
    const second = await connect(desk.deskId);
    second.send("hello", helloPayload("desk", desk.deskId, desk.token));

    await expect(first.closed()).resolves.toMatchObject({ code: 4409 });
    await expect(second.next()).resolves.toMatchObject({ type: "welcome" });
  });
});
