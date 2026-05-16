import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AnalystWindow } from "./AnalystWindow";
import {
  analystCatalogFixture,
  analystSnapshotsFixture,
  analystSessionsFixture,
  analystDailyKpisFixture,
  analystDefaultRange,
} from "@/test/analyst-fixtures";

function renderWindow() {
  return render(
    <AnalystWindow
      sources={analystCatalogFixture}
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
