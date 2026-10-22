/** Query exact current-zone calendar boundaries without inferring coverage. */
import { useMemo } from "react";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
import type { LocalHourTick } from "@/generated/LocalHourTick";
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

function validHourTicks(value: unknown, dateLocal: string, startUtc: string, endUtc: string): value is LocalHourTick[] {
  if (!Array.isArray(value) || value.length === 0) return false;
  const start = Date.parse(startUtc);
  const end = Date.parse(endUtc);
  let previous = start - 1;
  return value.every((item) => {
    if (typeof item !== "object" || item === null || Array.isArray(item)) return false;
    const tick = item as Partial<LocalHourTick>;
    if (typeof tick.at_utc !== "string" || !UTC_SUFFIX.test(tick.at_utc)
      || typeof tick.wall_time !== "string" || !/^([01]\d|2[0-3]):00$/.test(tick.wall_time)
      || !Number.isInteger(tick.utc_offset_secs) || Math.abs(tick.utc_offset_secs ?? 0) > 86_400) return false;
    const at = Date.parse(tick.at_utc);
    if (!Number.isFinite(at) || at < start || at >= end || at <= previous) return false;
    const local = new Date(at + tick.utc_offset_secs! * 1000).toISOString();
    if (local.slice(0, 10) !== dateLocal || local.slice(11, 16) !== tick.wall_time
      || local.slice(17, 23) !== "00.000") return false;
    previous = at;
    return true;
  });
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
    if (raw.status !== "ready") return raw;
    if (responseMatchesRange(raw.data, from, to)) {
      return { ...raw, data: raw.data.map((row) => ({
        ...row,
        hour_ticks: validHourTicks(row.hour_ticks, row.date_local, row.start_utc, row.end_utc) ? row.hour_ticks : null,
      })) };
    }
    return {
      status: "error",
      data: null,
      error: "Invalid local-day bounds response",
      refetch: raw.refetch,
      lastRefreshed: raw.lastRefreshed,
    };
  }, [raw, from, to]);
}
