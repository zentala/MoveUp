import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
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
});
