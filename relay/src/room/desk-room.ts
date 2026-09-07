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
 * Command routing (`command`, `command_result`) lives in `commands.ts` (T04);
 * this file only dispatches to it. Every command is answered — refused here,
 * refused because the PC is off, or executed by the desk — because a viewer
 * that gets nothing back cannot tell a slow command from a lost one.
 */
import type { Role } from "@app/generated/Role";
import type { Welcome } from "@app/generated/Welcome";

import { verifyToken, type AuthFailure } from "../auth/tokens";
import { numVar, type Env } from "../env";
import {
  announceDeskGone,
  broadcastToViewers,
  rememberSnapshot,
  replaceExistingDesk,
  welcomeFor,
} from "./fanout";
import {
  emptyPending,
  routeCommand,
  routeCommandResult,
  type CommandBox,
  type PendingMap,
} from "./commands";
import {
  CLOSE_CODES,
  encode,
  errorFrame,
  isFailure,
  readFrame,
  type KnownMessage,
} from "./messages";
import {
  DEFAULT_LOCKOUT_MS,
  DEFAULT_MAX_ATTEMPTS,
  DEFAULT_TTL_MS,
  emptyPairing,
  type PairingLimits,
  type PairingState,
} from "./pairing";
import {
  closeAll,
  closeViewer,
  issueCode,
  onlineViewers,
  redeemCode,
} from "./room-admin";
import {
  RPC_PREFIX,
  handleRoomRpc,
  type IssuedCode,
  type RedeemReply,
  type RoomOps,
} from "./rpc";
import { isAuthed, readState, writeState, type AuthedState } from "./sockets";
import { MIN_SWEEP_MS, sweep } from "./sweep";

const CLOSE_FOR: Record<AuthFailure, number> = {
  unauthorized: CLOSE_CODES.UNAUTHENTICATED,
  unentitled: CLOSE_CODES.UNENTITLED,
  revoked: CLOSE_CODES.REVOKED,
};

export class DeskRoom implements DurableObject, RoomOps, CommandBox {
  /**
   * Newest `snapshot` event seen from the desk, verbatim. Memory only, and
   * public because `fanout.ts` writes it — see `SnapshotBox` there.
   */
  snapshot: unknown = null;
  snapshotTs: number | null = null;

  /**
   * The outstanding pairing code and the brute-force lockout. Memory only, and
   * public because `room-admin.ts` replaces it — see `PairingBox` there.
   */
  pairing: PairingState = emptyPairing();
  readonly limits: PairingLimits;

  /**
   * Commands forwarded to the desk and still waiting for a `command_result`,
   * keyed by the command's envelope id. Memory only, and public because
   * `commands.ts` writes it — see `CommandBox` there.
   */
  pending: PendingMap = emptyPending();

  private readonly helloTimeoutMs: number;
  private readonly idleTimeoutMs: number;

  constructor(
    private readonly ctx: DurableObjectState,
    private readonly env: Env,
  ) {
    this.helloTimeoutMs = numVar(env.HELLO_TIMEOUT_MS, 5_000);
    this.idleTimeoutMs = numVar(env.IDLE_TIMEOUT_MS, 60_000);
    this.limits = {
      ttlMs: numVar(env.PAIRING_TTL_MS, DEFAULT_TTL_MS),
      lockoutMs: numVar(env.PAIRING_LOCKOUT_MS, DEFAULT_LOCKOUT_MS),
      maxAttempts: numVar(env.PAIRING_MAX_ATTEMPTS, DEFAULT_MAX_ATTEMPTS),
    };
  }

  // ─── Pairing and revocation (RoomOps) ─────────────────────────────────────
  // Bodies live in `room-admin.ts`; this file stays about the protocol.

  issuePairing = (): Promise<IssuedCode> => issueCode(this);

  redeemPairing = (code: string): Promise<RedeemReply> => redeemCode(this, code);

  revokeViewer = (viewerId: string): number =>
    closeViewer(this.ctx.getWebSockets(), viewerId);

  shutdown = (): number => closeAll(this, this.ctx.getWebSockets());

  onlineViewerIds = (): string[] => onlineViewers(this.ctx.getWebSockets());

  // ─── Upgrade ──────────────────────────────────────────────────────────────

  async fetch(request: Request): Promise<Response> {
    const pathname = new URL(request.url).pathname;
    if (pathname.startsWith(RPC_PREFIX)) {
      const answered = await handleRoomRpc(request, pathname, this);
      if (answered !== null) return answered;
      return new Response("no such room operation", { status: 404 });
    }

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

    const refreshed: AuthedState = { ...state, lastSeen: Date.now() };
    writeState(ws, refreshed);
    await this.handleAuthed(ws, refreshed, parsed);
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
        routeCommand(this, this.ctx.getWebSockets(), ws, state, message, Date.now());
        return;

      case "command_result":
        routeCommandResult(this, this.ctx.getWebSockets(), ws, state, message, Date.now());
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

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws, 1011, "socket error");
  }

  // ─── Alarm sweep ──────────────────────────────────────────────────────────

  /**
   * One alarm serves both deadlines (`sweep.ts`). It closes what is overdue and
   * re-arms itself only while something is still due, so an empty room costs
   * nothing.
   */
  async alarm(): Promise<void> {
    const nextIn = sweep(this.ctx.getWebSockets(), this.idleTimeoutMs, Date.now(), (ws) =>
      this.announceDeskGone(ws),
    );
    if (nextIn !== null) await this.scheduleSweep(nextIn);
  }

  /** Arms the alarm for `inMs`, unless an earlier one is already pending. */
  private async scheduleSweep(inMs: number): Promise<void> {
    const at = Date.now() + Math.max(inMs, MIN_SWEEP_MS);
    const existing = await this.ctx.storage.getAlarm();
    if (existing === null || existing > at) await this.ctx.storage.setAlarm(at);
  }

  // ─── Room state (bodies in `fanout.ts`) ───────────────────────────────────

  private announceDeskGone = (ws: WebSocket): void =>
    announceDeskGone(this.ctx.getWebSockets(), ws);

  private replaceExistingDesk = (incoming: WebSocket): void =>
    replaceExistingDesk(this.ctx.getWebSockets(), incoming);

  private welcomeFor = (role: Role, deskId: string): Welcome =>
    welcomeFor(this, this.ctx.getWebSockets(), role, deskId);

  private rememberSnapshot = (payload: unknown, ts: number): void =>
    rememberSnapshot(this, payload, ts);

  private broadcastToViewers = (frame: string, except?: WebSocket): void =>
    broadcastToViewers(this.ctx.getWebSockets(), frame, except);
}
