/**
 * useRemoteDesk.ts — connects to the desk backend via WebSocket.
 *
 * Provides the same UseDeskResult interface as useDesk(), but works
 * in any browser (no Tauri IPC needed). Used by the remote display
 * kiosk mode on a phone or secondary screen.
 *
 * Auto-reconnects with exponential backoff (1s -> 2s -> 4s -> max 10s).
 * Falls back to REST polling every 2s when WebSocket is disconnected.
 */
import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import type { UseDeskResult, TransitionInfo } from "./useDeskTypes";
import type {
  DeskState,
  SessionStateDto,
  MetricSnapshot,
  TodaySummaryDto,
  StateChangedPayload,
  PreviousSession,
  SessionEntry,
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

/** Default break reset threshold in seconds (10 min for full reset). */
const BREAK_RESET_THRESHOLD_SECS = 600;

/**
 * Connects to the desk backend via WebSocket.
 * Same interface as useDesk() but works in any browser.
 */
export function useRemoteDesk(): UseDeskResult {
  const [connected, setConnected] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);
  const [state, setState] = useState<DeskState>("Away");
  const [deskHeightCm, setDeskHeightCm] = useState(0);
  const [secsSinceLastBreak, setSecsSinceLastBreak] = useState(0);
  const [standingSeconds, setStandingSeconds] = useState(0);
  const [breakSeconds, setBreakSeconds] = useState(0);
  const [sessionLimitSecs, setSessionLimitSecs] = useState(0);
  const [standLimitSecs, setStandLimitSecs] = useState(900);
  const [positionChanges, setPositionChanges] = useState(0);
  const [limitUsedSecs, setLimitUsedSecs] = useState(0);
  const [dailyScore, setDailyScore] = useState(0);
  const [metrics, setMetrics] = useState<MetricSnapshot[]>([]);
  const [idleSecs, setIdleSecs] = useState(0);
  const [awayBoutSecs, setAwayBoutSecs] = useState(0);
  const [continuousComputerSecs, setContinuousComputerSecs] = useState(0);
  // TODO: wire setError to WS failure states
  const [error] = useState<string | null>(null);
  const [transition, setTransition] = useState<TransitionInfo | null>(null);
  const [todaySummary, setTodaySummary] = useState<TodaySummaryDto | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const wsConnectedRef = useRef(false);
  const reconnectDelay = useRef(1000);
  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const applySnapshot = useCallback((data: RemoteDisplayState) => {
    const dto = data.session;
    setState(dto.state);
    setDeskHeightCm(dto.desk_height_cm);
    setSecsSinceLastBreak(dto.secs_since_last_break);
    setStandingSeconds(dto.standing_seconds);
    setBreakSeconds(dto.break_seconds);
    setSessionLimitSecs(dto.session_limit_secs);
    if (dto.stand_limit_secs > 0) setStandLimitSecs(dto.stand_limit_secs);
    setPositionChanges(dto.position_changes);
    setLimitUsedSecs(dto.limit_used_secs);
    setDailyScore(dto.daily_score);
    setIdleSecs(dto.idle_secs ?? 0);
    setAwayBoutSecs(dto.away_bout_secs ?? 0);
    setContinuousComputerSecs(dto.continuous_computer_secs ?? 0);
    setMetrics(data.metrics);
    setTodaySummary(data.today);
    if (dto.state !== "Away" && dto.desk_height_cm > 0) {
      setConnected(true);
    }
  }, []);

  const applyStateChanged = useCallback((payload: StateChangedPayload) => {
    setConnected(true);
    setState(payload.state);
    setDeskHeightCm(payload.desk_height_cm);
    setLimitUsedSecs(payload.limit_used_secs);
    setStandingSeconds(payload.standing_seconds);
    setBreakSeconds(payload.break_seconds);
    setPositionChanges(payload.position_changes);
    if (transitionTimer.current) clearTimeout(transitionTimer.current);
    setTransition({
      lastBreakSecs: payload.last_break_secs,
      lastSittingSecs: payload.last_sitting_secs,
      breakCredit: payload.break_credit,
      transitionTo: payload.state,
    });
    transitionTimer.current = setTimeout(() => setTransition(null), 30_000);
  }, []);

  const applyDailyReset = useCallback(() => {
    setSecsSinceLastBreak(0);
    setStandingSeconds(0);
    setBreakSeconds(0);
    setPositionChanges(0);
    setLimitUsedSecs(0);
    setDailyScore(0);
    setTodaySummary({ sitting_secs: 0, standing_secs: 0, sessions: [] });
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
            setConnected(true);
            break;
          case "desk:device-lost":
            setConnected(false);
            break;
          case "desk:daily-reset":
            applyDailyReset();
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
  }, [applySnapshot, applyStateChanged, applyDailyReset]);

  // No-ops: settings not available in remote display mode
  const calibrate = useCallback(async () => { console.warn("Calibration unavailable in remote mode"); }, []);
  const setSitLimit = useCallback(async () => { console.warn("setSitLimit unavailable in remote mode"); }, []);
  const setStandLimit = useCallback(async () => { console.warn("setStandLimit unavailable in remote mode"); }, []);

  const previousSession = useMemo((): PreviousSession | null => {
    if (!todaySummary || todaySummary.sessions.length < 2) return null;
    const prev = todaySummary.sessions[todaySummary.sessions.length - 2];
    const isBreak =
      prev.state === "Standing" || prev.state === "Walking" || prev.state === "Away";
    return {
      state: prev.state,
      durationSecs: prev.duration_secs,
      wasEffective: isBreak && prev.duration_secs >= 300,
    };
  }, [todaySummary]);

  const todaySessions: SessionEntry[] = todaySummary?.sessions ?? [];
  const todayChanges = positionChanges;
  const todaySittingSecs = todaySummary?.sitting_secs ?? 0;
  const todayStandingSecs = todaySummary?.standing_secs ?? 0;
  const breakResetProgress = Math.min(breakSeconds / BREAK_RESET_THRESHOLD_SECS, 1.0);

  return {
    connected,
    port: null,
    state,
    deskHeightCm,
    secsSinceLastBreak,
    standingSeconds,
    breakSeconds,
    sessionLimitSecs,
    standLimitSecs,
    positionChanges,
    limitUsedSecs,
    limitRemaining: sessionLimitSecs - limitUsedSecs,
    limitRatio: sessionLimitSecs > 0 ? limitUsedSecs / sessionLimitSecs : 0,
    breakResetThreshold: BREAK_RESET_THRESHOLD_SECS,
    breakResetProgress,
    previousSession,
    todaySessions,
    todayChanges,
    todaySittingSecs,
    todayStandingSecs,
    dailyScore,
    metrics,
    error,
    idleSecs,
    awayBoutSecs,
    continuousComputerSecs,
    transition,
    wsConnected,
    calibrate,
    setSitLimit,
    setStandLimit,
  };
}
