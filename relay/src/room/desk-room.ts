/**
 * desk-room.ts — one Durable Object per desk (E022-T02).
 *
 * Holds at most one desk socket plus the room's viewers, fans desk `event`
 * frames out to those viewers, and keeps the newest snapshot in memory so a
 * viewer that arrives while the PC is off sees stale data marked
 * `desk_online: false` instead of a spinner. The viewer cap is a licence
 * question and is enforced at pairing time by T03, not here.
 *
 * The snapshot is deliberately **memory only**: it dies with the object, and
 * nothing about a person's day is ever written to Cloudflare storage
 * (PLAN.md §Architecture, `docs/PRIVACY.md`). A viewer that reconnects after an
 * eviction gets `snapshot: null` until the desk sends the next one — which the
 * desk does on every `welcome`.
 *
 * Command routing (`command`, `command_result`) is T04. Until then both are
 * answered with a non-fatal `error{not_implemented}` rather than dropped, so a
 * viewer built ahead of the relay learns the truth instead of waiting forever.
 */
import type { Role } from "@app/generated/Role";
import type { Welcome } from "@app/generated/Welcome";

import { verifyToken, type AuthFailure } from "../auth/tokens";
import { numVar, type Env } from "../env";
import {
  CLOSE_CODES,
  IDLE_CLOSE_CODE,
  encode,
  errorFrame,
  isFailure,
  readFrame,
  type KnownMessage,
} from "./messages";
import { isAuthed, readState, socketsInRole, writeState, type AuthedState } from "./sockets";

/** How often the alarm sweeps for late `hello`s and silent sockets. */
const MIN_SWEEP_MS = 50;

const CLOSE_FOR: Record<AuthFailure, number> = {
  unauthorized: CLOSE_CODES.UNAUTHENTICATED,
  unentitled: CLOSE_CODES.UNENTITLED,
  revoked: CLOSE_CODES.REVOKED,
};

export class DeskRoom implements DurableObject {
  /** Newest `snapshot` event seen from the desk, verbatim. Memory only. */
  private snapshot: unknown = null;
  private snapshotTs: number | null = null;

  private readonly helloTimeoutMs: number;
  private readonly idleTimeoutMs: number;

  constructor(
    private readonly ctx: DurableObjectState,
    private readonly env: Env,
  ) {
    this.helloTimeoutMs = numVar(env.HELLO_TIMEOUT_MS, 5_000);
    this.idleTimeoutMs = numVar(env.IDLE_TIMEOUT_MS, 60_000);
  }

