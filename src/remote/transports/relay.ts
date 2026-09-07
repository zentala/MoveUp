/**
 * relay.ts — the transport that reaches the desk from outside the LAN.
 *
 * Wire: `wss://<relay>/v1/desks/<desk_id>/ws`, one `DeskRoom` Durable Object
 * per desk. The token travels in the `hello` payload and **never** in the URL.
 *
 * Two facts this file exists to keep apart: the phone can be connected while
 * the desk is not (the room holds the last snapshot), and some closes are
 * terminal. Retrying a revoked token forever would look like a flaky network
 * while actually being a permanent state the user has to fix at `#/pair`.
 */
import { CLOSE_CODES, PROTOCOL_VERSION, parseMessage, validateCommand } from "../protocol";
import { clearRelayRecord, type RelayRecord } from "../storage";
import { Emitter } from "./emitter";
import type {
  CommandOutcome,
  Transport,
  TransportCapabilities,
  TransportMessage,
  TransportState,
  TransportStatus,
  Unsubscribe,
} from "./types";
import { VIEWER_CLIENT } from "./types";

const PING_MS = 25_000;
const RECONNECT_MIN_MS = 1000;
const RECONNECT_MAX_MS = 60_000;
const JITTER = 0.2;
/** How long a command may stay unanswered before the UI stops waiting. */
const COMMAND_TIMEOUT_MS = 10_000;

/** Close codes after which the socket must not be reopened. */
const TERMINAL: Record<number, TransportState> = {
  [CLOSE_CODES.UNENTITLED]: "error",
  [CLOSE_CODES.REVOKED]: "unpaired",
  [CLOSE_CODES.REPLACED]: "error",
};

/** Injection points; every one has a working default. */
export interface RelayTransportOptions {
  /** Deterministic jitter for tests. Returns 0..1. */
  random?: () => number;
  /** Envelope id source. Defaults to `crypto.randomUUID`. */
  newId?: () => string;
}

/** `https://host` → `wss://host`, `http://host` → `ws://host`. */
export function relayWsUrl(relayUrl: string, deskId: string): string {
  const base = relayUrl.replace(/\/+$/, "");
  const ws = base.replace(/^http:/, "ws:").replace(/^https:/, "wss:");
  return `${ws}/v1/desks/${encodeURIComponent(deskId)}/ws`;
}

let idCounter = 0;
const defaultNewId = () =>
  globalThis.crypto?.randomUUID?.() ?? `cmd-${Date.now()}-${(idCounter += 1)}`;

/** Authenticated, bidirectional transport through the cloud relay. */
export class RelayTransport implements Transport {
  readonly id = "relay" as const;
  readonly capabilities: TransportCapabilities = { control: true };

  private readonly url: string;
  private readonly record: RelayRecord;
  private readonly random: () => number;
  private readonly newId: () => string;
  private readonly messages = new Emitter<TransportMessage>();
  private readonly statuses = new Emitter<TransportStatus>();
  private readonly pending = new Map<
    string,
    { resolve: (o: CommandOutcome) => void; timer: ReturnType<typeof setTimeout> }
  >();

  private ws: WebSocket | null = null;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectDelay = RECONNECT_MIN_MS;
  private connected = false;
  private deskOnline = false;
  private open = false;
  private state: TransportState = "disconnected";
  private lastError: string | null = null;

  constructor(record: RelayRecord, options: RelayTransportOptions = {}) {
    this.record = record;
    this.url = relayWsUrl(record.relay_url, record.desk_id);
    this.random = options.random ?? Math.random;
    this.newId = options.newId ?? defaultNewId;
  }

  onMessage(handler: (message: TransportMessage) => void): Unsubscribe {
    return this.messages.on(handler);
  }

  onStatus(handler: (status: TransportStatus) => void): Unsubscribe {
    return this.statuses.on(handler);
  }

  connect(): void {
    if (this.open) return;
    this.open = true;
    this.openSocket();
  }

  close(): void {
    this.open = false;
    this.clearTimers();
    for (const [, entry] of this.pending) {
      clearTimeout(entry.timer);
      entry.resolve({ ok: false, error: { code: "closed", message: "transport closed" } });
    }
    this.pending.clear();
    this.ws?.close();
    this.ws = null;
    this.messages.clear();
    this.statuses.clear();
  }

