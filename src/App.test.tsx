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

  it("recovers a main-window reminder from the authoritative snapshot", async () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    };
    const { invoke } = await import("@tauri-apps/api/core");
    const message = { id: "session:1", source: "policy", reason: "sitting_limit", category: "urgent",
      title: "Time to stand up", body: "A policy-approved reason", is_test: false, elapsed_secs: 319, limit_secs: 1200 };
    vi.mocked(invoke).mockImplementation(async (command) => command === "get_overlay_state" ? null : {
      session_id: "session", revision: command === "get_app_reminder_snapshot" ? 1 : 2,
      active: message, pending: null, protected_flow: null, last_retirement: null,
      presentation_error: null, client_id: null,
    });
    const { default: App } = await import("./App");
    render(<App />);
    expect(await screen.findByRole("heading", { name: "Time to stand up" })).toBeInTheDocument();
    expect(screen.getByText("A policy-approved reason")).toBeInTheDocument();
  });

  it("reconnects a ready active notice with a presentation error without acknowledging or snoozing it", async () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
      metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    };
    const { invoke } = await import("@tauri-apps/api/core");
    const snapshot = { session_id: "presentation", revision: 0, active: { id: "presentation:1", source: "policy",
      reason: "sitting_limit", category: "warning", title: "Current reminder", body: "A policy-approved reason",
      is_test: false, elapsed_secs: 319, limit_secs: 1200 }, pending: null, protected_flow: null,
      last_retirement: null, presentation_error: "main_show_not_visible" as string | null, client_id: null as string | null };
    let flows = 0;
    vi.mocked(invoke).mockImplementation(async (command, raw) => {
      if (command === "get_overlay_state") return null;
      if (command === "claim_app_reminder_client") {
        snapshot.client_id = (raw as { clientId: string }).clientId; snapshot.revision++;
      }
      if (command === "set_app_reminder_flow") {
        snapshot.revision++; if (++flows > 1) snapshot.presentation_error = null;
      }
      return structuredClone(snapshot);
    });
    const { default: App } = await import("./App"); render(<App />);
    expect(await screen.findByRole("heading", { name: "Current reminder" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Retry the connection");
    fireEvent.click(screen.getByRole("button", { name: "Retry connection" }));
    await act(async () => { await new Promise(resolve => setTimeout(resolve, 0)); });
    expect(await screen.findByRole("heading", { name: "Current reminder" })).toBeInTheDocument();
    expect(snapshot.active.id).toBe("presentation:1");
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "claim_app_reminder_client")).toHaveLength(2);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => /acknowledge_app_reminder|snooze/.test(command))).toHaveLength(0);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
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
