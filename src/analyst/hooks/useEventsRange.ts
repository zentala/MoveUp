/**
 * useEventsRange.ts — Live event-log range query.
 *
 * Calls `get_events_range` and coerces the Rust `EventRow` to the fixture
 * `EventRow` shape (`type`/`message` rather than `kind`/`detail`).
 */
import { useMemo } from "react";
import type { EventRow } from "@/test/analyst-fixtures";
import { useRangeQuery, type RangeQueryState } from "./useRangeQuery";

interface WireEventRow {
  ts: string;
  kind: string;
  detail: string;
}

const KNOWN: EventRow["type"][] = [
  "STATE",
  "CREDIT",
  "NOTIF",
  "DEVICE",
  "ALERT",
  "RESET",
  "START",
  "AUTOSTART",
];

function coerce(row: WireEventRow): EventRow {
  const type = KNOWN.includes(row.kind as EventRow["type"])
    ? (row.kind as EventRow["type"])
    : "STATE";
  return { ts: row.ts, type, message: row.detail };
}

export function useEventsRange(
  from: string,
  to: string,
  skip = false,
): RangeQueryState<EventRow> {
  const raw = useRangeQuery<WireEventRow>({
    command: "get_events_range",
    from,
    to,
    skip,
  });
  return useMemo<RangeQueryState<EventRow>>(() => {
    if (raw.status !== "ready") return raw;
    return { status: "ready", data: raw.data.map(coerce) };
  }, [raw]);
}
