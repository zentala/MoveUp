import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import type { LiveObservation } from "@/types";
import { initialDeskState, selectDeskView } from "./deskReducer";
import type { UseDeskConnection } from "./useDeskTypes";

const mockDesk = vi.hoisted(() => ({ current: null as unknown }));

vi.mock("@/hooks/useDeskAuto", () => ({
  useDeskAuto: () => mockDesk.current,
}));

import { useWidgetData } from "./useWidgetData";

describe("useWidgetData", () => {
  beforeEach(() => {
    const liveObservation: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: "2026-10-25T08:00:00Z",
        observed_through: "2026-10-25T08:04:00Z",
      },
      last_observation_at: "2026-10-25T08:04:00Z",
      snapshot_at: "2026-10-25T08:09:01Z",
      quality: null,
      reason: "stale",
    };
    const state = {
      ...initialDeskState,
      connected: true,
      liveObservation,
      lastBreakCreditFacts: [{
        break_started_at: "2026-10-25T08:00:00Z",
        observed_break_secs: 300,
        minimum_break_secs: 60,
        multiplier: 2,
        requested_credit_secs: 600,
        prior_applied_credit_secs: 0,
        sitting_before_secs: 480,
        applied_credit_secs: 480,
        sitting_after_secs: 0,
        applied_at: "2026-10-25T08:05:00Z",
        no_credit_reason: null,
      }],
      dashboardRefresh: {
        loading: false,
        lastSuccessAt: "2026-10-25T08:09:02Z",
        error: "temporary refresh failure",
      },
      historyRefresh: {
        loading: false,
        lastSuccessAt: "2026-10-25T08:09:02Z",
        error: null,
      },
    };
    mockDesk.current = {
      ...selectDeskView(state),
      observationQuality: null,
      deskOnline: true,
      capabilities: { control: true },
      calibrate: vi.fn(),
      setSitLimit: vi.fn(),
      setStandLimit: vi.fn(),
    } satisfies UseDeskConnection;
  });

  it("passes producer freshness and independent view receipts to widget consumers", () => {
    const { result, rerender } = renderHook(() => useWidgetData(() => undefined));

    expect(result.current.sensorConnected).toBe(true);
    expect(result.current.observationQuality).toBeNull();
    expect(result.current.liveObservation?.reason).toBe("stale");
    expect(result.current.widgetProps.liveObservation).toEqual(result.current.liveObservation);
    expect(result.current.widgetProps.lastBreakCreditFacts).toEqual(
      (mockDesk.current as UseDeskConnection).lastBreakCreditFacts,
    );
    expect(result.current.widgetProps.dashboardRefresh).toEqual(result.current.dashboardRefresh);
    expect(result.current.dashboardRefresh).toEqual({
      loading: false,
      lastSuccessAt: "2026-10-25T08:09:02Z",
      error: "temporary refresh failure",
    });
    expect(result.current.historyRefresh.error).toBeNull();

    const updatedObservation: LiveObservation = {
      span: {
        state: "Standing",
        started_at: "2026-10-25T08:10:00Z",
        observed_through: "2026-10-25T08:11:00Z",
      },
      last_observation_at: "2026-10-25T08:11:00Z",
      snapshot_at: "2026-10-25T08:11:00Z",
      quality: "known",
      reason: null,
    };
    act(() => {
      mockDesk.current = {
        ...(mockDesk.current as UseDeskConnection),
        liveObservation: updatedObservation,
        dashboardRefresh: { loading: true, lastSuccessAt: "2026-10-25T08:09:02Z", error: null },
      } satisfies UseDeskConnection;
      rerender();
    });
    expect(result.current.widgetProps.liveObservation).toEqual(updatedObservation);
    expect(result.current.widgetProps.dashboardRefresh).toEqual({
      loading: true,
      lastSuccessAt: "2026-10-25T08:09:02Z",
      error: null,
    });
  });

  it("uses the saved activity preference and applies a successful settings-save update", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ show_activity_status: false } as never);
    const { result } = renderHook(() => useWidgetData(() => undefined));

    await waitFor(() => expect(result.current.widgetProps.showActivityStatus).toBe(false));

    act(() => {
      window.dispatchEvent(new CustomEvent("moveup:settings-saved", {
        detail: { show_activity_status: true },
      }));
    });
    expect(result.current.widgetProps.showActivityStatus).toBe(true);
  });

  it.each(["stale success", "stale failure"] as const)("keeps a newer saved preference over a %s initial settings read", async (outcome) => {
    let resolveRead!: (value: { show_activity_status: boolean }) => void;
    let rejectRead!: (reason: Error) => void;
    const initialRead = new Promise<{ show_activity_status: boolean }>((resolve, reject) => {
      resolveRead = resolve;
      rejectRead = reject;
    });
    vi.mocked(invoke).mockReturnValueOnce(initialRead as never);
    const { result, unmount } = renderHook(() => useWidgetData(() => undefined));

    act(() => {
      window.dispatchEvent(new CustomEvent("moveup:settings-saved", {
        detail: { show_activity_status: true },
      }));
    });
    expect(result.current.widgetProps.showActivityStatus).toBe(true);

    await act(async () => {
      if (outcome === "stale success") resolveRead({ show_activity_status: false });
      else rejectRead(new Error("stale read failed"));
      await initialRead.catch(() => undefined);
    });
    expect(result.current.widgetProps.showActivityStatus).toBe(true);
    unmount();
  });
});
