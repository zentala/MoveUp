import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, it, expect, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ApplicationReminder } from "./ApplicationReminder";
import type { AppReminderSnapshot } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const message: NonNullable<AppReminderSnapshot["active"]> = { id: "session:1", source: "policy", reason: "sitting_limit",
  category: "urgent", title: "Time to stand up", body: "Your configured reminder", is_test: false, elapsed_secs: 319, limit_secs: 1200 };
describe("ApplicationReminder", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockResolvedValue({ current_dismiss_minutes: 11, dismiss_durations_mins: [11, 23], snooze_minutes: 11 });
  });
  it("renders real non-test halfway Praise without claiming the full standing goal was reached", () => {
    render(<ApplicationReminder message={{ ...message, source: "session", reason: "praise", category: "success",
      title: "Halfway through your standing goal!", body: "Keep it up.", is_test: false,
      elapsed_secs: 450, limit_secs: 900 }} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    expect(screen.getByRole("heading", { name: "Halfway through your standing goal!" })).toBeInTheDocument();
    expect(screen.getByText("Progress")).toBeInTheDocument(); expect(screen.getByText("7m 30s / 15m 0s")).toBeInTheDocument();
    expect(screen.queryByText(/Goal reached|Test ·/)).not.toBeInTheDocument();
    return waitFor(() => expect(screen.getByText(/At opening: Snooze 11 min/i)).toBeInTheDocument());
  });
  it("shows the live nondefault delay and distinguishes this dismissal from the next one", async () => {
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    await waitFor(() => expect(screen.getByText(/At opening: Snooze 11 min; Dismiss step stays/i)).toBeInTheDocument());
    expect(screen.getByText(/Dismiss\/Close acknowledges, waits 11 min; later Dismiss uses next delay/i)).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledWith("get_active_profiles");
  });
  it("labels unsupported Snooze truthfully and keeps the current Dismiss delay", async () => {
    vi.mocked(invoke).mockResolvedValue({ current_dismiss_minutes: 0, dismiss_durations_mins: [0, 23], snooze_minutes: null });
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    await waitFor(() => expect(screen.getByText(/Snooze unavailable/i)).toBeInTheDocument());
    expect(screen.getByText(/Dismiss\/Close acknowledges, waits 0 min; Future Dismiss uses the next delay/i)).toBeInTheDocument();
  });
  it("reports timing unavailable when the live policy read fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("policy unavailable"));
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    await waitFor(() => expect(screen.getByText(/Current reminder timing is unavailable/i)).toBeInTheDocument());
  });
  it("keeps preview actions explicitly session and policy neutral", () => {
    render(<ApplicationReminder message={{ ...message, is_test: true }} busy={false} error={null} retrySave={false} onAction={vi.fn()} />);
    expect(screen.getByText(/Snooze or Dismiss closes this preview only; it does not change reminder policy or your session/i)).toBeInTheDocument();
  });
  it("routes the header X to its dedicated close path while footer Dismiss stays an ordinary acknowledgement", () => {
    const action = vi.fn(), close = vi.fn();
    const hasFocus = vi.spyOn(document, "hasFocus").mockReturnValue(true);
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={action} onClose={close} />);
    fireEvent.click(screen.getByRole("button", { name: "Close reminder" }));
    expect(close).toHaveBeenCalledOnce();
    expect(action).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(action).toHaveBeenCalledWith("dismiss");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(action).toHaveBeenCalledTimes(2);
    expect(close).toHaveBeenCalledOnce();
    hasFocus.mockRestore();
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
    return waitFor(() => expect(screen.getByText(/At opening: Snooze 11 min/i)).toBeInTheDocument());
  });
  it("handles Escape only in the focused window and never autofocuses", async () => {
    const action = vi.fn(); const focus = vi.spyOn(document, "hasFocus").mockReturnValue(false);
    render(<ApplicationReminder message={message} busy={false} error={null} retrySave={false} onAction={action} />);
    expect(document.activeElement).toBe(document.body);
    fireEvent.keyDown(window, { key: "Escape" }); expect(action).not.toHaveBeenCalled();
    focus.mockReturnValue(true); fireEvent.keyDown(window, { key: "Escape" }); expect(action).toHaveBeenCalledOnce();
    focus.mockRestore();
    await waitFor(() => expect(screen.getByText(/At opening: Snooze 11 min/i)).toBeInTheDocument());
  });
});
