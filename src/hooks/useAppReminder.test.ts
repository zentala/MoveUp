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
    else if (command === "acknowledge_app_reminder") backend = { ...backend, revision: backend.revision + 1, active: null, presentation_error: null };
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
