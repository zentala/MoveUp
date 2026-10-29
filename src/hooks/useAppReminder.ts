import { createContext, useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppReminderSnapshot } from "@/types";
import { DESK_APP_REMINDER } from "@/events";

export type ReminderCategory = NonNullable<AppReminderSnapshot["active"]>["category"];
type Flow = AppReminderSnapshot["protected_flow"];
type Action = "dismiss" | "snooze" | "retry";
export const AppReminderContext = createContext<{
  preview: (category: ReminderCategory) => Promise<AppReminderSnapshot>;
} | null>(null);

const connectionError = "Reminders are unavailable. Retry the connection.";
const actionError = "Could not save your response. Retry to finish.";

/** Main-only, event-driven presentation client. Rust owns all reasons and thresholds. */
export function useAppReminder(enabled: boolean) {
  const [snapshot, setSnapshot] = useState<AppReminderSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [ready, setReady] = useState(false);
  const [busy, setBusy] = useState(false);
  const [failedSaveId, setFailedSaveId] = useState<string | null>(null);
  const [protectedFlow, setProtectedFlow] = useState<Flow>(null);
  const [connectionAttempt, setConnectionAttempt] = useState(0);
  const current = useRef<AppReminderSnapshot | null>(null);
  const client = useRef(crypto.randomUUID());
  const generation = useRef(0);
  const desiredFlow = useRef<Flow>(null);
  const flowRequest = useRef(0);
  const sequence = useRef(0);
  const actionInFlight = useRef(false);
  const boot = useRef<Promise<void>>(Promise.resolve());

  const accept = useCallback((next: AppReminderSnapshot) => {
    if (!next || typeof next.session_id !== "string") throw new Error("invalid_reminder_snapshot");
    const previous = current.current;
    if (previous && (previous.session_id !== next.session_id || previous.revision >= next.revision)) return;
    current.current = next;
    setSnapshot(next);
  }, []);

  useEffect(() => {
    if (!enabled) return;
    const ownGeneration = ++generation.current;
    const alive = () => generation.current === ownGeneration;
    let unlisten: (() => void) | undefined;
    actionInFlight.current = false; setBusy(false);
    setReady(false);
    boot.current = (async () => {
      // Await registration before requesting any snapshot. An event can win either race.
      const dispose = await listen<AppReminderSnapshot>(DESK_APP_REMINDER, (event) => {
        if (alive()) accept(event.payload);
      });
      if (!alive()) { dispose(); return; }
      unlisten = dispose;
      for (let attempt = 0; attempt < 2; attempt++) {
        const observed = await invoke<AppReminderSnapshot>("get_app_reminder_snapshot");
        if (!alive()) return;
        accept(observed);
        try {
          const claimed = await invoke<AppReminderSnapshot>("claim_app_reminder_client", {
            previousClientId: current.current?.client_id ?? null, clientId: client.current,
          });
          if (!alive()) return;
          accept(claimed);
          // Remount reports its actual current flow; claim alone never clears protected edits.
          const flow = await invoke<AppReminderSnapshot>("set_app_reminder_flow", {
            clientId: client.current, sequence: ++sequence.current, flow: desiredFlow.current,
          });
          if (!alive()) return;
          accept(flow); setReady(true); setError(null);
          return;
        } catch (cause) {
          if (!String(cause).includes("stale_client") || attempt === 1) throw cause;
        }
      }
    })();
    // Keep rejection observable to callers without leaving an unhandled promise.
    void boot.current.catch(() => { if (alive()) setError(connectionError); });
    return () => { generation.current++; unlisten?.(); };
  }, [enabled, connectionAttempt, accept]);

  const protect = useCallback(async (flow: Flow) => {
    if (!enabled) return;
    const request = ++flowRequest.current;
    const previous = desiredFlow.current;
    desiredFlow.current = flow;
    // Enter protection synchronously before any await or render can replace a dialog.
    if (flow !== null) setProtectedFlow(flow);
    const ownGeneration = generation.current;
    try {
      await boot.current;
      if (generation.current !== ownGeneration) throw new Error("client_replaced");
      const response = await invoke<AppReminderSnapshot>("set_app_reminder_flow", {
        clientId: client.current, sequence: ++sequence.current, flow,
      });
      if (generation.current !== ownGeneration || request !== flowRequest.current) throw new Error("superseded_view");
      accept(response);
      if (request === flowRequest.current) { setProtectedFlow(flow); setError(null); }
    } catch (cause) {
      if (request === flowRequest.current && generation.current === ownGeneration) {
        desiredFlow.current = previous; setProtectedFlow(previous);
        setError("Could not protect this view. Your current view is unchanged. Retry.");
      }
      throw cause;
    }
  }, [enabled, accept]);

  const acknowledge = useCallback(async (action: Action) => {
    const message = current.current?.active;
    if (!enabled || !message || actionInFlight.current) return;
    actionInFlight.current = true; setBusy(true);
    const ownGeneration = generation.current;
    try {
      await boot.current;
      if (generation.current !== ownGeneration) return;
      const latest = current.current!;
      const response = await invoke<AppReminderSnapshot>("acknowledge_app_reminder", {
        clientId: client.current, id: message.id, revision: latest.revision, action,
      });
      if (generation.current === ownGeneration) { accept(response); setError(null); setFailedSaveId(null); }
    } catch (cause) {
      if (generation.current !== ownGeneration) return;
      setError(actionError);
      if (String(cause).includes("reminder_persistence_failed")) setFailedSaveId(message.id);
      try {
        const response = await invoke<AppReminderSnapshot>("get_app_reminder_snapshot");
        if (generation.current === ownGeneration) accept(response);
      } catch { /* Preserve the action error and current message for a deliberate retry. */ }
      if (current.current?.active?.id !== message.id) { setError(null); setFailedSaveId(null); }
    } finally {
      if (generation.current === ownGeneration) { actionInFlight.current = false; setBusy(false); }
    }
  }, [enabled, accept]);

  const preview = useCallback(async (category: ReminderCategory) => {
    if (!enabled) throw new Error("main_window_only");
    await boot.current;
    const ownGeneration = generation.current;
    const response = await invoke<AppReminderSnapshot>("trigger_test_notification", { category });
    if (generation.current === ownGeneration) accept(response);
    return response;
  }, [enabled, accept]);

  return { snapshot, ready, busy, protectedFlow, error: error ?? (snapshot?.presentation_error
    ? snapshot.presentation_error === "reminder_persistence_failed" ? actionError : "The reminder could not be presented. Retry the connection." : null),
    retrySave: snapshot?.presentation_error === "reminder_persistence_failed" || (!!snapshot?.active && failedSaveId === snapshot.active.id),
    protect, acknowledge, preview, retryConnection: () => setConnectionAttempt(value => value + 1) };
}
