/**
 * AnalystLive.test.tsx — smoke test for the production `/#/analyst` route.
 *
 * `AnalystWindow` is exercised in depth by its own test suite; this only
 * confirms the route renders it with a sane 7-day default range and no
 * fixture overrides (falling through to live Tauri-invoke hooks).
 */
import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import AnalystLive from "./AnalystLive";

describe("AnalystLive", () => {
  it("renders the Analyst window with tab triggers", async () => {
    render(<AnalystLive />);

    await waitFor(() => {
      expect(screen.getByText("Catalog")).toBeInTheDocument();
    });
    expect(screen.getByText("Explorer")).toBeInTheDocument();
  });

  it("renders a date range spanning the last 7 days", () => {
    render(<AnalystLive />);
    const dateInputs = document.querySelectorAll('input[type="date"]');
    expect(dateInputs.length).toBeGreaterThanOrEqual(2);

    const from = (dateInputs[0] as HTMLInputElement).value;
    const to = (dateInputs[1] as HTMLInputElement).value;
    const spanDays = (Date.parse(to) - Date.parse(from)) / 86_400_000;
    expect(spanDays).toBe(6);
  });
});
