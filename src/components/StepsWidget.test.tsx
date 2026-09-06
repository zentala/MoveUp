/**
 * StepsWidget.test.tsx — covers all five render branches and refresh flow.
 *
 * setup.ts globally mocks `@tauri-apps/api/core`; we override `invoke`
 * per test via `vi.mocked(invoke).mockResolvedValueOnce(...)`.
 *
 * The widget kicks off an automatic refresh ~500ms after mount, so each
 * test queues at least one extra mock response for `refresh_steps_now`
 * to keep the rejection log clean.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { StepsWidget } from "./StepsWidget";

const NOW = 1_715_000_000_000;

describe("StepsWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default fallback so any auto-refresh after the first invoke doesn't
    // reject loudly. The shape matches `StepsView`.
    vi.mocked(invoke).mockResolvedValue({
      configured: true,
      snapshot: { steps_today: 0, fetched_at_ms: NOW },
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
    expect(screen.queryByTestId("steps-count")).not.toBeInTheDocument();
  });

  it("renders the cached snapshot with locale-formatted count", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: { steps_today: 7421, fetched_at_ms: NOW },
    });
    render(<StepsWidget />);
    const count = await screen.findByTestId("steps-count");
    // Locale-aware: "7,421" en-US, "7 421" pl-PL — accept either by
    // stripping all non-digits.
    expect(count.textContent?.replace(/\D/g, "")).toBe("7421");
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
      .mockResolvedValueOnce({ configured: true, snapshot: null })
      .mockResolvedValueOnce({
        configured: true,
        snapshot: { steps_today: 100, fetched_at_ms: NOW },
      })
      .mockResolvedValueOnce({
        configured: true,
        snapshot: { steps_today: 9999, fetched_at_ms: NOW },
      });
    render(<StepsWidget />);
    await screen.findByTestId("steps-count");

    fireEvent.click(screen.getByRole("button", { name: /refresh steps/i }));
    await waitFor(() =>
      expect(screen.getByTestId("steps-count").textContent?.replace(/\D/g, "")).toBe(
        "9999",
      ),
    );
    expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("refresh_steps_now");
  });

  it("surfaces transient errors as a marker without crashing", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
      error_kind: "transient",
      error_message: "google fit: aggregate http 503",
    });
    render(<StepsWidget />);
    await waitFor(() =>
      expect(screen.getByTestId("steps-error")).toBeInTheDocument(),
    );
  });

  it("shows the reconnect CTA when the refresh token is revoked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
      error_kind: "auth_revoked",
      error_message: "google fit: refresh token revoked",
    });
    render(<StepsWidget />);
    await waitFor(() =>
      expect(screen.getByTestId("steps-reconnect")).toBeInTheDocument(),
    );
    expect(screen.getByText(/reconnect google fit/i)).toBeInTheDocument();
    // Reconnect state must NOT show the refresh button (no point).
    expect(
      screen.queryByRole("button", { name: /refresh steps/i }),
    ).not.toBeInTheDocument();
  });

  it("applies the stale class when fetched_at_ms is > 1h old", async () => {
    const oneHourPlus = Date.now() - 2 * 60 * 60 * 1000;
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: { steps_today: 500, fetched_at_ms: oneHourPlus },
    });
    render(<StepsWidget />);
    const badge = await screen.findByTestId("steps-widget");
    await waitFor(() => expect(badge.className).toMatch(/steps-widget--stale/));
  });
});

describe("StepsWidget in remote-display mode (no Tauri)", () => {
  afterEach(() => {
    // Restore the "running inside Tauri" default set by test/setup.ts.
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  });

  it("renders a clean 'not available' state instead of throwing/invoking Tauri", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    vi.resetModules();
    vi.clearAllMocks();
    const { invoke: freshInvoke } = await import("@tauri-apps/api/core");
    const { StepsWidget: FreshStepsWidget } = await import("./StepsWidget");

    render(<FreshStepsWidget />);

    expect(screen.getByTestId("steps-widget")).toHaveTextContent(
      "not available",
    );
    expect(freshInvoke).not.toHaveBeenCalled();
  });
});
