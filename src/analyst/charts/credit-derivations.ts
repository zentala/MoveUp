import type { SessionRow } from "@/test/analyst-fixtures";

export type CreditOutcome = "none" | "partial" | "full" | "unknown";
export type RecordedCredit = Exclude<CreditOutcome, "unknown">;

export interface CreditOutcomeCounts {
  none: number;
  partial: number;
  full: number;
  unknown: number;
}

export interface CreditOutcomeSummary {
  counts: CreditOutcomeCounts;
  known: number;
  eligible: number;
  excluded: number;
}

const RECORDED_OUTCOMES = new Set<string>(["none", "partial", "full"]);
const BREAK_STATES = new Set(["Standing", "Walking", "Away"]);

function endpoints(row: SessionRow): { start: number; end: number } | null {
  const start = row.startedAt == null ? Number.NaN : Date.parse(row.startedAt);
  const end = row.endedAt == null ? Number.NaN : Date.parse(row.endedAt);
  return Number.isFinite(start) && Number.isFinite(end) && end > start
    ? { start, end }
    : null;
}

/**
 * Counts stored break-credit outcomes, never durations or inferred credit.
 * The producer attaches a known classification to the break span closed when
 * returning to Sitting. An unclassified row is eligible only when it is a
 * source-qualified individual break span with a matching recorded Sitting
 * return; other null/unknown spans do not establish a completed outcome.
 */
export function creditOutcomes(rows: readonly SessionRow[]): CreditOutcomeSummary {
  const sittingReturnStarts = new Set(
    rows
      .filter((row) => row.historyKind === "individual" && row.state === "Sitting" && endpoints(row))
      .map((row) => endpoints(row)!.start),
  );
  const counts: CreditOutcomeCounts = { none: 0, partial: 0, full: 0, unknown: 0 };
  let excluded = 0;

  for (const row of rows) {
    const span = endpoints(row);
    if (!span || !BREAK_STATES.has(row.state ?? "")) {
      excluded++;
      continue;
    }

    const credit = row.breakCredit as string | null;
    if (credit != null && RECORDED_OUTCOMES.has(credit)) {
      // A stored classification is direct outcome evidence even when an older
      // row predates history_kind; that field qualifies posture intervals only.
      counts[credit as RecordedCredit]++;
      continue;
    }

    if (row.historyKind === "individual" && sittingReturnStarts.has(span.end)) {
      counts.unknown++;
      continue;
    }

    excluded++;
  }

  const known = counts.none + counts.partial + counts.full;
  return { counts, known, eligible: known + counts.unknown, excluded };
}
