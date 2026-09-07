/**
 * helpers.ts — one place that knows how to talk to the relay in a test.
 *
 * Every helper here waits for a real frame or a real close; none of them sleeps
 * a fixed amount and hopes. A test that times out says so, and a test that
 * passes did so because a message arrived.
 */
import { SELF } from "cloudflare:test";
import { expect } from "vitest";

import { PROTOCOL_VERSION } from "@app/remote/protocol";

export interface Frame {
  v: number;
  type: string;
  id: string;
  ts: number;
  payload: unknown;
}

export interface CloseInfo {
  code: number;
  reason: string;
}

/** A connected socket plus the queues a test reads its results from. */
export interface TestSocket {
  ws: WebSocket;
  /** Resolves with the next frame, or rejects if the socket closes first. */
  next(): Promise<Frame>;
  /** Resolves when the socket closes. */
  closed(): Promise<CloseInfo>;
  send(type: string, payload: unknown, id?: string): string;
  raw(text: string): void;
}

const DEFAULT_TIMEOUT_MS = 5_000;

export async function connect(deskId = "desk-1"): Promise<TestSocket> {
  const response = await SELF.fetch(`https://relay.test/v1/desks/${deskId}/ws`, {
    headers: { Upgrade: "websocket" },
  });
  expect(response.status).toBe(101);
  const ws = response.webSocket;
  if (!ws) throw new Error("upgrade returned no webSocket");
  ws.accept();

  const frames: Frame[] = [];
  const waiters: Array<(f: Frame) => void> = [];
  let closeInfo: CloseInfo | null = null;
  const closeWaiters: Array<(c: CloseInfo) => void> = [];

  ws.addEventListener("message", (event: MessageEvent) => {
    const frame = JSON.parse(String(event.data)) as Frame;
    const waiter = waiters.shift();
    if (waiter) waiter(frame);
    else frames.push(frame);
  });
  ws.addEventListener("close", (event: CloseEvent) => {
    closeInfo = { code: event.code, reason: event.reason };
    for (const w of closeWaiters.splice(0)) w(closeInfo);
  });

  const withTimeout = <T>(p: Promise<T>, what: string): Promise<T> =>
    Promise.race([
      p,
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error(`timed out waiting for ${what}`)), DEFAULT_TIMEOUT_MS),
      ),
    ]);

  return {
    ws,
    next: () =>
      withTimeout(
        new Promise<Frame>((resolve, reject) => {
          const queued = frames.shift();
          if (queued) return resolve(queued);
          if (closeInfo) return reject(new Error(`socket closed (${closeInfo.code}) before a frame`));
          waiters.push(resolve);
        }),
        "a frame",
      ),
    closed: () =>
      withTimeout(
        new Promise<CloseInfo>((resolve) => {
          if (closeInfo) return resolve(closeInfo);
          closeWaiters.push(resolve);
        }),
        "the socket to close",
      ),
    send(type, payload, id) {
      const messageId = id ?? crypto.randomUUID();
      ws.send(JSON.stringify({ v: PROTOCOL_VERSION, type, id: messageId, ts: Date.now(), payload }));
      return messageId;
    },
    raw: (text) => ws.send(text),
  };
}

export const helloPayload = (role: "desk" | "viewer", deskId = "desk-1") => ({
  role,
  desk_id: deskId,
  token: role === "desk" ? `mu_d_${"a".repeat(43)}` : `mu_v_${"b".repeat(43)}`,
  client: { app: role === "desk" ? "moveup-desktop" : "moveup-viewer", version: "0.7.0" },
});

/** Connects, says hello, and returns the socket together with its `welcome`. */
export async function join(
  role: "desk" | "viewer",
  deskId = "desk-1",
): Promise<{ socket: TestSocket; welcome: Frame }> {
  const socket = await connect(deskId);
  socket.send("hello", helloPayload(role, deskId));
  const welcome = await socket.next();
  expect(welcome.type).toBe("welcome");
  return { socket, welcome };
}

/** A `snapshot` event shaped the way `ws_broadcaster.rs` serialises one. */
export const snapshotEvent = (limitUsedSecs: number) => ({
  event: "snapshot",
  payload: { session: { state: "Sitting", limit_used_secs: limitUsedSecs } },
});
