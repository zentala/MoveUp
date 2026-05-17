/**
 * useSnapshotsRange.ts — Live snapshots range query.
 *
 * Calls `get_snapshots_range` and coerces the Rust snake_case `SnapshotRow`
 * shape into the camelCase shape consumed by the chart components.
 */
import { useMemo } from "react";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import type { DeskState } from "@/types";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

interface WireSnapshotRow {
  ts: string;
  state: string;
  sitting_seconds: number;
  standing_seconds: number;
  break_seconds: number;
  desk_height_cm: number;
  idle_secs: number;
  continuous_computer_secs: number;
  position_changes: number;
  daily_score: number | null;
}

function toDeskState(s: string): DeskState {
  const lower = s.toLowerCase();
  if (lower === "standing") return "Standing";
  if (lower === "walking") return "Walking";
  if (lower === "away") return "Away";
  return "Sitting";
}

function coerce(row: WireSnapshotRow): SnapshotRow {
  return {
    ts: row.ts,
    state: toDeskState(row.state),
    deskHeightCm: row.desk_height_cm,
    sittingSecs: row.sitting_seconds,
    standingSecs: row.standing_seconds,
    breakSecs: row.break_seconds,
    idleSecs: row.idle_secs,
    score: row.daily_score ?? 0,
  };
}

export function useSnapshotsRange(
  from: string,
  to: string,
  skip = false,
): RangeQueryState<SnapshotRow> {
  const raw = useRangeQuery<WireSnapshotRow>({
    command: "get_snapshots_range",
    from,
    to,
    skip,
  });
  return useMemo<RangeQueryState<SnapshotRow>>(() => {
    if (raw.status !== "ready") return raw;
    return { status: "ready", data: raw.data.map(coerce), refetch: raw.refetch };
  }, [raw]);
}
