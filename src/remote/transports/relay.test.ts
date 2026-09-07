/**
 * relay.test.ts — the cloud wire (E022-T08).
 *
 * Two behaviours carry most of the weight: the phone can be connected while
 * the desk is not, and a terminal close must NOT be retried. Both are
 * asserted as observable facts (a status value, a connection count), never as
 * "no error was thrown".
 */
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  MockWebSocket,
  makeEnvelope,
  makeSnapshot,
  setupMocks,
  teardownMocks,
} from "@/hooks/remoteDesk.test-helpers";
import { CLOSE_CODES } from "../protocol";
import { RELAY_STORAGE_KEY, saveRelayRecord, type RelayRecord } from "../storage";
import { RelayTransport, relayWsUrl } from "./relay";
import type { TransportMessage, TransportStatus } from "./types";

/** Last element, spelled out because the tsconfig targets ES2020 (no `Array.at`). */
const last = <T,>(items: T[]): T | undefined => items[items.length - 1];

const record: RelayRecord = {
  relay_url: "https://relay.desk.zentala.io",
  desk_id: "desk-1",
  viewer_id: "viewer-1",
  viewer_token: "mu_v_secret",
  desk_name: "Office",
};

let ids = 0;

function connected() {
  ids = 0;
  const transport = new RelayTransport(record, {
    random: () => 0.5,
    newId: () => `id-${(ids += 1)}`,
  });
  const messages: TransportMessage[] = [];
  const statuses: TransportStatus[] = [];
  transport.onMessage((m) => messages.push(m));
  transport.onStatus((s) => statuses.push(s));
  transport.connect();
  return { transport, messages, statuses, ws: MockWebSocket.latest() };
}

const welcome = (overrides: Record<string, unknown> = {}) =>
  makeEnvelope("welcome", {
    role: "viewer",
    desk_id: record.desk_id,
    desk_online: true,
    viewer_count: 1,
    snapshot: null,
    snapshot_ts: null,
    ...overrides,
  });

beforeEach(() => {
  setupMocks();
  localStorage.clear();
});
afterEach(teardownMocks);

describe("relayWsUrl", () => {
  it("upgrades the scheme and never puts a token in the URL", () => {
    const url = relayWsUrl("https://relay.desk.zentala.io/", "desk 1");
    expect(url).toBe("wss://relay.desk.zentala.io/v1/desks/desk%201/ws");
    expect(url).not.toContain(record.viewer_token);
  });

  it("keeps a plain-http relay on ws:// for local development", () => {
    expect(relayWsUrl("http://127.0.0.1:8787", "d")).toBe(
      "ws://127.0.0.1:8787/v1/desks/d/ws",
    );
  });
});

