/** Query exact current-zone calendar boundaries without inferring coverage. */
import { useMemo } from "react";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

const DAY_MS = 86_400_000;
const UTC_SUFFIX = /(?:Z|[+-]\d{2}:\d{2})$/i;

function expectedDate(from: string, index: number): string | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(from)) return null;
  const startMs = Date.parse(`${from}T00:00:00Z`);
  if (!Number.isFinite(startMs) || new Date(startMs).toISOString().slice(0, 10) !== from) return null;
  return new Date(startMs + index * DAY_MS).toISOString().slice(0, 10);
}

function isLocalDayBounds(value: unknown): value is LocalDayBounds {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const row = value as Partial<LocalDayBounds>;
  return typeof row.date_local === "string"
    && typeof row.start_utc === "string"
    && typeof row.end_utc === "string"
    && typeof row.zone_basis === "string"
    && typeof row.historical_source_zone === "string";
}

function responseMatchesRange(value: unknown, from: string, to: string): boolean {
  if (!Array.isArray(value) || !value.every(isLocalDayBounds)) return false;
  const rows: LocalDayBounds[] = value;
  const startMs = Date.parse(`${from}T00:00:00Z`);
  const endMs = Date.parse(`${to}T00:00:00Z`);
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs)
    || new Date(startMs).toISOString().slice(0, 10) !== from
    || new Date(endMs).toISOString().slice(0, 10) !== to) return false;
  if (endMs < startMs) return rows.length === 0;
  const count = Math.floor((endMs - startMs) / DAY_MS) + 1;
  if (rows.length !== count) return false;

  let previousEnd: number | null = null;
  return rows.every((row, index) => {
    const start = Date.parse(row.start_utc);
    const end = Date.parse(row.end_utc);
    const valid = row.date_local === expectedDate(from, index)
      && UTC_SUFFIX.test(row.start_utc)
      && UTC_SUFFIX.test(row.end_utc)
      && Number.isFinite(start)
      && Number.isFinite(end)
      && end > start
      && (previousEnd === null || previousEnd === start)
      && row.zone_basis === "current-query-local"
      && row.historical_source_zone === "unknown";
    previousEnd = end;
    return valid;
  });
}

export function useLocalDayBoundsRange(
  from: string,
  to: string,
  skip = false,
): RangeQueryState<LocalDayBounds> {
  const raw = useRangeQuery<LocalDayBounds>({
    command: "get_local_day_bounds_range",
    from,
    to,
    skip,
  });
  return useMemo(() => {
    if (raw.status !== "ready" || responseMatchesRange(raw.data, from, to)) return raw;
    return {
      status: "error",
      data: null,
      error: "Invalid local-day bounds response",
      refetch: raw.refetch,
      lastRefreshed: raw.lastRefreshed,
    };
  }, [raw, from, to]);
}
