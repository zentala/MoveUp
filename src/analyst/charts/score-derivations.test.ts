import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { scoreObservations, sourceLocalTimeLabel } from "./score-derivations";

function sample(ts: string, score: number | null, overrides: Partial<SnapshotRow> = {}): SnapshotRow {
  return {
    ts,
    state: null,
    deskHeightCm: 0,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score,
    localDate: "2026-10-22",
    timezoneOffsetSecs: 0,
    ...overrides,
  };
}

describe("score observations", () => {
  it("preserves signed values, measured zero, nulls, duplicates, and real timestamp order", () => {
    const rows = [
      sample("2026-10-22T09:02:00Z", 150),
      sample("2026-10-22T09:00:00Z", -90),
      sample("2026-10-22T09:01:00Z", 0),
      sample("2026-10-22T09:01:00Z", null),
    ];

    const result = scoreObservations(rows);

    expect(result.observations.map(({ score }) => score)).toEqual([-90, 0, null, 150]);
    expect(result.recorded).toBe(3);
    expect(result.unknown).toBe(1);
    expect(result.invalidTimestamps).toBe(0);
  });

  it("keeps samples with absent local date or offset unknown rather than deriving host time", () => {
    const result = scoreObservations([
      sample("2026-10-22T22:30:00Z", 1, {
        localDate: "2026-10-23",
        timezoneOffsetSecs: 7200,
      }),
      sample("2026-10-22T23:30:00Z", 2, { localDate: null }),
      sample("2026-10-23T00:30:00Z", 3, { timezoneOffsetSecs: null }),
      sample("bad timestamp", 4),
    ]);

    expect(result.observations).toHaveLength(4);
    expect(result.recorded).toBe(1);
    expect(result.unknown).toBe(3);
    expect(result.invalidTimestamps).toBe(1);
    expect(sourceLocalTimeLabel(result.observations[0])).toBe(
      "2026-10-23 00:30:00 UTC+02:00",
    );
    expect(sourceLocalTimeLabel(result.observations[1])).toBe("Source-local time unknown");
  });

  it("preserves the score but rejects a local date that conflicts with the captured instant and offset", () => {
    const result = scoreObservations([
      sample("2026-10-23T01:00:00Z", 8, {
        localDate: "2026-10-22",
        timezoneOffsetSecs: 7200,
      }),
    ]);

    expect(result.observations[0]).toMatchObject({
      score: 8,
      atMs: Date.parse("2026-10-23T01:00:00Z"),
      localDate: null,
      timezoneOffsetSecs: 7200,
    });
    expect(result.recorded).toBe(0);
    expect(result.unknown).toBe(1);
    expect(sourceLocalTimeLabel(result.observations[0])).toBe("Source-local time unknown");
  });

  it("formats offset seconds from the recorded offset without host timezone APIs", () => {
    const result = scoreObservations([
      sample("2026-10-22T18:18:44Z", -5, {
        localDate: "2026-10-23",
        timezoneOffsetSecs: 20_476,
      }),
    ]);
    expect(sourceLocalTimeLabel(result.observations[0])).toBe(
      "2026-10-23 00:00:00 UTC+05:41:16",
    );
  });
});
