/**
 * lan.test.ts — the read-only LAN wire (E022-T08).
 *
 * The interesting property here is that it accepts both frame shapes: a phone
 * and a desk update on their own schedules, so a pre-T11 desk (bare
 * `{event, payload}`) and a post-T11 one (v1 envelope) both have to work.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  MockWebSocket,
  makeEnvelope,
  makeSnapshot,
  setupMocks,
  teardownMocks,
} from "@/hooks/remoteDesk.test-helpers";
import { LanTransport } from "./lan";
import type { Transport, TransportMessage, TransportStatus } from "./types";

/** Last element, spelled out because the tsconfig targets ES2020 (no `Array.at`). */
const last = <T,>(items: T[]): T | undefined => items[items.length - 1];

beforeEach(setupMocks);
afterEach(teardownMocks);

function connected() {
  const transport = new LanTransport({ wsUrl: "ws://desk.test/display/ws" });
  const messages: TransportMessage[] = [];
  const statuses: TransportStatus[] = [];
  transport.onMessage((m) => messages.push(m));
  transport.onStatus((s) => statuses.push(s));
  transport.connect();
  return { transport, messages, statuses, ws: MockWebSocket.latest() };
}

describe("LanTransport", () => {
  it("is read-only and offers no command channel", () => {
    const transport: Transport = new LanTransport();
    expect(transport.capabilities.control).toBe(false);
    expect(transport.sendCommand).toBeUndefined();
  });

  it("delivers a legacy {event, payload} frame", () => {
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(makeSnapshot());

    expect(messages).toHaveLength(1);
    expect(messages[0]).toMatchObject({ kind: "event", event: "snapshot" });
  });

  it("delivers the same event wrapped in a v1 envelope", () => {
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(makeEnvelope("event", makeSnapshot()));

    expect(messages).toHaveLength(1);
    expect(messages[0]).toMatchObject({ kind: "event", event: "snapshot" });
  });

  it("ignores an envelope type this build does not know", () => {
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage(makeEnvelope("some_future_type", { a: 1 }));

    expect(messages).toHaveLength(0);
  });

  it("warns and ignores a frame that is not an envelope at all", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const { messages, ws } = connected();
    ws.simulateOpen();
    ws.simulateMessage({ v: 1, type: "event" });

    expect(messages).toHaveLength(0);
    expect(warn).toHaveBeenCalled();
    warn.mockRestore();
  });

  it("warns and ignores malformed JSON", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const { ws } = connected();
    ws.simulateOpen();
    ws.simulateRawMessage("not json{{");

    expect(warn).toHaveBeenCalledWith(expect.stringContaining("malformed"));
    warn.mockRestore();
  });

  it("reports desk liveness as the socket itself — one fact, not two", () => {
    const { statuses, ws } = connected();
    ws.simulateOpen();
    expect(last(statuses)).toMatchObject({ connected: true, deskOnline: true });

    ws.simulateClose();
    expect(last(statuses)).toMatchObject({ connected: false, deskOnline: false });
  });

  it("reconnects with backoff and stops once closed", () => {
    const { transport, ws } = connected();
    ws.simulateOpen();
    const before = MockWebSocket.instances.length;

    ws.simulateClose();
    vi.advanceTimersByTime(1000);
    expect(MockWebSocket.instances.length).toBe(before + 1);

    transport.close();
    MockWebSocket.latest().simulateClose();
    vi.advanceTimersByTime(60_000);
    expect(MockWebSocket.instances.length).toBe(before + 1);
  });

  it("polls the REST snapshot while the socket is down", async () => {
    const snapshot = { session: {}, metrics: [], today: {} };
    globalThis.fetch = vi.fn().mockResolvedValue({
      json: () => Promise.resolve(snapshot),
    }) as unknown as typeof fetch;

    const { messages } = connected();
    await vi.advanceTimersByTimeAsync(2000);

    expect(messages).toContainEqual({
      kind: "event",
      event: "snapshot",
      payload: snapshot,
    });
  });

  it("does not poll while the socket is up", async () => {
    const fetchMock = vi.fn().mockResolvedValue({ json: () => Promise.resolve({}) });
    globalThis.fetch = fetchMock as unknown as typeof fetch;

    const { ws } = connected();
    ws.simulateOpen();
    await vi.advanceTimersByTimeAsync(6000);

    expect(fetchMock).not.toHaveBeenCalled();
  });
});
