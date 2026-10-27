/**
 * App.test.tsx — verifies App gates Tauri-only event listening behind
 * `isTauri`, so it does not throw in remote-display (plain browser) mode.
 *
 * `isTauri` is a module-level constant computed at import time from
 * `window.__TAURI_INTERNALS__`, so each scenario resets modules and
 * re-imports App with the window flag set (or absent) beforehand.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";

vi.mock("@/pages/AnalystLive", () => ({ default: () => <div>Live analyst</div> }));
vi.mock("@/pages/AnalystMockup", () => ({ default: () => <div>Analyst mockup</div> }));
vi.mock("@/pages/MockupGallery", () => ({ default: () => <div>Mockup gallery</div> }));
vi.mock("@/mockup/VoiceCaptureGallery", () => ({ default: () => <div>Voice mockup</div> }));
vi.mock("@/remote/PairScreen", () => ({ PairScreen: () => <div>Pair screen</div> }));

vi.mock("@/hooks/useWidgetData", () => ({
  useWidgetData: () => ({
    widgetProps: {},
    wsConnected: undefined,
    sensorConnected: false,
  }),
}));

vi.mock("@/widgets/registry", () => ({
  ActiveWidget: () => <div data-testid="active-widget" />,
}));

vi.mock("@/components/ConnectionOverlay", () => ({
  ConnectionOverlay: () => <div data-testid="connection-overlay" />,
}));

vi.mock("@/components/SettingsPanel", () => ({
  default: ({ onClose }: { onClose: () => void }) => <button data-testid="settings-panel" onClick={onClose}>Close settings</button>,
}));

describe("App", () => {
  beforeEach(() => {
    delete window.__MOVEUP_UI_DIAGNOSTICS__;
    window.location.hash = "";
    vi.resetModules();
    vi.clearAllMocks();
  });

  afterEach(() => {
    window.location.hash = "";
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("does not call listen() when not running inside Tauri", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    const { listen } = await import("@tauri-apps/api/event");
    const { default: App } = await import("./App");
    render(<App />);
    expect(vi.mocked(listen)).not.toHaveBeenCalled();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({ boot: "mounted", view: "widget" });
  });

  it("calls listen() for tray commands when running inside Tauri", async () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
      transformCallback: vi.fn(),
    };
    const { listen } = await import("@tauri-apps/api/event");
    const { default: App } = await import("./App");
    render(<App />);
    expect(vi.mocked(listen)).toHaveBeenCalledWith(
      "desk:show-widget",
      expect.any(Function),
    );
    expect(vi.mocked(listen)).toHaveBeenCalledWith(
      "desk:show-settings",
      expect.any(Function),
    );
  });

  it("publishes actual widget/settings transitions without exposing route contents", async () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    const { listen } = await import("@tauri-apps/api/event");
    const { default: App } = await import("./App");
    render(<App />);
    expect(screen.getByTestId("active-widget")).toBeInTheDocument();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({ boot: "mounted", view: "widget" });
    const showSettings = vi.mocked(listen).mock.calls.find(([event]) => event === "desk:show-settings")?.[1];
    expect(showSettings).toBeDefined();
    act(() => showSettings!({ event: "desk:show-settings", id: 1, payload: null }));
    expect(screen.getByTestId("settings-panel")).toBeInTheDocument();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.view).toBe("settings");
    fireEvent.click(screen.getByTestId("settings-panel"));
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.view).toBe("widget");
  });

  it.each([
    ["#/analyst", "Live analyst", "analyst"],
    ["#/mockup/analyst", "Analyst mockup", "other"],
    ["#/mockup/voice", "Voice mockup", "other"],
    ["#/mockup", "Mockup gallery", "other"],
    ["#/pair?d=PRIVATE&c=SECRET", "Pair screen", "other"],
  ])("preserves route %s and publishes only its view category", async (hash, content, view) => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    window.location.hash = hash;
    const { default: App } = await import("./App");
    await act(async () => { render(<App />); });
    expect(screen.getByText(content)).toBeInTheDocument();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({ boot: "mounted", view });
    expect(JSON.stringify(window.__MOVEUP_UI_DIAGNOSTICS__)).not.toMatch(/PRIVATE|SECRET|#/);
  });
});
