import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import type { SessionRow } from "@/test/analyst-fixtures";
import { BreakCreditHistogram } from "./BreakCreditHistogram";

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

describe("BreakCreditHistogram", () => {
  it("shows recorded outcomes, including Unknown coverage, without duration labels", () => {
    const data = (["none", "partial", "full"] as const).map((breakCredit, index) =>
      equalDurationRow(index, breakCredit));
    data.push(row({
      id: 4,
      state: "Standing",
      startedAt: "2026-10-22T09:30:00+02:00",
      endedAt: "2026-10-22T09:32:00+02:00",
    }));
    data.push(row({
      id: 5,
      state: "Sitting",
      startedAt: "2026-10-22T09:32:00+02:00",
      endedAt: "2026-10-22T09:34:00+02:00",
    }));

    const { container } = render(<BreakCreditHistogram data={data} />);

    expect(container.querySelectorAll("rect[data-credit]")).toHaveLength(4);
    expect(container.querySelector('[data-credit="none"]')?.getAttribute("aria-label")).toContain("1");
    expect(container.querySelector('[data-credit="partial"]')?.getAttribute("aria-label")).toContain("1");
    expect(container.querySelector('[data-credit="full"]')?.getAttribute("aria-label")).toContain("1");
    expect(container.querySelector('[data-credit="unknown"]')?.getAttribute("aria-label")).toContain("1");
    expect(container.textContent).toContain("Coverage: 3 of 4 completed break returns classified");
    expect(container.textContent).not.toMatch(/<\s*60|60\s*[–-]\s*120|≥\s*120/);
    expect(container.textContent).not.toMatch(/medical|health effect/i);
  });

  it("distinguishes measured None from no eligible recorded break rows", () => {
    const measuredNone = render(<BreakCreditHistogram data={[row({ breakCredit: "none" })]} />);
    expect(measuredNone.container.textContent).toContain("Coverage: 1 of 1 completed break returns classified");
    expect(measuredNone.container.querySelector('[data-credit="none"]')?.getAttribute("aria-label")).toContain("1");

    const empty = render(<BreakCreditHistogram data={[]} />);
    expect(empty.container.textContent).toContain("No completed, recorded break outcomes");
    expect(empty.container.querySelector('[data-credit="none"]')?.getAttribute("aria-label")).toContain("0");
  });
});
