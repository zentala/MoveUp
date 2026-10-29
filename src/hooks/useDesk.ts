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
import { reportQueryResult } from "../uiDiagnostics";

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
    let active = true;
    let stateRequest = 0;
    let summaryRequest = 0;
    let stateInFlight = false;
    let summaryInFlight = false;
    let stateRefreshPending = false;
    let summaryRefreshPending = false;
    invoke("start_auto_connect").catch(console.error);

    const fetchState = async () => {
      if (!active) return;
      if (stateInFlight) {
        stateRefreshPending = true;
        return;
      }
      stateInFlight = true;
      stateRefreshPending = false;
      const request = ++stateRequest;
      dispatch({ type: "query-started", query: "dashboard" });
      try {
        const dashboard = await invoke<DashboardState>("get_dashboard_state");
        if (!active || request !== stateRequest) return;
        reportQueryResult("dashboard", "succeeded");
        const dto = dashboard.session;
        dispatch({ type: "snapshot", session: dto, metrics: dashboard.metrics });
        dispatch({ type: "query-succeeded", query: "dashboard", receivedAt: new Date().toISOString() });
        if (dto.state !== "Away" && dto.desk_height_cm > 0 && !portFetched.current) {
          portFetched.current = true;
          invoke<string | null>("get_connected_port")
            .then((p) => { if (active && request === stateRequest && p) dispatch({ type: "port", port: p }); })
            .catch(() => { portFetched.current = false; });
        }
      } catch (err) {
        if (!active || request !== stateRequest) return;
        reportQueryResult("dashboard", "failed");
        dispatch({
          type: "query-failed",
          query: "dashboard",
          message: err instanceof Error ? err.message : String(err),
        });
        console.debug("get_dashboard_state not ready:", err);
      } finally {
        stateInFlight = false;
        if (active && stateRefreshPending) void fetchState();
      }
    };

    const fetchSummary = async () => {
      if (!active) return;
      if (summaryInFlight) {
        summaryRefreshPending = true;
        return;
      }
      summaryInFlight = true;
      summaryRefreshPending = false;
      const request = ++summaryRequest;
      dispatch({ type: "query-started", query: "history" });
      try {
        const today = await invoke<TodaySummaryDto>("get_today_summary");
        if (!active || request !== summaryRequest) return;
        reportQueryResult("today_summary", "succeeded");
        dispatch({ type: "today-summary", today });
        dispatch({ type: "query-succeeded", query: "history", receivedAt: new Date().toISOString() });
      } catch (err) {
        if (!active || request !== summaryRequest) return;
        reportQueryResult("today_summary", "failed");
        dispatch({
          type: "query-failed",
          query: "history",
          message: err instanceof Error ? err.message : String(err),
        });
        console.debug("get_today_summary not ready:", err);
      } finally {
        summaryInFlight = false;
        if (active && summaryRefreshPending) void fetchSummary();
      }
    };

    fetchState();
    fetchSummary();

    const stateInterval = setInterval(fetchState, 1000);
    const summaryInterval = setInterval(fetchSummary, 10000);

    const cleanupFns: Array<() => void> = [];
    const register = <T,>(event: string, callback: (event: { payload: T }) => void) => {
      void listen<T>(event, callback).then((unsubscribe) => {
        if (active) cleanupFns.push(unsubscribe);
        else unsubscribe();
      }).catch(console.error);
    };

    register<DeviceConnectedPayload>("desk:device-connected", ({ payload }) => {
      dispatch({ type: "device-connected", port: payload.port });
      if (stateInFlight) stateRequest += 1;
      fetchState();
      fetchSummary();
    });

    register<null>("desk:device-lost", () => {
      // A read begun before the loss event cannot restore connection state.
      if (stateInFlight) stateRequest += 1;
      stateRefreshPending = false;
      dispatch({ type: "device-lost" });
    });

    register<StateChangedPayload>("desk:state-changed", ({ payload }) => {
      // A dashboard read started before this authoritative event must not undo it.
      if (stateInFlight) {
        stateRequest += 1;
        stateRefreshPending = true;
      }
      // A history read begun before this authoritative transition cannot
      // publish stale sessions or a misleading new receipt time.
      if (summaryInFlight) {
        summaryRequest += 1;
        summaryRefreshPending = true;
      }
      dispatch({ type: "state-changed", payload });
      if (transitionTimer.current) clearTimeout(transitionTimer.current);
      transitionTimer.current = setTimeout(
        () => dispatch({ type: "clear-transition" }),
        30_000,
      );
      fetchSummary();
    });

    register<SensorErrorPayload>("desk:sensor-error", ({ payload }) =>
      dispatch({ type: "error", message: payload.message }),
    );

    register<{ message: string }>("desk:db-error", ({ payload }) =>
      dispatch({ type: "error", message: payload.message }),
    );

    register<null>("desk:session-alert", () =>
      dispatch({ type: "error", message: "Time to take a break!" }),
    );

    return () => {
      active = false;
      stateRequest += 1;
      summaryRequest += 1;
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
