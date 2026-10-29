import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback } from "@tauri-apps/api/event";
import { useAppReminder } from "./useAppReminder";
import type { AppReminderSnapshot } from "@/types";

const message: NonNullable<AppReminderSnapshot["active"]> = { id: "session:1", source: "policy", reason: "sitting_limit",
  category: "urgent", title: "Time to stand up", body: "Policy text", is_test: false, elapsed_secs: 319, limit_secs: 1200 };
const empty = (): AppReminderSnapshot => ({ session_id: "session", revision: 0, active: null, pending: null,
  protected_flow: null, last_retirement: null, presentation_error: null, client_id: null });
let backend: AppReminderSnapshot;
let eventHandler: EventCallback<AppReminderSnapshot>;
const dispose = vi.fn();
function emit(next: AppReminderSnapshot) { act(() => eventHandler({ event: "desk:app-reminder", id: 1, payload: next })); }
beforeEach(() => {
  backend = empty(); vi.mocked(invoke).mockReset(); vi.mocked(listen).mockReset(); dispose.mockClear();
  vi.mocked(listen).mockImplementation(async (_event, callback) => { eventHandler = callback as EventCallback<AppReminderSnapshot>; return dispose; });
  vi.mocked(invoke).mockImplementation(async (command, raw) => {
    const args = raw as Record<string, unknown>;
    if (command === "get_app_reminder_snapshot") return structuredClone(backend);
    if (command === "claim_app_reminder_client") backend = { ...backend, revision: backend.revision + 1, client_id: args.clientId as string };
    else if (command === "set_app_reminder_flow") backend = { ...backend, revision: backend.revision + 1, protected_flow: args.flow as AppReminderSnapshot["protected_flow"] };
    else if (command === "acknowledge_app_reminder") backend = { ...backend, revision: backend.revision + 1,
      active: backend.pending, pending: null, presentation_error: null,
      last_retirement: { id: args.id as string, reason: "acknowledged" } };
    else if (command === "hide_main_window") return null;
    else throw new Error(`Unexpected IPC ${command}`);
    return structuredClone(backend);
  });
});

