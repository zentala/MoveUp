/**
 * remoteDesk.test-helpers.ts — Shared test utilities for useRemoteDesk tests.
 */
import { vi } from "vitest";

/** Minimal mock WebSocket for testing. */
export class MockWebSocket {
  static instances: MockWebSocket[] = [];
  url: string;
  onopen: (() => void) | null = null;
  onmessage: ((e: { data: string }) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  readyState = 0;
  closed = false;

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  send() { /* no-op for tests */ }

  close() {
    if (!this.closed) {
      this.closed = true;
      this.readyState = 3;
      this.onclose?.();
    }
  }

  simulateOpen() {
    this.readyState = 1;
    this.onopen?.();
  }

  simulateMessage(data: unknown) {
    this.onmessage?.({ data: JSON.stringify(data) });
  }

  simulateRawMessage(data: string) {
    this.onmessage?.({ data });
  }

  simulateClose() {
    this.close();
  }

  static reset() {
    MockWebSocket.instances = [];
  }

  static latest(): MockWebSocket {
    return MockWebSocket.instances[MockWebSocket.instances.length - 1];
  }
}

/** Factory for a valid snapshot payload. */
export function makeSnapshot(overrides: Record<string, unknown> = {}) {
  return {
    event: "snapshot",
    payload: {
      session: {
        state: "Sitting",
        sitting_seconds: 120,
        standing_seconds: 60,
        break_seconds: 0,
        session_limit_secs: 2700,
        stand_limit_secs: 900,
        desk_height_cm: 72.5,
        position_changes: 2,
        limit_used_secs: 120,
        daily_score: 5,
        standing_session_secs: 0,
        secs_since_last_break: 120,
        continuous_computer_secs: 120,
        longest_computer_session_secs: 120,
        sitting_seconds_total: 120,
        idle_secs: 0,
        away_bout_secs: 0,
        max_continuous_computer_secs: 7200,
        ...overrides,
      },
      metrics: [],
      today: { sitting_secs: 300, standing_secs: 100, sessions: [] },
    },
  };
}

/** Factory for a state-changed event payload. */
export function makeStateChanged(overrides: Record<string, unknown> = {}) {
  return {
    event: "desk:state-changed",
    payload: {
      state: "Standing",
      standing_seconds: 60,
      break_seconds: 10,
      desk_height_cm: 110.0,
      position_changes: 3,
      last_break_secs: 0,
      last_sitting_secs: 120,
      break_credit: "none",
      limit_used_secs: 0,
      ...overrides,
    },
  };
}

/** Save original WebSocket. */
export const OriginalWebSocket = globalThis.WebSocket;

/** Standard setup for remote desk tests. */
export function setupMocks() {
  vi.useFakeTimers();
  MockWebSocket.reset();
  (globalThis as unknown as Record<string, unknown>).WebSocket = MockWebSocket as unknown as typeof WebSocket;
  globalThis.fetch = vi.fn().mockRejectedValue(new Error("no fetch")) as unknown as typeof fetch;
}

/** Standard teardown for remote desk tests. */
export function teardownMocks() {
  vi.useRealTimers();
  (globalThis as unknown as Record<string, unknown>).WebSocket = OriginalWebSocket;
}

/** Dynamic import to pick up the mock WebSocket. */
export async function importHook() {
  const mod = await import("./useRemoteDesk");
  return mod.useRemoteDesk;
}
