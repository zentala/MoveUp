/**
 * useWidgetData.ts — assembles WidgetProps from useDesk data.
 *
 * Centralizes the mapping from raw desk state to the presentation
 * contract that widgets consume, keeping App.tsx lean.
 *
 * Uses useDeskAuto() which auto-selects Tauri IPC (desktop) or
 * WebSocket (remote display) based on runtime environment.
 */
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDeskAuto } from "@/hooks/useDeskAuto";
import type { LiveObservation, WidgetProps } from "@/types";
import type { QueryStatus } from "./useDeskTypes";

/** Extended result including connection metadata for the overlay. */
export interface WidgetDataResult {
  widgetProps: WidgetProps;
  /** Whether the WebSocket is connected (remote mode only; undefined in Tauri). */
  wsConnected?: boolean;
  /** Whether the physical sensor is connected to the PC. */
  sensorConnected: boolean;
  /** Whether the desk itself is reachable (false = relay up, desk asleep). */
  deskOnline: boolean;
  /** Producer sample quality, independent of the physical connection. */
  observationQuality: "known" | null;
  liveObservation: LiveObservation | null;
  dashboardRefresh: QueryStatus;
  historyRefresh: QueryStatus;
}

/**
 * Builds a complete `WidgetProps` object from live desk data.
 *
 * Also exposes raw connection flags for ConnectionOverlay.
 *
 * @param onOpenSettings - callback to open the settings panel
 * @returns WidgetDataResult with widgetProps and connection metadata
 */
export function useWidgetData(onOpenSettings: () => void): WidgetDataResult {
  const desk = useDeskAuto();
  const [showActivityStatus, setShowActivityStatus] = useState<boolean | null>(null);

  useEffect(() => {
    let disposed = false;
    let saveRevision = 0;
    const refresh = () => {
      const revisionAtStart = saveRevision;
      void invoke<{ show_activity_status: boolean }>("get_settings")
        .then((settings) => {
          if (!disposed && revisionAtStart === saveRevision) {
            setShowActivityStatus(settings.show_activity_status);
          }
        })
        .catch(() => {
          if (!disposed && revisionAtStart === saveRevision) setShowActivityStatus(null);
        });
    };
    const saved = (event: Event) => {
      const detail = (event as CustomEvent<{ show_activity_status: boolean }>).detail;
      if (typeof detail?.show_activity_status === "boolean") {
        saveRevision += 1;
        setShowActivityStatus(detail.show_activity_status);
      } else {
        refresh();
      }
    };
    refresh();
    window.addEventListener("moveup:settings-saved", saved);
    return () => {
      disposed = true;
      window.removeEventListener("moveup:settings-saved", saved);
    };
  }, []);

  return {
    widgetProps: {
      connected: desk.connected,
      liveObservation: desk.liveObservation,
      lastBreakCreditFacts: desk.lastBreakCreditFacts,
      dashboardRefresh: desk.dashboardRefresh,
      showActivityStatus: showActivityStatus ?? false,
      port: desk.port,
      state: desk.state,
      deskHeightCm: desk.deskHeightCm,
      limitUsedSecs: desk.limitUsedSecs,
      limitSecs: desk.sessionLimitSecs,
      standLimitSecs: desk.standLimitSecs,
      limitRemaining: desk.limitRemaining,
      limitRatio: desk.limitRatio,
      breakSecs: desk.breakSeconds,
      breakResetThreshold: desk.breakResetThreshold,
      breakResetProgress: desk.breakResetProgress,
      previousSession: desk.previousSession,
      todaySessions: desk.todaySessions,
      todayChanges: desk.todayChanges,
      todayStandingSecs: desk.todayStandingSecs,
      todaySittingSecs: desk.todaySittingSecs,
      todayScore: desk.dailyScore,
      idleSecs: desk.idleSecs,
      continuousComputerSecs: desk.continuousComputerSecs,
      metrics: desk.metrics,
      error: desk.error,
      onOpenSettings,
    },
    wsConnected: desk.wsConnected,
    sensorConnected: desk.connected,
    deskOnline: desk.deskOnline,
    observationQuality: desk.observationQuality,
    liveObservation: desk.liveObservation,
    dashboardRefresh: desk.dashboardRefresh,
    historyRefresh: desk.historyRefresh,
  };
}
