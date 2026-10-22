import type { SnapshotRow } from "@/test/analyst-fixtures";

export interface ScoreObservation {
  atMs: number | null;
  score: number | null;
  localDate: string | null;
  timezoneOffsetSecs: number | null;
}

export interface ScoreObservationSummary {
  observations: ScoreObservation[];
  recorded: number;
  unknown: number;
  invalidTimestamps: number;
}

const LOCAL_DATE = /^\d{4}-\d{2}-\d{2}$/;

function sourceDate(
  value: string | null | undefined,
  atMs: number | null,
  offsetSecs: number | null,
): string | null {
  if (!value || !LOCAL_DATE.test(value)) return null;
  const parsed = Date.parse(`${value}T00:00:00Z`);
  if (!Number.isFinite(parsed) || new Date(parsed).toISOString().slice(0, 10) !== value) return null;
  if (atMs === null || offsetSecs === null) return null;

  const localDate = new Date(atMs + offsetSecs * 1000);
  return Number.isFinite(localDate.getTime()) && localDate.toISOString().slice(0, 10) === value
    ? value
    : null;
}

function sourceOffset(value: number | null | undefined): number | null {
  return typeof value === "number" && Number.isInteger(value) && Math.abs(value) < 86_400
    ? value
    : null;
}

function hasRecordedScoreTime(sample: ScoreObservation): boolean {
  return sample.atMs !== null && sample.localDate !== null && sample.timezoneOffsetSecs !== null;
}

/** Keeps one result per source row; missing values never become zero or a bucket maximum. */
export function scoreObservations(rows: readonly SnapshotRow[]): ScoreObservationSummary {
  let invalidTimestamps = 0;
  const observations = rows.map((row): ScoreObservation => {
    const parsed = Date.parse(row.ts);
    const atMs = Number.isFinite(parsed) ? parsed : null;
    if (atMs === null) invalidTimestamps++;
    const timezoneOffsetSecs = sourceOffset(row.timezoneOffsetSecs);
    return {
      atMs,
      score: typeof row.score === "number" && Number.isFinite(row.score) ? row.score : null,
      // Keep the known score and offset, but do not claim a source-local date
      // that disagrees with the captured instant plus its recorded offset.
      localDate: sourceDate(row.localDate, atMs, timezoneOffsetSecs),
      timezoneOffsetSecs,
    };
  });

  // Stable sorting keeps every duplicate-instant observation and its source order.
  observations.sort((a, b) => (a.atMs ?? Number.POSITIVE_INFINITY) - (b.atMs ?? Number.POSITIVE_INFINITY));
  const recorded = observations.filter((sample) => sample.score !== null && hasRecordedScoreTime(sample)).length;
  return {
    observations,
    recorded,
    unknown: observations.length - recorded,
    invalidTimestamps,
  };
}

/** Formats wall-clock time only from the offset stored with that sample. */
export function sourceLocalTimeLabel(sample: ScoreObservation): string {
  if (!hasRecordedScoreTime(sample)) return "Source-local time unknown";
  const local = new Date(sample.atMs! + sample.timezoneOffsetSecs! * 1000);
  if (!Number.isFinite(local.getTime())) return "Source-local time unknown";

  const clock = local.toISOString().slice(11, 19);
  const offset = sample.timezoneOffsetSecs!;
  const sign = offset < 0 ? "-" : "+";
  const absolute = Math.abs(offset);
  const hours = String(Math.floor(absolute / 3600)).padStart(2, "0");
  const minutes = String(Math.floor((absolute % 3600) / 60)).padStart(2, "0");
  const seconds = absolute % 60;
  const secondsSuffix = seconds === 0 ? "" : `:${String(seconds).padStart(2, "0")}`;
  return `${sample.localDate} ${clock} UTC${sign}${hours}:${minutes}${secondsSuffix}`;
}

export function formatScore(score: number): string {
  if (score === 0) return "0";
  const value = String(Math.abs(score));
  return score < 0 ? `−${value}` : `+${value}`;
}
