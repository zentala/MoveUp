/**
 * Event fan-out, the cached snapshot, and `desk_status`.
 */
import { describe, expect, it } from "vitest";

import { connect, join, snapshotEvent } from "../helpers";

describe("event fan-out", () => {
  it("happy: a desk event reaches every viewer verbatim", async () => {
    const { socket: desk } = await join("desk", "fanout-happy");
    const { socket: a } = await join("viewer", "fanout-happy");
    const { socket: b } = await join("viewer", "fanout-happy");

    const event = { event: "desk:state-changed", payload: { state: "Standing" } };
    const id = desk.send("event", event);

    for (const viewer of [a, b]) {
      const frame = await viewer.next();
      expect(frame).toMatchObject({ type: "event", id, payload: event });
    }
  });

  it("empty: a desk event with no viewers is cached, not an error", async () => {
    const { socket: desk } = await join("desk", "fanout-empty");
    desk.send("event", snapshotEvent(900));

    const { welcome } = await join("viewer", "fanout-empty");
    expect(welcome.payload).toMatchObject({
      desk_online: true,
      snapshot: snapshotEvent(900),
    });
    expect((welcome.payload as { snapshot_ts: number }).snapshot_ts).toBeGreaterThan(0);
  });

  it("caches only snapshots — a delta would lie to a late viewer", async () => {
    const { socket: desk } = await join("desk", "fanout-delta");
    desk.send("event", snapshotEvent(100));
    desk.send("event", { event: "desk:state-changed", payload: { state: "Standing" } });

    const { welcome } = await join("viewer", "fanout-delta");
    expect(welcome.payload).toMatchObject({ snapshot: snapshotEvent(100) });
  });

  it("the newest snapshot wins", async () => {
    const { socket: desk } = await join("desk", "fanout-newest");
    desk.send("event", snapshotEvent(100));
    desk.send("event", snapshotEvent(200));

    const { welcome } = await join("viewer", "fanout-newest");
    expect(welcome.payload).toMatchObject({ snapshot: snapshotEvent(200) });
  });

  it("error: a viewer may not publish events", async () => {
    const { socket: viewer } = await join("viewer", "fanout-forbidden");
    viewer.send("event", snapshotEvent(1));
    await expect(viewer.next()).resolves.toMatchObject({
      type: "error",
      payload: { code: "forbidden" },
    });
  });
});

describe("desk_status", () => {
  it("viewers are told when a desk arrives", async () => {
    const { socket: viewer } = await join("viewer", "status-arrive");
    await join("desk", "status-arrive");

    const frame = await viewer.next();
    expect(frame.type).toBe("desk_status");
    expect(frame.payload).toMatchObject({ online: true });
  });

  it("viewers are told when the desk drops", async () => {
    const { socket: desk } = await join("desk", "status-drop");
    const { socket: viewer } = await join("viewer", "status-drop");

    desk.ws.close(1000, "bye");
    const frame = await viewer.next();
    expect(frame.type).toBe("desk_status");
    expect(frame.payload).toMatchObject({ online: false });
  });

  it("a viewer joining an occupied room sees the right viewer_count", async () => {
    await join("desk", "status-count");
    await join("viewer", "status-count");
    const { welcome } = await join("viewer", "status-count");
    // The joining socket is authenticated before `welcome` is built, so it
    // counts itself: one earlier viewer plus this one.
    expect(welcome.payload).toMatchObject({ viewer_count: 2, desk_online: true });
  });

  it("a second desk replaces the first with 4409", async () => {
    const { socket: first } = await join("desk", "status-replace");
    const second = await connect("status-replace");
    second.send("hello", {
      role: "desk",
      desk_id: "status-replace",
      token: `mu_d_${"c".repeat(43)}`,
      client: { app: "moveup-desktop", version: "0.7.0" },
    });

    await expect(first.closed()).resolves.toMatchObject({ code: 4409 });
    await expect(second.next()).resolves.toMatchObject({ type: "welcome" });
  });
});
