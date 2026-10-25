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
      if (command === "get_dashboard_state") return {
        session: {
          state: "Sitting",
          observation_quality: null,
          live_observation: {
            span: {
              state: "Sitting",
              started_at: "2026-10-25T08:00:00Z",
              observed_through: "2026-10-25T08:04:00Z",
            },
            last_observation_at: "2026-10-25T08:04:00Z",
            snapshot_at: "2026-10-25T08:09:01Z",
            quality: null,
            reason: "stale",
          },
          sitting_seconds: 120, standing_seconds: 0, break_seconds: 0,
          session_limit_secs: 2400, stand_limit_secs: 900, desk_height_cm: 72,
          position_changes: 0, limit_used_secs: 120, daily_score: 0,
          standing_session_secs: 0, secs_since_last_break: 120,
          continuous_computer_secs: 120, longest_computer_session_secs: 120,
          sitting_seconds_total: 120, idle_secs: 0, away_bout_secs: 0,
          max_continuous_computer_secs: 7200,
        },
        metrics: [],
      } as never;
      throw new Error(`Unexpected Analyst route command: ${command}`);
    });
  });

  it("renders the Analyst window with tab triggers", async () => {
    render(<AnalystLive />);

    await expectSettledRange();
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_dashboard_state"));
    expect(invoke).not.toHaveBeenCalledWith("start_auto_connect", undefined);
    expect(screen.getByTestId("analyst-live-observation")).toHaveAttribute("data-observation-reason", "stale");
    expect(screen.getByTestId("analyst-live-observation")).toHaveAttribute("data-observed-through", "2026-10-25T08:04:00Z");
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
