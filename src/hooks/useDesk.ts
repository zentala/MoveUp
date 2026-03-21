/**
 * useDesk.ts — custom hook that connects to the Tauri desk backend.
 *
 * Fetches initial session state on mount and subscribes to all `desk:*`
 * Tauri events, cleaning up listeners on unmount.
 */
import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeskState,
  SessionStateDto,
  DeviceConnectedPayload,
  StateChangedPayload,
  SensorErrorPayload,
} from "@/types";

/** Transition info shown for 30s after a state change. */
export interface TransitionInfo {
  lastBreakSecs: number;
  lastSittingSecs: number;
  breakCredit: "none" | "partial" | "full";
  transitionTo: DeskState;
}

/** Shape returned by the useDesk hook. */
export interface UseDeskResult {
  connected: boolean;
  port: string | null;
  state: DeskState | null;
  deskHeightCm: number;
  /** Current sitting session seconds (resets after break credit). */
  sittingSeconds: number;
  standingSeconds: number;
  breakSeconds: number;
  sessionLimitSecs: number;
  positionChanges: number;
  limitUsedSecs: number;
  limitRemaining: number;
  limitRatio: number;
  dailyScore: number;
  error: string | null;
  /** Transition info (auto-clears after 30s). */
  transition: TransitionInfo | null;
  calibrate: (position: "sitting" | "standing") => Promise<void>;
  setSitLimit: (mins: number) => Promise<void>;
  setStandLimit: (mins: number) => Promise<void>;
}

/**
 * Subscribes to all Tauri `desk:*` events and exposes current desk state.
 * Auto-fetches initial state via `get_session_state()` on mount, then:
 * - Polls `get_session_state()` every 1 second (live updates when event system lags)
 * - Polls `get_today_summary()` every 10 seconds
 * - Starts auto-connect on mount
 * - Exposes calibration and settings commands
 */
export function useDesk(): UseDeskResult {
  const [connected, setConnected] = useState(false);
  const [port, setPort] = useState<string | null>(null);
  const [state, setState] = useState<DeskState | null>(null);
  const [deskHeightCm, setDeskHeightCm] = useState(0);
  const [sittingSeconds, setSittingSeconds] = useState(0);
  const [standingSeconds, setStandingSeconds] = useState(0);
  const [breakSeconds, setBreakSeconds] = useState(0);
  const [sessionLimitSecs, setSessionLimitSecs] = useState(0);
  const [positionChanges, setPositionChanges] = useState(0);
  const [limitUsedSecs, setLimitUsedSecs] = useState(0);
  const [dailyScore, setDailyScore] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [transition, setTransition] = useState<TransitionInfo | null>(null);
  const transitionTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Memoized commands
  const calibrate = useCallback(
    async (position: "sitting" | "standing") => {
      try {
        // Get current height
        const dto = await invoke<SessionStateDto>("get_session_state");
        const heightMm = Math.round(dto.desk_height_cm * 10);

        // Save calibration
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
    // Start auto-connect on mount
    invoke("start_auto_connect").catch(console.error);

    // Fetch current session state on mount
    const fetchState = async () => {
      try {
        const dto = await invoke<SessionStateDto>("get_session_state");
        setState(dto.state);
        setDeskHeightCm(dto.desk_height_cm);
        setSittingSeconds(dto.current_session_secs);
        setStandingSeconds(dto.standing_seconds);
        setBreakSeconds(dto.break_seconds);
        setSessionLimitSecs(dto.session_limit_secs);
        setPositionChanges(dto.position_changes);
        setLimitUsedSecs(dto.limit_used_secs);
        setDailyScore(dto.daily_score);
      } catch (err) {
        // Backend may not be connected yet; expected on cold start
        console.debug("get_session_state not ready:", err);
      }
    };

    fetchState();

    // Poll session state every 1 second
    const stateInterval = setInterval(fetchState, 1000);

    // Poll today summary every 10 seconds (used by TodayStats)
    const summaryInterval = setInterval(() => {
      invoke("get_today_summary").catch((err) => {
        console.debug("get_today_summary not ready:", err);
      });
    }, 10000);

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
          setState(payload.state);
          setDeskHeightCm(payload.desk_height_cm);
          setSittingSeconds(payload.current_session_secs);
          setStandingSeconds(payload.standing_seconds);
          setBreakSeconds(payload.break_seconds);
          setPositionChanges(payload.position_changes);
          // Show transition banner for 30s
          if (transitionTimer.current) clearTimeout(transitionTimer.current);
          setTransition({
            lastBreakSecs: payload.last_break_secs,
            lastSittingSecs: payload.last_sitting_secs,
            breakCredit: payload.break_credit,
            transitionTo: payload.state,
          });
          transitionTimer.current = setTimeout(() => setTransition(null), 30_000);
        },
      );

      const unError = await listen<SensorErrorPayload>(
        "desk:sensor-error",
        ({ payload }) => {
          setError(payload.message);
        },
      );

      const unDbError = await listen<{ message: string }>(
        "desk:db-error",
        ({ payload }) => {
          setError(payload.message);
        },
      );

      // desk:session-alert has no payload — just show a generic reminder
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

  return {
    connected,
    port,
    state,
    deskHeightCm,
    sittingSeconds,
    standingSeconds,
    breakSeconds,
    sessionLimitSecs,
    positionChanges,
    limitUsedSecs,
    limitRemaining: sessionLimitSecs - limitUsedSecs,
    limitRatio: sessionLimitSecs > 0 ? limitUsedSecs / sessionLimitSecs : 0,
    dailyScore,
    error,
    transition,
    calibrate,
    setSitLimit,
    setStandLimit,
  };
}
