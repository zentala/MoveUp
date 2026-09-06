/**
 * App.test.tsx — verifies App gates Tauri-only event listening behind
 * `isTauri`, so it does not throw in remote-display (plain browser) mode.
 *
 * `isTauri` is a module-level constant computed at import time from
 * `window.__TAURI_INTERNALS__`, so each scenario resets modules and
 * re-imports App with the window flag set (or absent) beforehand.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render } from "@testing-library/react";

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
  default: () => <div data-testid="settings-panel" />,
}));

describe("App", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
  });

  afterEach(() => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("does not call listen() when not running inside Tauri", async () => {
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    const { listen } = await import("@tauri-apps/api/event");
    const { default: App } = await import("./App");
    render(<App />);
    expect(vi.mocked(listen)).not.toHaveBeenCalled();
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
});
