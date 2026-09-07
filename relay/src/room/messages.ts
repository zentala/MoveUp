/**
 * messages.ts — the relay's half of the v1 envelope (E022-T02).
 *
 * Parsing is not re-implemented here: `@app/remote/protocol` (E022-T01) is the
 * single schema the desk, the phone and the relay all validate against. This
 * module only adds what a *sender* needs — building envelopes and deciding what
 * a parse failure should do to the socket.
 */
import {
  CLOSE_CODES,
  PROTOCOL_VERSION,
  parseMessage,
  type Envelope,
  type KnownMessage,
  type MessageType,
  type ParseOutcome,
} from "@app/remote/protocol";
import type { ErrorBody } from "@app/generated/ErrorBody";

export {
  CLOSE_CODES,
  PROTOCOL_VERSION,
  parseMessage,
  type Envelope,
  type ErrorBody,
  type KnownMessage,
  type MessageType,
  type ParseOutcome,
};

/**
 * Closed with 1001 rather than a 4xxx code: an idle timeout is not the client's
 * fault and reconnecting is the right response, which is exactly what a
 * standard "going away" means. The 4xxx codes in `CLOSE_CODES` all say
 * "something about you is wrong", and three of them stop the desk from
 * retrying at all (`TERMINAL_CLOSE_CODES`).
 */
export const IDLE_CLOSE_CODE = 1001;

/** Builds an outbound envelope. `id` is echoed by replies, so callers may pin it. */
export function envelope<T>(type: MessageType, payload: T, id?: string): Envelope<T> {
  return {
    v: PROTOCOL_VERSION,
    type,
    id: id ?? crypto.randomUUID(),
    ts: Date.now(),
    payload,
  };
}

/** Serialises an envelope for `ws.send`. */
export function encode<T>(type: MessageType, payload: T, id?: string): string {
  return JSON.stringify(envelope(type, payload, id));
}

/** An `error` envelope — non-fatal by contract; the socket stays open. */
export function errorFrame(code: string, message: string, id?: string): string {
  return encode("error", { code, message } satisfies ErrorBody, id);
}

/**
 * What a frame that failed to parse should do to the connection.
 *
 * `close` means the peer is speaking a protocol we cannot follow at all; `send`
 * means one message was wrong and the conversation continues. Keeping the two
 * apart is the whole reason this returns a decision instead of throwing.
 */
export type ParseFailure =
  | { action: "close"; code: number; reason: string }
  | { action: "send"; frame: string };

/** The sender's envelope id, when the frame had one worth echoing. */
function idOf(json: unknown): string | undefined {
  const id = (json as { id?: unknown } | null)?.id;
  return typeof id === "string" && id.length > 0 ? id : undefined;
}

/**
 * Parses a raw frame. Returns the message, or the decision its failure implies.
 *
 * An `error` answer carries the offending frame's own id, so a client waiting
 * on a reply to message X learns that X is what went wrong instead of seeing an
 * unrelated error and continuing to wait.
 *
 * `unknown_type` is deliberately non-fatal: the contract says a client ignores
 * types it does not know, and a newer peer must not be disconnected for being
 * newer.
 */
export function readFrame(raw: string): KnownMessage | ParseFailure {
  let json: unknown;
  try {
    json = JSON.parse(raw);
  } catch {
    return { action: "close", code: CLOSE_CODES.PROTOCOL_ERROR, reason: "malformed JSON" };
  }

  const replyId = idOf(json);
  const outcome: ParseOutcome = parseMessage(json);
  if (outcome.status === "ok") return outcome.message;

  if (outcome.status === "unknown_type") {
    return {
      action: "send",
      frame: errorFrame("unknown_type", `unsupported message type: ${outcome.type}`, replyId),
    };
  }

  // A wrong version or a frame that is not an envelope at all cannot be
  // answered inside the protocol — there is no agreed shape left to answer in.
  if (outcome.error.code === "unsupported_version" || outcome.error.code === "bad_envelope") {
    return {
      action: "close",
      code: CLOSE_CODES.PROTOCOL_ERROR,
      reason: outcome.error.message.slice(0, 120),
    };
  }
  return { action: "send", frame: errorFrame(outcome.error.code, outcome.error.message, replyId) };
}

/** Type guard separating a parsed message from a parse decision. */
export function isFailure(x: KnownMessage | ParseFailure): x is ParseFailure {
  return "action" in x;
}
