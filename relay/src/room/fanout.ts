/**
 * fanout.ts — who is in the room, and what each of them is sent.
 *
 * Functions over the socket list rather than methods, so `desk-room.ts` can be
 * read as the protocol state machine it is. Nothing here parses or validates:
 * frames arrive already decided.
 */
import type { Role } from "@app/generated/Role";
import type { Welcome } from "@app/generated/Welcome";

import { CLOSE_CODES, encode } from "./messages";
import { socketsInRole } from "./sockets";

/** The newest snapshot the desk sent. Memory only — it dies with the object. */
export interface SnapshotBox {
  snapshot: unknown;
  snapshotTs: number | null;
}

export const deskSocket = (all: WebSocket[]): WebSocket | null =>
  socketsInRole(all, "desk")[0]?.ws ?? null;

export const viewerSockets = (all: WebSocket[]): WebSocket[] =>
  socketsInRole(all, "viewer").map((v) => v.ws);

export function broadcastToViewers(all: WebSocket[], frame: string, except?: WebSocket): void {
  for (const ws of viewerSockets(all)) {
    if (ws === except) continue;
    try {
      ws.send(frame);
    } catch {
      // A socket that died between the scan and the send is closed by the
      // runtime; the close handler cleans up.
    }
  }
}

/**
 * A newer desk connection wins: the app was restarted or moved machines, and
 * the old socket is a ghost the user cannot see or close.
 */
export function replaceExistingDesk(all: WebSocket[], incoming: WebSocket): void {
  for (const { ws } of socketsInRole(all, "desk")) {
    if (ws === incoming) continue;
    ws.close(CLOSE_CODES.REPLACED, "replaced by a newer desk connection");
  }
}

/**
 * Tells the viewers the desk is gone — but only once the *last* desk socket has
 * left. `leaving` is excluded explicitly because a socket being closed may still
 * appear in `getWebSockets()` while its handler runs, which would make a
 * departing desk look like a desk that is still there.
 */
export function announceDeskGone(all: WebSocket[], leaving: WebSocket): void {
  const remaining = socketsInRole(all, "desk").filter((d) => d.ws !== leaving);
  if (remaining.length > 0) return;
  broadcastToViewers(all, encode("desk_status", { online: false, since: Date.now() }));
}

export function welcomeFor(
  box: SnapshotBox,
  all: WebSocket[],
  role: Role,
  deskId: string,
): Welcome {
  return {
    role,
    desk_id: deskId,
    desk_online: role === "desk" || deskSocket(all) !== null,
    viewer_count: viewerSockets(all).length,
    snapshot: (box.snapshot ?? null) as Welcome["snapshot"],
    snapshot_ts: box.snapshotTs,
  };
}

/**
 * Caches the desk's `snapshot` event. Other event kinds pass through untouched
 * — they are deltas, and replaying one to a late viewer would tell it about a
 * change it never had the "before" for.
 */
export function rememberSnapshot(box: SnapshotBox, payload: unknown, ts: number): void {
  if ((payload as { event?: unknown } | null)?.event !== "snapshot") return;
  box.snapshot = payload;
  box.snapshotTs = ts;
}
