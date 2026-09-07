/**
 * deskReducer.ts — the desk-state machine shared by every transport.
 *
 * `useDesk` (Tauri IPC) and `useRemoteDesk` (WebSocket + REST) own only
 * their transport; both translate raw backend payloads into the actions
 * below and read the same derived view through `selectDeskView`.
 */
import type {
  DeskState,
  SessionStateDto,
  MetricSnapshot,
  StateChangedPayload,
  TodaySummaryDto,
  PreviousSession,
  SessionEntry,
} from "@/types";
import type { HealthView } from "@/generated/HealthView";
import type { TransitionInfo, UseDeskResult } from "./useDeskTypes";

/** Default break reset threshold in seconds (10 min for full reset). */
export const BREAK_RESET_THRESHOLD_SECS = 600;

/** A break counts as effective from this duration on (seconds). */
const EFFECTIVE_BREAK_SECS = 300;

/** A day with nothing recorded yet — every counter genuinely zero. */
const EMPTY_TODAY_SUMMARY: TodaySummaryDto = {
  sitting_secs: 0,
  standing_secs: 0,
  yesterday_sitting_secs: 0,
  yesterday_standing_secs: 0,
  position_changes: 0,
  sessions: [],
};

const DESK_STATES: readonly string[] = ["Sitting", "Standing", "Walking", "Away"];

/**
 * Narrows a session row's `state` column, which the backend types as a plain
 * string. Mirrors Rust's `DeskState::from_db_str`: an unrecognised value is
 * unknown, never silently mapped onto a real state.
 */
function toDeskState(raw: string): DeskState | null {
  return DESK_STATES.includes(raw) ? (raw as DeskState) : null;
}

/** Everything a transport-agnostic desk view needs to render. */
export interface DeskReducerState {
  connected: boolean;
  port: string | null;
  state: DeskState;
  deskHeightCm: number;
  secsSinceLastBreak: number;
  standingSeconds: number;
  breakSeconds: number;
  sessionLimitSecs: number;
  standLimitSecs: number;
  positionChanges: number;
  limitUsedSecs: number;
  dailyScore: number;
  metrics: MetricSnapshot[];
  error: string | null;
  idleSecs: number;
  awayBoutSecs: number;
  continuousComputerSecs: number;
  transition: TransitionInfo | null;
  todaySummary: TodaySummaryDto | null;
  /**
   * Health view carried by the remote snapshot (E021-T03/T04).
   *
   * null means "no snapshot has arrived yet" — distinct from a snapshot
   * that arrived saying `configured: false`. The desktop transport leaves
   * it null and reads health over IPC instead.
   */
  health: HealthView | null;
}

export const initialDeskState: DeskReducerState = {
  connected: false,
  port: null,
  state: "Away",
  deskHeightCm: 0,
  secsSinceLastBreak: 0,
  standingSeconds: 0,
  breakSeconds: 0,
  sessionLimitSecs: 0,
  standLimitSecs: 900,
  positionChanges: 0,
  limitUsedSecs: 0,
  dailyScore: 0,
  metrics: [],
  error: null,
  idleSecs: 0,
  awayBoutSecs: 0,
  continuousComputerSecs: 0,
  transition: null,
  todaySummary: null,
  health: null,
};

/** Every way a transport may move the desk state forward. */
export type DeskAction =
  | {
      type: "snapshot";
      session: SessionStateDto;
      metrics: MetricSnapshot[];
      today?: TodaySummaryDto;
      health?: HealthView;
    }
  | { type: "today-summary"; today: TodaySummaryDto }
  | { type: "state-changed"; payload: StateChangedPayload }
  | { type: "device-connected"; port: string | null }
  | { type: "device-lost" }
  | { type: "port"; port: string | null }
  | { type: "error"; message: string }
  | { type: "daily-reset" }
  | { type: "clear-transition" };

/**
 * Pure desk-state transition. No timers, no I/O — a transport adapter owns
 * those and calls back in with the action the payload represents.
 */