  async sendCommand(
    name: string,
    args: Record<string, unknown>,
  ): Promise<CommandOutcome> {
    // The desk validates again; this copy just avoids a pointless round trip.
    const invalid = validateCommand(name, args);
    if (invalid) return { ok: false, error: invalid };
    if (!this.connected || !this.ws) {
      return { ok: false, error: { code: "offline", message: "not connected to the relay" } };
    }

    const id = this.newId();
    this.send("command", { name, args, viewer_id: null }, id);

    return new Promise<CommandOutcome>((resolve) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        resolve({ ok: false, error: { code: "timeout", message: "no reply from the desk" } });
      }, COMMAND_TIMEOUT_MS);
      this.pending.set(id, { resolve, timer });
    });
  }

  // ─── internals ────────────────────────────────────────────────────────────

  private clearTimers(): void {
    if (this.pingTimer) clearInterval(this.pingTimer);
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.pingTimer = null;
    this.reconnectTimer = null;
  }

  private emitStatus(): void {
    this.statuses.emit({
      connected: this.connected,
      deskOnline: this.deskOnline,
      state: this.state,
      error: this.lastError,
    });
  }

  private send(type: string, payload: unknown, id = this.newId()): void {
    this.ws?.send(
      JSON.stringify({ v: PROTOCOL_VERSION, type, id, ts: Date.now(), payload }),
    );
  }

  private openSocket(): void {
    this.state = "connecting";
    this.emitStatus();
    const ws = new WebSocket(this.url);
    this.ws = ws;

    ws.onopen = () => {
      this.connected = true;
      this.state = "connected";
      this.lastError = null;
      this.send("hello", {
        role: "viewer",
        desk_id: this.record.desk_id,
        token: this.record.viewer_token,
        client: VIEWER_CLIENT,
      });
      this.pingTimer = setInterval(() => this.send("ping", null), PING_MS);
      this.emitStatus();
    };

    ws.onmessage = (e: MessageEvent) => this.handleFrame(e.data as string);
    ws.onclose = (e?: CloseEvent) => this.handleClose(e?.code ?? 1006);
    ws.onerror = () => ws.close();
  }

  private handleClose(code: number): void {
    this.connected = false;
    this.deskOnline = false;
    if (this.pingTimer) clearInterval(this.pingTimer);
    this.pingTimer = null;

    const terminal = TERMINAL[code];
    if (terminal) {
      // A revoked viewer is unpaired for good: drop the credential so the
      // next load lands on the LAN path (or the pairing screen) rather than
      // retrying a token the relay has already forgotten.
      if (code === CLOSE_CODES.REVOKED) clearRelayRecord();
      this.open = false;
      this.state = terminal;
      this.lastError = `relay closed the connection (${code})`;
      this.emitStatus();
      return;
    }

    this.state = "disconnected";
    this.emitStatus();
    if (!this.open) return;
    this.reconnectTimer = setTimeout(() => this.openSocket(), this.jittered());
    this.reconnectDelay = Math.min(this.reconnectDelay * 2, RECONNECT_MAX_MS);
  }

  /** Backoff with ±20 % spread, so a relay restart does not get a thundering herd. */
  private jittered(): number {
    const spread = this.reconnectDelay * JITTER;
    return Math.round(this.reconnectDelay - spread + this.random() * spread * 2);
  }

  private handleFrame(data: string): void {
    let raw: unknown;
    try {
      raw = JSON.parse(data);
    } catch {
      console.warn("Relay: malformed WS message, ignoring");
      return;
    }
    const outcome = parseMessage(raw);
    if (outcome.status === "error") {
      console.warn(`Relay: rejected frame (${outcome.error.code})`);
      return;
    }
    if (outcome.status !== "ok") return;
    const message = outcome.message;

    switch (message.type) {
      case "welcome": {
        this.reconnectDelay = RECONNECT_MIN_MS;
        this.deskOnline = message.payload.desk_online;
        this.emitStatus();
        this.messages.emit({
          kind: "welcome",
          deskOnline: message.payload.desk_online,
          snapshot: message.payload.snapshot,
        });
        break;
      }
      case "event": {
        const inner = message.payload as { event?: unknown; payload?: unknown };
        if (typeof inner.event !== "string") return;
        this.messages.emit({ kind: "event", event: inner.event, payload: inner.payload });
        break;
      }
      case "desk_status": {
        this.deskOnline = message.payload.online;
        this.emitStatus();
        this.messages.emit({ kind: "desk_status", online: message.payload.online });
        break;
      }
      case "command_result": {
        const { command_id: commandId, ok, error } = message.payload;
        const entry = this.pending.get(commandId);
        if (entry) {
          clearTimeout(entry.timer);
          this.pending.delete(commandId);
          entry.resolve({ ok, error });
        }
        this.messages.emit({ kind: "command_result", commandId, ok, error });
        break;
      }
      case "ping":
        this.send("pong", null, message.id);
        break;
      case "error":
        console.warn(`Relay: ${message.payload.code} — ${message.payload.message}`);
        break;
      default:
        break;
    }
  }
}
