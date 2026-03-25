/**
 * remote-display.test.ts — integration tests for the remote display server.
 *
 * These tests require the app to be running (`pnpm tauri:dev`).
 * They verify the HTTP endpoints and WebSocket behavior of the
 * embedded remote display server on port 3390.
 */
import { describe, it, expect } from "vitest";

const BASE_URL = "http://localhost:3390";
const WS_URL = "ws://localhost:3390/display/ws";

describe("Remote Display Server", () => {
  it("GET /display/api returns valid session state", async () => {
    const res = await fetch(`${BASE_URL}/display/api`);
    expect(res.ok).toBe(true);
    const data = await res.json();
    expect(data).toHaveProperty("state");
    expect(data).toHaveProperty("sitting_seconds");
  });

  it("GET /display serves HTML", async () => {
    const res = await fetch(`${BASE_URL}/display`);
    expect(res.ok).toBe(true);
    const text = await res.text();
    expect(text).toContain("<!DOCTYPE html>");
  });

  it("WebSocket connects and receives full snapshot", async () => {
    const ws = new WebSocket(WS_URL);
    try {
      const firstMessage = await new Promise<Record<string, unknown>>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("WS timeout")), 5000);
        ws.onmessage = (e) => {
          clearTimeout(timeout);
          resolve(JSON.parse(e.data as string));
        };
        ws.onerror = () => {
          clearTimeout(timeout);
          reject(new Error("WS connection error"));
        };
      });
      expect(firstMessage.event).toBe("snapshot");
      const payload = firstMessage.payload as Record<string, unknown>;
      expect(payload).toHaveProperty("session");
      expect(payload).toHaveProperty("metrics");
      expect(payload).toHaveProperty("today");
      const session = payload.session as Record<string, unknown>;
      expect(session).toHaveProperty("state");
    } finally {
      ws.close();
    }
  });

  it("WebSocket receives heartbeat within 7 seconds", async () => {
    const ws = new WebSocket(WS_URL);
    try {
      const messages: Array<Record<string, unknown>> = [];
      ws.onmessage = (e) => messages.push(JSON.parse(e.data as string));
      await new Promise((r) => setTimeout(r, 7000));
      const hasHeartbeat = messages.some((m) => m.event === "heartbeat");
      expect(hasHeartbeat).toBe(true);
    } finally {
      ws.close();
    }
  }, 10000);
});
