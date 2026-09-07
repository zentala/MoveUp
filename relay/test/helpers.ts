/**
 * helpers.ts — one place that knows how to talk to the relay in a test.
 *
 * Every socket helper waits for a real frame or a real close; none of them
 * sleeps a fixed amount and hopes. A test that times out says so, and a test
 * that passes did so because a message arrived.
 *
 * Credentials are **real**. A desk is registered through `POST /v1/desks/register`
 * and a viewer is paired through `POST /v1/pair`, so the room tests exercise the
 * same `verifyToken` path production does. Only the licence row is seeded
 * directly — licences are minted out of band by `scripts/mint-license.mjs`, not
 * by any route.
 */
import { SELF, env as testBindings } from "cloudflare:test";
import { expect } from "vitest";

import { PROTOCOL_VERSION } from "@app/remote/protocol";

import { sha256Hex } from "../src/auth/licenses";
import type { TestEnv } from "./env";

export const env = testBindings as TestEnv;

/** The database, or a failure that names what is missing. */
export function testDb(): D1Database {
  if (!env.DB) throw new Error("test: no DB binding — check vitest.config.ts");
  return env.DB;
}

export const url = (path: string) => `https://relay.test${path}`;

export interface ApiOptions {
  method?: string;
  token?: string;
  body?: unknown;
  /** Rate-limit bucket. Unique per call by default, so tests do not throttle
   * each other; a throttling test pins it deliberately. */
  ip?: string;
}

export async function api(path: string, options: ApiOptions = {}): Promise<Response> {
  const headers: Record<string, string> = {
    "CF-Connecting-IP": options.ip ?? crypto.randomUUID(),
  };
  if (options.token) headers.Authorization = `Bearer ${options.token}`;
  if (options.body !== undefined) headers["content-type"] = "application/json";

  return SELF.fetch(url(path), {
    method: options.method ?? (options.body === undefined ? "GET" : "POST"),
    headers,
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  });
}

// ─── Credentials ────────────────────────────────────────────────────────────

export interface LicenseOptions {
  plan?: string;
  maxDesks?: number;
  maxViewers?: number;
  /** Unix ms, or `null` for a licence that never expires. */
  expiresAt?: number | null;
}

/** Inserts one licence and returns its plaintext key. */
export async function seedLicense(options: LicenseOptions = {}): Promise<string> {
  const key = `mu_lic_${crypto.randomUUID()}`;
  await testDb()
    .prepare(
      "INSERT INTO licenses (key_hash, plan, max_desks, max_viewers, expires_at, created_at)" +
        " VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(
      await sha256Hex(key),
      options.plan ?? "founder",
      options.maxDesks ?? 1,
      options.maxViewers ?? 10,
      options.expiresAt === undefined ? null : options.expiresAt,
      Date.now(),
    )
    .run();
  return key;
}

export interface Desk {
  deskId: string;
  token: string;
  licenseKey: string;
  plan: string;
}

/** Registers a desk against an existing licence. */
export async function registerDesk(licenseKey: string, deskName = "Test desk"): Promise<Desk> {
  const response = await api("/v1/desks/register", {
    body: { license_key: licenseKey, desk_name: deskName, app_version: "0.7.0" },
  });
  expect(response.status).toBe(201);
  const body = (await response.json()) as { desk_id: string; desk_token: string; plan: string };
  return { deskId: body.desk_id, token: body.desk_token, licenseKey, plan: body.plan };
}

/** A licence plus one desk registered against it — the usual starting point. */
export async function newDesk(options: LicenseOptions = {}): Promise<Desk> {
  return registerDesk(await seedLicense(options));
}

export interface Viewer {
  viewerId: string;
  token: string;
}

/** Asks the desk for a code and redeems it, the way a phone does. */
export async function pairViewer(desk: Desk, deviceName = "Test phone"): Promise<Viewer> {
  const issued = await api(`/v1/desks/${desk.deskId}/pairings`, {
    method: "POST",
    token: desk.token,
    body: {},
  });
  expect(issued.status).toBe(201);
  const { code } = (await issued.json()) as { code: string };

  const paired = await api("/v1/pair", {
    body: { desk_id: desk.deskId, code, device_name: deviceName },
  });
  expect(paired.status).toBe(201);
  const body = (await paired.json()) as { viewer_id: string; viewer_token: string };
  return { viewerId: body.viewer_id, token: body.viewer_token };
}

// ─── Sockets ────────────────────────────────────────────────────────────────

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

export async function connect(deskId: string): Promise<TestSocket> {
  const response = await SELF.fetch(url(`/v1/desks/${deskId}/ws`), {
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

export const helloPayload = (role: "desk" | "viewer", deskId: string, token: string) => ({
  role,
  desk_id: deskId,
  token,
  client: { app: role === "desk" ? "moveup-desktop" : "moveup-viewer", version: "0.7.0" },
});

export interface Joined {
  socket: TestSocket;
  welcome: Frame;
}

/** Connects, says hello, and returns the socket together with its `welcome`. */
export async function join(
  role: "desk" | "viewer",
  deskId: string,
  token: string,
): Promise<Joined> {
  const socket = await connect(deskId);
  socket.send("hello", helloPayload(role, deskId, token));
  const welcome = await socket.next();
  expect(welcome.type).toBe("welcome");
  return { socket, welcome };
}

export const joinDesk = (desk: Desk): Promise<Joined> => join("desk", desk.deskId, desk.token);

export const joinViewer = (desk: Desk, viewer: Viewer): Promise<Joined> =>
  join("viewer", desk.deskId, viewer.token);

/** A desk with one paired phone, both connected — the common fixture. */
export async function pairedRoom(options: LicenseOptions = {}): Promise<{
  desk: Desk;
  viewer: Viewer;
}> {
  const desk = await newDesk(options);
  const viewer = await pairViewer(desk);
  return { desk, viewer };
}

/** A `snapshot` event shaped the way `ws_broadcaster.rs` serialises one. */
export const snapshotEvent = (limitUsedSecs: number) => ({
  event: "snapshot",
  payload: { session: { state: "Sitting", limit_used_secs: limitUsedSecs } },
});
