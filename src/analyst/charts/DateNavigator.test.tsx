import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { TimelineInterval } from "@/timeline/intervals";
import { DateNavigator } from "./DateNavigator";

vi.mock("../hooks/useLocalDayBoundsRange", () => ({
  useLocalDayBoundsRange: (from: string, to: string) => {
    const rows: Array<{ date_local: string; start_utc: string; end_utc: string; zone_basis: string; historical_source_zone: string }> = [];
    const day = new Date(`${from}T00:00:00`);
    const last = new Date(`${to}T00:00:00`);
    while (day <= last) {
      const date_local = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
      const start = new Date(day.getTime());
      const end = new Date(day.getTime());
      end.setDate(end.getDate() + 1);
      rows.push({ date_local, start_utc: start.toISOString(), end_utc: end.toISOString(), zone_basis: "current-query-local", historical_source_zone: "unknown" });
      day.setDate(day.getDate() + 1);
    }
    return { status: "ready", data: rows, error: null, refetch: () => undefined, lastRefreshed: null };
  },
}));

const range = { from: "2026-05-15", to: "2026-05-17" };
const firstDayStart = new Date("2026-05-15T00:00:00").getTime();
const intervals: TimelineInterval[] = [
  { startMs: firstDayStart + 8 * 3_600_000, endMs: firstDayStart + 9 * 3_600_000, state: "Standing", provenance: "session" },
];

class ResizeObserverMock {
  static latest: ResizeObserverMock | null = null;
  constructor(private readonly callback: ResizeObserverCallback) { ResizeObserverMock.latest = this; }
  observe(): void {}
  disconnect(): void {}
  unobserve(): void {}
  trigger(): void { this.callback([], this as unknown as ResizeObserver); }
}

afterEach(() => { vi.unstubAllGlobals(); ResizeObserverMock.latest = null; });

