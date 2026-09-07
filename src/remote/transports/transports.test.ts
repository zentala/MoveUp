/**
 * transports.test.ts — the seam itself (E022-T08).
 *
 * One fixture, two wires, one reducer: if the LAN and relay paths ever stop
 * producing identical state from identical data, the phone shows different
 * numbers depending on where its owner is standing. That is the regression
 * this file exists to catch, so it drives the real `useRemoteDesk` against
 * real transports — nothing between them is stubbed except the socket.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach } from "vitest";
import {
  MockWebSocket,
  makeEnvelope,
  setupMocks,
  teardownMocks,
  importHook,
} from "@/hooks/remoteDesk.test-helpers";
import { saveRelayRecord, type RelayRecord } from "../storage";
import { LanTransport } from "./lan";
import { RelayTransport } from "./relay";
import { selectTransport } from "./index";

const record: RelayRecord = {
  relay_url: "http://127.0.0.1:8787",
  desk_id: "desk-1",
  viewer_id: "viewer-1",
  viewer_token: "mu_v_secret",
  desk_name: "Office",
};

/**
 * The T01 contract fixtures — the same bytes the Rust side parses.
 *
 * Loaded through the bundler's glob rather than a hand-written literal, so a
 * fixture that is renamed or removed cannot leave this test quietly passing
 * against a stale copy. An empty glob is a failure, never a pass.
 */
const fixtures = import.meta.glob("../../../tests/fixtures/relay-protocol/*.json", {
  eager: true,
  import: "default",
}) as Record<string, unknown>;

const fixture = <T,>(name: string): T => {
  const key = Object.keys(fixtures).find((k) => k.endsWith(`/${name}.json`));
  if (!key) throw new Error(`fixture ${name}.json not found among ${Object.keys(fixtures).length}`);
  return fixtures[key] as T;
};

const eventSnapshot = fixture<{ payload: { event: string; payload: unknown } }>(
  "event-snapshot",
);

beforeEach(() => {
  setupMocks();
  localStorage.clear();
});
afterEach(teardownMocks);

/** Everything the UI reads, minus the parts that legitimately differ. */
function view(result: Record<string, unknown>) {
  const { calibrate, setSitLimit, setStandLimit, capabilities, deskOnline, wsConnected, ...rest } =
    result;
  void calibrate;
  void setSitLimit;
  void setStandLimit;
  void capabilities;
  void deskOnline;
  void wsConnected;
  return rest;
}

describe("transport parity", () => {
  it("both wires drive the reducer to the same state from the same fixture", async () => {
    const useRemoteDesk = await importHook();

    const lan = renderHook(() =>
      useRemoteDesk({ transport: new LanTransport({ wsUrl: "ws://desk.test/display/ws" }) }),
    );
    const lanWs = MockWebSocket.latest();
    act(() => {
      lanWs.simulateOpen();
      // Pre-T11 desks send the DisplayEvent bare.
      lanWs.simulateMessage(eventSnapshot.payload);
    });

    const relay = renderHook(() => useRemoteDesk({ transport: new RelayTransport(record) }));
    const relayWs = MockWebSocket.latest();
    act(() => {
      relayWs.simulateOpen();
      relayWs.simulateMessage(eventSnapshot);
    });

    expect(view(relay.result.current as unknown as Record<string, unknown>)).toEqual(
      view(lan.result.current as unknown as Record<string, unknown>),
    );
    // …and it is the fixture's state, not two identical empties.
    expect(lan.result.current.limitUsedSecs).toBe(900);
    expect(relay.result.current.limitUsedSecs).toBe(900);
  });

  it("a null welcome snapshot leaves the reducer at its initial state", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() =>
      useRemoteDesk({ transport: new RelayTransport(record) }),
    );
    const ws = MockWebSocket.latest();
    act(() => {
      ws.simulateOpen();
      ws.simulateMessage(
        makeEnvelope("welcome", {
          role: "viewer",
          desk_id: record.desk_id,
          desk_online: false,
          viewer_count: 1,
          snapshot: null,
          snapshot_ts: null,
        }),
      );
    });

    expect(result.current.limitUsedSecs).toBe(0);
    // The distinction the overlay needs: connected, but the desk is not there.
    expect(result.current.wsConnected).toBe(true);
    expect(result.current.deskOnline).toBe(false);
  });

  it("exposes the transport's capabilities to the UI", async () => {
    const useRemoteDesk = await importHook();
    const lan = renderHook(() => useRemoteDesk({ transport: new LanTransport() }));
    expect(lan.result.current.capabilities.control).toBe(false);

    const relay = renderHook(() => useRemoteDesk({ transport: new RelayTransport(record) }));
    expect(relay.result.current.capabilities.control).toBe(true);
  });
});

describe("selectTransport", () => {
  it("picks the LAN wire when this phone has never been paired", () => {
    expect(selectTransport()).toBeInstanceOf(LanTransport);
  });

  it("picks the relay wire once a pairing is stored", () => {
    saveRelayRecord(record);
    expect(selectTransport()).toBeInstanceOf(RelayTransport);
  });
});
