/**
 * useRemoteDesk.ts — WebSocket transport for the shared desk reducer.
 *
 * Provides the same UseDeskResult interface as useDesk(), but works
 * in any browser (no Tauri IPC needed). Used by the remote display
 * kiosk mode on a phone or secondary screen.
 *
 * Auto-reconnects with exponential backoff (1s -> 2s -> 4s -> max 10s).
 * Falls back to REST polling every 2s when WebSocket is disconnected.
 */
import { useState, useEffect, useRef, useCallback, useReducer, useMemo } from "react";
import type { UseDeskResult } from "./useDeskTypes";
import { deskReducer, initialDeskState, selectDeskView } from "./deskReducer";
import type {
  SessionStateDto,
  MetricSnapshot,
  TodaySummaryDto,
  StateChangedPayload,
} from "@/types";

/** WebSocket event shape (matches Rust DisplayEvent serialization). */
interface WsEvent {
  event: string;
  payload: unknown;
}

/**
 * Acknowledgement of a dictated voice note, pushed over the WS stream.
 *
 * Wire name `desk:voice-ack`, payload shape mirrors the Rust
 * `DisplayEvent::VoiceAck { transcript, intent, reply }` (E021-T06).
 */
export interface VoiceAck {
  transcript: string;
  intent: string;
  reply?: string | null;
}

type VoiceAckListener = (ack: VoiceAck) => void;

/**
 * Listeners for `desk:voice-ack`, kept at module level.
 *
 * The WS connection is owned by whoever calls `useRemoteDesk` (App, through
 * `useDeskAuto`), while the component that renders the ack — `VoiceCapture` —
 * sits elsewhere in the tree and cannot be handed the hook's options. Both
 * paths land here: the hook registers its `onVoiceAck` option as a listener,
 * and any component can subscribe directly.
 */
const voiceAckListeners = new Set<VoiceAckListener>();

/** Subscribes to `desk:voice-ack`. Returns the unsubscribe function. */
export function subscribeVoiceAck(listener: VoiceAckListener): () => void {
  voiceAckListeners.add(listener);
  return () => {
    voiceAckListeners.delete(listener);
  };
}

/** Fans a `desk:voice-ack` out to every subscriber. */
export function emitVoiceAck(ack: VoiceAck): void {
  for (const listener of voiceAckListeners) listener(ack);
}

/** Options for {@link useRemoteDesk}. */
export interface UseRemoteDeskOptions {
  /** Called for every `desk:voice-ack` event received on the stream. */
  onVoiceAck?: VoiceAckListener;
}

/** Full snapshot sent by the server on connect and every ~1s. */
interface RemoteDisplayState {
  session: SessionStateDto;
  metrics: MetricSnapshot[];
  today: TodaySummaryDto;
}

/**
 * Connects to the desk backend via WebSocket.
 * Same interface as useDesk() but works in any browser.
 */
export function useRemoteDesk(options: UseRemoteDeskOptions = {}): UseDeskResult {
  const [state, dispatch] = useReducer(deskReducer, initialDeskState);
  const [wsConnected, setWsConnected] = useState(false);

  const wsRef = useRef<WebSocket | null>(null);
  const wsConnectedRef = useRef(false);
  const reconnectDelay = useRef(1000);
  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const applySnapshot = useCallback((data: RemoteDisplayState) => {
    dispatch({
      type: "snapshot",
      session: data.session,
      metrics: data.metrics,
      today: data.today,
    });
  }, []);

  const applyStateChanged = useCallback((payload: StateChangedPayload) => {
    dispatch({ type: "state-changed", payload });
    if (transitionTimer.current) clearTimeout(transitionTimer.current);
    transitionTimer.current = setTimeout(
      () => dispatch({ type: "clear-transition" }),
      30_000,
    );
  }, []);

  const { onVoiceAck } = options;
  useEffect(() => {
    if (!onVoiceAck) return;
    return subscribeVoiceAck(onVoiceAck);
  }, [onVoiceAck]);

  useEffect(() => {
    let mounted = true;

    const wsUrl = `ws://${window.location.host}/display/ws`;
    const apiUrl = `/display/api`;

    function connect() {
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        setWsConnected(true);
        wsConnectedRef.current = true;
        reconnectDelay.current = 1000;
      };

      ws.onmessage = (e) => {
        let msg: WsEvent;
        try {
          msg = JSON.parse(e.data);
        } catch {
          console.warn("Remote display: malformed WS message, ignoring");
          return;
        }
        switch (msg.event) {
          case "snapshot":
            applySnapshot(msg.payload as RemoteDisplayState);
            break;
          case "desk:state-changed":
            applyStateChanged(msg.payload as StateChangedPayload);
            break;
          case "desk:device-connected":
            // Remote mode never exposes a serial port — it is not local.
            dispatch({ type: "device-connected", port: null });
            break;
          case "desk:device-lost":
            dispatch({ type: "device-lost" });
            break;
          case "desk:daily-reset":
            dispatch({ type: "daily-reset" });
            break;
          case "desk:voice-ack":
            emitVoiceAck(msg.payload as VoiceAck);
            break;
          case "heartbeat":
            break;
        }
      };

      ws.onclose = () => {
        setWsConnected(false);
        wsConnectedRef.current = false;
        if (mounted) {
          setTimeout(connect, reconnectDelay.current);
          reconnectDelay.current = Math.min(reconnectDelay.current * 2, 10000);
        }
      };

      ws.onerror = () => ws.close();
    }

    connect();

    const fallbackInterval = setInterval(() => {
      if (!wsConnectedRef.current) {
        fetch(apiUrl)
          .then((r) => r.json())
          .then((data) => applySnapshot(data as RemoteDisplayState))
          .catch(() => {});
      }
    }, 2000);

    return () => {
      mounted = false;
      clearInterval(fallbackInterval);
      if (transitionTimer.current) clearTimeout(transitionTimer.current);
      wsRef.current?.close();
    };
  }, [applySnapshot, applyStateChanged]);

  // No-ops: settings not available in remote display mode
  const calibrate = useCallback(async () => { console.warn("Calibration unavailable in remote mode"); }, []);
  const setSitLimit = useCallback(async () => { console.warn("setSitLimit unavailable in remote mode"); }, []);
  const setStandLimit = useCallback(async () => { console.warn("setStandLimit unavailable in remote mode"); }, []);

  const view = useMemo(() => selectDeskView(state), [state]);

  return {
    ...view,
    port: null,
    wsConnected,
    calibrate,
    setSitLimit,
    setStandLimit,
  };
}
