import { fireEvent, render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ApplicationReminder } from "./ApplicationReminder";
import type { AppReminderSnapshot } from "@/types";

const message: NonNullable<AppReminderSnapshot["active"]> = { id: "session:1", source: "policy", reason: "sitting_limit",
  category: "urgent", title: "Time to stand up", body: "Your configured reminder", is_test: false, elapsed_secs: 319, limit_secs: 1200 };
describe("ApplicationReminder", () => {
  it("renders real non-test halfway Praise without claiming the full standing goal was reached", () => {
    render(<ApplicationReminder message={{ ...message, source: "session", reason: "praise", category: "success",
      title: "Halfway through your standing goal!", body: "Keep it up.", is_test: false,
      elapsed_secs: 450, limit_secs: 900 }} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    expect(screen.getByRole("heading", { name: "Halfway through your standing goal!" })).toBeInTheDocument();
    expect(screen.getByText("Progress")).toBeInTheDocument(); expect(screen.getByText("7m 30s / 15m 0s")).toBeInTheDocument();
    expect(screen.queryByText(/Goal reached|Test ·/)).not.toBeInTheDocument();
  });
  it.each(["info", "warning", "urgent", "success"] as const)("labels %s and uses only authoritative values", (category) => {
    const action = vi.fn();
    render(<ApplicationReminder message={{ ...message, category, is_test: true }} busy={false} error={null} retrySave={false} onAction={action} />);
    expect(screen.getByText(/Test ·/)).toBeInTheDocument();
    expect(screen.getByText("5m 19s / 20m 0s")).toBeInTheDocument();
    expect(action).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" })); expect(action).toHaveBeenCalledWith("dismiss");
  });
  it("keeps failed saves visible and sends only authoritative retry", () => {
    const action = vi.fn();
    render(<ApplicationReminder message={{ ...message, elapsed_secs: null, limit_secs: null }} busy={false}
      error="Could not save your response." retrySave onAction={action} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Could not save");
    expect(screen.getByRole("button", { name: "Close reminder" })).toBeDisabled();
    expect(screen.queryByRole("button", { name: "Retry connection" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry saving" })); expect(action).toHaveBeenCalledWith("retry");
    expect(screen.queryByText(/40m/)).not.toBeInTheDocument();
  });
  it("handles Escape only in the focused window and never autofocuses", () => {
    const action = vi.fn(); const focus = vi.spyOn(document, "hasFocus").mockReturnValue(false);
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={action} />);
    expect(document.activeElement).toBe(document.body);
    fireEvent.keyDown(window, { key: "Escape" }); expect(action).not.toHaveBeenCalled();
    focus.mockReturnValue(true); fireEvent.keyDown(window, { key: "Escape" }); expect(action).toHaveBeenCalledOnce();
    focus.mockRestore();
  });
});
