/**
 * DebugSection.test.tsx — unit tests for the debug data panel.
 *
 * Test Coverage:
 * - Renders all fixed groups with data from useDesk() defaults
 * - Renders Config group once get_settings resolves
 * - Renders "(no metrics)" and "(empty)" placeholders with no data
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import DebugSection from "./DebugSection";

// setup.ts provides global vi.mock for @tauri-apps/api/core

beforeEach(() => {
  vi.mocked(invoke).mockClear();
});

describe("DebugSection", () => {
  it("renders the fixed state groups", async () => {
    render(<DebugSection />);

    await waitFor(() => {
      expect(screen.getByText("Current State")).toBeInTheDocument();
    });
    expect(screen.getByText("Sitting Session")).toBeInTheDocument();
    expect(screen.getByText("Break / Standing")).toBeInTheDocument();
    expect(screen.getByText("Today Totals")).toBeInTheDocument();
    expect(screen.getByText("Transition")).toBeInTheDocument();
    expect(screen.getByText("Previous Session")).toBeInTheDocument();
    expect(screen.getByText("Notification Criteria")).toBeInTheDocument();
    expect(screen.getByText("KPI Metrics")).toBeInTheDocument();
  });

  it("shows placeholders when there are no metrics or sessions yet", async () => {
    render(<DebugSection />);

    await waitFor(() => {
      expect(screen.getByText("(no metrics)")).toBeInTheDocument();
    });
    expect(screen.getByText("(empty)")).toBeInTheDocument();
    expect(screen.getByText("(no recent transition)")).toBeInTheDocument();
    expect(screen.getByText("(none yet)")).toBeInTheDocument();
  });

  it("renders the Config group once get_settings resolves", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") {
        return {
          sit_limit_mins: 40,
          stand_limit_mins: 15,
          standing_target_mins: 20,
          notify_inactivity: true,
          notify_daily_posture_balance: false,
          notify_praise_halfway: true,
        };
      }
      return null;
    });

    render(<DebugSection />);

    await waitFor(() => {
      expect(screen.getByText("Config")).toBeInTheDocument();
    });
    expect(screen.getByText("sit_limit_mins")).toBeInTheDocument();
  });

  it("renders KPI metrics returned by get_dashboard_state", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_dashboard_state") {
        return {
          session: {
            state: "sitting",
            sitting_seconds: 0,
            standing_seconds: 0,
            break_seconds: 0,
            session_limit_secs: 2700,
            stand_limit_secs: 0,
            desk_height_cm: 75.0,
            position_changes: 0,
            limit_used_secs: 0,
            daily_score: 0,
            standing_session_secs: 0,
            secs_since_last_break: 0,
            continuous_computer_secs: 0,
            longest_computer_session_secs: 0,
            sitting_seconds_total: 0,
            idle_secs: 0,
            away_bout_secs: 0,
            max_continuous_computer_secs: 7200,
          },
          metrics: [
            {
              id: "standing_pct",
              label: "Standing",
              result: { value: 0.5, display: "50%", level: "green", is_personal_best: false },
            },
          ],
        };
      }
      return null;
    });

    render(<DebugSection />);

    await waitFor(() => {
      expect(screen.getByText(/Standing \(standing_pct\)/)).toBeInTheDocument();
    });
  });
});