describe("DateNavigator", () => {
  it("renders one composition tab and ring for every date in range", () => {
    render(<DateNavigator range={range} selectedDay="2026-05-17" intervals={intervals} panelId="selected-day-panel"
      nowMs={new Date(2026, 4, 16, 12).getTime()} onSelectDay={() => undefined} />);
    expect(screen.getByRole("tab", { name: /2026-05-15/ })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /2026-05-16/ })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /2026-05-17/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByText("Today")).toBeInTheDocument();
    const ringLabel = screen.getByTestId("day-ring-2026-05-16").getAttribute("aria-label") ?? "";
    expect(ringLabel).toContain("Unknown 50.00 percent");
    expect(ringLabel).toContain("Future 50.00 percent");
    expect(screen.getByTestId("day-cols").querySelectorAll('[role="tab"]')).toHaveLength(3);
  });

  it("selects a clicked date through the actual nav callback", () => {
    const onSelectDay = vi.fn();
    render(<DateNavigator range={range} selectedDay="2026-05-17" intervals={intervals} panelId="selected-day-panel"
      nowMs={Date.parse("2026-05-18T00:00:00Z")} onSelectDay={onSelectDay} />);
    fireEvent.click(screen.getByRole("tab", { name: /2026-05-16/ }));
    expect(onSelectDay).toHaveBeenCalledWith("2026-05-16");
  });

  it("reserves the same tab width while composition changes from loading to known coverage", () => {
    const fullDayStart = new Date("2026-05-17T00:00:00").getTime();
    const fullDayInterval: TimelineInterval = {
      startMs: fullDayStart,
      endMs: new Date("2026-05-18T00:00:00").getTime(),
      state: "Standing",
      provenance: "session",
    };
    const props = { range: { from: "2026-05-17", to: "2026-05-17" }, selectedDay: "2026-05-17",
      intervals: [fullDayInterval], panelId: "selected-day-panel", nowMs: Date.parse("2026-05-18T00:00:00Z"),
      onSelectDay: () => undefined };
    const { rerender } = render(<DateNavigator {...props} busy />);
    const tab = screen.getByRole("tab", { name: /2026-05-17/ });
    const loadingCoverage = screen.getByText("Loading");
    expect(loadingCoverage).toHaveClass("day-tab__coverage--reserved");
    expect(tab).toHaveAccessibleName(/Composition loading; no duration asserted/);

    rerender(<DateNavigator {...props} busy={false} />);

    expect(screen.getByRole("tab", { name: /2026-05-17/ })).toHaveAccessibleName(/100\.0%/);
    const readyCoverage = screen.getByText("100.0% recorded coverage");
    expect(readyCoverage).toHaveClass("day-tab__coverage--reserved");
    expect(readyCoverage).toBeInTheDocument();
  });

  it("moves roving focus with arrows and Home/End without selecting", () => {
    const onSelectDay = vi.fn();
    render(<DateNavigator range={range} selectedDay="2026-05-16" intervals={intervals} panelId="selected-day-panel"
      nowMs={Date.parse("2026-05-18T00:00:00Z")} onSelectDay={onSelectDay} />);
    const middle = screen.getByRole("tab", { name: /2026-05-16/ });
    middle.focus();
    fireEvent.keyDown(middle, { key: "ArrowRight" });
    expect(screen.getByRole("tab", { name: /2026-05-17/ })).toHaveFocus();
    expect(onSelectDay).not.toHaveBeenCalled();
    fireEvent.keyDown(screen.getByRole("tab", { name: /2026-05-17/ }), { key: "Home" });
    expect(screen.getByRole("tab", { name: /2026-05-15/ })).toHaveFocus();
    fireEvent.keyDown(screen.getByRole("tab", { name: /2026-05-15/ }), { key: "End" });
    expect(screen.getByRole("tab", { name: /2026-05-17/ })).toHaveFocus();
  });

  it("reveals a selected tab at the clipped left edge using viewport coordinates, then preserves visible scroll on resize", () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    render(<DateNavigator range={range} selectedDay="2026-05-17" intervals={intervals} panelId="selected-day-panel"
      nowMs={Date.parse("2026-05-18T00:00:00Z")} onSelectDay={() => undefined} />);
    const tabs = screen.getByTestId("day-cols");
    const selected = screen.getByRole("tab", { name: /2026-05-17/ });
    Object.defineProperty(tabs, "clientWidth", { configurable: true, value: 100 });
    Object.defineProperty(selected, "offsetWidth", { configurable: true, value: 88 });
    Object.defineProperty(tabs, "scrollLeft", { configurable: true, writable: true, value: 40 });
    vi.spyOn(tabs, "getBoundingClientRect").mockReturnValue({ left: 300, right: 400, top: 0, bottom: 120, width: 100, height: 120, x: 300, y: 0, toJSON: () => ({}) } as DOMRect);
    vi.spyOn(selected, "getBoundingClientRect").mockReturnValue({ left: 250, right: 338, top: 0, bottom: 100, width: 88, height: 100, x: 250, y: 0, toJSON: () => ({}) } as DOMRect);

    act(() => ResizeObserverMock.latest?.trigger());
    expect(tabs.scrollLeft).toBe(0);
    expect(selected).not.toHaveFocus();

    vi.spyOn(selected, "getBoundingClientRect").mockReturnValue({ left: 380, right: 468, top: 0, bottom: 100, width: 88, height: 100, x: 380, y: 0, toJSON: () => ({}) } as DOMRect);
    act(() => ResizeObserverMock.latest?.trigger());
    expect(tabs.scrollLeft).toBe(68);

    tabs.scrollLeft = 200;
    vi.spyOn(selected, "getBoundingClientRect").mockReturnValue({ left: 305, right: 393, top: 0, bottom: 100, width: 88, height: 100, x: 305, y: 0, toJSON: () => ({}) } as DOMRect);
    act(() => ResizeObserverMock.latest?.trigger());
    expect(tabs.scrollLeft).toBe(200);
  });
});
