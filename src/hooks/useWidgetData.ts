/**
 * useWidgetData.ts — assembles WidgetProps from useDesk data.
 *
 * Centralizes the mapping from raw desk state to the presentation
 * contract that widgets consume, keeping App.tsx lean.
 *
 * Uses useDeskAuto() which auto-selects Tauri IPC (desktop) or
 * WebSocket (remote display) based on runtime environment.
 */
import { useDeskAuto } from "@/hooks/useDeskAuto";
import type { WidgetProps } from "@/types";

/** Extended result including connection metadata for the overlay. */
export interface WidgetDataResult {
  widgetProps: WidgetProps;
  /** Whether the WebSocket is connected (remote mode only; undefined in Tauri). */
  wsConnected?: boolean;
  /** Whether the physical sensor is connected to the PC. */
  sensorConnected: boolean;
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

  return {
    widgetProps: {
      connected: desk.connected,
      port: desk.port,
      state: desk.state,
      deskHeightCm: desk.deskHeightCm,
      currentSessionSecs: desk.sittingSeconds,
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
      metrics: desk.metrics,
      error: desk.error,
      onOpenSettings,
    },
    wsConnected: desk.wsConnected,
    sensorConnected: desk.connected,
  };
}
