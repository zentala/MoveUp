/**
 * useRemoteDesk.test.ts — Core tests for event handling and state mapping.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  MockWebSocket, makeSnapshot, makeStateChanged,
  setupMocks, teardownMocks, importHook,
} from "./remoteDesk.test-helpers";

beforeEach(setupMocks);
afterEach(teardownMocks);

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
    expect(result.current.sittingSeconds).toBe(120);
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
    expect(result.current.sittingSeconds).toBe(120);
    expect(result.current.dailyScore).toBe(5);

    act(() => { ws.simulateMessage({ event: "desk:daily-reset", payload: null }); });

    expect(result.current.sittingSeconds).toBe(0);
    expect(result.current.standingSeconds).toBe(0);
    expect(result.current.breakSeconds).toBe(0);
    expect(result.current.positionChanges).toBe(0);
    expect(result.current.dailyScore).toBe(0);
  });

  it("port is always null in remote mode", async () => {
    const useRemoteDesk = await importHook();
    const { result } = renderHook(() => useRemoteDesk());
    expect(result.current.port).toBeNull();
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
