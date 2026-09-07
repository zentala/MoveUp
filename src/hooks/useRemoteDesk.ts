/**
 * useRemoteDesk.ts — the browser half of the desk display (E022-T08).
 *
 * Provides the same UseDeskResult interface as useDesk(), but works in any
 * browser (no Tauri IPC needed). Used by the remote display kiosk mode on a
 * phone or secondary screen.
 *
 * The socket itself no longer lives here: `src/remote/transports/` owns the
 * wire (LAN or relay, chosen by `selectTransport`) and this hook owns the
 * reducer. That split is what lets one phone build serve both paths — and it
 * is why the hook now also reports `capabilities` and `deskOnline`, two facts
 * that only the transport can know.
 */
import { useState, useEffect, useRef, useCallback, useReducer, useMemo } from "react";
import { selectTransport } from "@/remote/transports";
import type {
  Transport,
  TransportCapabilities,
  TransportMessage,
  TransportStatus,
} from "@/remote/transports";
import type { UseDeskConnection } from "./useDeskTypes";
import { deskReducer, initialDeskState, selectDeskView } from "./deskReducer";
import { publishRemoteHealth } from "./useHealth";
import type { HealthView } from "@/generated/HealthView";
import type {
  SessionStateDto,
  MetricSnapshot,
  TodaySummaryDto,
  StateChangedPayload,
} from "@/types";

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
  /**
   * Transport to use instead of the one `selectTransport()` would pick.
   * Read once, on the first render — swapping wires mid-session is a reload.
   */
  transport?: Transport;
}

/** Full snapshot sent by the server on connect and every ~1s. */
interface RemoteDisplayState {
  session: SessionStateDto;
  metrics: MetricSnapshot[];
  today: TodaySummaryDto;
  /**
   * Merged health view (E021-T03). Optional on the wire only so an older
   * backend cannot break the display; a current one always sends it.
   */
  health?: HealthView;
}

/**
 * Connects to the desk backend via WebSocket.
 * Same interface as useDesk() but works in any browser.
 */
export function useRemoteDesk(options: UseRemoteDeskOptions = {}): UseDeskConnection {
  const [state, dispatch] = useReducer(deskReducer, initialDeskState);
  const [wsConnected, setWsConnected] = useState(false);
  const [deskOnline, setDeskOnline] = useState(false);

  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // One transport per mount, built lazily on the first render (it opens
  // nothing until `connect()`), so `capabilities` is stable from the very
  // first paint instead of flickering the controls in.
  const [transport] = useState<Transport>(
    () => options.transport ?? selectTransport(),
  );
  const capabilities: TransportCapabilities = transport.capabilities;

  const applySnapshot = useCallback((data: RemoteDisplayState) => {
    dispatch({
      type: "snapshot",
      session: data.session,
      metrics: data.metrics,
      today: data.today,
      health: data.health,
    });
    // The health widget lives outside this hook's subtree, so hand it the
    // view directly rather than widening `UseDeskResult` for one consumer.
    if (data.health) publishRemoteHealth(data.health);
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

  /** Routes one `DisplayEvent` into the reducer. Shared by both transports. */
  const applyEvent = useCallback(
    (event: string, payload: unknown) => {
      switch (event) {
        case "snapshot":
          applySnapshot(payload as RemoteDisplayState);
          break;
        case "desk:state-changed":
          applyStateChanged(payload as StateChangedPayload);
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
          emitVoiceAck(payload as VoiceAck);
          break;
        case "heartbeat":
          break;
      }
    },
    [applySnapshot, applyStateChanged],
  );

  useEffect(() => {
    const onMessage = (message: TransportMessage) => {
      switch (message.kind) {
        case "event":
          applyEvent(message.event, message.payload);
          break;
        case "welcome":
          // A room that has never seen the desk sends `snapshot: null`. Leave
          // the reducer at its initial state and let the overlay say "desk
          // offline" — rendering zeros would read as a real reading of zero.
          if (message.snapshot) applySnapshot(message.snapshot as RemoteDisplayState);
          break;
        case "desk_status":
        case "command_result":
          // Liveness arrives through onStatus; results through sendCommand.
          break;
      }
    };

    const onStatus = (status: TransportStatus) => {
      setWsConnected(status.connected);
      setDeskOnline(status.deskOnline);
    };

    const offMessage = transport.onMessage(onMessage);
    const offStatus = transport.onStatus(onStatus);
    transport.connect();

    return () => {
      offMessage();
      offStatus();
      if (transitionTimer.current) clearTimeout(transitionTimer.current);
      transport.close();
    };
  }, [transport, applyEvent, applySnapshot]);

  // No-ops: settings not available in remote display mode
  const calibrate = useCallback(async () => { console.warn("Calibration unavailable in remote mode"); }, []);
  const setSitLimit = useCallback(async () => { console.warn("setSitLimit unavailable in remote mode"); }, []);
  const setStandLimit = useCallback(async () => { console.warn("setStandLimit unavailable in remote mode"); }, []);

  const view = useMemo(() => selectDeskView(state), [state]);

  return {
    ...view,
    port: null,
    wsConnected,
    deskOnline,
    capabilities,
    calibrate,
    setSitLimit,
    setStandLimit,
  };
}