export function deskReducer(state: DeskReducerState, action: DeskAction): DeskReducerState {
  switch (action.type) {
    case "snapshot": {
      const dto = action.session;
      // A non-Away state with real height data proves the sensor is alive,
      // even when the device-connected event landed before we subscribed.
      const sensorAlive = dto.state !== "Away" && dto.desk_height_cm > 0;
      return {
        ...state,
        state: dto.state,
        deskHeightCm: dto.desk_height_cm,
        secsSinceLastBreak: dto.secs_since_last_break,
        standingSeconds: dto.standing_seconds,
        breakSeconds: dto.break_seconds,
        sessionLimitSecs: dto.session_limit_secs,
        standLimitSecs: dto.stand_limit_secs > 0 ? dto.stand_limit_secs : state.standLimitSecs,
        positionChanges: dto.position_changes,
        limitUsedSecs: dto.limit_used_secs,
        dailyScore: dto.daily_score,
        idleSecs: dto.idle_secs ?? 0,
        awayBoutSecs: dto.away_bout_secs ?? 0,
        continuousComputerSecs: dto.continuous_computer_secs ?? 0,
        metrics: action.metrics,
        todaySummary: action.today ?? state.todaySummary,
        health: action.health ?? state.health,
        connected: sensorAlive ? true : state.connected,
      };
    }

    case "today-summary":
      return { ...state, todaySummary: action.today };

    case "state-changed": {
      const p = action.payload;
      return {
        ...state,
        connected: true,
        state: p.state,
        deskHeightCm: p.desk_height_cm,
        limitUsedSecs: p.limit_used_secs,
        standingSeconds: p.standing_seconds,
        breakSeconds: p.break_seconds,
        positionChanges: p.position_changes,
        transition: {
          lastBreakSecs: p.last_break_secs,
          lastSittingSecs: p.last_sitting_secs,
          breakCredit: p.break_credit,
          transitionTo: p.state,
        },
      };
    }

    case "device-connected":
      return { ...state, connected: true, port: action.port, error: null };

    case "device-lost":
      return { ...state, connected: false, port: null };

    case "port":
      return { ...state, port: action.port };

    case "error":
      return { ...state, error: action.message };

    case "daily-reset":
      return {
        ...state,
        secsSinceLastBreak: 0,
        standingSeconds: 0,
        breakSeconds: 0,
        positionChanges: 0,
        limitUsedSecs: 0,
        dailyScore: 0,
        todaySummary: EMPTY_TODAY_SUMMARY,
      };

    case "clear-transition":
      return { ...state, transition: null };
  }
}

/** Previous completed session, or null before a second session exists. */
export function selectPreviousSession(state: DeskReducerState): PreviousSession | null {
  const sessions = state.todaySummary?.sessions;
  if (!sessions || sessions.length < 2) return null;
  const prev = sessions[sessions.length - 2];
  const prevState = toDeskState(prev.state);
  if (prevState === null) return null;
  const isBreak =
    prevState === "Standing" || prevState === "Walking" || prevState === "Away";
  // A row written before the duration column is unknown, so it cannot have
  // earned credit — 0 keeps it out of `wasEffective` without claiming a length.
  const durationSecs = prev.duration_secs ?? 0;
  return {
    state: prevState,
    durationSecs,
    wasEffective: isBreak && durationSecs >= EFFECTIVE_BREAK_SECS,
  };
}

/** Sessions tracked today — an empty list when nothing was recorded yet. */
export function selectTodaySessions(state: DeskReducerState): SessionEntry[] {
  return state.todaySummary?.sessions ?? [];
}

/** 0.0-1.0 progress toward a full break reset. */
export function selectBreakResetProgress(state: DeskReducerState): number {
  return Math.min(state.breakSeconds / BREAK_RESET_THRESHOLD_SECS, 1.0);
}

/** The part of `UseDeskResult` that does not depend on the transport. */
export type DeskView = Omit<UseDeskResult, "calibrate" | "setSitLimit" | "setStandLimit" | "wsConnected">;

/** Projects reducer state onto the public hook contract. */
export function selectDeskView(state: DeskReducerState): DeskView {
  return {
    connected: state.connected,
    port: state.port,
    state: state.state,
    deskHeightCm: state.deskHeightCm,
    secsSinceLastBreak: state.secsSinceLastBreak,
    standingSeconds: state.standingSeconds,
    breakSeconds: state.breakSeconds,
    sessionLimitSecs: state.sessionLimitSecs,
    standLimitSecs: state.standLimitSecs,
    positionChanges: state.positionChanges,
    limitUsedSecs: state.limitUsedSecs,
    limitRemaining: state.sessionLimitSecs - state.limitUsedSecs,
    limitRatio: state.sessionLimitSecs > 0 ? state.limitUsedSecs / state.sessionLimitSecs : 0,
    breakResetThreshold: BREAK_RESET_THRESHOLD_SECS,
    breakResetProgress: selectBreakResetProgress(state),
    previousSession: selectPreviousSession(state),
    todaySessions: selectTodaySessions(state),
    todayChanges: state.positionChanges,
    todaySittingSecs: state.todaySummary?.sitting_secs ?? 0,
    todayStandingSecs: state.todaySummary?.standing_secs ?? 0,
    dailyScore: state.dailyScore,
    metrics: state.metrics,
    error: state.error,
    idleSecs: state.idleSecs,
    awayBoutSecs: state.awayBoutSecs,
    continuousComputerSecs: state.continuousComputerSecs,
    transition: state.transition,
  };
}
