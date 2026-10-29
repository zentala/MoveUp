import { createContext, useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppReminderSnapshot } from "@/types";
import { DESK_APP_REMINDER } from "@/events";

export type ReminderCategory = NonNullable<AppReminderSnapshot["active"]>["category"];
type Flow = AppReminderSnapshot["protected_flow"];
type Action = "dismiss" | "snooze" | "retry";
type CloseIntent = { id: string; sessionId: string; revision: number; generation: number; stage: "ack" | "hide" };
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
  const [closeError, setCloseError] = useState<string | null>(null);
  const [closeAckRetry, setCloseAckRetry] = useState(false);
  const [protectedFlow, setProtectedFlow] = useState<Flow>(null);
  const [connectionAttempt, setConnectionAttempt] = useState(0);
  const current = useRef<AppReminderSnapshot | null>(null);
  const client = useRef(crypto.randomUUID());
  const generation = useRef(0);
  const desiredFlow = useRef<Flow>(null);
  const flowRequest = useRef(0);
  const sequence = useRef(0);
  const actionInFlight = useRef(false);
  const closeIntent = useRef<CloseIntent | null>(null);
  const boot = useRef<Promise<void>>(Promise.resolve());

  const accept = useCallback((next: AppReminderSnapshot) => {
    if (!next || typeof next.session_id !== "string") throw new Error("invalid_reminder_snapshot");
    const previous = current.current;
    if (previous && (previous.session_id !== next.session_id || previous.revision >= next.revision)) return;
    const intent = closeIntent.current;
    if (intent && (next.session_id !== intent.sessionId || next.protected_flow !== null || next.pending !== null
      || (next.active !== null && next.active.id !== intent.id))) {
      closeIntent.current = null;
      setCloseError(null);
      setCloseAckRetry(false);
    }
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
    return () => { generation.current++; closeIntent.current = null; unlisten?.(); };
  }, [enabled, connectionAttempt, accept]);

  const protect = useCallback(async (flow: Flow) => {
    if (!enabled) return;
    const request = ++flowRequest.current;
    const previous = desiredFlow.current;
    desiredFlow.current = flow;
    // Enter protection synchronously before any await or render can replace a dialog.
    if (flow !== null) {
      closeIntent.current = null;
      setCloseError(null);
      setCloseAckRetry(false);
      setProtectedFlow(flow);
    }
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

  const finishClose = useCallback(async (response: AppReminderSnapshot | null, intent: CloseIntent) => {
    if (closeIntent.current !== intent || generation.current !== intent.generation || desiredFlow.current !== null) return;
    const latest = current.current;
    const expectedRevision = intent.stage === "hide" ? intent.revision : response?.revision;
    if (!response || response.session_id !== intent.sessionId || response.active !== null || response.pending !== null
      || response.protected_flow !== null || !latest || latest.session_id !== intent.sessionId
      || response.revision !== expectedRevision || latest.revision !== expectedRevision
      || latest.active !== null || latest.pending !== null || latest.protected_flow !== null) {
      closeIntent.current = null;
      setCloseError(null);
      setCloseAckRetry(false);
      return;
    }
    intent.stage = "hide";
    intent.revision = response.revision;
    try {
      await invoke("hide_main_window", { reminderClose: {
        sessionId: intent.sessionId, clientId: client.current, reminderId: intent.id, revision: intent.revision,
      } });
      const afterHide = current.current;
      if (closeIntent.current === intent && generation.current === intent.generation && desiredFlow.current === null
        && afterHide?.session_id === intent.sessionId && !afterHide.active && !afterHide.pending
        && !afterHide.protected_flow) {
        closeIntent.current = null;
        setCloseError(null);
        setCloseAckRetry(false);
      }
    } catch {
      if (closeIntent.current === intent && generation.current === intent.generation) {
        setCloseError("The reminder was dismissed, but the popup could not be hidden. Retry closing.");
      }
    }
  }, []);

  const acknowledge = useCallback(async (action: Action): Promise<AppReminderSnapshot | null> => {
    const initial = current.current;
    const message = initial?.active;
    if (!enabled || !message || !initial || actionInFlight.current) return null;
    actionInFlight.current = true; setBusy(true);
    const ownGeneration = generation.current;
    let response: AppReminderSnapshot | null = null;
    try {
      await boot.current;
      if (generation.current !== ownGeneration) return null;
      const latest = current.current;
      if (!latest || latest.session_id !== initial.session_id || latest.active?.id !== message.id) {
        const intent = closeIntent.current;
        if (intent?.stage === "ack" && intent.id === message.id && intent.generation === ownGeneration) {
          closeIntent.current = null;
          setCloseAckRetry(false); setCloseError(null);
        }
        return null;
      }
      const intent = closeIntent.current;
      if (intent?.stage === "ack" && intent.id === message.id && intent.generation === ownGeneration) intent.revision = latest.revision;
      response = await invoke<AppReminderSnapshot>("acknowledge_app_reminder", {
        clientId: client.current, id: message.id, revision: latest.revision, action,
      });
      if (generation.current === ownGeneration) {
        const close = closeIntent.current;
        if (action === "dismiss" && close?.stage === "ack" && close.id === message.id
          && response.session_id === close.sessionId && response.active?.id === message.id
          && response.pending === null && response.protected_flow === null) {
          accept(response);
          setError(actionError); setCloseAckRetry(true);
          response = null;
        } else {
          accept(response);
          setError(null); setFailedSaveId(null);
        }
      }
    } catch (cause) {
      if (generation.current !== ownGeneration) return null;
      setError(actionError);
      const intent = closeIntent.current;
      const persistenceFailure = String(cause).includes("reminder_persistence_failed");
      if (persistenceFailure) setFailedSaveId(message.id);
      try {
        const observed = await invoke<AppReminderSnapshot>("get_app_reminder_snapshot");
        if (generation.current === ownGeneration) {
          accept(observed);
          const ownsIntent = !!intent && closeIntent.current === intent && intent.generation === ownGeneration;
          const confirmedClose = !!intent && ownsIntent && intent.stage === "ack" && intent.id === message.id
            && observed.session_id === intent.sessionId && observed.active === null && observed.pending === null
            && observed.protected_flow === null && observed.last_retirement?.id === message.id
            && observed.last_retirement.reason === "acknowledged";
          if (confirmedClose) response = observed;
          else if (intent && ownsIntent && intent.stage === "ack" && intent.id === message.id && observed.active?.id === message.id) {
            if (!persistenceFailure) setCloseAckRetry(true);
          }
        }
      } catch { /* Preserve the action error and current message for a deliberate retry. */ }
      if (current.current?.active?.id !== message.id) { setError(null); setFailedSaveId(null); }
    } finally {
      if (generation.current === ownGeneration) { actionInFlight.current = false; setBusy(false); }
    }
    if (response && action === "retry") {
      const intent = closeIntent.current;
      if (intent?.stage === "ack" && intent.id === message.id && intent.generation === ownGeneration) {
        await finishClose(response, intent);
      }
    }
    return response;
  }, [enabled, accept, finishClose]);

  const closeReminder = useCallback(async () => {
    const message = current.current?.active;
    const latest = current.current;
    if (!enabled || !message || !latest || actionInFlight.current || desiredFlow.current !== null
      || latest.protected_flow !== null) return;
    const intent: CloseIntent = { id: message.id, sessionId: latest.session_id, revision: latest.revision,
      generation: generation.current, stage: "ack" };
    closeIntent.current = intent;
    setCloseError(null);
    setCloseAckRetry(false);
    const response = await acknowledge("dismiss");
    if (response) await finishClose(response, intent);
  }, [enabled, acknowledge, finishClose]);

  const retryCloseAcknowledgement = useCallback(async () => {
    const intent = closeIntent.current;
    if (!enabled || !intent || intent.stage !== "ack" || intent.generation !== generation.current) return;
    const response = await acknowledge("dismiss");
    if (response) await finishClose(response, intent);
  }, [enabled, acknowledge, finishClose]);

  const retryClose = useCallback(async () => {
    const intent = closeIntent.current;
    const latest = current.current;
    if (!enabled || !intent || intent.stage !== "hide" || !latest || actionInFlight.current) return;
    await finishClose(latest, intent);
  }, [enabled, finishClose]);

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
    protect, acknowledge, closeReminder, closeError, closeAckRetry, retryCloseAcknowledgement, retryClose, preview,
    retryConnection: () => setConnectionAttempt(value => value + 1) };
}
