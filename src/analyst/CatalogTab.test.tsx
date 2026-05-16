/**
 * CatalogTab.test.tsx — Unit tests for the Catalog tab.
 *
 * Verifies:
 * - Renders rows from explicit `data` prop (fixture / mockup path)
 * - Without `data` prop: invokes `get_data_catalog` once, shows loading first,
 *   then transitions to the table.
 * - Surfaces error card when the invoke rejects.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { CatalogTab } from "./CatalogTab";
import type { DataCatalog } from "./types/catalog";

const CATALOG: DataCatalog = {
  generated_at: "2026-05-16T12:00:00Z",
  sources: [
    {
      id: "sensor",
      name: "VL53L1X distance sensor",
      kind: "serial",
      location: "COM3 @ 115200",
      retention: "stream",
      fields: [{ name: "mm", type: "u16", description: "distance in mm" }],
      sample_row: null,
      description: "Distance sensor.",
    },
    {
      id: "sessions",
      name: "Session history",
      kind: "sqlite",
      location: "{app_data}/desk.db",
      retention: "no limit",
      fields: [{ name: "id", type: "INTEGER PK", description: "row id" }],
      sample_row: null,
      description: "Session table.",
    },
  ],
};

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("CatalogTab", () => {
  it("renders rows from explicit data prop without invoking", async () => {
    render(<CatalogTab data={CATALOG} />);

    expect(screen.getByTestId("catalog-tab")).toBeInTheDocument();
    expect(screen.getByText("VL53L1X distance sensor")).toBeInTheDocument();
    expect(screen.getByText("Session history")).toBeInTheDocument();
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("calls invoke once and shows loading first when no data prop", async () => {
    let resolveFn: (v: DataCatalog) => void = () => {};
    const pending = new Promise<DataCatalog>((res) => {
      resolveFn = res;
    });
    vi.mocked(invoke).mockReturnValueOnce(pending);

    render(<CatalogTab />);

    expect(screen.getByTestId("catalog-tab-loading")).toBeInTheDocument();
    expect(vi.mocked(invoke)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_data_catalog");

    resolveFn(CATALOG);
    await waitFor(() => expect(screen.getByTestId("catalog-tab")).toBeInTheDocument());
    expect(screen.getByText("VL53L1X distance sensor")).toBeInTheDocument();
  });

  it("renders error card when invoke rejects", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("kaboom"));

    render(<CatalogTab />);

    await waitFor(() =>
      expect(screen.getByTestId("catalog-tab-error")).toBeInTheDocument(),
    );
    expect(screen.getByTestId("catalog-tab-error")).toHaveTextContent("kaboom");
  });
});
