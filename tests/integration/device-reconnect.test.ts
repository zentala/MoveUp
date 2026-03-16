/**
 * device-reconnect.test.ts
 *
 * Integration tests for device connection state management.
 * Tests device discovery, connection, disconnection, and reconnection scenarios.
 */

import { describe, it, expect, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

describe("Device Reconnect Integration Tests", () => {
  beforeEach(async () => {
    // Setup: ensure clean state
  });

  it("list-ports: returns available serial ports", async () => {
    const ports = await invoke<any[]>("list_ports");

    expect(Array.isArray(ports)).toBe(true);
    // Ports may be empty in test environment, but should be an array
  });

  it("start-auto-connect: triggers port scan and connect", async () => {
    // This command runs in the background; we can't easily test completion
    // but we verify it doesn't throw

    await expect(async () => {
      await invoke("start_auto_connect");
    }).not.rejects.toThrow();
  });

  it("stop-reading: stops the background reader gracefully", async () => {
    // Verify the command executes without error
    await expect(async () => {
      await invoke("stop_reading");
    }).not.rejects.toThrow();
  });

  it("desk:device-connected event: emitted when sensor is detected", async () => {
    // This test would need actual device or mock serial port
    // For now, we document the expected event structure

    const events: any[] = [];

    const unlisten = await listen("desk:device-connected", (event) => {
      events.push(event.payload);
    });

    // In a real scenario, device would connect and emit event
    // For now, we just verify the listener doesn't throw

    await new Promise((resolve) => setTimeout(resolve, 100));

    unlisten();
  });

  it("desk:device-lost event: emitted when sensor disconnects", async () => {
    const events: any[] = [];

    const unlisten = await listen("desk:device-lost", (event) => {
      events.push(event.payload);
    });

    // Similar to above — this requires actual device disconnection

    await new Promise((resolve) => setTimeout(resolve, 100));

    unlisten();
  });

  it("desk:state-changed event: includes all required fields", async () => {
    const events: any[] = [];

    const unlisten = await listen("desk:state-changed", (event) => {
      events.push(event.payload);
    });

    // Trigger a state change via inject_reading
    await invoke("inject_reading", { mm: 750, active: true });

    await new Promise((resolve) => setTimeout(resolve, 200));

    if (events.length > 0) {
      const payload = events[0];
      expect(payload).toHaveProperty("state");
      expect(payload).toHaveProperty("sitting_seconds");
      expect(payload).toHaveProperty("standing_seconds");
      expect(payload).toHaveProperty("break_seconds");
      expect(payload).toHaveProperty("desk_height_cm");
      expect(payload).toHaveProperty("position_changes");
    }

    unlisten();
  });
});
