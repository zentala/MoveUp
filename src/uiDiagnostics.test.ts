import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { startUiDiagnostics } from "./uiDiagnostics";

const { logError } = vi.hoisted(() => ({ logError: vi.fn() }));

vi.mock("@tauri-apps/plugin-log", () => ({ error: logError }));

type RegisteredHandlers = {
  stop: () => void;
  onError: (event: ErrorEvent) => void;
  onRejection: (event: PromiseRejectionEvent) => void;
};

const cleanups: Array<() => void> = [];

function startDiagnostics(): RegisteredHandlers {
  const addListener = vi.spyOn(window, "addEventListener");
  const stop = startUiDiagnostics();
  const calls = addListener.mock.calls as unknown as Array<
    [string, EventListenerOrEventListenerObject]
  >;
  const handlerFor = (type: string): EventListener => {
    const registration = calls.find(([eventType]) => eventType === type);
    if (!registration) throw new Error(`Missing ${type} listener`);
    const listener = registration[1];
    return typeof listener === "function"
      ? listener
      : listener.handleEvent.bind(listener);
  };
  cleanups.push(stop);
  return {
    stop,
    onError: handlerFor("error") as (event: ErrorEvent) => void,
    onRejection: handlerFor("unhandledrejection") as (
      event: PromiseRejectionEvent,
    ) => void,
  };
}

function errorEvent(options: Partial<ErrorEvent>): ErrorEvent {
  return Object.assign(new Event("error"), options) as ErrorEvent;
}

function rejectionEvent(reason: unknown): PromiseRejectionEvent {
  return Object.assign(new Event("unhandledrejection"), {
    reason,
  }) as PromiseRejectionEvent;
}

