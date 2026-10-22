import { describe, expect, it } from "vitest";
import type { SessionRow } from "@/test/analyst-fixtures";
import { creditOutcomes } from "./credit-derivations";

function row(overrides: Partial<SessionRow> = {}): SessionRow {
  return {
    id: 1,
    startedAt: "2026-10-22T09:00:00+02:00",
    endedAt: "2026-10-22T09:02:00+02:00",
    state: "Away",
    durationSecs: 120,
    sittingSecs: 0,
    standingSecs: 0,
    positionChanges: null,
    breakCredit: null,
    historyKind: "individual",
    dateLocal: "2026-10-22",
    ...overrides,
  };
}

function equalDurationRow(id: number, breakCredit: SessionRow["breakCredit"]): SessionRow {
  const minute = String(id * 5).padStart(2, "0");
  const endMinute = String(id * 5 + 2).padStart(2, "0");
  return row({
    id,
    startedAt: `2026-10-22T09:${minute}:00+02:00`,
    endedAt: `2026-10-22T09:${endMinute}:00+02:00`,
    breakCredit,
  });
}

describe("recorded credit outcomes", () => {
  it("counts different stored outcomes for equal-duration break rows and keeps unknown coverage", () => {
    const rows = (["none", "partial", "full"] as const).map((breakCredit, index) =>
      equalDurationRow(index, breakCredit));
    rows.push(row({
      id: 4,
      state: "Standing",
      startedAt: "2026-10-22T09:30:00+02:00",
      endedAt: "2026-10-22T09:32:00+02:00",
    }));
    rows.push(row({
      id: 5,
      state: "Sitting",
      startedAt: "2026-10-22T09:32:00+02:00",
      endedAt: "2026-10-22T09:34:00+02:00",
    }));
    rows.push(row({
      id: 6,
      state: "Walking",
      startedAt: "2026-10-22T10:00:00+02:00",
      endedAt: "2026-10-22T10:02:00+02:00",
    }));

    expect(creditOutcomes(rows)).toEqual({
      counts: { none: 1, partial: 1, full: 1, unknown: 1 },
      known: 3,
      eligible: 4,
      excluded: 2,
    });
  });

  it("counts stored legacy classifications but needs a proven return for unknown credit", () => {
    const rows = [
      row({ historyKind: null, breakCredit: "full" }),
      row({ historyKind: "summary", breakCredit: "partial" }),
      row({ state: "Sitting", breakCredit: "full" }),
      row({ state: null, breakCredit: "full" }),
      row({ endedAt: null, breakCredit: "full" }),
      row({ endedAt: "bad timestamp", breakCredit: "full" }),
      row({
        startedAt: "2026-10-22T09:02:00+02:00",
        endedAt: "2026-10-22T09:04:00+02:00",
      }),
      row({
        startedAt: "2026-10-22T09:04:00+02:00",
        endedAt: "2026-10-22T09:10:00+02:00",
      }),
      row({
        state: "Sitting",
        startedAt: "2026-10-22T09:10:00+02:00",
        endedAt: "2026-10-22T09:12:00+02:00",
      }),
    ];

    expect(creditOutcomes(rows)).toEqual({
      counts: { none: 0, partial: 1, full: 1, unknown: 1 },
      known: 2,
      eligible: 3,
      excluded: 6,
    });
  });

  it("keeps measured None distinct from no eligible records", () => {
    const measuredNone = creditOutcomes([row({ breakCredit: "none" })]);
    const noRecords = creditOutcomes([]);

    expect(measuredNone.counts.none).toBe(1);
    expect(measuredNone.eligible).toBe(1);
    expect(noRecords).toEqual({
      counts: { none: 0, partial: 0, full: 0, unknown: 0 },
      known: 0,
      eligible: 0,
      excluded: 0,
    });
  });

  it("treats unknown stored categories as unknown only for a proven completed return", () => {
    const unknown = row({
      breakCredit: "other" as SessionRow["breakCredit"],
      endedAt: "2026-10-22T09:02:00+02:00",
    });
    const sitting = row({
      state: "Sitting",
      startedAt: "2026-10-22T09:02:00+02:00",
      endedAt: "2026-10-22T09:04:00+02:00",
    });
    expect(creditOutcomes([unknown, sitting]).counts.unknown).toBe(1);
  });
});
