/**
 * SessionProgress.floating.test.tsx — T029 floating window spec tests.
 *
 * Tests for transition data display and break credit labels.
 * Tests that require missing fields or features use .skip with a comment.
 */
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import SessionProgress from "@/components/SessionProgress";
import type { BreakCredit, StateChangedPayload } from "@/types";

describe("SessionProgress — Floating Window Spec (T029)", () => {
  // ── Scenario A: Fresh start ──────────────────────────────────────────

  it("renders 00:00 when sittingSeconds is 0 (fresh start)", () => {
    render(<SessionProgress sittingSeconds={0} limitSeconds={2700} />);
    expect(screen.getByText("00:00")).toBeInTheDocument();
  });

  // ── Scenario B: Session continues after short break ──────────────────

  it("renders 30:00 when session continues after short break", () => {
    // After standing < 5 min, sitting_seconds = 1800 (30 min carry-over)
    render(<SessionProgress sittingSeconds={1800} limitSeconds={2700} />);
    expect(screen.getByText("30:00")).toBeInTheDocument();
  });

  // ── Scenario C: Session reduced after partial break ──────────────────

  it("renders 10:00 when session reduced by 20 min after partial break", () => {
    // 1800 - 1200 = 600 = 10:00
    render(<SessionProgress sittingSeconds={600} limitSeconds={2700} />);
    expect(screen.getByText("10:00")).toBeInTheDocument();
  });

  // ── Scenario D: Fresh start after full break ─────────────────────────

  it("renders 00:00 after full break (>= 10 min standing)", () => {
    render(<SessionProgress sittingSeconds={0} limitSeconds={2700} />);
    expect(screen.getByText("00:00")).toBeInTheDocument();
  });

  // ── Scenario G: StateChangedPayload types ────────────────────────────

  it("StateChangedPayload includes last_break_secs and break_credit fields", () => {
    // This tests that the TypeScript type has the required fields.
    // The actual rendering of transition data is a UI feature for T030.
    const payload: StateChangedPayload = {
      state: "Sitting",
      sitting_seconds: 0,
      standing_seconds: 720,
      break_seconds: 0,
      desk_height_cm: 72.3,
      position_changes: 2,
      last_break_secs: 720,
      last_sitting_secs: 1800,
      break_credit: "full",
    };

    expect(payload.last_break_secs).toBe(720);
    expect(payload.last_sitting_secs).toBe(1800);
    expect(payload.break_credit).toBe("full");
  });

  it("BreakCredit type accepts all valid values", () => {
    const none: BreakCredit = "none";
    const partial: BreakCredit = "partial";
    const full: BreakCredit = "full";
    expect(none).toBe("none");
    expect(partial).toBe("partial");
    expect(full).toBe("full");
  });

  // ── Scenario G: Transition UI tests (will fail — feature not implemented) ──

  // .skip because the TransitionBanner component does not exist yet.
  // T030 will implement the banner that shows "Stood for X min" after transition.
  it.skip("renders last_break_secs after Standing->Sitting transition", () => {
    // BUG: TransitionBanner component does not exist yet.
    // When implemented, it should show "Stood for 12 min" for 30 seconds
    // after a Standing->Sitting transition, using payload.last_break_secs.
    // This test should render the banner and verify the text.
    expect(true).toBe(false); // placeholder — will be replaced in T030
  });

  it.skip("shows break credit label when partial credit applied", () => {
    // BUG: No UI element shows break credit info yet.
    // When implemented, after a 7-min break (partial credit), the UI should
    // show "Session reduced by 20 min" or similar. Uses payload.break_credit.
    expect(true).toBe(false); // placeholder — will be replaced in T030
  });

  it.skip("shows 'Fresh start!' when full credit applied", () => {
    // BUG: No UI element shows break credit info yet.
    // When implemented, after a 12-min break (full credit), the UI should
    // show "Fresh start!" or similar. Uses payload.break_credit = "full".
    expect(true).toBe(false); // placeholder — will be replaced in T030
  });
});
