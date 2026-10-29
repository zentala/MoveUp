/**
 * OneBarWidget.test.tsx — verifies the One Bar widget renders correctly
 * across all states and temperature levels.
 */
import { describe, it, expect, vi, beforeAll, afterAll } from "vitest";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { OneBarWidget } from "./OneBarWidget";
import type { WidgetProps } from "@/types";
import type { LiveObservation } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { OPEN_ANALYST_DAY_KEY } from "@/analyst/open-day";

// Native modal behavior is covered in the real App browser tests, not jsdom.
beforeAll(() => {
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true, value(this: HTMLDialogElement) { this.setAttribute("open", ""); },
  });
  Object.defineProperty(HTMLDialogElement.prototype, "close", {
    configurable: true, value(this: HTMLDialogElement) { this.removeAttribute("open"); },
  });
});
afterAll(() => {
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});

function props(overrides: Partial<WidgetProps> = {}): WidgetProps {
  const merged: WidgetProps = {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72.5,
    limitUsedSecs: 600,
    limitSecs: 2400,
    standLimitSecs: 900,
    limitRemaining: 1800,
    limitRatio: 0.25,
    breakSecs: 0,
    breakResetThreshold: 600,
    breakResetProgress: 0,
    previousSession: null,
    todaySessions: [],
    todayChanges: 3,
    todayStandingSecs: 0,
    todaySittingSecs: 600,
    todayScore: 0,
    idleSecs: 0,
    continuousComputerSecs: 0,
    metrics: [],
    error: null,
    onOpenSettings: vi.fn(),
    ...overrides,
  };
  if (overrides.liveObservation === undefined) {
    merged.liveObservation = {
      span: {
        state: merged.state ?? "Sitting",
        started_at: "2026-10-29T10:00:00Z",
        observed_through: "2026-10-29T10:10:00Z",
      },
      last_observation_at: "2026-10-29T10:10:00Z",
      snapshot_at: "2026-10-29T10:10:00Z",
      quality: "known",
      reason: null,
    };
  }
  return merged;
}

it("keeps a protected Share dialog open and retryable when closing it fails", async () => {
  const protect = vi.fn()
    .mockResolvedValueOnce(undefined)
    .mockRejectedValueOnce(new Error("stale_client_update"))
    .mockResolvedValueOnce(undefined);
  render(<OneBarWidget {...props({ onShareChange: protect })} />);
  fireEvent.click(screen.getByRole("button", { name: "Share my stats" }));
  const dialog = await screen.findByRole("dialog", { name: "share my stats" });

  fireEvent.click(screen.getByRole("button", { name: "Close" }));
  await waitFor(() => expect(dialog).toBeInTheDocument());
  expect(dialog).toContainElement(screen.getByRole("alert"));
  expect(screen.getByRole("alert")).toHaveTextContent("Could not change the Share view");
  expect(screen.getByRole("button", { name: "Retry close" })).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Retry close" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  expect(protect).toHaveBeenCalledTimes(3);
});

