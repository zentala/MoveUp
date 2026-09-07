/**
 * lan.ts — the transport that has always existed, now behind the interface.
 *
 * Wire: `ws://<host>/display/ws`, served by `remote_server.rs` on the LAN.
 * Read-only by design (`capabilities.control === false`): that socket is
 * unauthenticated, so it never carries a write — see ADR 023.
 *
 * It accepts **two** frame shapes on purpose. Today the server sends a bare
 * `{event, payload}`; from T11 it wraps the same thing in the v1 envelope.
 * A phone and a desk are updated separately, so both shapes have to work at
 * once — and an unrecognised shape is warned about, never silently dropped.
 */
import { parseMessage } from "../protocol";
import { Emitter } from "./emitter";
import type {
  Transport,
  TransportCapabilities,
  TransportMessage,
  TransportStatus,
  Unsubscribe,
} from "./types";

/** Backoff floor and ceiling, unchanged from the pre-T08 hook. */
const RECONNECT_MIN_MS = 1000;
const RECONNECT_MAX_MS = 10_000;
/** REST poll cadence while the socket is down. */
const FALLBACK_POLL_MS = 2000;

/** Injection points; every one has a working default. */
export interface LanTransportOptions {
  /** Defaults to `ws://<location.host>/display/ws`. */
  wsUrl?: string;
  /** Defaults to `/display/api`. */
  apiUrl?: string;
}

const defaultWsUrl = () =>
  `ws://${typeof window === "undefined" ? "localhost" : window.location.host}/display/ws`;

/** Read-only LAN transport. */
export class LanTransport implements Transport {
  readonly id = "lan" as const;
  readonly capabilities: TransportCapabilities = { control: false };

  private readonly wsUrl: string;
  private readonly apiUrl: string;
  private readonly messages = new Emitter<TransportMessage>();
  private readonly statuses = new Emitter<TransportStatus>();

  private ws: WebSocket | null = null;
  private reconnectDelay = RECONNECT_MIN_MS;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private pollTimer: ReturnType<typeof setInterval> | null = null;
  private connected = false;
  private open = false;

  constructor(options: LanTransportOptions = {}) {
    this.wsUrl = options.wsUrl ?? defaultWsUrl();
    this.apiUrl = options.apiUrl ?? "/display/api";
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
    this.pollTimer = setInterval(() => this.pollFallback(), FALLBACK_POLL_MS);
  }

  close(): void {
    this.open = false;
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    if (this.pollTimer) clearInterval(this.pollTimer);
    this.reconnectTimer = null;
    this.pollTimer = null;
    this.ws?.close();
    this.ws = null;
    this.messages.clear();
    this.statuses.clear();
  }

  private emitStatus(): void {
    this.statuses.emit({
      connected: this.connected,
      // The LAN server *is* the desk: one liveness fact, not two.
      deskOnline: this.connected,
      state: this.connected ? "connected" : "disconnected",
      error: null,
    });
  }

  private openSocket(): void {
    const ws = new WebSocket(this.wsUrl);
    this.ws = ws;

    ws.onopen = () => {
      this.connected = true;
      this.reconnectDelay = RECONNECT_MIN_MS;
      this.emitStatus();
    };

    ws.onmessage = (e: MessageEvent) => this.handleFrame(e.data as string);

    ws.onclose = () => {
      this.connected = false;
      this.emitStatus();
      if (!this.open) return;
      this.reconnectTimer = setTimeout(() => this.openSocket(), this.reconnectDelay);
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, RECONNECT_MAX_MS);
    };

    ws.onerror = () => ws.close();
  }

  private handleFrame(data: string): void {
    let raw: unknown;
    try {
      raw = JSON.parse(data);
    } catch {
      console.warn("Remote display: malformed WS message, ignoring");
      return;
    }
    if (typeof raw !== "object" || raw === null) {
      console.warn("Remote display: malformed WS message, ignoring");
      return;
    }

    // Pre-T11 servers send the DisplayEvent itself; newer ones wrap it.
    if ("event" in raw) {
      const legacy = raw as { event: unknown; payload?: unknown };
      if (typeof legacy.event !== "string") {
        console.warn("Remote display: malformed WS message, ignoring");
        return;
      }
      this.messages.emit({ kind: "event", event: legacy.event, payload: legacy.payload });
      return;
    }

    const outcome = parseMessage(raw);
    if (outcome.status === "error") {
      console.warn(`Remote display: rejected frame (${outcome.error.code})`);
      return;
    }
    // A type this build does not know is ignored by contract, not an error.
    if (outcome.status !== "ok" || outcome.message.type !== "event") return;

    const inner = outcome.message.payload as { event?: unknown; payload?: unknown };
    if (typeof inner.event !== "string") return;
    this.messages.emit({ kind: "event", event: inner.event, payload: inner.payload });
  }

  /** Polls the REST snapshot while the socket is down, as before T08. */
  private pollFallback(): void {
    if (this.connected) return;
    fetch(this.apiUrl)
      .then((r) => r.json())
      .then((data) => this.messages.emit({ kind: "event", event: "snapshot", payload: data }))
      .catch(() => {});
  }
}
