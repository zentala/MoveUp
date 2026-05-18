/**
 * StepsWidget.test.tsx — covers the three render states and refresh flow.
 *
 * setup.ts globally mocks `@tauri-apps/api/core`; we override `invoke` per
 * test via `vi.mocked(invoke).mockResolvedValueOnce(...)`.
 *
 * The widget kicks off an automatic refresh ~500ms after mount, so each
 * test queues at least one additional mock response for `refresh_steps_now`
 * to keep the rejection log clean.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { StepsWidget } from "./StepsWidget";

const NOW = 1_715_000_000_000;

describe("StepsWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default fallback so the post-mount auto-refresh doesn't reject loudly.
    vi.mocked(invoke).mockResolvedValue({
      steps_today: 0,
      fetched_at_ms: NOW,
    });
  });

  it("shows the connect hint when Google Fit is not configured", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: false,
      snapshot: null,
    });
    render(<StepsWidget />);
    await waitFor(() =>
      expect(screen.getByText(/connect google fit/i)).toBeInTheDocument(),
    );
    // The count slot must not appear in the unconfigured state.
    expect(screen.queryByTestId("steps-count")).not.toBeInTheDocument();
  });

  it("renders the cached snapshot with locale-formatted count", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: { steps_today: 7421, fetched_at_ms: NOW },
    });
    render(<StepsWidget />);
    const count = await screen.findByTestId("steps-count");
    expect(count.textContent).toBe("7,421");
  });

  it("shows a placeholder while configured but with no snapshot yet", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
    });
    render(<StepsWidget />);
    const count = await screen.findByTestId("steps-count");
    expect(count.textContent).toBe("—");
  });

  it("refreshes when the refresh button is clicked", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ configured: true, snapshot: null }) // get_steps_today
      .mockResolvedValueOnce({ steps_today: 100, fetched_at_ms: NOW }) // auto-refresh
      .mockResolvedValueOnce({ steps_today: 9999, fetched_at_ms: NOW }); // manual click
    render(<StepsWidget />);
    await screen.findByTestId("steps-count");

    fireEvent.click(screen.getByRole("button", { name: /refresh steps/i }));
    await waitFor(() =>
      expect(screen.getByTestId("steps-count").textContent).toBe("9,999"),
    );
    expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("refresh_steps_now");
  });

  it("surfaces backend errors as a marker without crashing", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("network down");
    render(<StepsWidget />);
    await waitFor(() =>
      expect(screen.getByTestId("steps-error")).toBeInTheDocument(),
    );
  });
});
