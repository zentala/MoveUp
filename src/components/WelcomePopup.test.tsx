/**
 * WelcomePopup.test.tsx — Unit tests for the welcome popup component.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import { WelcomePopup } from "./WelcomePopup";

const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("WelcomePopup", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockInvoke.mockResolvedValue(undefined);
  });

  it("introduces sensor setup, real readings, calibration, balance and history in English", () => {
    render(<WelcomePopup />);
    expect(screen.getByRole("heading", { name: "Your first reading" })).toBeInTheDocument();
    expect(screen.getByText(/USB controller detection alone does not confirm that the assembled sensor sends readings/i)).toBeInTheDocument();
    expect(screen.getByText(/Move the desk and confirm that the height reading changes/i)).toBeInTheDocument();
    expect(screen.getByText(/Sitting balance includes break credit; history shows recorded elapsed time/i)).toBeInTheDocument();
    expect(screen.getByText(/without a reading, the state remains unknown/i)).toBeInTheDocument();
  });

  it("uses readable English first-run labels rather than escaped text", () => {
    const { container } = render(<WelcomePopup />);
    const text = container.textContent ?? "";
    expect(text).toContain("MOVEUP · FIRST RUN");
    expect(text).toContain("Don't show this at next launch");
    expect(text).not.toMatch(/\\u[0-9a-fA-F]{4}/);
  });

  it("opens MoveUp from the primary action with dontShowAgain false", async () => {
    render(<WelcomePopup />);
    fireEvent.click(screen.getByRole("button", { name: "Open MoveUp" }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("dismiss_welcome", {
        dontShowAgain: false,
      });
    });
  });

  it("dismiss with checkbox calls dismiss_welcome with dontShowAgain true", async () => {
    render(<WelcomePopup />);
    const checkbox = screen.getByRole("checkbox", { name: "Don't show this at next launch" });
    fireEvent.click(checkbox);
    fireEvent.click(screen.getByRole("button", { name: "Open MoveUp" }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("dismiss_welcome", {
        dontShowAgain: true,
      });
    });
  });

  it("test notification button calls trigger_test_notification", async () => {
    render(<WelcomePopup />);
    const testBtn = screen.getByRole("button", { name: "Test reminder" });
    fireEvent.click(testBtn);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("trigger_test_notification");
    });
  });

  it("shows notification status after test", async () => {
    mockInvoke.mockResolvedValueOnce({ active: { is_test: true }, pending: null, presentation_error: null });
    render(<WelcomePopup />);
    const testBtn = screen.getByRole("button", { name: "Test reminder" });
    fireEvent.click(testBtn);
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Test preview sent to the main window");
    });
  });

  it("announces preparation and queued delivery from the main-window reminder snapshot", async () => {
    let resolvePreview!: (snapshot: unknown) => void;
    mockInvoke.mockReturnValueOnce(new Promise((resolve) => { resolvePreview = resolve; }));
    render(<WelcomePopup />);

    const testBtn = screen.getByRole("button", { name: "Test reminder" });
    fireEvent.click(testBtn);

    expect(screen.getByRole("status")).toHaveTextContent("Preparing test preview");
    expect(testBtn).toBeDisabled();
    resolvePreview({ active: { is_test: false }, pending: { is_test: true }, presentation_error: null });

    expect(await screen.findByRole("status")).toHaveTextContent("Test preview queued in the main window");
    expect(testBtn).toBeEnabled();
  });

  it("shows a readable error and keeps first-run dismissal available when preview is rejected", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Rate limited: wait before testing again"));
    render(<WelcomePopup />);

    fireEvent.click(screen.getByRole("button", { name: "Test reminder" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Could not start test preview");
    expect(screen.getByRole("alert")).toHaveTextContent("Rate limited: wait before testing again");
    expect(screen.getByRole("button", { name: "Open MoveUp" })).toBeEnabled();
    const checkbox = screen.getByRole("checkbox", { name: "Don't show this at next launch" });
    fireEvent.click(checkbox);
    fireEvent.click(screen.getByRole("button", { name: "Open MoveUp" }));
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("dismiss_welcome", { dontShowAgain: true }));
  });

  it("keeps the Welcome action visible and retryable when opening MoveUp fails", async () => {
    mockInvoke.mockRejectedValueOnce("welcome_main_window_focus_failed").mockResolvedValueOnce(undefined);
    render(<WelcomePopup />);
    const openMoveUp = screen.getByRole("button", { name: "Open MoveUp" });

    fireEvent.click(openMoveUp);
    expect(await screen.findByRole("alert")).toHaveTextContent("MoveUp could not be focused. Try again.");
    expect(openMoveUp).toBeEnabled();

    fireEvent.click(openMoveUp);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(2));
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("has data-tauri-drag-region for draggability", () => {
    const { container } = render(<WelcomePopup />);
    const dragRegion = container.querySelector("[data-tauri-drag-region]");
    expect(dragRegion).toBeTruthy();
  });

  it("keeps first-run controls outside the only scrollable region at 480×420", () => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 480 });
    Object.defineProperty(window, "innerHeight", { configurable: true, value: 420 });
    render(<WelcomePopup />);

    const shell = screen.getByTestId("welcome-shell");
    const content = screen.getByTestId("welcome-content");
    const footer = screen.getByTestId("welcome-footer");

    expect(shell.style.height).toBe("100vh");
    expect(shell.style.overflow).toBe("hidden");
    expect(content.style.overflowY).toBe("auto");
    expect(content.style.minHeight).toBe("0px");
    expect(footer.style.flexShrink).toBe("0");
    expect(shell.contains(content)).toBe(true);
    expect(shell.contains(footer)).toBe(true);
    expect(content.contains(footer)).toBe(false);
    expect(within(footer).getByRole("checkbox")).toBeInTheDocument();
    expect(within(footer).getByRole("button", { name: "Test reminder" })).toBeInTheDocument();
    expect(within(footer).getByRole("button", { name: "Open MoveUp" })).toBeInTheDocument();
  });
});
