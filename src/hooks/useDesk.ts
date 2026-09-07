/**
 * useDesk.ts — Tauri IPC transport for the shared desk reducer.
 *
 * Owns polling (`get_dashboard_state` every 1s, `get_today_summary` every
 * 10s) and the `desk:*` event subscriptions; all state derivation lives in
 * `deskReducer.ts`.
 */
import { useEffect, useReducer, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  SessionStateDto,
  DashboardState,
  DeviceConnectedPayload,
  StateChangedPayload,
  SensorErrorPayload,
  TodaySummaryDto,
} from "@/types";
import type { TransportCapabilities } from "@/remote/transports";
import type { UseDeskConnection } from "./useDeskTypes";
import { deskReducer, initialDeskState, selectDeskView } from "./deskReducer";

export type { TransitionInfo, UseDeskResult, UseDeskConnection } from "./useDeskTypes";

/**
 * Capabilities of the local IPC path.
 *
 * Constant, not a transport lookup: inside Tauri there is no wire to
 * negotiate, so the same shape the remote transports report is filled in with
 * the one answer that is always true here.
 */
const LOCAL_CAPABILITIES: TransportCapabilities = { control: true };

/**
 * Subscribes to all Tauri `desk:*` events and exposes current desk state.
 * Auto-fetches initial state on mount, then:
 * - Polls `get_dashboard_state()` every 1 second
 * - Polls `get_today_summary()` every 10 seconds
 * - Starts auto-connect on mount
 * - Exposes calibration and settings commands
 */
export function useDesk(): UseDeskConnection {
  const [state, dispatch] = useReducer(deskReducer, initialDeskState);
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
        dispatch({ type: "snapshot", session: dto, metrics: dashboard.metrics });
        if (dto.state !== "Away" && dto.desk_height_cm > 0 && !portFetched.current) {
          portFetched.current = true;
          invoke<string | null>("get_connected_port")
            .then((p) => { if (p) dispatch({ type: "port", port: p }); })
            .catch(() => { portFetched.current = false; });
        }
      } catch (err) {
        console.debug("get_dashboard_state not ready:", err);
      }
    };

    const fetchSummary = () => {
      invoke<TodaySummaryDto>("get_today_summary")
        .then((today) => dispatch({ type: "today-summary", today }))
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
        ({ payload }) => dispatch({ type: "device-connected", port: payload.port }),
      );

      const unLost = await listen<null>("desk:device-lost", () =>
        dispatch({ type: "device-lost" }),
      );

      const unState = await listen<StateChangedPayload>(
        "desk:state-changed",
        ({ payload }) => {
          dispatch({ type: "state-changed", payload });
          if (transitionTimer.current) clearTimeout(transitionTimer.current);
          transitionTimer.current = setTimeout(
            () => dispatch({ type: "clear-transition" }),
            30_000,
          );
          // Refresh summary on state change
          fetchSummary();
        },
      );

      const unError = await listen<SensorErrorPayload>(
        "desk:sensor-error",
        ({ payload }) => dispatch({ type: "error", message: payload.message }),
      );

      const unDbError = await listen<{ message: string }>(
        "desk:db-error",
        ({ payload }) => dispatch({ type: "error", message: payload.message }),
      );

      const unAlert = await listen<null>("desk:session-alert", () =>
        dispatch({ type: "error", message: "Time to take a break!" }),
      );

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

  const view = useMemo(() => selectDeskView(state), [state]);

  return {
    ...view,
    // The desktop app talks to its own backend over IPC: the desk is by
    // definition present, and every command is already available locally.
    deskOnline: true,
    capabilities: LOCAL_CAPABILITIES,
    calibrate,
    setSitLimit,
    setStandLimit,
  };
}
