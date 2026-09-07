/**
 * commands.ts — viewer → desk command routing (E022-T04).
 *
 * Three rules the rest of this file exists to keep:
 *
 * 1. **A command is always answered.** Refused on the relay, refused because
 *    the PC is off, or executed by the desk — the viewer gets a
 *    `command_result` or an `error` carrying the command's own envelope id.
 *    A dropped command is indistinguishable from a slow one, and the phone
 *    would spin forever.
 * 2. **A `command_result` reaches exactly one viewer.** The desk answers with
 *    the `command_id` it was given; the room looks that id up in `pending` and
 *    sends the reply there and nowhere else. Broadcasting a result would tell
 *    every paired phone what another phone just did.
 * 3. **The forwarded envelope keeps its id and its `ts`.** Only `viewer_id` is
 *    stamped on. Re-timestamping would defeat the desk's own staleness check
 *    (T07 ignores a command older than 30 s), and re-issuing the id would
 *    break the reply lookup.
 *
 * The relay validates arguments even though the desk validates them again.
 * That is not redundancy: the relay's copy is what lets a bad command be
 * refused without waking the desk, and the desk's copy is what makes the relay
 * untrusted (ADR 023).
 */
import type { Command } from "@app/generated/Command";
import type { CommandResult } from "@app/generated/CommandResult";
import { validateCommand } from "@app/remote/protocol";

import {
  COMMAND_LIMIT,
  claim,
  remember,
  takeCommandToken,
  type CommandBox,
} from "./command-table";
import { deskSocket } from "./fanout";
import { encode, errorFrame, type Envelope, type KnownMessage } from "./messages";
import { socketsInRole, type AuthedState } from "./sockets";

export {
  COMMAND_LIMIT,
  COMMAND_WINDOW_MS,
  MAX_PENDING,
  PENDING_TTL_MS,
  claim,
  emptyPending,
  remember,
  takeCommandToken,
  type CommandBox,
  type PendingMap,
} from "./command-table";

// ─── Routing ────────────────────────────────────────────────────────────────

/** A relay-authored `command_result` — used for every refusal. */
function resultFrame(commandId: string, code: string, message: string): string {
  return encode("command_result", {
    command_id: commandId,
    ok: false,
    error: { code, message },
  } satisfies CommandResult);
}

/**
 * Handles a `command` from a viewer socket.
 *
 * A refusal that is about *this message* (bad name, bad arguments) is an
 * `error` frame, matching the contract's "non-fatal (rate limit, unknown type,
 * bad args)". A refusal that is about the *world* (the PC is off) is a
 * `command_result`, because from the phone's point of view the command ran and
 * failed — the pending UI has to resolve either way.
 */
export function routeCommand(
  box: CommandBox,
  sockets: WebSocket[],
  ws: WebSocket,
  state: AuthedState,
  message: Envelope<Command> & { type: "command" },
  now: number,
): void {
  if (state.role !== "viewer") {
    ws.send(errorFrame("forbidden", "only a viewer may send commands", message.id));
    return;
  }

  const throttled = takeCommandToken(ws, state, now);
  if (throttled) {
    ws.send(
      errorFrame(
        "rate_limited",
        `at most ${COMMAND_LIMIT} commands a minute; retry in ${Math.ceil(
          throttled.retryAfterMs / 1000,
        )}s`,
        message.id,
      ),
    );
    return;
  }

  const invalid = validateCommand(message.payload.name, message.payload.args);
  if (invalid) {
    ws.send(errorFrame(invalid.code, invalid.message, message.id));
    return;
  }

  const desk = deskSocket(sockets);
  if (!desk) {
    ws.send(resultFrame(message.id, "desk_offline", "the desk is not connected"));
    return;
  }

  // `viewer_id` is stamped by the relay and is the only field it adds. Id and
  // `ts` are the viewer's own, on purpose — see the header.
  const forwarded: Envelope<Command> = {
    ...message,
    payload: { ...message.payload, viewer_id: state.viewerId },
  };

  try {
    desk.send(JSON.stringify(forwarded));
  } catch {
    ws.send(resultFrame(message.id, "desk_offline", "the desk connection dropped"));
    return;
  }

  if (state.viewerId !== null) remember(box, message.id, state.viewerId, now);
}

/**
 * Handles a `command_result` from the desk socket.
 *
 * An unmatched `command_id` is answered rather than dropped: it means the room
 * forgot who asked (eviction, or a desk that took longer than 30 s), and the
 * desk deserves to know its answer went nowhere.
 */
export function routeCommandResult(
  box: CommandBox,
  sockets: WebSocket[],
  ws: WebSocket,
  state: AuthedState,
  message: KnownMessage & { type: "command_result" },
  now: number,
): void {
  if (state.role !== "desk") {
    ws.send(errorFrame("forbidden", "only the desk may answer commands", message.id));
    return;
  }

  const viewerId = claim(box, message.payload.command_id, now);
  if (viewerId === null) {
    ws.send(
      errorFrame(
        "unknown_command_id",
        `no viewer is waiting on ${message.payload.command_id}`,
        message.id,
      ),
    );
    return;
  }

  const frame = JSON.stringify(message);
  let delivered = 0;
  for (const viewer of socketsInRole(sockets, "viewer")) {
    if (viewer.state.viewerId !== viewerId) continue;
    try {
      viewer.ws.send(frame);
      delivered += 1;
    } catch {
      // Closed between the scan and the send; the close handler cleans up.
    }
  }

  if (delivered === 0) {
    ws.send(
      errorFrame(
        "viewer_gone",
        `viewer ${viewerId} left before the result arrived`,
        message.id,
      ),
    );
  }
}
