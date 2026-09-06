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
export function useRemoteDesk(): UseDeskResult {
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
