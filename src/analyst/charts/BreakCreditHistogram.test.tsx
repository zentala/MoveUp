import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { BreakCreditHistogram } from "./BreakCreditHistogram";
import { analystSessionsFixture } from "@/test/analyst-fixtures";

describe("BreakCreditHistogram", () => {
  it("renders an SVG with three bars", () => {
    const { container } = render(<BreakCreditHistogram data={analystSessionsFixture} />);
    expect(container.querySelector("svg")).toBeTruthy();
    const rects = container.querySelectorAll("rect");
    expect(rects.length).toBeGreaterThanOrEqual(3);
  });

  it("labels the three credit buckets", () => {
    const { container } = render(<BreakCreditHistogram data={analystSessionsFixture} />);
    expect(container.textContent).toContain("None");
    expect(container.textContent).toContain("Partial");
    expect(container.textContent).toContain("Full");
  });
});