describe("UI diagnostics", () => {
  beforeEach(() => {
    delete window.__MOVEUP_UI_DIAGNOSTICS__;
    logError.mockReset().mockResolvedValue(undefined);
  });

  afterEach(() => {
    cleanups.splice(0).forEach((stop) => stop());
    vi.restoreAllMocks();
  });

  it("logs a recognized same-origin source frame without raw error data", () => {
    const handlers = startDiagnostics();
    const error = new TypeError("PRIVATE message with token");
    error.stack = `TypeError: PRIVATE message with token\n    at render (${window.location.origin}/src/Popup.tsx?token=SECRET:12:8)`;

    handlers.onError(
      errorEvent({
        error,
        filename: `${window.location.origin}/api/private.js?token=SECRET`,
        lineno: 4,
        colno: 6,
      }),
    );

    expect(logError).toHaveBeenCalledWith(
      "UI_FAILURE source=window kind=TypeError module=Popup.tsx line=12 column=8",
    );
    const [message] = logError.mock.calls[0] as [string];
    expect(message).not.toContain("PRIVATE");
    expect(message).not.toContain("SECRET");
    expect(message).not.toContain("http://");
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.last_error).toEqual({
      source: "window", kind: "TypeError",
      location: { module: "Popup.tsx", line: 12, column: 8 },
    });
  });

  it("captures rejection location while discarding its message and query", () => {
    const handlers = startDiagnostics();
    const reason = new TypeError("PRIVATE rejection");
    reason.stack = `TypeError: PRIVATE rejection\n    at ${window.location.origin}/assets/rejection-chunk.js?token=SECRET:22:4`;

    handlers.onRejection(rejectionEvent(reason));

    expect(logError).toHaveBeenCalledWith(
      "UI_FAILURE source=rejection kind=TypeError module=rejection-chunk.js line=22 column=4",
    );
    const [message] = logError.mock.calls[0] as [string];
    expect(message).not.toContain("PRIVATE");
    expect(message).not.toContain("SECRET");
    expect(message).not.toContain("http://");
  });

  it("omits foreign locations and untrusted error names", () => {
    const handlers = startDiagnostics();
    const error = new Error("PRIVATE message");
    error.name = "SECRET_TOKEN";
    error.stack = `SECRET_TOKEN: https://other.example/private.js:3:9`;

    handlers.onError(
      errorEvent({
        error,
        filename: "https://other.example/private.js?secret=SECRET",
        lineno: 3,
        colno: 9,
      }),
    );

    expect(logError).toHaveBeenCalledWith("UI_FAILURE source=window kind=Error");
  });

  it("does not parse a URL in the error header or message as a frame", () => {
    const handlers = startDiagnostics();
    const error = new Error(
      `PRIVATE ${window.location.origin}/src/mentioned.js:8:2`,
    );
    error.stack = `Error: PRIVATE ${window.location.origin}/src/mentioned.js:8:2`;

    handlers.onError(
      errorEvent({
        error,
        filename: `${window.location.origin}/src/Popup.tsx`,
        lineno: 8,
        colno: 2,
      }),
    );

    expect(logError).toHaveBeenCalledWith("UI_FAILURE source=window kind=Error");
  });

  it("skips frame-shaped lines in a multiline message and keeps the following frame", () => {
    const handlers = startDiagnostics();
    const error = new Error(
      `prefix\n    at fn (${window.location.origin}/assets/private.js:8:2)`,
    );
    error.stack = `Error: ${error.message}\n    at render (${window.location.origin}/src/Popup.tsx:10:4)`;

    handlers.onError(errorEvent({ error }));

    expect(logError).toHaveBeenCalledWith(
      "UI_FAILURE source=window kind=Error module=Popup.tsx line=10 column=4",
    );
  });

  it("omits same-origin JavaScript paths outside source and asset routes", () => {
    const handlers = startDiagnostics();
    const error = new Error("PRIVATE");
    error.stack = `Error: PRIVATE\n    at load (${window.location.origin}/api/private.js:8:2)`;

    handlers.onError(errorEvent({ error }));

    expect(logError).toHaveBeenCalledWith("UI_FAILURE source=window kind=Error");
  });

  it("does not recurse or throw when the existing logger rejects", async () => {
    logError.mockRejectedValueOnce(new Error("PRIVATE logger failure"));
    const handlers = startDiagnostics();

    expect(() =>
      handlers.onError(
        errorEvent({
          error: new Error("PRIVATE render failure"),
          filename: `${window.location.origin}/src/Popup.tsx`,
        }),
      ),
    ).not.toThrow();

    await Promise.resolve();
    expect(logError).toHaveBeenCalledOnce();
  });

  it("does not throw when the existing logger throws synchronously", () => {
    logError.mockImplementationOnce(() => {
      throw new Error("PRIVATE logger failure");
    });
    const handlers = startDiagnostics();

    expect(() =>
      handlers.onError(errorEvent({ error: new Error("PRIVATE") })),
    ).not.toThrow();
    expect(logError).toHaveBeenCalledOnce();
  });

  it("removes both listeners when stopped", () => {
    const handlers = startDiagnostics();
    const removeListener = vi.spyOn(window, "removeEventListener");

    handlers.stop();

    expect(removeListener).toHaveBeenCalledWith("error", handlers.onError);
    expect(removeListener).toHaveBeenCalledWith(
      "unhandledrejection",
      handlers.onRejection,
    );
  });

  it("initializes exactly the v1 fields without inventing completed requests", () => {
    startDiagnostics();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toEqual({
      schema_version: 1, boot: "before_app", view: "other", last_error: null,
      queries: {
        dashboard: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
        today_summary: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
      },
    });
  });

  it.each(["throw", "reject"])("retains a safe error before the logger can %s", async (mode) => {
    const handlers = startDiagnostics();
    logError.mockImplementationOnce(() => {
      expect(window.__MOVEUP_UI_DIAGNOSTICS__?.last_error).toEqual({
        source: "window", kind: "Error", location: null,
      });
      if (mode === "throw") throw new Error("PRIVATE sink");
      return Promise.reject(new Error("PRIVATE sink"));
    });
    handlers.onError(errorEvent({ error: new Error("PRIVATE user data") }));
    await Promise.resolve();
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.last_error).toEqual({
      source: "window", kind: "Error", location: null,
    });
    expect(logError).toHaveBeenCalledOnce();
  });
});
