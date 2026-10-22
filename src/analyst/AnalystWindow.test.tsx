import { beforeEach, describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
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

const testCatalog: DataCatalog = {
  generated_at: "2026-05-16T00:00:00Z",
  sources: analystCatalogFixture.map((s) => ({
    id: s.id,
    name: s.name,
    kind: s.kind,
    location: s.location,
    retention: s.retention,
    fields: s.fields.map((f) => ({ name: f.name, type: f.type, description: "" })),
    sample_row: null,
    description: "",
  })),
};

function renderWindow() {
  return render(
    <AnalystWindow
      catalog={testCatalog}
      snapshots={analystSnapshotsFixture}
      sessions={analystSessionsFixture}
      kpis={analystDailyKpisFixture}
      defaultRange={analystDefaultRange}
    />,
  );
}

describe("AnalystWindow", { timeout: 20000 }, () => {
  beforeEach(() => {
    localStorage.removeItem("desk:analyst:range");
    localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
  });

  it("renders both tab triggers", () => {
    renderWindow();
    expect(screen.getByRole("tab", { name: /catalog/i })).toBeTruthy();
    expect(screen.getByRole("tab", { name: /explorer/i })).toBeTruthy();
  });

  it("defaults to the Explorer tab and switches to Catalog on click", () => {
    renderWindow();
    expect(screen.getByTestId("explorer-tab")).toBeTruthy();
    fireEvent.click(screen.getByRole("tab", { name: /catalog/i }));
    expect(screen.getByTestId("catalog-tab")).toBeTruthy();
    expect(screen.queryByTestId("explorer-tab")).toBeNull();
  });

  it("opens Explorer and retains the valid request pending a selected-day consumer", async () => {
    const request = createOpenDayRequest("2026-05-17", Date.now(), "pending-1");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(request));
    renderWindow();

    expect(await screen.findByTestId("explorer-tab")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
      expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBe(JSON.stringify(request));
    });
  });

  it("switches from Catalog to Explorer for a repeated same-day request", async () => {
    renderWindow();
    fireEvent.click(screen.getByRole("tab", { name: "Catalog" }));
    expect(screen.getByTestId("catalog-tab")).toBeInTheDocument();

    const request = createOpenDayRequest("2026-05-17", Date.now(), "repeat-1");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(request));
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY, newValue: JSON.stringify(request) }));

    await waitFor(() => {
      expect(screen.getByTestId("explorer-tab")).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: "Explorer" })).toHaveAttribute("aria-selected", "true");
      expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBe(JSON.stringify(request));
    });
  });

  it("ignores malformed or stale storage events and removes the listener on unmount", () => {
    const remove = vi.spyOn(window, "removeEventListener");
    const view = renderWindow();
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify({
      day: "2026-05-17",
      requestedAtMs: Date.now() - 10 * 60_000,
      requestId: "stale",
    }));
    fireEvent(window, new StorageEvent("storage", { key: OPEN_ANALYST_DAY_KEY }));
    expect(screen.getByTestId("explorer-tab")).toBeInTheDocument();
    expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBeNull();
    view.unmount();
    expect(remove).toHaveBeenCalledWith("storage", expect.any(Function));
    remove.mockRestore();
  });
});
