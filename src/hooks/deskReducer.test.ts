/**
 * deskReducer.test.ts — pure reducer tests. No Tauri, no WebSocket.
 */
import { describe, it, expect } from "vitest";
import {
  deskReducer,
  initialDeskState,
  selectDeskView,
  selectPreviousSession,
  selectTodaySessions,
  selectBreakResetProgress,
  BREAK_RESET_THRESHOLD_SECS,
} from "./deskReducer";
import type { DeskReducerState } from "./deskReducer";
import type { SessionStateDto, StateChangedPayload, TodaySummaryDto } from "@/types";

function makeSession(overrides: Partial<SessionStateDto> = {}): SessionStateDto {
  return {
    state: "Sitting",
    sitting_seconds: 120,
    standing_seconds: 60,
    break_seconds: 0,
    session_limit_secs: 2400,
    stand_limit_secs: 900,
    desk_height_cm: 72.5,
    position_changes: 2,
    limit_used_secs: 120,
    daily_score: 5,
    standing_session_secs: 0,
    secs_since_last_break: 120,
    continuous_computer_secs: 120,
    longest_computer_session_secs: 120,
    sitting_seconds_total: 120,
    idle_secs: 0,
    away_bout_secs: 0,
    max_continuous_computer_secs: 7200,
    ...overrides,
  };
}

function makeStateChanged(overrides: Partial<StateChangedPayload> = {}): StateChangedPayload {
  return {
    state: "Standing",
    standing_seconds: 60,
    break_seconds: 10,
    desk_height_cm: 110,
    position_changes: 3,
    last_break_secs: 0,
    last_sitting_secs: 120,
    break_credit: "none",
    limit_used_secs: 900,
    ...overrides,
  };
}

const summary: TodaySummaryDto = {
  sitting_secs: 300,
  standing_secs: 100,
  yesterday_sitting_secs: 0,
  yesterday_standing_secs: 0,
  position_changes: 2,
  sessions: [
    { start: "09:00", end: "09:20", state: "Standing", duration_secs: 600, break_credit: "full" },
    { start: "09:20", end: null, state: "Sitting", duration_secs: 120, break_credit: null },
  ],
};

describe("deskReducer — happy path", () => {
  it("snapshot copies every session field onto state", () => {
    const s = deskReducer(initialDeskState, {
      type: "snapshot",
      session: makeSession(),
      metrics: [],
    });

    expect(s.state).toBe("Sitting");
    expect(s.deskHeightCm).toBe(72.5);
    expect(s.limitUsedSecs).toBe(120);
    expect(s.secsSinceLastBreak).toBe(120);
    expect(s.standLimitSecs).toBe(900);
    expect(s.dailyScore).toBe(5);
    // Non-Away + real height means the sensor is alive.
    expect(s.connected).toBe(true);
  });

  it("state-changed updates the credited counter and opens a transition", () => {
    const s = deskReducer(initialDeskState, {
      type: "state-changed",
      payload: makeStateChanged(),
    });

    expect(s.state).toBe("Standing");
    expect(s.limitUsedSecs).toBe(900);
    expect(s.positionChanges).toBe(3);
    expect(s.connected).toBe(true);
    expect(s.transition).toEqual({
      lastBreakSecs: 0,
      lastSittingSecs: 120,
      breakCredit: "none",
      transitionTo: "Standing",
    });
  });

  it("clear-transition drops the transition without touching counters", () => {
    const opened = deskReducer(initialDeskState, {
      type: "state-changed",
      payload: makeStateChanged(),
    });
    const cleared = deskReducer(opened, { type: "clear-transition" });

    expect(cleared.transition).toBeNull();
    expect(cleared.limitUsedSecs).toBe(900);
  });

  it("device connect/lost and explicit port updates track the serial port", () => {
    const connected = deskReducer(
      { ...initialDeskState, error: "boom" },
      { type: "device-connected", port: "COM3" },
    );
    expect(connected).toMatchObject({ connected: true, port: "COM3", error: null });

    const repointed = deskReducer(connected, { type: "port", port: "COM7" });
    expect(repointed.port).toBe("COM7");

    const lost = deskReducer(repointed, { type: "device-lost" });
    expect(lost).toMatchObject({ connected: false, port: null });
  });

  it("daily-reset zeroes the daily counters and empties today's summary", () => {
    const withData = deskReducer(initialDeskState, {
      type: "snapshot",
      session: makeSession(),
      metrics: [],
      today: summary,
    });
    const reset = deskReducer(withData, { type: "daily-reset" });

    expect(reset.secsSinceLastBreak).toBe(0);
    expect(reset.standingSeconds).toBe(0);
    expect(reset.limitUsedSecs).toBe(0);
    expect(reset.dailyScore).toBe(0);
    expect(reset.todaySummary).toEqual({
      sitting_secs: 0,
      standing_secs: 0,
      yesterday_sitting_secs: 0,
      yesterday_standing_secs: 0,
      position_changes: 0,
      sessions: [],
    });
  });

  it("keeps the previous stand limit when the backend reports 0", () => {
    const s = deskReducer(initialDeskState, {
      type: "snapshot",
      session: makeSession({ stand_limit_secs: 0 }),
      metrics: [],
    });
    expect(s.standLimitSecs).toBe(900);
  });

  it("an Away snapshot does not claim the sensor is connected", () => {
    const s = deskReducer(initialDeskState, {
      type: "snapshot",
      session: makeSession({ state: "Away", desk_height_cm: 0 }),
      metrics: [],
    });
    expect(s.connected).toBe(false);
  });
});

