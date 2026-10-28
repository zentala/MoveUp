import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AnalystWindow } from "./AnalystWindow";
import type { DataCatalog } from "./types/catalog";
import {
  analystCatalogFixture,
  analystSnapshotsFixture,
  analystSessionsFixture,
  analystDailyKpisFixture,
  analystDefaultRange,
} from "@/test/analyst-fixtures";
import { createOpenDayRequest, OPEN_ANALYST_DAY_KEY } from "./open-day";
import type { LiveObservation } from "@/generated/LiveObservation";
import type { QueryStatus } from "@/hooks/useDeskTypes";

const testCatalog: DataCatalog = {
  generated_at: "2026-05-16T00:00:00Z",
  sources: analystCatalogFixture.map((source) => ({
    id: source.id, name: source.name, kind: source.kind, location: source.location, retention: source.retention,
    fields: source.fields.map((field) => ({ name: field.name, type: field.type, description: "" })),
    sample_row: null, description: "",
  })),
};

function renderWindow(overrides: { liveObservation?: LiveObservation | null; dashboardRefresh?: QueryStatus } = {}) {
  return render(<AnalystWindow catalog={testCatalog} snapshots={analystSnapshotsFixture} sessions={analystSessionsFixture}
    kpis={analystDailyKpisFixture} defaultRange={analystDefaultRange} {...overrides} />);
}

