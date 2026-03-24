/**
 * scenario-helpers.ts — Helper functions for building scenario data.
 */
import type { SessionEntry, MetricSnapshot } from "@/types";

/** No-op callback for onOpenSettings in scenarios. */
export const noop = () => {};

/** Create a metric snapshot with defaults. */
export function mkMetric(
  id: string,
  label: string,
  display: string,
  level: "green" | "yellow" | "red",
): MetricSnapshot {
  return {
    id,
    label,
    result: { value: 0, display, level, is_personal_best: false },
  };
}

/** Build a session list from simple entries (sequential, starting at 8:00). */
export function mkSessions(
  entries: Array<{ state: string; mins: number }>,
): SessionEntry[] {
  let t = new Date();
  t.setHours(8, 0, 0, 0);
  return entries.map((e) => {
    const start = t.toISOString();
    t = new Date(t.getTime() + e.mins * 60_000);
    return {
      start,
      end: t.toISOString(),
      state: e.state as SessionEntry["state"],
      duration_secs: e.mins * 60,
    };
  });
}
