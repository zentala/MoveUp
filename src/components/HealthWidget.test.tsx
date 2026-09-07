/**
 * HealthWidget.test.tsx — every render branch plus the refresh flow.
 *
 * setup.ts globally mocks `@tauri-apps/api/core`; we override `invoke`
 * per test via `vi.mocked(invoke).mockResolvedValueOnce(...)`.
 *
 * The hook kicks off an automatic refresh ~500ms after mount, so each test
 * queues a fallback response for `refresh_health_now` to keep the rejection
 * log clean.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { HealthWidget } from "./HealthWidget";

const NOW = 1_715_000_000_000;

function snapshot(overrides: Record<string, unknown> = {}) {
  return {
    steps_today: 0,
    source_id: "google_fit",
    fetched_at_ms: NOW,
    ...overrides,
  };
}

describe("HealthWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue({
      configured: true,
      snapshot: snapshot(),
    });
  });

  it("shows the connect hint when no health source is configured", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: false,
      snapshot: null,
    });
    render(<HealthWidget />);
    await waitFor(() =>
      expect(screen.getByText(/connect health source/i)).toBeInTheDocument(),
    );
    expect(screen.queryByTestId("health-steps")).not.toBeInTheDocument();
  });

  it("renders the cached snapshot with a locale-formatted step count", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({ steps_today: 7421 }),
    });
    render(<HealthWidget />);
    const count = await screen.findByTestId("health-steps");
    // Locale-aware: "7,421" en-US, "7 421" pl-PL — compare digits only.
    expect(count.textContent?.replace(/\D/g, "")).toBe("7421");
  });

  it("shows a placeholder while configured but with no snapshot yet", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
    });
    render(<HealthWidget />);
    const count = await screen.findByTestId("health-steps");
    expect(count.textContent).toBe("—");
    // No snapshot means no source to name.
    expect(screen.queryByTestId("health-source")).not.toBeInTheDocument();
  });

  it("renders the HR badge only when heart_rate_bpm is present", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({ steps_today: 100, heart_rate_bpm: 62 }),
    });
    render(<HealthWidget />);
    const hr = await screen.findByTestId("health-hr");
    expect(hr.textContent?.replace(/\D/g, "")).toBe("62");
  });

  it("omits the HR badge when the source reports no heart rate", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({ steps_today: 100 }),
    });
    render(<HealthWidget />);
    await screen.findByTestId("health-steps");
    expect(screen.queryByTestId("health-hr")).not.toBeInTheDocument();
  });

  it("labels the source and warns about the Google Fit sunset", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({ steps_today: 10, source_id: "google_fit" }),
    });
    render(<HealthWidget />);
    const badge = await screen.findByTestId("health-widget");
    expect(screen.getByTestId("health-source").textContent).toBe("Google Fit");
    await waitFor(() =>
      expect(badge.getAttribute("title")).toMatch(/Google Fit ends late 2026/),
    );
  });

  it("drops the sunset hint when another source produced the reading", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({ steps_today: 10, source_id: "push" }),
    });
    render(<HealthWidget />);
    const badge = await screen.findByTestId("health-widget");
    expect(screen.getByTestId("health-source").textContent).toBe("phone");
    expect(badge.getAttribute("title")).not.toMatch(/ends late 2026/);
  });

  it("refreshes when the refresh button is clicked", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce({ configured: true, snapshot: null })
      .mockResolvedValueOnce({
        configured: true,
        snapshot: snapshot({ steps_today: 100 }),
      })
      .mockResolvedValueOnce({
        configured: true,
        snapshot: snapshot({ steps_today: 9999 }),
      });
    render(<HealthWidget />);
    await screen.findByTestId("health-steps");

    fireEvent.click(screen.getByRole("button", { name: /refresh health/i }));
    await waitFor(() =>
      expect(
        screen.getByTestId("health-steps").textContent?.replace(/\D/g, ""),
      ).toBe("9999"),
    );
    expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("refresh_health_now");
  });

  it("surfaces transient errors as a marker without crashing", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
      error_kind: "transient",
      error_message: "google fit: aggregate http 503",
    });
    render(<HealthWidget />);
    await waitFor(() =>
      expect(screen.getByTestId("health-error")).toBeInTheDocument(),
    );
  });

  it("shows the reconnect CTA when the refresh token is revoked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: null,
      error_kind: "auth_revoked",
      error_message: "google fit: refresh token revoked",
    });
    render(<HealthWidget />);
    await waitFor(() =>
      expect(screen.getByTestId("health-reconnect")).toBeInTheDocument(),
    );
    // Reconnect state must NOT show the refresh button (no point).
    expect(
      screen.queryByRole("button", { name: /refresh health/i }),
    ).not.toBeInTheDocument();
  });

  it("applies the stale class when fetched_at_ms is > 1h old", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      configured: true,
      snapshot: snapshot({
        steps_today: 500,
        fetched_at_ms: Date.now() - 2 * 60 * 60 * 1000,
      }),
    });
    render(<HealthWidget />);
    const badge = await screen.findByTestId("health-widget");
    await waitFor(() => expect(badge.className).toMatch(/health-widget--stale/));
  });
});

describe("HealthWidget in remote-display mode (no Tauri)", () => {
  afterEach(() => {
    // Restore the "running inside Tauri" default set by test/setup.ts.
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  });

  it("renders the pushed snapshot without ever calling Tauri IPC", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    vi.resetModules();
    vi.clearAllMocks();
    const { invoke: freshInvoke } = await import("@tauri-apps/api/core");
    const { publishRemoteHealth } = await import("@/hooks/useHealth");
    const { HealthWidget: FreshWidget } = await import("./HealthWidget");

    publishRemoteHealth({
      configured: true,
      snapshot: {
        steps_today: 1234,
        source_id: "curl",
        fetched_at_ms: Date.now(),
      },
    });

    render(<FreshWidget />);

    expect(
      screen.getByTestId("health-steps").textContent?.replace(/\D/g, ""),
    ).toBe("1234");
    expect(screen.getByTestId("health-source").textContent).toBe("curl");
    // No refresh affordance on the phone — the desktop owns outbound calls.
    expect(
      screen.queryByRole("button", { name: /refresh health/i }),
    ).not.toBeInTheDocument();
    expect(freshInvoke).not.toHaveBeenCalled();
  });
});
