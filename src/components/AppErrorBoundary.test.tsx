import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import AppErrorBoundary from "./AppErrorBoundary";

const { logError } = vi.hoisted(() => ({ logError: vi.fn() }));

vi.mock("@tauri-apps/plugin-log", () => ({ error: logError }));

function BrokenPanel(): never {
  throw new TypeError("PRIVATE render message");
}

describe("AppErrorBoundary", () => {
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    delete window.__MOVEUP_UI_DIAGNOSTICS__;
    logError.mockReset().mockResolvedValue(undefined);
    consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
  });

  afterEach(() => {
    consoleError.mockRestore();
  });

  it("shows a fixed accessible fallback and logs only sanitized render details", () => {
    let renderEscapedBoundary = false;
    try {
      render(
        <AppErrorBoundary>
          <BrokenPanel />
        </AppErrorBoundary>,
      );
    } catch {
      renderEscapedBoundary = true;
    }

    expect(renderEscapedBoundary).toBe(false);
    expect(screen.getByRole("alert").textContent).toContain(
      "View unavailable.",
    );
    expect(screen.getByRole("alert").textContent).toBe("View unavailable.");
    expect(logError).toHaveBeenCalledOnce();
    const [message] = logError.mock.calls[0] as [string];
    expect(message).toMatch(/^UI_FAILURE source=render kind=TypeError/);
    expect(message).not.toContain("PRIVATE render message");
    expect(message).not.toContain("AppErrorBoundary.test.tsx?");
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({
      boot: "fallback", last_error: { source: "render", kind: "TypeError", location: null },
    });
  });

  it("keeps rendering children when they do not fail", () => {
    render(
      <AppErrorBoundary>
        <p>Popup content</p>
      </AppErrorBoundary>,
    );

    expect(screen.getByText("Popup content")).toBeTruthy();
    expect(logError).not.toHaveBeenCalled();
  });

  it("retains fallback evidence when plugin logging fails", async () => {
    logError.mockRejectedValue(new Error("PRIVATE logger failure"));
    render(<AppErrorBoundary><BrokenPanel /></AppErrorBoundary>);
    await Promise.resolve();
    expect(screen.getByRole("alert")).toHaveTextContent("View unavailable.");
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toMatchObject({
      boot: "fallback", last_error: { source: "render", kind: "TypeError" },
    });
    expect(JSON.stringify(window.__MOVEUP_UI_DIAGNOSTICS__)).not.toContain("PRIVATE");
  });
});
