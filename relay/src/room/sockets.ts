/**
 * sockets.ts — what the room knows about one open WebSocket.
 *
 * The state lives in the socket's *attachment*, not in a field on the Durable
 * Object, because the room uses the WebSocket Hibernation API: the object can
 * be evicted between two messages and reconstructed with its sockets intact,
 * and only the attachment survives that.
 *
 * It is also not in the accept-time *tags*. A socket's role is not known at
 * upgrade — it arrives in `hello`, one round trip later — and tags cannot be
 * changed after `acceptWebSocket`. Putting the role in the URL to make it
 * taggable would have leaked connection metadata into a place the protocol
 * deliberately keeps empty (PLAN.md: "token is never in the URL"), so the
 * lookup is a filter over `getWebSockets()` instead. One desk and a handful of
 * viewers per room make that scan free.
 */
import type { Role } from "@app/generated/Role";

export interface PendingState {
  state: "pending";
  /** Unix ms after which a socket that has not said `hello` is closed 4401. */
  helloBy: number;
}

export interface AuthedState {
  state: "authed";
  role: Role;
  /** Null for a desk, and for a viewer until T03 can resolve one. */
  viewerId: string | null;
  /** Unix ms of the last frame received. Drives the idle close. */
  lastSeen: number;
  /** Unix ms the socket was authenticated — `desk_status.since` for a desk. */
  since: number;
  /**
   * Unix ms of each command this socket sent inside the current rate-limit
   * window (E022-T04). Absent on a socket that has sent none, and on every
   * desk socket. It rides in the attachment rather than in Durable Object
   * memory so an eviction cannot forgive a viewer's burst.
   */
  commandHits?: number[];
}

export type SocketState = PendingState | AuthedState;

export function readState(ws: WebSocket): SocketState | null {
  const raw = ws.deserializeAttachment() as SocketState | null | undefined;
  return raw ?? null;
}

export function writeState(ws: WebSocket, state: SocketState): void {
  ws.serializeAttachment(state);
}

export const isAuthed = (s: SocketState | null): s is AuthedState => s?.state === "authed";

/** Every authenticated socket in the given role, newest state first read. */
export function socketsInRole(
  all: WebSocket[],
  role: Role,
): Array<{ ws: WebSocket; state: AuthedState }> {
  const found: Array<{ ws: WebSocket; state: AuthedState }> = [];
  for (const ws of all) {
    const state = readState(ws);
    if (isAuthed(state) && state.role === role) found.push({ ws, state });
  }
  return found;
}
