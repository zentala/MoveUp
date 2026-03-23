/**
 * useDeskTypes.ts — type definitions for the useDesk hook return value.
 *
 * Extracted from useDesk.ts to keep both files under the 250-line limit.
 */
import type { DeskState, MetricSnapshot, PreviousSession, SessionEntry } from "@/types";

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
  state: DeskState;
  deskHeightCm: number;
  /** Current sitting session seconds (resets after break credit). */
  sittingSeconds: number;
  standingSeconds: number;
  breakSeconds: number;
  sessionLimitSecs: number;
  positionChanges: number;
  limitUsedSecs: number;
  /** limitSecs - limitUsedSecs. Can go negative (overtime). */
  limitRemaining: number;
  /** limitUsedSecs / limitSecs. 0 when limitSecs=0. Can exceed 1.0. */
  limitRatio: number;
  /** Break reset threshold in seconds (from config). */
  breakResetThreshold: number;
  /** 0.0-1.0 progress toward a full break reset. */
  breakResetProgress: number;
  /** Previous completed session, or null on first session. */
  previousSession: PreviousSession | null;
  /** All sessions tracked today. */
  todaySessions: SessionEntry[];
  /** Number of position changes today. */
  todayChanges: number;
  /** Total sitting seconds today. */
  todaySittingSecs: number;
  /** Total standing seconds today. */
  todayStandingSecs: number;
  dailyScore: number;
  /** KPI metric snapshots from MetricEngine. */
  metrics: MetricSnapshot[];
  error: string | null;
  /** Transition info (auto-clears after 30s). */
  transition: TransitionInfo | null;
  calibrate: (position: "sitting" | "standing") => Promise<void>;
  setSitLimit: (mins: number) => Promise<void>;
  setStandLimit: (mins: number) => Promise<void>;
}
