import { useEffect, useMemo, useReducer, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DashboardState } from "@/types";
import { deskReducer, initialDeskState, selectDeskView } from "./deskReducer";

/** Reuse the existing local dashboard cadence; this hook never auto-connects. */
const SNAPSHOT_REFRESH_MS = 1_000;

export function useLiveObservation() {
  const [state, dispatch] = useReducer(deskReducer, initialDeskState);
  const latestRequest = useRef(0);

  useEffect(() => {
    let active = true;
    let inFlight = false;
    let refreshPending = false;

    const refresh = async () => {
      if (!active) return;
      if (inFlight) {
        refreshPending = true;
        return;
      }
      inFlight = true;
      refreshPending = false;
      const request = ++latestRequest.current;
      dispatch({ type: "query-started", query: "dashboard" });
      try {
        const dashboard = await invoke<DashboardState>("get_dashboard_state");
        if (!active || request !== latestRequest.current) return;
        dispatch({ type: "snapshot", session: dashboard.session, metrics: dashboard.metrics });
        dispatch({ type: "query-succeeded", query: "dashboard", receivedAt: new Date().toISOString() });
      } catch (error) {
        if (!active || request !== latestRequest.current) return;
        dispatch({
          type: "query-failed",
          query: "dashboard",
          message: error instanceof Error ? error.message : String(error),
        });
      } finally {
        inFlight = false;
        if (active && refreshPending) void refresh();
      }
    };

    void refresh();
    const interval = setInterval(() => void refresh(), SNAPSHOT_REFRESH_MS);
    return () => {
      active = false;
      latestRequest.current += 1;
      clearInterval(interval);
    };
  }, []);

  const view = useMemo(() => selectDeskView(state), [state]);
  return {
    liveObservation: view.liveObservation,
    observationQuality: view.observationQuality,
    refresh: view.dashboardRefresh,
  };
}