it("does not open Analyst with a missing day when saving the request fails and can retry", async () => {
  localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
  vi.mocked(invoke).mockClear();
  const storageWrite = vi.spyOn(Storage.prototype, "setItem").mockImplementationOnce(() => {
    throw new Error("storage unavailable");
  });
  const { container } = render(<OneBarWidget {...props()} />);
  const todayButton = container.querySelector<HTMLButtonElement>(".one-bar__today-btn");
  expect(todayButton).not.toBeNull();

  fireEvent.click(todayButton!);
  await screen.findByRole("alert");
  expect(screen.getByRole("alert")).toHaveTextContent("Could not open Analyst for today");
  expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull();
  expect(invoke).not.toHaveBeenCalledWith("open_analyst_window");

  storageWrite.mockRestore();
  fireEvent.click(screen.getByRole("button", { name: "Retry opening Analyst" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("open_analyst_window"));
  expect(JSON.parse(localStorage.getItem(OPEN_ANALYST_DAY_KEY) ?? "null")).toMatchObject({
    day: expect.any(String), requestId: expect.any(String),
  });
});

describe("OneBarWidget", () => {
  it("keeps main actions above the full status and hides the popup through its native command", async () => {
    vi.mocked(invoke).mockClear();
    const { container } = render(<OneBarWidget {...props({ onShareChange: vi.fn(), idleSecs: 0 })} />);

    const actionRow = container.querySelector<HTMLElement>(".one-bar__header-actions");
    expect(actionRow).not.toBeNull();
    expect(actionRow!.children).toHaveLength(4);
    expect(screen.getByRole("button", { name: "Share my stats" })).toBeInTheDocument();
    expect(within(actionRow!).getByRole("button", { name: "Open Analyst for today's local day" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Settings" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Close popup to tray" })).toBeInTheDocument();
    expect(screen.getByText("Estimated Sitting")).toBeInTheDocument();
    expect(screen.getByText("73 cm")).toBeInTheDocument();
    expect(document.querySelector(".state-indicator")?.textContent).not.toContain("Recent input");

    fireEvent.click(screen.getByRole("button", { name: "Close popup to tray" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("hide_main_window"));
  });

  it("keeps the popup visible and offers retry when the native hide fails", async () => {
    let failuresRemaining = 1;
    vi.mocked(invoke).mockReset().mockImplementation(async (command: string) => {
      if (command === "hide_main_window" && failuresRemaining-- > 0) throw new Error("native hide failed");
      return null;
    });
    render(<OneBarWidget {...props({ onShareChange: vi.fn() })} />);
    fireEvent.click(screen.getByRole("button", { name: "Close popup to tray" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Could not close the popup. Please retry.");
    fireEvent.click(screen.getByRole("button", { name: "Retry close" }));
    await waitFor(() => expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "hide_main_window")).toHaveLength(2));
    expect(screen.getByTestId("one-bar-widget")).toBeInTheDocument();
  });

  it("accepts another explicit X request while an earlier hide acknowledgement is unresolved", async () => {
    let hideRequests = 0;
    vi.mocked(invoke).mockReset().mockImplementation((command: string) => {
      if (command === "hide_main_window") {
        hideRequests += 1;
        // The native window can be hidden/re-shown while this WebView stays mounted.
        // Model a successful first hide whose JavaScript acknowledgement never settles.
        if (hideRequests === 1) return new Promise<null>(() => {});
      }
      return Promise.resolve(null);
    });
    render(<OneBarWidget {...props({ onShareChange: vi.fn() })} />);
    const close = screen.getByRole("button", { name: "Close popup to tray" });

    fireEvent.click(close);
    await waitFor(() => expect(hideRequests).toBe(1));

    // A later explicit close must reach the idempotent native hide again.
    fireEvent.click(close);
    await waitFor(() => expect(hideRequests).toBe(2));
  });

  it("renders without crashing", () => {
    render(<OneBarWidget {...props()} />);
    expect(screen.getByTestId("one-bar-widget")).toBeInTheDocument();
  });

  it("applies calm temperature class when sitting < 50%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.3 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--calm");
  });

  it("applies warm temperature class when sitting 50-80%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.6 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--warm");
  });

  it("applies hot temperature class when sitting 80-100%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.9 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--hot");
  });

  it("applies burning temperature class when over limit", () => {
    render(<OneBarWidget {...props({ limitRatio: 1.2 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--burning");
  });

  it("applies standing temperature class", () => {
    render(
      <OneBarWidget
        {...props({ state: "Standing", breakResetProgress: 0.5 })}
      />,
    );
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--standing");
  });

  it("applies away temperature class", () => {
    render(<OneBarWidget {...props({ state: "Away" })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--away");
  });

  it("applies reset temperature class", () => {
    render(
      <OneBarWidget
        {...props({ state: "Standing", breakResetProgress: 1.0 })}
      />,
    );
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--reset");
  });

  it("shows desk height in header via StateIndicator", () => {
    render(<OneBarWidget {...props({ deskHeightCm: 72.4 })} />);
    expect(screen.getByText("72 cm")).toBeInTheDocument();
  });

  it("shows state label in header via StateIndicator", () => {
    const { rerender } = render(<OneBarWidget {...props({ state: "Sitting" })} />);
    expect(screen.getByText("Estimated Sitting")).toBeInTheDocument();

    rerender(<OneBarWidget {...props({ state: "Standing" })} />);
    expect(screen.getByText("Estimated Standing")).toBeInTheDocument();

    rerender(<OneBarWidget {...props({ state: "Away" })} />);
    expect(screen.getByText("Estimated Away")).toBeInTheDocument();
  });

  it("moves enabled activity status into the accessible details", () => {
    render(<OneBarWidget {...props({ idleSecs: 0 })} />);
    expect(document.querySelector(".state-indicator")?.textContent).not.toContain("Recent input");
    expect(screen.getByText("Recent input")).toBeInTheDocument();
  });

  it("opens today's local Analyst day from the compact header entry", () => {
    localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
    vi.mocked(invoke).mockClear();
    const ordering: string[] = [];
    const originalSetItem = Storage.prototype.setItem;
    const storageWrite = vi.spyOn(Storage.prototype, "setItem").mockImplementation(function (this: Storage, key: string, value: string) {
      if (key === OPEN_ANALYST_DAY_KEY) ordering.push("request");
      originalSetItem.call(this, key, value);
    });
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === "open_analyst_window") ordering.push("open");
      return null;
    });
    const before = Date.now();
    const { container } = render(<OneBarWidget {...props()} />);

    const todayButton = container.querySelector<HTMLButtonElement>(".one-bar__today-btn");
    expect(todayButton).not.toBeNull();
    fireEvent.click(todayButton!);

    const request = JSON.parse(localStorage.getItem(OPEN_ANALYST_DAY_KEY) ?? "null") as { day: string; requestedAtMs: number; requestId: string };
    const now = new Date(request.requestedAtMs);
    const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
    expect(request.day).toBe(today);
    expect(request.requestedAtMs).toBeGreaterThanOrEqual(before);
    expect(request.requestId).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("open_analyst_window");
    expect(ordering).toEqual(["request", "open"]);
    storageWrite.mockRestore();
  });

  it("hides recent-input status when the saved activity preference is off", () => {
    render(<OneBarWidget {...props({ idleSecs: 150, showActivityStatus: false })} />);
    expect(screen.queryByText("No recent input · 2m 30s")).not.toBeInTheDocument();
    expect(screen.queryByText("Recent input")).not.toBeInTheDocument();
    expect(screen.getByText("Estimated Sitting")).toBeInTheDocument();
  });

  it("shows observation and view-response status without changing compact strip geometry", () => {
    const liveObservation: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: "2026-10-24T10:00:00Z",
        observed_through: "2026-10-24T10:20:00Z",
      },
      last_observation_at: "2026-10-24T10:20:00Z",
      snapshot_at: "2026-10-24T11:00:00Z",
      quality: null,
      reason: "stale",
    };
    render(<OneBarWidget {...props({
      liveObservation,
      dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-24T11:00:01Z", error: "temporary read failure" },
    })} />);

    const status = screen.getByTestId("timeline-observation-status");
    expect(status).toHaveAttribute("aria-label", expect.stringContaining("Sitting observed through"));
    expect(status).toHaveTextContent("Stale · View error");
    expect(screen.getByTestId("view-response-details")).toHaveTextContent("View update failed: temporary read failure");
    expect(screen.getByTestId("timeline-strip")).toHaveStyle({ height: "24px", minHeight: "24px" });
  });

  it("separates an unavailable observation from recent input and keeps the credited balance", () => {
    const liveObservation: LiveObservation = {
      span: null,
      last_observation_at: null,
      snapshot_at: "2026-10-29T00:00:00.000Z",
      quality: null,
      reason: "no_observation",
    };
    render(<OneBarWidget {...props({
      state: "Sitting",
      deskHeightCm: 72,
      idleSecs: 0,
      limitUsedSecs: 600,
      liveObservation,
      dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-29T00:00:01Z", error: null },
    })} />);

    expect(document.querySelector(".state-indicator__label")).toHaveTextContent("Unknown");
    expect(screen.queryByText("Sitting")).not.toBeInTheDocument();
    expect(screen.getByText("Recent input")).toBeInTheDocument();
    expect(screen.queryByText("72 cm")).not.toBeInTheDocument();
    expect(screen.getByTestId("one-bar-big-number")).toHaveTextContent("10:00 / 40:00");

    const disclosure = screen.getByTestId("observation-details");
    const summary = screen.getByTestId("timeline-observation-status");
    expect(disclosure).not.toHaveAttribute("aria-live");
    fireEvent.click(summary);
    const explanation = screen.getByRole("region", { name: "How to read this status" });
    expect(explanation).toHaveTextContent("desk-height observations and recent Windows keyboard/mouse input");
    expect(explanation).toHaveTextContent("No recent input can also mean reading, listening, thinking, or AI work");
    expect(explanation).toHaveTextContent("Timeline colors identify recorded desk state and time");
    expect(explanation).toHaveTextContent("Break credit reduces the current Sitting balance, but never removes recorded intervals");
    expect(explanation).toHaveTextContent("A sitting-colored interval alone does not mean the limit is exceeded");
    expect(explanation).toHaveTextContent("View response received");
    expect(explanation.querySelectorAll('[aria-live="polite"]')).toHaveLength(0);
    expect(screen.getByTestId("view-response-details")).toHaveAttribute("aria-live", "off");
  });

  it("qualifies a stale state as the last estimate and does not show its height as current", () => {
    const liveObservation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-24T10:00:00Z", observed_through: "2026-10-24T10:20:00Z" },
      last_observation_at: "2026-10-24T10:20:00Z",
      snapshot_at: "2026-10-24T11:00:00Z",
      quality: null,
      reason: "stale",
    };
    render(<OneBarWidget {...props({ state: "Sitting", deskHeightCm: 72, liveObservation })} />);
    expect(screen.getByText("Last estimate Sitting")).toBeInTheDocument();
    expect(screen.queryByText("72 cm")).not.toBeInTheDocument();
    expect(screen.getByTestId("observation-details")).toHaveTextContent("sensor observation stale");
  });

  it("qualifies retained known observations after a view refresh fails without calling the sensor stale", () => {
    const liveObservation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-24T10:00:00Z", observed_through: "2026-10-24T10:20:00Z" },
      last_observation_at: "2026-10-24T10:20:00Z",
      snapshot_at: "2026-10-24T10:20:00Z",
      quality: "known",
      reason: null,
    };
    render(<OneBarWidget {...props({
      state: "Sitting",
      deskHeightCm: 72,
      liveObservation,
      dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-24T10:20:01Z", error: "snapshot unavailable" },
    })} />);

    expect(screen.getByText("Last estimate Sitting")).toBeInTheDocument();
    expect(screen.getByText("72 cm")).toBeInTheDocument();
    expect(screen.getByTestId("timeline-observation-status")).toHaveAttribute("aria-label", expect.stringContaining("View refresh failed"));
    expect(screen.getByTestId("timeline-observation-status")).toHaveAttribute("aria-label", expect.stringContaining("Sitting observed through"));
    expect(screen.getByTestId("timeline-observation-status")).toHaveTextContent("View error");
    expect(screen.getByTestId("view-response-details")).toHaveTextContent("View update failed: snapshot unavailable");
    expect(screen.getByTestId("view-response-details")).toHaveTextContent("last response received");
  });

  it("does not announce routine request-start and response changes in the live status", () => {
    const liveObservation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-24T10:00:00Z", observed_through: "2026-10-24T10:20:00Z" },
      last_observation_at: "2026-10-24T10:20:00Z",
      snapshot_at: "2026-10-24T10:20:00Z",
      quality: "known",
      reason: null,
    };
    const common = { ...props({ liveObservation }) };
    const view = render(<OneBarWidget {...{ ...common,
      dashboardRefresh: { loading: true, lastSuccessAt: "2026-10-24T11:00:00Z", error: null } }} />);
    const status = screen.getByTestId("timeline-observation-announcement");
    const announcement = status.textContent;
    view.rerender(<OneBarWidget {...{ ...common,
      dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-24T11:00:01Z", error: null } }} />);
    expect(screen.getByTestId("timeline-observation-announcement").textContent).toBe(announcement);
    expect(screen.getByTestId("view-response-details")).toHaveTextContent("View response received");
    expect(screen.getByTestId("view-response-details")).toHaveAttribute("aria-live", "off");
  });

  it("shows elapsed/total as big number", () => {
    render(
      <OneBarWidget
        {...props({ limitUsedSecs: 600, limitSecs: 2400 })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 600s = 10:00, 2400s = 40:00
    expect(bigNum.textContent).toContain("10:00");
    expect(bigNum.textContent).toContain("40:00");
  });

  it("renders all sub-components", () => {
    render(<OneBarWidget {...props()} />);
    expect(screen.getByTestId("one-bar-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("one-bar-timer")).toBeInTheDocument();
    expect(screen.getByTestId("progress-bar-container")).toBeInTheDocument();
  });

  it("renders KpiStrip with metrics", () => {
    render(
      <OneBarWidget
        {...props({
          metrics: [
            {
              id: "standing_pct",
              label: "\u2195 Standing",
              result: { value: 15, display: "15%", level: "green", is_personal_best: false },
            },
            {
              id: "position_rate",
              label: "\u21c4 Changes",
              result: { value: 1.2, display: "1.2/h", level: "green", is_personal_best: true },
            },
          ],
        })}
      />,
    );
    expect(screen.getByTestId("kpi-strip")).toBeInTheDocument();
    expect(screen.getByText("15%")).toBeInTheDocument();
    expect(screen.getByText("1.2/h")).toBeInTheDocument();
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpen = vi.fn();
    render(<OneBarWidget {...props({ onOpenSettings: onOpen })} />);
    screen.getByTitle("Settings").click();
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it("exposes named share and settings icon controls and opens sharing", () => {
    render(<OneBarWidget {...props()} />);
    const share = screen.getByRole("button", { name: "Share my stats" });
    expect(share).toHaveAttribute("title", "Share my stats");
    expect(screen.getByRole("button", { name: "Settings" })).toHaveAttribute("title", "Settings");
    fireEvent.click(share);
    expect(screen.getByText("share my stats")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(share).toHaveFocus();
  });

  it("big number shows break elapsed/standing target for standing", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Standing",
          breakSecs: 180,
          standLimitSecs: 600,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    expect(bigNum.textContent).toContain("03:00");
    expect(bigNum.textContent).toContain("10:00");
  });

  it("away state has gray color scheme on progress bar", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Away",
          limitUsedSecs: 120,
          limitSecs: 600,
        })}
      />,
    );
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill.className).toContain("progress-bar__fill--gray");
  });

  it("shows previous session info when available", () => {
    render(
      <OneBarWidget
        {...props({
          previousSession: {
            state: "Standing",
            durationSecs: 960,
            wasEffective: true,
          },
        })}
      />,
    );
    expect(screen.getByText(/previously/)).toBeInTheDocument();
    expect(screen.getByText(/stood/)).toBeInTheDocument();
  });
});
