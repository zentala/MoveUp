/**
 * useSnapshotsRange.ts — Live snapshots range query.
 *
 * Calls `get_snapshots_range` and coerces the Rust snake_case `SnapshotRow`
 * shape into the camelCase shape consumed by the chart components.
 */
import { useMemo } from "react";
import type { SnapshotRow as SnapshotWireRow } from "@/generated/SnapshotRow";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { coerceSnapshotRow } from "../explorer-derivations";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

export function useSnapshotsRange(
  from: string,
  to: string,
  skip = false,
): RangeQueryState<SnapshotRow> {
  const raw = useRangeQuery<SnapshotWireRow>({
    command: "get_snapshots_range",
    from,
    to,
    skip,
  });
  return useMemo<RangeQueryState<SnapshotRow>>(() => {
    if (raw.status !== "ready") return raw;
    return {
      status: "ready",
      data: raw.data.map(coerceSnapshotRow),
      refetch: raw.refetch,
      lastRefreshed: raw.lastRefreshed,
    };
  }, [raw]);
}