describe("AnalystWindow selected-day receiver", () => {
  beforeEach(() => {
    localStorage.removeItem("desk:analyst:range");
    localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
  });

  it("keeps tabs linked and uses manual arrow/Home/End focus before activation", () => {
    renderWindow();
    const explorer = screen.getByRole("tab", { name: "Explorer" });
    const catalog = screen.getByRole("tab", { name: "Catalog" });
    expect(explorer).toHaveAttribute("aria-controls", "analyst-panel-explorer");
    expect(document.getElementById("analyst-panel-explorer")).toHaveAttribute("aria-labelledby", "analyst-tab-explorer");
    fireEvent.keyDown(explorer, { key: "ArrowLeft" });
    expect(catalog).toHaveFocus();
    expect(catalog).toHaveAttribute("aria-selected", "false");
    fireEvent.keyDown(catalog, { key: "End" });
    expect(explorer).toHaveFocus();
    expect(explorer).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(explorer, { key: "ArrowLeft" });
    fireEvent.keyDown(catalog, { key: "Enter" });
    expect(catalog).toHaveAttribute("aria-selected", "true");
  });

  it("shows both app tabs and opens Explorer from Catalog on a valid request", async () => {
    renderWindow();
    expect(screen.getByRole("tab", { name: "Catalog" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
    fireEvent.click(screen.getByRole("tab", { name: "Catalog" }));
    const request = createOpenDayRequest("2026-05-17", Date.now(), "catalog-day");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(request));
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(request) }));
    expect(await screen.findByRole("region", { name: "Daily timeline for 2026-05-17" })).toBeInTheDocument();
    await waitFor(() => expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull());
    expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
  });

  it("relays live observation and independent view refresh updates into Explorer status", () => {
    const now = Date.now();
    const knownThrough = new Date(now - 30_000).toISOString();
    const known: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: new Date(now - 90_000).toISOString(),
        observed_through: knownThrough,
      },
      last_observation_at: knownThrough,
      snapshot_at: knownThrough,
      quality: "known",
      reason: null,
    };
    const stale: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: new Date(now - 11 * 60_000).toISOString(),
        observed_through: new Date(now - 10 * 60_000).toISOString(),
      },
      last_observation_at: new Date(now - 10 * 60_000).toISOString(),
      snapshot_at: new Date(now - 5 * 60_000).toISOString(),
      quality: null,
      reason: "stale",
    };
    const response = (lastSuccessAt: string | null, error: string | null): QueryStatus => ({
      loading: false, lastSuccessAt, error,
    });
    const view = renderWindow({ liveObservation: known, dashboardRefresh: response(new Date(now - 28_000).toISOString(), null) });
    const status = screen.getByTestId("live-observation-status");
    expect(status).toHaveTextContent("Observed Sitting through");
    expect(status).toHaveTextContent("View response received");

    view.rerender(<AnalystWindow catalog={testCatalog} snapshots={analystSnapshotsFixture} sessions={analystSessionsFixture}
      kpis={analystDailyKpisFixture} defaultRange={analystDefaultRange} liveObservation={stale}
      dashboardRefresh={response(new Date(now - 28_000).toISOString(), "temporary read failure")} />);
    expect(status).toHaveTextContent("Last confirmed Sitting through");
    expect(status).toHaveTextContent("sensor observation stale");
    expect(status).toHaveTextContent("View update failed: temporary read failure");
    expect(status).toHaveTextContent("last view response received");

    view.rerender(<AnalystWindow catalog={testCatalog} snapshots={analystSnapshotsFixture} sessions={analystSessionsFixture}
      kpis={analystDailyKpisFixture} defaultRange={analystDefaultRange} liveObservation={null}
      dashboardRefresh={response(null, "offline")} />);
    expect(status).toHaveTextContent("Current observation unavailable");
    expect(status).toHaveTextContent("View update failed: offline");
  });

  it("explicitly selects an in-range historical request, independent of the 02:00 initializer", async () => {
    const request = createOpenDayRequest("2026-05-16", Date.now(), "historical-in-range");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(request));
    renderWindow();
    expect(await screen.findByRole("region", { name: "Daily timeline for 2026-05-16" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: /2026-05-16/ })).toHaveAttribute("aria-selected", "true");
    await waitFor(() => expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull());
  });

  it("consumes a repeated same-day request with a distinct request ID", async () => {
    renderWindow();
    const first = createOpenDayRequest("2026-05-17", Date.now(), "repeat-one");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(first));
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(first) }));
    await waitFor(() => expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull());
    const second = createOpenDayRequest("2026-05-17", Date.now(), "repeat-two");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(second));
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(second) }));
    await waitFor(() => expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull());
    expect(screen.getByRole("region", { name: "Daily timeline for 2026-05-17" })).toBeInTheDocument();
  });

  it("processes a newer request that arrives before the older pending request is acknowledged", async () => {
    renderWindow();
    const older = createOpenDayRequest("2026-05-16", Date.now(), "older");
    const newer = createOpenDayRequest("2026-05-17", Date.now(), "newer");
    act(() => {
      localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(older));
      fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(older) }));
      localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(newer));
      fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(newer) }));
    });
    expect(await screen.findByRole("region", { name: "Daily timeline for 2026-05-17" })).toBeInTheDocument();
    await waitFor(() => expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull());
    expect(screen.getByRole("tab", { name: /2026-05-17/ })).toHaveAttribute("aria-selected", "true");
  });

  it("does not acknowledge or clear a valid request if storage access fails", () => {
    const request = createOpenDayRequest("2026-05-16", Date.now(), "storage-denied");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(request));
    const original = Storage.prototype.getItem;
    const read = vi.spyOn(Storage.prototype, "getItem").mockImplementation(function (this: Storage, key: string) {
      if (key === OPEN_ANALYST_DAY_KEY) throw new DOMException("Storage blocked", "SecurityError");
      return original.call(this, key);
    });
    renderWindow();
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(request) }));
    expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
    read.mockRestore();
    expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBe(JSON.stringify(request));
  });

  it("ignores stale requests and removes the storage listener on unmount", () => {
    const remove = vi.spyOn(window, "removeEventListener");
    const view = renderWindow();
    const stale = JSON.stringify({ day: "2026-05-17", requestedAtMs: Date.now() - 10 * 60_000, requestId: "stale" });
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, stale);
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: stale }));
    expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull();
    view.unmount();
    expect(remove).toHaveBeenCalledWith("storage", expect.any(Function));
    remove.mockRestore();
  });
});
