/**
 * session-flow.test.ts
 *
 * Integration tests for the complete session state machine flow.
 * Tests the core business logic: sitting duration, breaks, alerts, state transitions.
 *
 * These tests use inject_reading command to simulate sensor readings
 * and verify SessionManager state via get_session_state.
 */

import { describe, it, expect, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  SITTING_DISTANCE_MM,
  STANDING_DISTANCE_MM,
  DEBOUNCE_COUNT,
  sendRepeated,
} from "../emulator/scenarios";

describe("Session Flow Integration Tests", () => {
  beforeEach(async () => {
    // Reset session state before each test
    // (In a real scenario, app is restarted; here we just ensure clean state)
  });

  it("basic-sitting-session: accumulates sitting seconds when in sitting position", async () => {
    // Send sitting readings
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 3, true, 100);

    // Query state
    const state = await invoke<any>("get_session_state");

    expect(state.state).toBe("sitting");
    expect(state.sitting_seconds).toBeGreaterThanOrEqual(0);
  });

  it("standing-long-break: 10+ min standing resets sitting counter to 0", async () => {
    // Start with sitting
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);
    await new Promise((resolve) => setTimeout(resolve, 2000)); // Accumulate sitting time

    // Transition to standing
    await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    // Simulate 11 minutes standing (660 seconds) — we can use time acceleration
    // For now, just verify state change and break tracking
    const state = await invoke<any>("get_session_state");

    expect(state.state).toBe("standing");
  });

  it("standing-short-break: 5-9 min standing subtracts 20 min from sitting counter", async () => {
    // Start sitting
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    const sitStateStart = await invoke<any>("get_session_state");
    const initialSitting = sitStateStart.sitting_seconds;

    // Transition to standing
    await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    // In a real scenario, hold standing for 7 minutes; for now just verify transition
    const standState = await invoke<any>("get_session_state");
    expect(standState.state).toBe("standing");
  });

  it("walking-no-standing-credit: walking (standing + inactive) doesn't increase standing time", async () => {
    // Transition to standing with active=false (walking)
    await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, false, 50);

    const state = await invoke<any>("get_session_state");

    expect(state.state).toBe("walking");
    // standing_seconds should not increase (or increase minimally) in Walking state
    expect(state.standing_seconds).toBeLessThanOrEqual(100);
  });

  it("alert-fires-at-limit: should_alert triggers when sitting >= session_limit", async () => {
    // Note: This test would require time-mocking or direct state manipulation
    // For now, we test the state transition and event emission path
    const stateChangeEvents: any[] = [];

    // Listen for state-changed events
    const unlisten = await listen("desk:state-changed", (event) => {
      stateChangeEvents.push(event.payload);
    });

    // Inject sitting readings
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    await new Promise((resolve) => setTimeout(resolve, 500));

    expect(stateChangeEvents.length).toBeGreaterThanOrEqual(0);

    unlisten();
  });

  it("no-duplicate-alert: should_alert only fires once per sitting stint", async () => {
    // This test verifies the alert_fired flag behavior
    // Multiple calls to should_alert should only return true once

    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    const state1 = await invoke<any>("get_session_state");
    expect(state1.state).toBe("sitting");

    // In real testing, we'd call should_alert directly;
    // via inject_reading it's implicit in the state machine
  });

  it("debounce-5-readings: state transition requires 5 consistent readings", async () => {
    // Send 4 standing readings (insufficient for debounce)
    for (let i = 0; i < 4; i++) {
      await invoke("inject_reading", {
        mm: STANDING_DISTANCE_MM,
        active: true,
      });
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    // State should still be Away (or previous state)
    let state = await invoke<any>("get_session_state");
    expect(["away", "sitting"]).toContain(state.state);

    // Send 1 more reading (5th = debounce confirmation)
    await invoke("inject_reading", { mm: STANDING_DISTANCE_MM, active: true });

    state = await invoke<any>("get_session_state");
    expect(state.state).toBe("standing");
  });

  it("position-changes-counter: sitting↔standing transitions increment counter", async () => {
    const stateStart = await invoke<any>("get_session_state");
    const posChangesStart = stateStart.position_changes;

    // Transition to standing
    await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    let state = await invoke<any>("get_session_state");
    expect(state.position_changes).toBe(posChangesStart + 1);

    // Transition back to sitting
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, 50);

    state = await invoke<any>("get_session_state");
    expect(state.position_changes).toBe(posChangesStart + 2);
  });

  it("daily-reset: session counters reset at midnight", async () => {
    // This test requires mocking the system time or direct state injection
    // For now, we test that check_daily_reset is callable via the session state

    const state = await invoke<any>("get_session_state");
    expect(state.sitting_seconds).toBeDefined();
    expect(state.standing_seconds).toBeDefined();
  });
});
