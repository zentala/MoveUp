import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AnalystWindow } from "./AnalystWindow";
import type { DataCatalog } from "./types/catalog";
import {
  analystCatalogFixture,
  analystSnapshotsFixture,
  analystSessionsFixture,
  analystDailyKpisFixture,
  analystDefaultRange,
} from "@/test/analyst-fixtures";

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
});
