/**
 * AnalystLive.test.tsx — smoke test for the production `/#/analyst` route.
 *
 * `AnalystWindow` is exercised in depth by its own test suite; this only
 * confirms the route renders it with a sane 7-day default range and no
 * fixture overrides (falling through to live Tauri-invoke hooks).
 */
import { beforeEach, describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import AnalystLive from "./AnalystLive";

const rangeCommands = [
  "get_snapshots_range",
  "get_sessions_range",
  "get_events_range",
];

async function expectSettledRange() {
  await screen.findByText(/0 snapshots · 0 sessions/);
  expect(screen.getByRole("button", { name: "Refresh" })).toBeEnabled();
  const dateInputs = document.querySelectorAll<HTMLInputElement>('input[type="date"]');
  expect(dateInputs.length).toBeGreaterThanOrEqual(2);
  const from = dateInputs[0].value;
  const to = dateInputs[1].value;
  await waitFor(() => {
    for (const command of rangeCommands) {
      expect(invoke).toHaveBeenCalledWith(command, { from, to });
    }
  });
  return { from, to };
}

describe("AnalystLive", () => {
  beforeEach(() => {
    localStorage.removeItem("desk:analyst:range");
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation(async (command) => {
      // Real range commands return vectors, including an empty history.
      if (rangeCommands.includes(command)) return [] as never;
      throw new Error(`Unexpected Analyst route command: ${command}`);
    });
  });

  it("renders the Analyst window with tab triggers", async () => {
    render(<AnalystLive />);

    await expectSettledRange();
    expect(screen.getByRole("tab", { name: "Catalog" })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByTestId("explorer-tab")).toBeInTheDocument();
  }, 10000);

  it("renders a date range spanning the last 7 days", async () => {
    render(<AnalystLive />);
    const { from, to } = await expectSettledRange();
    const spanDays = (Date.parse(to) - Date.parse(from)) / 86_400_000;
    expect(spanDays).toBe(6);
    const now = new Date();
    const localToday = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    expect(to).toBe(localToday);
  });
});
