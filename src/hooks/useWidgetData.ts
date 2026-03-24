/**
 * useWidgetData.ts — assembles WidgetProps from useDesk data.
 *
 * Centralizes the mapping from raw desk state to the presentation
 * contract that widgets consume, keeping App.tsx lean.
 *
 * The backend already sends live values (computed via DateTime arithmetic)
 * every 1s, so no client-side timer interpolation is needed.
 */
import { useDesk } from "@/hooks/useDesk";
import type { WidgetProps } from "@/types";

/**
 * Builds a complete `WidgetProps` object from live desk data.
 *
 * @param onOpenSettings - callback to open the settings panel
 * @returns WidgetProps ready to pass to any widget component
 */
export function useWidgetData(onOpenSettings: () => void): WidgetProps {
  const desk = useDesk();

  return {
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
  };
}
