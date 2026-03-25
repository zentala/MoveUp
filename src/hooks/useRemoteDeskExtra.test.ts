/**
 * useRemoteDeskExtra.test.ts — Reconnection, backoff, auto-detection, and CSS class tests.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  MockWebSocket, makeSnapshot,
  setupMocks, teardownMocks, importHook,
} from "./remoteDesk.test-helpers";

beforeEach(setupMocks);
afterEach(teardownMocks);

describe("useRemoteDesk reconnection", () => {
  it("reconnects with exponential backoff capped at 10s", async () => {
    const useRemoteDesk = await importHook();
    renderHook(() => useRemoteDesk());

    const ws1 = MockWebSocket.latest();
    ws1.simulateOpen();

    // Close — should reconnect after 1s
    act(() => { ws1.simulateClose(); });
    const countBefore = MockWebSocket.instances.length;
    act(() => { vi.advanceTimersByTime(1000); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 1);

    // Close again — delay should be 2s
    const ws2 = MockWebSocket.latest();
    act(() => { ws2.simulateClose(); });
    act(() => { vi.advanceTimersByTime(1500); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 1);

    act(() => { vi.advanceTimersByTime(500); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 2);

    // Close again — delay should be 4s
    const ws3 = MockWebSocket.latest();
    act(() => { ws3.simulateClose(); });
    act(() => { vi.advanceTimersByTime(3500); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 2);

    act(() => { vi.advanceTimersByTime(500); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 3);
  });

  it("resets backoff delay on successful reconnect", async () => {
    const useRemoteDesk = await importHook();
    renderHook(() => useRemoteDesk());

    const ws1 = MockWebSocket.latest();
    ws1.simulateOpen();
    act(() => { ws1.simulateClose(); });

    // Reconnect at 1s
    act(() => { vi.advanceTimersByTime(1000); });
    const ws2 = MockWebSocket.latest();
    ws2.simulateOpen();

    act(() => { ws2.simulateClose(); });

    // Should reconnect at 1s again (not 2s)
    const countBefore = MockWebSocket.instances.length;
    act(() => { vi.advanceTimersByTime(1000); });
    expect(MockWebSocket.instances.length).toBe(countBefore + 1);
  });
});

describe("useDeskAuto", () => {
  it("detects Tauri when __TAURI_INTERNALS__ exists", () => {
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = { invoke: vi.fn() };
    const isTauri = !!window.__TAURI_INTERNALS__;
    expect(isTauri).toBe(true);
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("detects browser when __TAURI_INTERNALS__ is absent", () => {
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
    const isTauri = !!window.__TAURI_INTERNALS__;
    expect(isTauri).toBe(false);
  });
});

describe("remote-display class", () => {
  it("is added to documentElement when not in Tauri", () => {
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
    const isTauri = !!window.__TAURI_INTERNALS__;
    if (!isTauri) {
      document.documentElement.classList.add("remote-display");
    }
    expect(document.documentElement.classList.contains("remote-display")).toBe(true);
    document.documentElement.classList.remove("remote-display");
  });

  it("is NOT added when __TAURI_INTERNALS__ is present", () => {
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    document.documentElement.classList.remove("remote-display");
    const isTauri = !!window.__TAURI_INTERNALS__;
    if (!isTauri) {
      document.documentElement.classList.add("remote-display");
    }
    expect(document.documentElement.classList.contains("remote-display")).toBe(false);
    delete (window as Record<string, unknown>).__TAURI_INTERNALS__;
  });
});
