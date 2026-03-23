/**
 * useWidgetData.ts — assembles WidgetProps from useDesk + useTimer hooks.
 *
 * Centralizes the mapping from raw desk state to the presentation
 * contract that widgets consume, keeping App.tsx lean.
 */
import { useDesk } from "@/hooks/useDesk";
import { useTimer } from "@/hooks/useTimer";
import type { WidgetProps } from "@/types";

/**
 * Builds a complete `WidgetProps` object from live desk data.
 *
 * @param onOpenSettings - callback to open the settings panel
 * @returns WidgetProps ready to pass to any widget component
 */
export function useWidgetData(onOpenSettings: () => void): WidgetProps {
  const desk = useDesk();

  const liveSitting = useTimer(
    desk.sittingSeconds,
    desk.state === "Sitting",
  );
  const liveBreak = useTimer(
    desk.breakSeconds,
    desk.state !== "Sitting",
  );

  return {
    connected: desk.connected,
    port: desk.port,
    state: desk.state,
    deskHeightCm: desk.deskHeightCm,
    currentSessionSecs: liveSitting,
    limitSecs: desk.sessionLimitSecs,
    limitRemaining: desk.limitRemaining,
    limitRatio: desk.limitRatio,
    breakSecs: liveBreak,
    breakResetThreshold: desk.breakResetThreshold,
    breakResetProgress: desk.breakResetProgress,
    previousSession: desk.previousSession,
    todaySessions: desk.todaySessions,
    todayChanges: desk.todayChanges,
    todayStandingSecs: desk.todayStandingSecs,
    todaySittingSecs: desk.todaySittingSecs,
    todayScore: desk.dailyScore,
    error: desk.error,
    onOpenSettings,
    elapsed: desk.state === "Sitting" ? liveSitting : liveBreak,
    total: desk.state === "Sitting" ? desk.sessionLimitSecs : desk.breakResetThreshold,
    colorScheme: desk.state === "Sitting"
      ? "sitting" as const
      : desk.state === "Away"
        ? "gray" as const
        : "standing" as const,
  };
}
