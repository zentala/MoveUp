import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import SettingsTabBar, { SETTINGS_TABS, type SettingsTabIndex } from "./SettingsTabBar";

function SettingsTabsHarness() {
  const [activeTab, setActiveTab] = useState<SettingsTabIndex>(0);
  return (
    <>
      <SettingsTabBar activeTab={activeTab} onTabChange={setActiveTab} />
      <div id="settings-panel" role="tabpanel" aria-labelledby={`settings-tab-${activeTab}`} tabIndex={0}>
        {SETTINGS_TABS[activeTab]}
      </div>
    </>
  );
}

describe("SettingsTabBar keyboard interaction", () => {
  it("roves focus with arrows and Home/End without selecting until activation", () => {
    render(<SettingsTabsHarness />);
    const profiles = screen.getByRole("tab", { name: "Profiles" });
    const calibration = screen.getByRole("tab", { name: "Calibr." });
    const debug = screen.getByRole("tab", { name: "Debug" });

    expect(profiles).toHaveAttribute("tabindex", "0");
    expect(calibration).toHaveAttribute("tabindex", "-1");
    profiles.focus();
    fireEvent.keyDown(profiles, { key: "ArrowRight" });
    expect(calibration).toHaveFocus();
    expect(calibration).toHaveAttribute("tabindex", "0");
    expect(profiles).toHaveAttribute("tabindex", "-1");
    expect(profiles).toHaveAttribute("aria-selected", "true");
    fireEvent.keyDown(calibration, { key: "End" });
    expect(debug).toHaveFocus();
    expect(profiles).toHaveAttribute("aria-selected", "true");
    fireEvent.click(debug);

    expect(debug).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tabpanel")).toHaveAttribute("aria-labelledby", debug.id);
    expect(debug).toHaveAttribute("aria-controls", "settings-panel");
  });

  it("keeps keyboard navigation within the tab range", () => {
    render(<SettingsTabsHarness />);
    const profiles = screen.getByRole("tab", { name: "Profiles" });
    const debug = screen.getByRole("tab", { name: "Debug" });
    profiles.focus();
    fireEvent.keyDown(profiles, { key: "ArrowLeft" });
    expect(profiles).toHaveFocus();
    debug.focus();
    fireEvent.keyDown(debug, { key: "ArrowRight" });
    expect(debug).toHaveFocus();
  });
});
