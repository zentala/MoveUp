import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { waitFor } from "@testing-library/react";

const { renderRoot, createRoot, logError } = vi.hoisted(() => {
  const renderRoot = vi.fn();
  return { renderRoot, createRoot: vi.fn(() => ({ render: renderRoot })), logError: vi.fn() };
});
vi.mock("react-dom/client", () => ({ createRoot }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: logError }));

describe("early application bootstrap", () => {
  let listenerCalls: Array<[string, EventListenerOrEventListenerObject]>;

  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    logError.mockResolvedValue(undefined);
    delete window.__MOVEUP_UI_DIAGNOSTICS__;
    document.body.innerHTML = '<div id="root"></div>';
    listenerCalls = vi.spyOn(window, "addEventListener").mock.calls as unknown as typeof listenerCalls;
  });

  afterEach(() => {
    for (const [type, listener] of listenerCalls) {
      if (type === "error" || type === "unhandledrejection") {
        window.removeEventListener(type, listener);
      }
    }
    vi.doUnmock("./App");
    vi.restoreAllMocks();
    delete window.__MOVEUP_UI_DIAGNOSTICS__;
    document.body.innerHTML = "";
  });

  it("captures an App module evaluation failure after installing diagnostics", async () => {
    let snapshotAtEvaluation: unknown;
    let listenersAtEvaluation: string[] = [];
    vi.doMock("./App", () => {
      snapshotAtEvaluation = structuredClone(window.__MOVEUP_UI_DIAGNOSTICS__);
      listenersAtEvaluation = listenerCalls.map(([type]) => type);
      const error = new TypeError("PRIVATE module initialization");
      error.stack = `TypeError: PRIVATE module initialization\n    at ${window.location.origin}/src/App.tsx?token=SECRET:7:3`;
      throw error;
    });

    // A factory throwing while the real entry point imports App models a module
    // evaluation failure, rather than a missing test helper/export.
    await import("./main");
    await waitFor(() => expect(logError).toHaveBeenCalledOnce());

    expect(snapshotAtEvaluation).toMatchObject({ schema_version: 1, boot: "before_app", last_error: null });
    expect(listenersAtEvaluation).toEqual(expect.arrayContaining(["error", "unhandledrejection"]));
    // The test module loader may wrap the thrown TypeError in its own Error.
    // Both follow the same existing sanitizer; no private cause is traversed.
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({
      boot: "before_app", last_error: { source: "rejection", kind: expect.stringMatching(/^(?:TypeError|Error)$/) },
    });
    expect(JSON.stringify(window.__MOVEUP_UI_DIAGNOSTICS__)).not.toMatch(/PRIVATE|SECRET|https?:/);
    expect(createRoot).not.toHaveBeenCalled();
  });

  it("does not claim React mounted merely because App imported and render was scheduled", async () => {
    vi.doMock("./App", () => ({ default: () => null }));
    await import("./main");
    await waitFor(() => expect(renderRoot).toHaveBeenCalledOnce());
    expect(createRoot).toHaveBeenCalledWith(document.getElementById("root"));
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.boot).toBe("before_app");
    expect(logError).not.toHaveBeenCalled();
  });

  it("initializes capture even when the root element is missing", async () => {
    document.body.innerHTML = "";
    await expect(import("./main")).rejects.toThrow("Root element #root not found");
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.boot).toBe("before_app");
    expect(listenerCalls.map(([type]) => type)).toEqual(expect.arrayContaining(["error", "unhandledrejection"]));
  });
});