describe("RelayTransport", () => {
  it("sends hello with the viewer role and the stored token", () => {
    const { ws } = connected();
    ws.simulateOpen();

    const hello = ws.sentOfType("hello")[0];
    expect(hello.payload).toMatchObject({
      role: "viewer",
      desk_id: record.desk_id,
      token: record.viewer_token,
    });
    expect(hello.v).toBe(1);
  });

  it("reports the desk offline while staying connected", () => {
    const { statuses, messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(welcome({ desk_online: false }));

    expect(last(statuses)).toMatchObject({ connected: true, deskOnline: false });
    expect(last(messages)).toMatchObject({ kind: "welcome", deskOnline: false, snapshot: null });
  });

  it("passes the room's last snapshot through welcome", () => {
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(welcome({ snapshot: { session: {} }, snapshot_ts: 1 }));

    expect(last(messages)).toMatchObject({ kind: "welcome", snapshot: { session: {} } });
  });

  it("tracks desk_status while the socket stays up", () => {
    const { statuses, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(welcome());
    ws.simulateMessage(makeEnvelope("desk_status", { online: false, since: 5 }));

    expect(last(statuses)).toMatchObject({ connected: true, deskOnline: false });
  });

  it("unwraps an event envelope into the DisplayEvent the reducer expects", () => {
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(makeEnvelope("event", makeSnapshot()));

    expect(last(messages)).toMatchObject({ kind: "event", event: "snapshot" });
  });

  it("pings every 25 s and answers a server ping with a pong", () => {
    const { ws } = connected();
    ws.simulateOpen();

    vi.advanceTimersByTime(25_000);
    expect(ws.sentOfType("ping")).toHaveLength(1);

    ws.simulateMessage(makeEnvelope("ping", null, "server-ping"));
    const pong = ws.sentOfType("pong")[0];
    expect(pong.id).toBe("server-ping");
  });

  it("rejects a command with bad arguments before it reaches the wire", async () => {
    const { transport, ws } = connected();
    ws.simulateOpen();

    const outcome = await transport.sendCommand!("set_limits", {});
    expect(outcome).toMatchObject({ ok: false, error: { code: "bad_args" } });
    expect(ws.sentOfType("command")).toHaveLength(0);
  });

  it("refuses a command while disconnected instead of queueing it", async () => {
    const { transport } = connected();
    const outcome = await transport.sendCommand!("ack_alert", {});
    expect(outcome).toMatchObject({ ok: false, error: { code: "offline" } });
  });

  it("resolves a command when its result comes back", async () => {
    const { transport, ws } = connected();
    ws.simulateOpen();

    const pending = transport.sendCommand!("ack_alert", {});
    const sent = ws.sentOfType("command")[0];
    expect(sent.payload).toMatchObject({ name: "ack_alert", args: {} });

    ws.simulateMessage(
      makeEnvelope("command_result", {
        command_id: sent.id,
        ok: true,
        error: null,
      }),
    );
    await expect(pending).resolves.toEqual({ ok: true, error: null });
  });

  it("gives up on a command that is never answered", async () => {
    const { transport, ws } = connected();
    ws.simulateOpen();

    const pending = transport.sendCommand!("ack_alert", {});
    await vi.advanceTimersByTimeAsync(10_000);
    await expect(pending).resolves.toMatchObject({
      ok: false,
      error: { code: "timeout" },
    });
  });

  it("forgets the pairing and stops retrying when the viewer is revoked", () => {
    saveRelayRecord(record);
    const { statuses, ws } = connected();
    ws.simulateOpen();
    const before = MockWebSocket.instances.length;

    ws.simulateClose(CLOSE_CODES.REVOKED);
    vi.advanceTimersByTime(120_000);

    expect(localStorage.getItem(RELAY_STORAGE_KEY)).toBeNull();
    expect(last(statuses)?.state).toBe("unpaired");
    expect(MockWebSocket.instances.length).toBe(before);
  });

  it("stops retrying — but keeps the pairing — when the licence is unentitled", () => {
    saveRelayRecord(record);
    const { statuses, ws } = connected();
    ws.simulateOpen();
    const before = MockWebSocket.instances.length;

    ws.simulateClose(CLOSE_CODES.UNENTITLED);
    vi.advanceTimersByTime(120_000);

    expect(last(statuses)?.state).toBe("error");
    expect(localStorage.getItem(RELAY_STORAGE_KEY)).not.toBeNull();
    expect(MockWebSocket.instances.length).toBe(before);
  });

  it("reconnects after an ordinary drop, with growing backoff", () => {
    const { ws } = connected();
    ws.simulateOpen();
    const before = MockWebSocket.instances.length;

    // random() === 0.5 puts the jittered delay exactly on the nominal value.
    ws.simulateClose(1006);
    vi.advanceTimersByTime(999);
    expect(MockWebSocket.instances.length).toBe(before);
    vi.advanceTimersByTime(1);
    expect(MockWebSocket.instances.length).toBe(before + 1);

    MockWebSocket.latest().simulateClose(1006);
    vi.advanceTimersByTime(1999);
    expect(MockWebSocket.instances.length).toBe(before + 1);
    vi.advanceTimersByTime(1);
    expect(MockWebSocket.instances.length).toBe(before + 2);
  });

  it("keeps the jittered delay within ±20 % of the nominal backoff", () => {
    const seen: number[] = [];
    for (const r of [0, 1]) {
      MockWebSocket.reset();
      const transport = new RelayTransport(record, { random: () => r });
      transport.connect();
      const ws = MockWebSocket.latest();
      ws.simulateOpen();
      ws.simulateClose(1006);
      const before = MockWebSocket.instances.length;
      vi.advanceTimersByTime(799);
      const early = MockWebSocket.instances.length;
      vi.advanceTimersByTime(401);
      seen.push(early === before && MockWebSocket.instances.length === before + 1 ? 1 : 0);
      transport.close();
    }
    // Both extremes of the random source land inside 800..1200 ms.
    expect(seen).toEqual([1, 1]);
  });
});
