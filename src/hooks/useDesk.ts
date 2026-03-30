/**
 * useDesk.ts — custom hook that connects to the Tauri desk backend.
 *
 * Fetches initial session state on mount and subscribes to all `desk:*`
 * Tauri events, cleaning up listeners on unmount.
 */
import { useEffect, useState, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeskState,
  SessionStateDto,
  DashboardState,
  MetricSnapshot,
  DeviceConnectedPayload,
  StateChangedPayload,
  SensorErrorPayload,
  TodaySummaryDto,
  PreviousSession,
  SessionEntry,
} from "@/types";
import type { TransitionInfo, UseDeskResult } from "./useDeskTypes";

export type { TransitionInfo, UseDeskResult } from "./useDeskTypes";

/** Default break reset threshold in seconds (10 min for full reset). */
const BREAK_RESET_THRESHOLD_SECS = 600;

/**
 * Subscribes to all Tauri `desk:*` events and exposes current desk state.
 * Auto-fetches initial state via `get_session_state()` on mount, then:
 * - Polls `get_session_state()` every 1 second
 * - Polls `get_today_summary()` every 10 seconds
 * - Starts auto-connect on mount
 * - Exposes calibration and settings commands
 */
export function useDesk(): UseDeskResult {
  const [connected, setConnected] = useState(false);
  const [port, setPort] = useState<string | null>(null);
  const [state, setState] = useState<DeskState>("Away");
  const [deskHeightCm, setDeskHeightCm] = useState(0);
  const [sittingSeconds, setSittingSeconds] = useState(0);
  const [standingSeconds, setStandingSeconds] = useState(0);
  const [breakSeconds, setBreakSeconds] = useState(0);
  const [sessionLimitSecs, setSessionLimitSecs] = useState(0);
  const [standLimitSecs, setStandLimitSecs] = useState(900);
  const [positionChanges, setPositionChanges] = useState(0);
  const [limitUsedSecs, setLimitUsedSecs] = useState(0);
  const [dailyScore, setDailyScore] = useState(0);
  const [metrics, setMetrics] = useState<MetricSnapshot[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [transition, setTransition] = useState<TransitionInfo | null>(null);
  const [idleSecs, setIdleSecs] = useState(0);
  const [awayBoutSecs, setAwayBoutSecs] = useState(0);
  const [continuousComputerSecs, setContinuousComputerSecs] = useState(0);
  const [todaySummary, setTodaySummary] = useState<TodaySummaryDto | null>(null);
  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const portFetched = useRef(false);

  const calibrate = useCallback(
    async (position: "sitting" | "standing") => {
      try {
        const dto = await invoke<SessionStateDto>("get_session_state");
        const heightMm = Math.round(dto.desk_height_cm * 10);
        if (position === "sitting") {
          await invoke("calibrate", { sitting_mm: heightMm });
        } else {
          await invoke("calibrate", { standing_mm: heightMm });
        }
      } catch (err) {
        console.error(`Calibration failed for ${position}:`, err);
        throw err;
      }
    },
    [],
  );

  const setSitLimit = useCallback(async (mins: number) => {
    try {
      await invoke("set_session_limit", { minutes: mins });
    } catch (err) {
      console.error("Failed to set sitting limit:", err);
      throw err;
    }
  }, []);

  const setStandLimit = useCallback(async (mins: number) => {
    try {
      await invoke("set_stand_limit", { minutes: mins });
    } catch (err) {
      console.error("Failed to set standing limit:", err);
      throw err;
    }
  }, []);

  useEffect(() => {
    invoke("start_auto_connect").catch(console.error);

    const fetchState = async () => {
      try {
        const dashboard = await invoke<DashboardState>("get_dashboard_state");
        const dto = dashboard.session;
        setMetrics(dashboard.metrics);
        setState(dto.state);
        setDeskHeightCm(dto.desk_height_cm);
        setSittingSeconds(dto.current_session_secs);
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
        // If we get a non-Away state with height data, sensor is connected.
        // Fixes race where device-connected fires before listener mounts.
        if (dto.state !== "Away" && dto.desk_height_cm > 0) {
          setConnected(true);
          if (!portFetched.current) {
            portFetched.current = true;
            invoke<string | null>("get_connected_port")
              .then((p) => { if (p) setPort(p); })
              .catch(() => { portFetched.current = false; });
          }
        }
      } catch (err) {
        console.debug("get_dashboard_state not ready:", err);
      }
    };

    const fetchSummary = () => {
      invoke<TodaySummaryDto>("get_today_summary")
        .then(setTodaySummary)
        .catch((err) => console.debug("get_today_summary not ready:", err));
    };

    fetchState();
    fetchSummary();

    const stateInterval = setInterval(fetchState, 1000);
    const summaryInterval = setInterval(fetchSummary, 10000);

    let cleanupFns: Array<() => void> = [];

    async function subscribe() {
      const unConnected = await listen<DeviceConnectedPayload>(
        "desk:device-connected",
        ({ payload }) => {
          setConnected(true);
          setPort(payload.port);
          setError(null);
        },
      );

      const unLost = await listen<null>("desk:device-lost", () => {
        setConnected(false);
        setPort(null);
      });

      const unState = await listen<StateChangedPayload>(
        "desk:state-changed",
        ({ payload }) => {
          // If we receive state changes, sensor must be connected
          // (fixes race condition where device-connected fires before listener mounts)
          setConnected(true);
          setState(payload.state);
          setDeskHeightCm(payload.desk_height_cm);
          setSittingSeconds(payload.current_session_secs);
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
          // Refresh summary on state change
          fetchSummary();
        },
      );

      const unError = await listen<SensorErrorPayload>(
        "desk:sensor-error",
        ({ payload }) => setError(payload.message),
      );

      const unDbError = await listen<{ message: string }>(
        "desk:db-error",
        ({ payload }) => setError(payload.message),
      );

      const unAlert = await listen<null>("desk:session-alert", () => {
        setError("Time to take a break!");
      });

      cleanupFns = [unConnected, unLost, unState, unError, unDbError, unAlert];
    }

    subscribe().catch(console.error);

    return () => {
      clearInterval(stateInterval);
      clearInterval(summaryInterval);
      if (transitionTimer.current) clearTimeout(transitionTimer.current);
      cleanupFns.forEach((fn) => fn());
    };
  }, []);

  // Derived: previous session from today's sessions list
  const previousSession = useMemo((): PreviousSession | null => {
    if (!todaySummary || todaySummary.sessions.length < 2) return null;
    const prev = todaySummary.sessions[todaySummary.sessions.length - 2];
    const isBreak = prev.state === "Standing" || prev.state === "Walking" || prev.state === "Away";
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
    connected, port, state, deskHeightCm,
    sittingSeconds, standingSeconds, breakSeconds,
    sessionLimitSecs, standLimitSecs, positionChanges,
    limitUsedSecs,
    limitRemaining: sessionLimitSecs - limitUsedSecs,
    limitRatio: sessionLimitSecs > 0 ? limitUsedSecs / sessionLimitSecs : 0,
    breakResetThreshold: BREAK_RESET_THRESHOLD_SECS,
    breakResetProgress,
    previousSession,
    todaySessions, todayChanges, todaySittingSecs, todayStandingSecs,
    dailyScore, metrics, error,
    idleSecs, awayBoutSecs, continuousComputerSecs,
    transition, calibrate, setSitLimit, setStandLimit,
  };
}