  // ─── Upgrade ──────────────────────────────────────────────────────────────

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get("Upgrade")?.toLowerCase() !== "websocket") {
      return new Response("expected a websocket upgrade", { status: 426 });
    }

    const pair = new WebSocketPair();
    const [client, server] = [pair[0], pair[1]];

    // Accepted before `hello` arrives, so the role is not yet known and cannot
    // become an accept-time tag — see `sockets.ts` for why that is fine.
    this.ctx.acceptWebSocket(server);
    writeState(server, { state: "pending", helloBy: Date.now() + this.helloTimeoutMs });
    await this.scheduleSweep(this.helloTimeoutMs);

    return new Response(null, { status: 101, webSocket: client });
  }

  // ─── Frames ───────────────────────────────────────────────────────────────

  async webSocketMessage(ws: WebSocket, raw: ArrayBuffer | string): Promise<void> {
    if (typeof raw !== "string") {
      ws.close(CLOSE_CODES.PROTOCOL_ERROR, "binary frames are not part of v1");
      return;
    }

    const parsed = readFrame(raw);
    if (isFailure(parsed)) {
      if (parsed.action === "close") ws.close(parsed.code, parsed.reason);
      else ws.send(parsed.frame);
      return;
    }

    const state = readState(ws);
    if (!isAuthed(state)) {
      await this.handleHello(ws, parsed);
      return;
    }

    writeState(ws, { ...state, lastSeen: Date.now() });
    await this.handleAuthed(ws, state, parsed);
  }

  private async handleHello(ws: WebSocket, message: KnownMessage): Promise<void> {
    if (message.type !== "hello") {
      // Anything before `hello` is unauthenticated by definition, whatever it
      // claims to be.
      ws.close(CLOSE_CODES.UNAUTHENTICATED, "hello must come first");
      return;
    }

    const { role, desk_id: deskId, token } = message.payload;
    const auth = await verifyToken(this.env, deskId, role, token);
    if (!auth.ok) {
      ws.close(CLOSE_FOR[auth.reason], auth.reason);
      return;
    }

    const now = Date.now();
    if (role === "desk") this.replaceExistingDesk(ws);

    const authed: AuthedState = {
      state: "authed",
      role,
      viewerId: auth.viewerId,
      lastSeen: now,
      since: now,
    };
    writeState(ws, authed);

    ws.send(encode("welcome", this.welcomeFor(role, deskId), message.id));

    if (role === "desk") {
      this.broadcastToViewers(encode("desk_status", { online: true, since: now }));
    }
    await this.scheduleSweep(this.idleTimeoutMs);
  }

  private async handleAuthed(
    ws: WebSocket,
    state: AuthedState,
    message: KnownMessage,
  ): Promise<void> {
    switch (message.type) {
      case "ping":
        ws.send(encode("pong", null, message.id));
        return;

      case "pong":
        // `lastSeen` was already refreshed; a pong carries nothing else.
        return;

      case "hello":
        ws.send(errorFrame("already_authenticated", "this socket already said hello", message.id));
        return;

      case "event":
        if (state.role !== "desk") {
          ws.send(errorFrame("forbidden", "only a desk may publish events", message.id));
          return;
        }
        this.rememberSnapshot(message.payload, message.ts);
        this.broadcastToViewers(JSON.stringify(message), ws);
        return;

      case "command":
      case "command_result":
        // T04. Answering is the point: a silent drop is indistinguishable from
        // a delivered command that the desk chose to ignore.
        ws.send(
          errorFrame(
            "not_implemented",
            `${message.type} routing lands in E022-T04`,
            message.id,
          ),
        );
        return;

      default:
        ws.send(errorFrame("unexpected_type", `${message.type} is not sent by a client`, message.id));
    }
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
    const state = readState(ws);
    // Workers requires the server side to be closed explicitly after the peer
    // closes; without it the socket lingers in `getWebSockets()`.
    try {
      ws.close(code, reason);
    } catch {
      // Already closed — nothing to undo.
    }
    if (isAuthed(state) && state.role === "desk") this.announceDeskGone(ws);
  }

  /**
   * Tells the viewers the desk is gone — but only once the *last* desk socket
   * has left. `ws` is excluded explicitly because a socket being closed may
   * still appear in `getWebSockets()` while its handler runs, which would make
   * a departing desk look like a desk that is still there.
   */
  private announceDeskGone(ws: WebSocket): void {
    const remaining = socketsInRole(this.ctx.getWebSockets(), "desk").filter((d) => d.ws !== ws);
    if (remaining.length > 0) return;
    this.broadcastToViewers(encode("desk_status", { online: false, since: Date.now() }));
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws, 1011, "socket error");
  }

  // ─── Alarm sweep ──────────────────────────────────────────────────────────

  /**
   * One alarm serves both deadlines. It closes what is overdue and re-arms
   * itself only while sockets remain, so an empty room costs nothing.
   */
  async alarm(): Promise<void> {
    const now = Date.now();
    let nextIn = Number.POSITIVE_INFINITY;

    for (const ws of this.ctx.getWebSockets()) {
      const state = readState(ws);
      if (state === null) continue;

      if (state.state === "pending") {
        if (state.helloBy <= now) ws.close(CLOSE_CODES.UNAUTHENTICATED, "no hello in time");
        else nextIn = Math.min(nextIn, state.helloBy - now);
        continue;
      }

      const silentFor = now - state.lastSeen;
      if (silentFor >= this.idleTimeoutMs) {
        ws.close(IDLE_CLOSE_CODE, "idle");
        // A close we initiate does not call `webSocketClose`, so the viewers
        // would otherwise keep showing a desk that timed out.
        if (state.role === "desk") this.announceDeskGone(ws);
      } else {
        nextIn = Math.min(nextIn, this.idleTimeoutMs - silentFor);
      }
    }

    if (Number.isFinite(nextIn)) await this.scheduleSweep(nextIn);
  }

  /** Arms the alarm for `inMs`, unless an earlier one is already pending. */
  private async scheduleSweep(inMs: number): Promise<void> {
    const at = Date.now() + Math.max(inMs, MIN_SWEEP_MS);
    const existing = await this.ctx.storage.getAlarm();
    if (existing === null || existing > at) await this.ctx.storage.setAlarm(at);
  }

  // ─── Room state ───────────────────────────────────────────────────────────

  private deskSocket(): WebSocket | null {
    return socketsInRole(this.ctx.getWebSockets(), "desk")[0]?.ws ?? null;
  }

  private viewerSockets(): WebSocket[] {
    return socketsInRole(this.ctx.getWebSockets(), "viewer").map((v) => v.ws);
  }

  /**
   * A newer desk connection wins: the app was restarted or moved machines, and
   * the old socket is a ghost the user cannot see or close.
   */
  private replaceExistingDesk(incoming: WebSocket): void {
    for (const { ws } of socketsInRole(this.ctx.getWebSockets(), "desk")) {
      if (ws === incoming) continue;
      ws.close(CLOSE_CODES.REPLACED, "replaced by a newer desk connection");
    }
  }

  private welcomeFor(role: Role, deskId: string): Welcome {
    const deskOnline = role === "desk" || this.deskSocket() !== null;
    return {
      role,
      desk_id: deskId,
      desk_online: deskOnline,
      viewer_count: this.viewerSockets().length,
      snapshot: (this.snapshot ?? null) as Welcome["snapshot"],
      snapshot_ts: this.snapshotTs,
    };
  }

  /** Viewer capacity is enforced at pairing time by T03, not here. */

  /**
   * Caches the desk's `snapshot` event. Other event kinds pass through
   * untouched — they are deltas, and replaying one to a late viewer would tell
   * it about a change it never had the "before" for.
   */
  private rememberSnapshot(payload: unknown, ts: number): void {
    const name = (payload as { event?: unknown } | null)?.event;
    if (name !== "snapshot") return;
    this.snapshot = payload;
    this.snapshotTs = ts;
  }

  private broadcastToViewers(frame: string, except?: WebSocket): void {
    for (const ws of this.viewerSockets()) {
      if (ws === except) continue;
      try {
        ws.send(frame);
      } catch {
        // A socket that died between the scan and the send is closed by the
        // runtime; the close handler cleans up.
      }
    }
  }
}
