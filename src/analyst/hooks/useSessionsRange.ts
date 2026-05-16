/**
 * useSessionsRange.ts — Live sessions range query.
 *
 * Calls `get_sessions_range` and coerces the Rust `SessionRow` shape (sparse)
 * to the chart-facing `SessionRow` shape. Fields not stored in the DB are
 * synthesised: `breakCredit` defaults to `"full"` (sessions are persisted
 * only when they completed naturally), other counters default to 0.
 */
import { useMemo } from "react";
import type { SessionRow } from "@/test/analyst-fixtures";
import type { DeskState } from "@/types";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

interface WireSessionRow {
  start: string;
  end: string | null;
  state: string;
  duration_secs: number | null;
}

function toDeskState(s: string): DeskState {
  const lower = s.toLowerCase();
  if (lower === "standing") return "Standing";
  if (lower === "walking") return "Walking";
  if (lower === "away") return "Away";
  return "Sitting";
}

function coerce(row: WireSessionRow, idx: number): SessionRow {
  const duration = row.duration_secs ?? 0;
  const state = toDeskState(row.state);
  const isSit = state === "Sitting";
  return {
    id: idx,
    startedAt: row.start,
    endedAt: row.end ?? row.start,
    state,
    durationSecs: duration,
    sittingSecs: isSit ? duration : 0,
    standingSecs: isSit ? 0 : duration,
    positionChanges: 0,
    breakCredit: duration >= 120 ? "full" : duration >= 60 ? "partial" : "none",
    dateLocal: row.start.slice(0, 10),
  };
}

export function useSessionsRange(
  from: string,
  to: string,
  skip = false,
): RangeQueryState<SessionRow> {
  const raw = useRangeQuery<WireSessionRow>({
    command: "get_sessions_range",
    from,
    to,
    skip,
  });
  return useMemo<RangeQueryState<SessionRow>>(() => {
    if (raw.status !== "ready") return raw;
    return { status: "ready", data: raw.data.map(coerce) };
  }, [raw]);
}