describe("main reminder transport", () => {
  it("registers listener before snapshot, lets newer events win, and ignores old revisions/sessions", async () => {
    let register!: (dispose: () => void) => void;
    vi.mocked(listen).mockImplementationOnce((_event, callback) => {
      eventHandler = callback as EventCallback<AppReminderSnapshot>;
      return new Promise(resolve => { register = resolve; });
    });
    const { result } = renderHook(() => useAppReminder(true));
    expect(invoke).not.toHaveBeenCalled();
    await act(async () => register(dispose));
    await waitFor(() => expect(result.current.ready).toBe(true));
    backend = { ...backend, revision: 10, active: message }; emit(backend);
    emit({ ...backend, revision: 9, active: null });
    emit({ ...backend, session_id: "retired-process", revision: 999, active: null });
    expect(result.current.snapshot?.active?.id).toBe(message.id);
  });

  it("does not overwrite an event with a delayed older snapshot", async () => {
    let resolveSnapshot!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { resolveSnapshot = resolve; }));
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(resolveSnapshot).toBeDefined());
    backend = { ...backend, revision: 10, active: message }; emit(backend);
    await act(async () => resolveSnapshot(empty()));
    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current.snapshot?.active?.id).toBe(message.id);
  });

  it("cleans pending subscriptions and makes no late claim after unmount", async () => {
    let resolveRegistration!: (dispose: () => void) => void;
    vi.mocked(listen).mockImplementationOnce(() => new Promise(resolve => { resolveRegistration = resolve; }));
    const { unmount } = renderHook(() => useAppReminder(true)); unmount();
    await act(async () => resolveRegistration(dispose));
    expect(dispose).toHaveBeenCalledOnce(); expect(invoke).not.toHaveBeenCalled();
  });

  it("recovers a failed save after remount with server-owned retry, without guessing the lost action", async () => {
    backend = { ...empty(), active: message, client_id: "old-client", presentation_error: "reminder_persistence_failed" };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current.error).toContain("Could not save"); expect(result.current.retrySave).toBe(true);
    await act(async () => result.current.acknowledge("retry"));
    expect(invoke).toHaveBeenCalledWith("acknowledge_app_reminder", expect.objectContaining({ id: message.id, action: "retry" }));
    expect(result.current.snapshot?.active).toBeNull(); expect(result.current.error).toBeNull();
  });

  it("retains the save error when recovery read fails and prevents duplicate actions", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    vi.mocked(invoke).mockRejectedValueOnce("reminder_persistence_failed").mockRejectedValueOnce("read failed");
    await act(async () => { await Promise.all([result.current.acknowledge("snooze"), result.current.acknowledge("snooze")]); });
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "acknowledge_app_reminder")).toHaveLength(1);
    expect(result.current.error).toContain("Could not save"); expect(result.current.retrySave).toBe(true);
    expect(result.current.snapshot?.active?.id).toBe(message.id);
  });

  it("persists the active reminder before hiding for its dedicated close action", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));

    await act(async () => result.current.closeReminder());

    const calls = vi.mocked(invoke).mock.calls.map(([command, args]) => [command, args]);
    const ack = calls.findIndex(([command, args]) => command === "acknowledge_app_reminder" && (args as { action?: string }).action === "dismiss");
    const hide = calls.findIndex(([command]) => command === "hide_main_window");
    expect(ack).toBeGreaterThan(-1);
    expect(hide).toBeGreaterThan(ack);
    expect(vi.mocked(invoke).mock.calls[hide]).toEqual(["hide_main_window", { reminderClose: {
      sessionId: "session", clientId: expect.any(String), reminderId: message.id, revision: expect.any(Number),
    } }]);
    expect(result.current.snapshot?.active).toBeNull();
    expect(result.current.closeError).toBeNull();
  });

  it("accepts a fresh X for a later reminder after an earlier close completed", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    await act(async () => result.current.closeReminder());

    const next = { ...message, id: "session:later", title: "Later reminder" };
    backend = { ...backend, revision: backend.revision + 1, active: next, last_retirement: null };
    emit(backend);
    expect(result.current.snapshot?.active?.id).toBe(next.id);
    await act(async () => result.current.closeReminder());

    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "acknowledge_app_reminder")).toHaveLength(2);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "hide_main_window")).toHaveLength(2);
  });

  it("retains close intent through a failed acknowledgement, then retries save before hiding", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    vi.mocked(invoke).mockRejectedValueOnce("reminder_persistence_failed");

    await act(async () => result.current.closeReminder());
    expect(result.current.retrySave).toBe(true);
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);

    await act(async () => result.current.acknowledge("retry"));
    expect(result.current.snapshot?.active).toBeNull();
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "acknowledge_app_reminder").map(([, args]) => (args as { action: string }).action))
      .toEqual(["dismiss", "retry"]);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "hide_main_window")).toHaveLength(1);
  });

  it("retries a rejected close request as Dismiss, not as reserved persistence recovery", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    vi.mocked(invoke).mockRejectedValueOnce(new Error("temporary transport failure"));

    await act(async () => result.current.closeReminder());
    expect(result.current.closeAckRetry).toBe(true);
    expect(result.current.retrySave).toBe(false);
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);

    await act(async () => result.current.retryCloseAcknowledgement());
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "acknowledge_app_reminder").map(([, args]) => (args as { action: string }).action))
      .toEqual(["dismiss", "dismiss"]);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "hide_main_window")).toHaveLength(1);
  });

  it("retries a failed post-acknowledgement hide without acknowledging again", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    vi.mocked(invoke).mockImplementation(async (command, raw) => {
      if (command === "hide_main_window") throw new Error("native hide failed");
      const args = raw as Record<string, unknown>;
      if (command === "acknowledge_app_reminder") backend = { ...backend, revision: backend.revision + 1, active: null };
      else if (command === "get_app_reminder_snapshot") return structuredClone(backend);
      else if (command === "claim_app_reminder_client") backend = { ...backend, revision: backend.revision + 1, client_id: args.clientId as string };
      else if (command === "set_app_reminder_flow") backend = { ...backend, revision: backend.revision + 1, protected_flow: args.flow as AppReminderSnapshot["protected_flow"] };
      else throw new Error(`Unexpected IPC ${command}`);
      return structuredClone(backend);
    });

    await act(async () => result.current.closeReminder());
    expect(result.current.closeError).toContain("could not be hidden");
    expect(result.current.snapshot?.active).toBeNull();
    await act(async () => result.current.retryClose());
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "acknowledge_app_reminder")).toHaveLength(1);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "hide_main_window")).toHaveLength(2);
  });

  it("cancels a pending close when a newer reminder arrives before the old acknowledgement completes", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let resolveAck!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementation((command) => command === "acknowledge_app_reminder"
      ? new Promise(resolve => { resolveAck = resolve; })
      : Promise.resolve(command === "get_app_reminder_snapshot" ? structuredClone(backend) : null));

    let closing!: Promise<void>;
    act(() => { closing = result.current.closeReminder(); });
    await waitFor(() => expect(resolveAck).toBeDefined());
    const newer = { ...message, id: "session:2", title: "Newer reminder" };
    backend = { ...backend, revision: 10, active: newer };
    emit(backend);
    await act(async () => { resolveAck({ ...empty(), revision: 9 }); await closing; });
    expect(result.current.snapshot?.active?.id).toBe(newer.id);
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it.each(["dismiss", "snooze"] as const)("keeps ordinary %s separate from hiding", async (action) => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    await act(async () => result.current.acknowledge(action));
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it("acknowledges the visible reminder but never hides a promoted queued reminder", async () => {
    const newer = { ...message, id: "session:queued", title: "Queued reminder" };
    backend = { ...empty(), active: message, pending: newer };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));

    await act(async () => result.current.closeReminder());

    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "acknowledge_app_reminder")).toBe(true);
    expect(result.current.snapshot?.active?.id).toBe(newer.id);
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it("does not hide when Settings protection starts while the close acknowledgement is pending", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let resolveAck!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementation((command, raw) => {
      if (command === "acknowledge_app_reminder") return new Promise(resolve => { resolveAck = resolve; });
      if (command === "set_app_reminder_flow") {
        backend = { ...backend, revision: backend.revision + 1, protected_flow: (raw as { flow: "settings" | null }).flow };
        return Promise.resolve(structuredClone(backend));
      }
      if (command === "get_app_reminder_snapshot") return Promise.resolve(structuredClone(backend));
      if (command === "claim_app_reminder_client") return Promise.resolve(structuredClone(backend));
      return Promise.resolve(null);
    });
    let closing!: Promise<void>;
    act(() => { closing = result.current.closeReminder(); });
    await waitFor(() => expect(resolveAck).toBeDefined());
    await act(async () => result.current.protect("settings"));
    await act(async () => { resolveAck({ ...backend, revision: backend.revision + 1, active: null }); await closing; });
    expect(result.current.protectedFlow).toBe("settings");
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it("does not restore a canceled close after a delayed recovery read completes", async () => {
    backend = { ...empty(), active: message };
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let rejectAck!: (reason: Error) => void;
    let resolveRecovery!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementation((command, raw) => {
      if (command === "acknowledge_app_reminder") return new Promise((_, reject) => { rejectAck = reject; });
      if (command === "get_app_reminder_snapshot") return new Promise(resolve => { resolveRecovery = resolve; });
      if (command === "set_app_reminder_flow") {
        backend = { ...backend, revision: backend.revision + 1, protected_flow: (raw as { flow: "settings" | null }).flow };
        return Promise.resolve(structuredClone(backend));
      }
      return Promise.resolve(null);
    });

    let closing!: Promise<void>;
    act(() => { closing = result.current.closeReminder(); });
    await waitFor(() => expect(rejectAck).toBeDefined());
    await act(async () => rejectAck(new Error("temporary transport failure")));
    await waitFor(() => expect(resolveRecovery).toBeDefined());
    await act(async () => result.current.protect("settings"));
    const staleAcknowledged = { ...backend, revision: backend.revision - 1, active: null,
      protected_flow: null, last_retirement: { id: message.id, reason: "acknowledged" as const } };
    await act(async () => { resolveRecovery(staleAcknowledged); await closing; });

    expect(result.current.protectedFlow).toBe("settings");
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it("does not dispatch late hide work when the owning client generation unmounts", async () => {
    backend = { ...empty(), active: message };
    const { result, unmount } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let resolveAck!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementation((command) => command === "acknowledge_app_reminder"
      ? new Promise(resolve => { resolveAck = resolve; })
      : Promise.resolve(command === "get_app_reminder_snapshot" ? structuredClone(backend) : null));
    let closing!: Promise<void>;
    act(() => { closing = result.current.closeReminder(); });
    await waitFor(() => expect(resolveAck).toBeDefined());
    unmount();
    await act(async () => { resolveAck({ ...empty(), revision: 10, last_retirement: { id: message.id, reason: "acknowledged" } }); await closing; });
    expect(vi.mocked(invoke).mock.calls.some(([command]) => command === "hide_main_window")).toBe(false);
  });

  it("enters protection before an asynchronous handshake and retains it on late events", async () => {
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let resolveFlow!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { resolveFlow = resolve; }));
    let protectedRequest!: Promise<void>;
    act(() => { protectedRequest = result.current.protect("share"); });
    expect(result.current.protectedFlow).toBe("share");
    await waitFor(() => expect(resolveFlow).toBeDefined());
    backend = { ...backend, revision: 10, active: message, protected_flow: "share" }; emit(backend);
    await act(async () => { resolveFlow(backend); await protectedRequest; });
    expect(result.current.protectedFlow).toBe("share");
  });

  it("never starts IPC or a listener outside the owning main surface", () => {
    renderHook(() => useAppReminder(false)); expect(listen).not.toHaveBeenCalled(); expect(invoke).not.toHaveBeenCalled();
  });

  it("rejects a superseded view request instead of committing an older view after the latest one", async () => {
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.ready).toBe(true));
    let first!: (snapshot: AppReminderSnapshot) => void, second!: (snapshot: AppReminderSnapshot) => void;
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { first = resolve; }))
      .mockImplementationOnce(() => new Promise(resolve => { second = resolve; }));
    let older!: Promise<void>, latest!: Promise<void>;
    act(() => { older = result.current.protect("settings"); latest = result.current.protect("share"); });
    const olderFailure = older.catch(cause => String(cause));
    await waitFor(() => expect(second).toBeDefined());
    await act(async () => { second({ ...backend, revision: 10, protected_flow: "share" }); await latest; });
    await act(async () => first({ ...backend, revision: 9, protected_flow: "settings" }));
    expect(await olderFailure).toContain("superseded_view"); expect(result.current.protectedFlow).toBe("share");
  });

  it("shows subscription failures and can reconnect without a timer or poller", async () => {
    vi.mocked(listen).mockRejectedValueOnce("bridge unavailable");
    const { result } = renderHook(() => useAppReminder(true));
    await waitFor(() => expect(result.current.error).toContain("Reminders are unavailable"));
    expect(invoke).not.toHaveBeenCalled();
    act(() => result.current.retryConnection());
    await waitFor(() => expect(result.current.ready).toBe(true)); expect(result.current.error).toBeNull();
  });
});
