/**
 * command-table.ts — the two pieces of bookkeeping command routing needs (E022-T04).
 *
 * Both are pure functions over state the caller owns, so they can be tested
 * without a Durable Object: the per-viewer throttle over a socket attachment,
 * and the table of commands still waiting for their result.
 */
import { writeState, type AuthedState } from "./sockets";

/** Commands one viewer may send per window (PLAN.md §Protocol). */
export const COMMAND_LIMIT = 10;
export const COMMAND_WINDOW_MS = 60_000;

/** How long the relay remembers who asked, waiting for the desk's answer. */
export const PENDING_TTL_MS = 30_000;

/**
 * Ceiling on the in-flight table. Ten commands a minute a viewer and a 30 s
 * memory make the honest maximum small; the cap is what stops a desk that
 * never answers from growing the map without bound.
 */
export const MAX_PENDING = 512;

/** Who asked for a command, and until when the room still cares. */
interface PendingCommand {
  viewerId: string;
  expiresAt: number;
}

/**
 * The in-flight table. Memory only, like the snapshot: a Durable Object
 * eviction between a command and its result loses the routing entry, and the
 * result is then answered with `unknown_command_id` rather than guessed at.
 */
export type PendingMap = Map<string, PendingCommand>;

/** A mutable holder so these functions can be tested without a real room. */
export interface CommandBox {
  pending: PendingMap;
}

export const emptyPending = (): PendingMap => new Map();

// ─── Per-viewer throttle ────────────────────────────────────────────────────

/**
 * Takes one token from a viewer's window.
 *
 * The hit list lives in the socket's attachment rather than a field on the
 * Durable Object, for the same reason the rest of the socket state does: the
 * object may be evicted between two commands, and only the attachment survives
 * that. A throttle kept in object memory would forgive a burst every eviction.
 *
 * @returns `null` when allowed, or how long until the caller may retry.
 */
export function takeCommandToken(
  ws: WebSocket,
  state: AuthedState,
  now: number,
): { retryAfterMs: number } | null {
  const recent = (state.commandHits ?? []).filter((at) => now - at < COMMAND_WINDOW_MS);

  if (recent.length >= COMMAND_LIMIT) {
    writeState(ws, { ...state, commandHits: recent });
    return { retryAfterMs: COMMAND_WINDOW_MS - (now - recent[0]) };
  }

  recent.push(now);
  writeState(ws, { ...state, commandHits: recent });
  return null;
}

// ─── In-flight table ────────────────────────────────────────────────────────

/**
 * Files a command under its envelope id.
 *
 * Expired entries are dropped first, so the cap is only reached by commands
 * that are genuinely still in flight. If it is reached anyway, the oldest
 * entry goes — `Map` preserves insertion order, so that is the first key.
 */
export function remember(box: CommandBox, commandId: string, viewerId: string, now: number): void {
  for (const [id, entry] of box.pending) {
    if (entry.expiresAt <= now) box.pending.delete(id);
  }
  while (box.pending.size >= MAX_PENDING) {
    const oldest = box.pending.keys().next();
    if (oldest.done) break;
    box.pending.delete(oldest.value);
  }
  box.pending.set(commandId, { viewerId, expiresAt: now + PENDING_TTL_MS });
}

/** The viewer a result belongs to, or `null` when the entry is gone or stale. */
export function claim(box: CommandBox, commandId: string, now: number): string | null {
  const entry = box.pending.get(commandId);
  if (!entry) return null;
  box.pending.delete(commandId);
  return entry.expiresAt <= now ? null : entry.viewerId;
}
