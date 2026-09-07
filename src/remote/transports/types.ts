/**
 * types.ts — the contract every phone-side transport implements (E022-T08).
 *
 * The viewer app talks to the desk over one of two wires: the LAN WebSocket
 * the app has always served on `:3390`, or the Cloudflare relay. They differ
 * in reachability and in what they are allowed to do — the LAN path is
 * read-only, the relay path can send commands — and in nothing else. This
 * interface is where that difference is declared once, so `useRemoteDesk`
 * never asks which wire it is on.
 */
import type { ErrorBody } from "../../generated/ErrorBody";

/** Which wire a transport speaks. */
export type TransportId = "lan" | "relay";

/**
 * What a transport is allowed to do.
 *
 * `control` gates the whole `RemoteControls` surface (T09): the LAN socket is
 * unauthenticated, so it never carries a write.
 */
export interface TransportCapabilities {
  control: boolean;
}

/** One inbound message, normalised across both wires. */
export type TransportMessage =
  /** A `DisplayEvent` — `{event, payload}` exactly as the reducer expects. */
  | { kind: "event"; event: string; payload: unknown }
  /** Relay handshake: the last snapshot the room held, plus desk liveness. */
  | { kind: "welcome"; deskOnline: boolean; snapshot: unknown | null }
  /** The desk came or went while we stayed connected. */
  | { kind: "desk_status"; online: boolean }
  /** Reply to a `sendCommand` call, routed by envelope id. */
  | {
      kind: "command_result";
      commandId: string;
      ok: boolean;
      error: ErrorBody | null;
    };

/**
 * Why the transport is in the state it is in.
 *
 * `unpaired` is terminal and distinct from `error`: the credential is gone,
 * so retrying cannot help and the UI must send the user back to `#/pair`.
 */
export type TransportState =
  | "connecting"
  | "connected"
  | "disconnected"
  | "unpaired"
  | "error";

/** Link status, pushed on every change. */
export interface TransportStatus {
  /** The socket to the peer (desk on LAN, relay on the cloud path) is up. */
  connected: boolean;
  /**
   * The desk itself is reachable. On LAN this equals `connected` — the desk
   * *is* the server. On the relay the two come apart, which is the whole
   * reason `ConnectionOverlay` needs a fourth state.
   */
  deskOnline: boolean;
  state: TransportState;
  error: string | null;
}

/** Outcome of one `sendCommand` call. */
export interface CommandOutcome {
  ok: boolean;
  error: ErrorBody | null;
}

/** Unsubscribe handle returned by `onMessage` / `onStatus`. */
export type Unsubscribe = () => void;

/** One wire to the desk. */
export interface Transport {
  readonly id: TransportId;
  readonly capabilities: TransportCapabilities;
  /** Opens the connection. Idempotent: a second call while open is a no-op. */
  connect(): void;
  /** Closes the connection and cancels every timer. Safe to call twice. */
  close(): void;
  /** Present only when `capabilities.control` is true. */
  sendCommand?(
    name: string,
    args: Record<string, unknown>,
  ): Promise<CommandOutcome>;
  onMessage(handler: (message: TransportMessage) => void): Unsubscribe;
  onStatus(handler: (status: TransportStatus) => void): Unsubscribe;
}

/**
 * Version this build reports in `hello.client`.
 *
 * Read from the Vite env rather than hardcoded, because a hardcoded version
 * drifts silently and then lies in the relay's logs. `dev` when unset.
 */
export const VIEWER_VERSION: string =
  (import.meta.env?.VITE_APP_VERSION as string | undefined) ?? "dev";

/** `hello.client` for the phone viewer. */
export const VIEWER_CLIENT = {
  app: "moveup-viewer",
  version: VIEWER_VERSION,
} as const;
