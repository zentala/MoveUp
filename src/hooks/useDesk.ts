/**
 * useDesk.ts — custom hook that connects to the Tauri desk backend.
 *
 * Fetches initial session state on mount and subscribes to all `desk:*`
 * Tauri events, cleaning up listeners on unmount.
 */
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  DeskState,
  SessionStateDto,
  DeviceConnectedPayload,
  StateChangedPayload,
  SensorErrorPayload,
} from "@/types";

/** Shape returned by the useDesk hook. */
export interface UseDeskResult {
  /** Whether a device is actively connected. */
  connected: boolean;
  /** Serial port name, e.g. "COM3", or null when not connected. */
  port: string | null;
  /** Current ergonomic state. */
  state: DeskState | null;
  /** Current measured desk height in centimeters. */
  deskHeightCm: number;
  /** Seconds spent in current sitting session. */
  sittingSeconds: number;
  /** Seconds spent standing today. */
  standingSeconds: number;
  /** Seconds spent in current break. */
  breakSeconds: number;
  /** Configured session limit in seconds. */
  sessionLimitSecs: number;
  /** Number of position changes (Sitting↔Standing transitions) today. */
  positionChanges: number;
  /** Last sensor or connection error message, if any. */
  error: string | null;
}

/**
 * Subscribes to all Tauri `desk:*` events and exposes current desk state.
 * Auto-fetches initial state via `get_session_state()` on mount.
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
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Fetch current session state on mount
    invoke<SessionStateDto>("get_session_state")
      .then((dto) => {
        setState(dto.state);
        setDeskHeightCm(dto.desk_height_cm);
        setSittingSeconds(dto.sitting_seconds);
        setStandingSeconds(dto.standing_seconds);
        setBreakSeconds(dto.break_seconds);
        setSessionLimitSecs(dto.session_limit_secs);
        setPositionChanges(dto.position_changes);
      })
      .catch(() => {
        // Backend may not be connected yet; that is expected on cold start
      });

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
          setSittingSeconds(payload.sitting_seconds);
          setStandingSeconds(payload.standing_seconds);
          setBreakSeconds(payload.break_seconds);
          setPositionChanges(payload.position_changes);
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
    error,
  };
}