describe("deskReducer selectors", () => {
  it("nil — no summary yet means no previous session", () => {
    expect(selectPreviousSession(initialDeskState)).toBeNull();
    expect(selectTodaySessions(initialDeskState)).toEqual([]);
  });

  it("nil — a row with an unrecorded duration is not an effective break", () => {
    const s: DeskReducerState = {
      ...initialDeskState,
      todaySummary: {
        ...summary,
        sessions: [
          { start: "09:00", end: "09:20", state: "Standing", duration_secs: null, break_credit: null },
          summary.sessions[1],
        ],
      },
    };
    const prev = selectPreviousSession(s);
    // Unknown length cannot earn credit, and must not read as a long break.
    expect(prev).toEqual({ state: "Standing", durationSecs: 0, wasEffective: false });
  });

  it("error — an unrecognised state column yields no previous session", () => {
    const s: DeskReducerState = {
      ...initialDeskState,
      todaySummary: {
        ...summary,
        sessions: [
          { start: "09:00", end: "09:20", state: "Levitating", duration_secs: 600, break_credit: null },
          summary.sessions[1],
        ],
      },
    };
    // Mirrors Rust's DeskState::from_db_str — unknown, never coerced.
    expect(selectPreviousSession(s)).toBeNull();
  });

  it("empty — a summary with zero sessions yields an empty list, not null", () => {
    const s: DeskReducerState = {
      ...initialDeskState,
      todaySummary: { ...summary, sitting_secs: 0, standing_secs: 0, sessions: [] },
    };
    expect(selectTodaySessions(s)).toEqual([]);
    expect(selectPreviousSession(s)).toBeNull();
  });

  it("a single session is not yet a PREVIOUS session", () => {
    const s: DeskReducerState = {
      ...initialDeskState,
      todaySummary: { ...summary, sessions: [summary.sessions[0]] },
    };
    expect(selectPreviousSession(s)).toBeNull();
  });

  it("marks a long break as effective", () => {
    const s: DeskReducerState = { ...initialDeskState, todaySummary: summary };
    expect(selectPreviousSession(s)).toEqual({
      state: "Standing",
      durationSecs: 600,
      wasEffective: true,
    });
  });

  it("a short break is not effective", () => {
    const s: DeskReducerState = {
      ...initialDeskState,
      todaySummary: {
        ...summary,
        sessions: [
          { start: "09:00", end: "09:01", state: "Standing", duration_secs: 60, break_credit: "none" },
          summary.sessions[1],
        ],
      },
    };
    expect(selectPreviousSession(s)?.wasEffective).toBe(false);
  });

  it("break reset progress is clamped to 1.0", () => {
    expect(selectBreakResetProgress({ ...initialDeskState, breakSeconds: 300 })).toBeCloseTo(0.5);
    expect(
      selectBreakResetProgress({
        ...initialDeskState,
        breakSeconds: BREAK_RESET_THRESHOLD_SECS * 3,
      }),
    ).toBe(1.0);
  });

  it("selectDeskView derives limitRemaining and limitRatio", () => {
    const s = deskReducer(initialDeskState, {
      type: "snapshot",
      session: makeSession(),
      metrics: [],
      today: summary,
    });
    const view = selectDeskView(s);

    expect(view.limitRemaining).toBe(2400 - 120);
    expect(view.limitRatio).toBeCloseTo(120 / 2400);
    expect(view.todaySittingSecs).toBe(300);
    expect(view.todayStandingSecs).toBe(100);
    expect(view.todayChanges).toBe(2);
    expect(view.breakResetThreshold).toBe(BREAK_RESET_THRESHOLD_SECS);
  });

  it("limitRatio is 0 when no limit is configured", () => {
    const view = selectDeskView({ ...initialDeskState, limitUsedSecs: 50 });
    expect(view.limitRatio).toBe(0);
    expect(view.limitRemaining).toBe(-50);
  });

  it("error — a reported sensor failure reaches the view", () => {
    const s = deskReducer(initialDeskState, { type: "error", message: "sensor gone" });
    expect(selectDeskView(s).error).toBe("sensor gone");
  });
});
