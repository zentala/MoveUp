/**
 * WelcomePopup.test.tsx — Unit tests for the welcome popup component.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
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

  it("renders heading and feature list", () => {
    render(<WelcomePopup />);
    expect(screen.getByText(/Jestem Twoim osobistym/)).toBeTruthy();
    expect(screen.getByText(/Pasek na/)).toBeTruthy();
    expect(screen.getByText(/przesiedzisz za/)).toBeTruthy();
    expect(screen.getByText(/nagradzam/)).toBeTruthy();
  });

  it("renders real Polish diacritic characters, not escape sequences", () => {
    const { container } = render(<WelcomePopup />);
    const text = container.textContent ?? "";
    expect(text).toContain("Cześć");
    expect(text).toContain("działam");
    expect(text).not.toMatch(/\\u[0-9a-fA-F]{4}/);
  });

  it("dismiss button calls dismiss_welcome with dontShowAgain false", async () => {
    render(<WelcomePopup />);
    const dismissBtn = screen.getByText(/Gotowy! Zaczynamy!/);
    fireEvent.click(dismissBtn);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("dismiss_welcome", {
        dontShowAgain: false,
      });
    });
  });

  it("dismiss with checkbox calls dismiss_welcome with dontShowAgain true", async () => {
    render(<WelcomePopup />);
    const checkbox = screen.getByRole("checkbox");
    fireEvent.click(checkbox);
    const dismissBtn = screen.getByText(/Gotowy! Zaczynamy!/);
    fireEvent.click(dismissBtn);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("dismiss_welcome", {
        dontShowAgain: true,
      });
    });
  });

  it("test notification button calls trigger_test_notification", async () => {
    render(<WelcomePopup />);
    const testBtn = screen.getByText(/Przetestuj powiadomienia/);
    fireEvent.click(testBtn);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("trigger_test_notification");
    });
  });

  it("shows notification status after test", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    render(<WelcomePopup />);
    const testBtn = screen.getByText(/Przetestuj powiadomienia/);
    fireEvent.click(testBtn);
    await waitFor(() => {
      expect(screen.getByText(/Sent!/)).toBeTruthy();
    });
  });

  it("has data-tauri-drag-region for draggability", () => {
    const { container } = render(<WelcomePopup />);
    const dragRegion = container.querySelector("[data-tauri-drag-region]");
    expect(dragRegion).toBeTruthy();
  });
});
