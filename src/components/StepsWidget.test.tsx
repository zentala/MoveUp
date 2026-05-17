/**
 * StepsWidget.test.tsx — covers the three render states and refresh flow.
 *
 * Mocking strategy: setup.ts globally mocks `@tauri-apps/api/core`, so we
 * just override `invoke` per test via `vi.mocked(invoke).mockResolvedValueOnce(...)`.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { StepsWidget } from "./StepsWidget";

const NOW = 1_715_000_000_000;

describe("StepsWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows the setup hint when Google Fit is not configured", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ configured: false, snapshot: null });
    render(<StepsWidget />);
    await waitFor(() =>
      expect(screen.getByText(/connect google fit/i)).toBeInTheDocument(),
    );
    // The count display must not appear in the unconfigured state.
    expect(screen.queryByTestId("steps-count")).not.toBeInTheDocument();
  });

  it("renders the cached snapshot with locale-formatted count", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: { steps_today: 7421, fetched_at_ms: NOW },
    });
    render(<StepsWidget />);
    const count = await screen.findByTestId("steps-count");
    // en-US formatting → "7,421"
    expect(count.textContent).toBe("7,421");
  });

  it("shows a placeholder while configured but with no snapshot yet", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ configured: true, snapshot: null });
    render(<StepsWidget />);
    const count = await screen.findByTestId("steps-count");
    expect(count.textContent).toBe("—");
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("refreshes when the refresh button is clicked", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ configured: true, snapshot: null })
      .mockResolvedValueOnce({ steps_today: 9999, fetched_at_ms: NOW });
    render(<StepsWidget />);
    await screen.findByTestId("steps-count");

    fireEvent.click(screen.getByRole("button", { name: /refresh steps/i }));
    await waitFor(() =>
      expect(screen.getByTestId("steps-count").textContent).toBe("9,999"),
    );
    expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("refresh_steps_now");
  });

  it("surfaces backend errors inline without crashing", async () => {
    vi.mocked(invoke).mockRejectedValueOnce("network down");
    render(<StepsWidget />);
    await waitFor(() => expect(screen.getByTestId("steps-error")).toBeInTheDocument());
    expect(screen.getByTestId("steps-error").textContent).toContain("network down");
  });
});
