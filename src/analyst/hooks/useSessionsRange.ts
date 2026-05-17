/**
 * useSessionsRange.ts — Live sessions range query.
 *
 * Thin wrapper over `useRangeQuery`. Wire-shape coercion (including the
 * `breakCredit` heuristic) lives in `explorer-derivations.ts` so the hook
 * stays I/O-only and the derivation rule is unit-testable in isolation.
 */
import { useMemo } from "react";
import type { SessionRow } from "@/test/analyst-fixtures";
import {
  coerceSessionRow,
  type WireSessionRow,
} from "../explorer-derivations";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

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
    return { status: "ready", data: raw.data.map(coerceSessionRow), refetch: raw.refetch };
  }, [raw]);
}
