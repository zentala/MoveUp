/**
 * useDeskTypes.ts — type definitions for the useDesk hook return value.
 *
 * Extracted from useDesk.ts to keep both files under the 250-line limit.
 */
import type { TransportCapabilities } from "@/remote/transports";
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
  /**
   * Seconds since the last position change, uncredited.
   * **Debug tab only** — never a timer, bar, colour band or notification.
   */
  secsSinceLastBreak: number;
  standingSeconds: number;
  breakSeconds: number;
  sessionLimitSecs: number;
  /** Standing target in seconds (from config, e.g. 900 = 15 min). */
  standLimitSecs: number;
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
  /** Current system idle time in seconds. */
  idleSecs: number;
  /** Current continuous Away bout duration in seconds. */
  awayBoutSecs: number;
  /** Continuous seconds at the computer. Resets after 5+ min Away. */
  continuousComputerSecs: number;
  /** Transition info (auto-clears after 30s). */
  transition: TransitionInfo | null;
  /** True when the WebSocket to the backend is connected (remote mode only). */
  wsConnected?: boolean;
  calibrate: (position: "sitting" | "standing") => Promise<void>;
  setSitLimit: (mins: number) => Promise<void>;
  setStandLimit: (mins: number) => Promise<void>;
}

/**
 * The two facts only the transport knows (E022-T08).
 *
 * Deliberately NOT on `UseDeskResult`: `deskReducer.DeskView` is defined by
 * omission from that interface, so anything added there would have to be
 * produced by the pure reducer — which cannot know whether a socket is up.
 */
export interface DeskTransportFacts {
  /**
   * True when the desk itself is reachable.
   *
   * On Tauri and on the LAN this is the same fact as being connected. Through
   * the relay it is not: the phone can hold a healthy socket to a room whose
   * desk is asleep, and the display must say so rather than spin
   * "Reconnecting…" at a connection that is fine.
   */
  deskOnline: boolean;
  /** What this transport is allowed to do — gates the remote controls. */
  capabilities: TransportCapabilities;
}

/** What every desk hook returns: the reducer view plus its transport facts. */
export interface UseDeskConnection extends UseDeskResult, DeskTransportFacts {}
