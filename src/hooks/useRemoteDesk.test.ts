/**
 * useRemoteDesk.test.ts — Core tests for event handling and state mapping.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  MockWebSocket, makeSnapshot, makeStateChanged,
  setupMocks, teardownMocks, importHook, importRemoteDeskModule,
} from "./remoteDesk.test-helpers";

beforeEach(setupMocks);
afterEach(teardownMocks);

/** Snapshot frame carrying the health slice the display server sends (E021-T03). */
function makeSnapshotWithHealth() {
  const frame = makeSnapshot();
  (frame.payload as Record<string, unknown>).health = {
    configured: true,
    snapshot: {
      steps_today: 1234,
      source_id: "google_fit",
      fetched_at_ms: 1_715_000_000_000,
    },
  };
  return frame;
}

describe("useRemoteDesk", () => {
  it("parses snapshot event and updates all state fields", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => {
      ws.simulateMessage(makeSnapshot());
    });

    expect(result.current.state).toBe("Sitting");
    expect(result.current.deskHeightCm).toBe(72.5);
    expect(result.current.secsSinceLastBreak).toBe(120);
    expect(result.current.standingSeconds).toBe(60);
    expect(result.current.sessionLimitSecs).toBe(2700);
    expect(result.current.positionChanges).toBe(2);
    expect(result.current.limitUsedSecs).toBe(120);
    expect(result.current.dailyScore).toBe(5);
    expect(result.current.connected).toBe(true);
  });

  it("parses desk:state-changed event correctly", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => {
      ws.simulateMessage(makeStateChanged());
    });

    expect(result.current.state).toBe("Standing");
    expect(result.current.deskHeightCm).toBe(110.0);
    expect(result.current.breakSeconds).toBe(10);
    expect(result.current.positionChanges).toBe(3);
    expect(result.current.transition).not.toBeNull();
  });

  it("state-changed carries the credited counter, it never resets to 0", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => { ws.simulateMessage(makeSnapshot()); });
    expect(result.current.limitUsedSecs).toBe(120);

    act(() => {
      ws.simulateMessage(
        makeStateChanged({ state: "Sitting", limit_used_secs: 1500 }),
      );
    });

    expect(result.current.limitUsedSecs).toBe(1500);
  });

  it("handles desk:device-connected and desk:device-lost", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => {
      ws.simulateMessage({ event: "desk:device-connected", payload: { port: "COM3" } });
    });
    expect(result.current.connected).toBe(true);

    act(() => {
      ws.simulateMessage({ event: "desk:device-lost", payload: null });
    });
    expect(result.current.connected).toBe(false);
  });

  it("ignores heartbeat events without error", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => {
      ws.simulateMessage({ event: "heartbeat", payload: null });
    });

    expect(result.current.state).toBe("Away");
  });

  it("returns no-op functions for calibrate/setSitLimit/setStandLimit", async () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    await result.current.calibrate("sitting");
    await result.current.setSitLimit(30);
    await result.current.setStandLimit(15);

    expect(warnSpy).toHaveBeenCalledTimes(3);
    warnSpy.mockRestore();
  });

  it("catches malformed JSON messages without throwing", async () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => {
      ws.simulateRawMessage("this is not valid json{{{");
    });

    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining("malformed"),
    );
    expect(result.current.state).toBe("Away");
    warnSpy.mockRestore();
  });

  it("handles desk:daily-reset event — resets daily counters", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    act(() => { ws.simulateMessage(makeSnapshot()); });
    expect(result.current.secsSinceLastBreak).toBe(120);
    expect(result.current.dailyScore).toBe(5);

    act(() => { ws.simulateMessage({ event: "desk:daily-reset", payload: null }); });

    expect(result.current.secsSinceLastBreak).toBe(0);
    expect(result.current.standingSeconds).toBe(0);
    expect(result.current.breakSeconds).toBe(0);
    expect(result.current.positionChanges).toBe(0);
    expect(result.current.dailyScore).toBe(0);
  });

  it("routes a desk:voice-ack frame to the onVoiceAck option and subscribers", async () => {
    const { useRemoteDesk, subscribeVoiceAck } = await importRemoteDeskModule();
    const viaOption = vi.fn();
    const viaSubscription = vi.fn();
    const off = subscribeVoiceAck(viaSubscription);

    renderHook(() => useRemoteDesk({ onVoiceAck: viaOption }));
    const ws = MockWebSocket.latest();
    act(() => { ws.simulateOpen(); });

    const payload = { transcript: "drzemka 5", intent: "Snooze(5)", reply: "Ok." };
    act(() => { ws.simulateMessage({ event: "desk:voice-ack", payload }); });

    expect(viaOption).toHaveBeenCalledWith(payload);
    expect(viaSubscription).toHaveBeenCalledWith(payload);
    off();
  });

  it("stops delivering acks to an unmounted consumer", async () => {
    const { useRemoteDesk } = await importRemoteDeskModule();
    const viaOption = vi.fn();
    const { unmount } = renderHook(() => useRemoteDesk({ onVoiceAck: viaOption }));
    const ws = MockWebSocket.latest();
    act(() => { ws.simulateOpen(); });
    unmount();

    act(() => {
      ws.simulateMessage({
        event: "desk:voice-ack",
        payload: { transcript: "x", intent: "Note", reply: null },
      });
    });

    expect(viaOption).not.toHaveBeenCalled();
  });

  it("publishes the snapshot's health view to health subscribers", async () => {
    const useRemoteDesk = await importHook();
    const { subscribeRemoteHealth, resetRemoteHealth } = await import("./useHealth");
    resetRemoteHealth();
    const seen = vi.fn();
    const off = subscribeRemoteHealth(seen);

    renderHook(() => useRemoteDesk());
    const ws = MockWebSocket.latest();
    ws.simulateOpen();
    act(() => { ws.simulateMessage(makeSnapshotWithHealth()); });

    expect(seen).toHaveBeenCalledWith(
      expect.objectContaining({
        configured: true,
        snapshot: expect.objectContaining({
          steps_today: 1234,
          source_id: "google_fit",
        }),
      }),
    );
    off();
  });

  it("a snapshot without health leaves the last health view alone", async () => {
    const useRemoteDesk = await importHook();
    const { subscribeRemoteHealth, resetRemoteHealth } = await import("./useHealth");
    resetRemoteHealth();
    const seen = vi.fn();
    const off = subscribeRemoteHealth(seen);

    renderHook(() => useRemoteDesk());
    const ws = MockWebSocket.latest();
    ws.simulateOpen();

    // A backend older than E021-T03 sends no `health` key at all.
    act(() => { ws.simulateMessage(makeSnapshot()); });

    expect(seen).not.toHaveBeenCalled();
    off();
  });

  it("port is always null in remote mode", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());
    expect(result.current.port).toBeNull();
  });

  it("defaults to the read-only LAN transport when nothing is paired", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    expect(result.current.capabilities.control).toBe(false);
    expect(result.current.deskOnline).toBe(false);

    const ws = MockWebSocket.latest();
    act(() => { ws.simulateOpen(); });
    // On the LAN the server is the desk: one connection, one liveness fact.
    expect(result.current.deskOnline).toBe(true);
  });

  it("wsConnected tracks WebSocket connection state", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());

    expect(result.current.wsConnected).toBe(false);

    const ws = MockWebSocket.latest();
    act(() => { ws.simulateOpen(); });
    expect(result.current.wsConnected).toBe(true);

    act(() => { ws.simulateClose(); });
    expect(result.current.wsConnected).toBe(false);
  });
});
