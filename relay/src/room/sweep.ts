/**
 * sweep.ts — the room's two deadlines, served by one alarm.
 *
 * A socket that never says `hello` and a socket that has gone silent are both
 * closed here. The function returns how long until the next thing falls due, or
 * `null` when nothing does, so an empty room stops costing alarms.
 */
import { CLOSE_CODES, IDLE_CLOSE_CODE } from "./messages";
import { readState } from "./sockets";

/** Smallest alarm delay; below this the sweep would busy-loop. */
export const MIN_SWEEP_MS = 50;

/**
 * Closes whatever is overdue.
 *
 * @param onDeskGone Called for a desk closed by *this* sweep. A close we
 *   initiate does not call `webSocketClose`, so without it the viewers would go
 *   on showing a desk that timed out.
 * @returns Milliseconds until the next deadline, or `null` if there is none.
 */
export function sweep(
  all: WebSocket[],
  idleTimeoutMs: number,
  now: number,
  onDeskGone: (ws: WebSocket) => void,
): number | null {
  let nextIn = Number.POSITIVE_INFINITY;

  for (const ws of all) {
    const state = readState(ws);
    if (state === null) continue;

    if (state.state === "pending") {
      if (state.helloBy <= now) ws.close(CLOSE_CODES.UNAUTHENTICATED, "no hello in time");
      else nextIn = Math.min(nextIn, state.helloBy - now);
      continue;
    }

    const silentFor = now - state.lastSeen;
    if (silentFor >= idleTimeoutMs) {
      ws.close(IDLE_CLOSE_CODE, "idle");
      if (state.role === "desk") onDeskGone(ws);
    } else {
      nextIn = Math.min(nextIn, idleTimeoutMs - silentFor);
    }
  }

  return Number.isFinite(nextIn) ? nextIn : null;
}
