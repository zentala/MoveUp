import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { BreakCreditDetails } from "./BreakCreditDetails";
import type { LastBreakCreditFacts } from "@/types";

const fact: LastBreakCreditFacts = {
  break_started_at: "2026-10-29T10:00:00Z",
  observed_break_secs: 300,
  minimum_break_secs: 60,
  multiplier: 2,
  requested_credit_secs: 600,
  prior_applied_credit_secs: 0,
  sitting_before_secs: 480,
  applied_credit_secs: 480,
  sitting_after_secs: 0,
  applied_at: "2026-10-29T10:05:00Z",
  no_credit_reason: null,
};

describe("BreakCreditDetails", () => {
  it("shows source-captured rule and capped before/applied/after facts in a native disclosure", () => {
    render(<BreakCreditDetails facts={[fact]} />);
    const disclosure = screen.getByTestId("break-credit-details");
    expect(disclosure.tagName).toBe("DETAILS");
    expect(screen.getByText("Break credit details").tagName).toBe("SUMMARY");
    expect(disclosure).toHaveTextContent("Observed duration");
    expect(disclosure).toHaveTextContent("05:00");
    expect(disclosure).toHaveTextContent("Credit requested");
    expect(disclosure).toHaveTextContent("10:00");
    expect(disclosure).toHaveTextContent("Credit applied");
    expect(disclosure).toHaveTextContent("08:00");
    expect(disclosure).toHaveTextContent("00:00");
    expect(disclosure).not.toHaveTextContent("×1");
  });

  it("states that legacy or absent producer facts are unavailable, not zero", () => {
    render(<BreakCreditDetails facts={null} />);
    expect(screen.getByText("Break credit details unavailable")).toBeInTheDocument();
    expect(screen.queryByTestId("break-credit-details")).not.toBeInTheDocument();
  });

  it("shows the producer reason for a no-credit result", () => {
    render(<BreakCreditDetails facts={[{ ...fact, requested_credit_secs: 0, applied_credit_secs: 0, no_credit_reason: "break_below_minimum" }]} />);
    expect(screen.getByText("Break was shorter than the captured minimum.")).toBeInTheDocument();
  });

  it("keeps separate applications and their captured rules visible after a profile change", () => {
    const laterApplication = {
      ...fact,
      observed_break_secs: 600,
      multiplier: 1,
      requested_credit_secs: 120,
      prior_applied_credit_secs: 480,
      sitting_before_secs: 720,
      applied_credit_secs: 120,
      sitting_after_secs: 600,
      applied_at: "2026-10-29T10:10:00Z",
    };
    const noCreditAttempt = {
      ...laterApplication,
      requested_credit_secs: 120,
      prior_applied_credit_secs: 600,
      sitting_before_secs: 600,
      applied_credit_secs: 0,
      sitting_after_secs: 600,
      applied_at: "2026-10-29T10:11:00Z",
      no_credit_reason: "already_credited_prefix" as const,
    };
    render(<BreakCreditDetails facts={[fact, laterApplication, noCreditAttempt]} />);
    const disclosure = screen.getByTestId("break-credit-details");
    fireEvent.click(screen.getByText("Break credit details"));
    const scrollBody = screen.getByRole("region", { name: "Break credit entries" });
    expect(disclosure).toHaveTextContent("×2");
    expect(disclosure).toHaveTextContent("×1");
    expect(disclosure).toHaveTextContent("Earlier credit");
    expect(disclosure).toHaveTextContent("08:00");
    expect(disclosure.querySelectorAll("ol > li")).toHaveLength(3);
    expect(scrollBody).toHaveAttribute("tabindex", "0");
    expect(scrollBody).toHaveTextContent("already met this rule");
    const firstRule = entriesText(disclosure, 0);
    const secondRule = entriesText(disclosure, 1);
    expect(firstRule).not.toBe(secondRule);
  });

  it("keeps the panel heading and Close action reachable and returns focus to the status disclosure", () => {
    render(<BreakCreditDetails facts={[fact]} />);
    const summary = screen.getByText("Break credit details");

    fireEvent.click(summary);
    expect(screen.getByRole("heading", { name: "Captured break credit" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Close break credit details" })).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Close break credit details" }));
    expect(screen.getByTestId("break-credit-details")).not.toHaveAttribute("open");
    expect(summary).toHaveFocus();
  });

  it("explains when a positive rule rounds to less than one credited second", () => {
    render(<BreakCreditDetails facts={[{ ...fact, multiplier: 0.001, requested_credit_secs: 0, applied_credit_secs: 0, no_credit_reason: "requested_credit_rounded_to_zero" }]} />);
    expect(screen.getByText("The captured rule requested less than one second of credit.")).toBeInTheDocument();
  });
});

function entriesText(disclosure: HTMLElement, index: number): string {
  return disclosure.querySelectorAll("ol > li")[index]?.querySelector("dl > div:nth-child(3) dd")?.textContent ?? "";
}
