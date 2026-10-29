import { useEffect, useId, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle, CheckCircle2, Info, X, CircleAlert } from "lucide-react";
import { UiIcon } from "./UiIcon";
import type { AppReminderSnapshot } from "@/types";
import "./application-reminder.css";

type Message = NonNullable<AppReminderSnapshot["active"]>;
type ReminderTiming = { current_dismiss_minutes: number; dismiss_durations_mins: number[]; snooze_minutes: number | null };
const categories = {
  info: { label: "Information", icon: Info }, warning: { label: "Warning", icon: AlertTriangle },
  urgent: { label: "Action needed", icon: CircleAlert }, success: { label: "Progress", icon: CheckCircle2 },
};

export function ApplicationReminder({ message, busy, error, retrySave, closeAckRetry, onAction, onClose, onCloseRetry, onReconnect }: {
  message: Message; busy: boolean; error: string | null; retrySave: boolean;
  closeAckRetry?: boolean;
  onAction: (action: "dismiss" | "snooze" | "retry") => void;
  onClose?: () => void;
  onCloseRetry?: () => void;
  onReconnect?: () => void;
}) {
  const headingId = useId(), bodyId = useId();
  const [timing, setTiming] = useState<{ reminderId: string; value: ReminderTiming | null } | null>(null);
  useEffect(() => {
    let mounted = true;
    if (message.is_test) return () => { mounted = false; };
    void invoke<ReminderTiming | null>("get_active_profiles")
      .then(value => {
        if (!mounted) return;
        const valid = value !== null && Number.isInteger(value.current_dismiss_minutes)
          && Array.isArray(value.dismiss_durations_mins) && value.dismiss_durations_mins.every(Number.isInteger)
          && (value.snooze_minutes === null || Number.isInteger(value.snooze_minutes));
        setTiming({ reminderId: message.id, value: valid ? value : null });
      })
      .catch(() => { if (mounted) setTiming({ reminderId: message.id, value: null }); });
    return () => { mounted = false; };
  }, [message.id, message.is_test]);
  const category = categories[message.category];
  const currentTiming = timing?.reminderId === message.id ? timing.value : undefined;
  const futureDismissTiming = currentTiming && currentTiming.dismiss_durations_mins.length > 1
    ? "Future Dismiss uses the next delay; last repeats."
    : "This delay repeats.";
  useEffect(() => {
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !event.defaultPrevented && document.hasFocus() && !busy && !retrySave) {
        event.preventDefault(); onAction("dismiss");
      }
    };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [onAction, busy, retrySave]);
  return <section className={`application-reminder application-reminder--${message.category}`} aria-labelledby={headingId}
    aria-describedby={bodyId} aria-live="polite" data-testid="application-reminder">
    <header className="application-reminder__header">
      <div className="application-reminder__category"><UiIcon icon={category.icon} />
        <span>{message.is_test ? `Test · ${category.label}` : category.label}</span></div>
      <button type="button" className="btn application-reminder__close" aria-label="Close reminder"
        disabled={busy || retrySave} onClick={() => onClose ? onClose() : onAction("dismiss")}><UiIcon icon={X} /></button>
      <h1 id={headingId}>{message.title}</h1>
    </header>
    <div className="application-reminder__body" role="region" aria-label="Reminder details" tabIndex={0}
      onKeyDown={event => {
        if (event.altKey || event.ctrlKey || event.metaKey) return;
        const body = event.currentTarget;
        const maxScroll = Math.max(0, body.scrollHeight - body.clientHeight);
        const arrowStep = Math.max(1, Math.min(40, body.clientHeight - 1));
        let nextScroll: number | null = null;
        switch (event.key) {
          case "ArrowDown": nextScroll = body.scrollTop + arrowStep; break;
          case "ArrowUp": nextScroll = body.scrollTop - arrowStep; break;
          case "PageDown": nextScroll = body.scrollTop + body.clientHeight; break;
          case "PageUp": nextScroll = body.scrollTop - body.clientHeight; break;
          case "Home": nextScroll = 0; break;
          case "End": nextScroll = maxScroll; break;
        }
        if (nextScroll !== null) {
          body.scrollTop = Math.min(maxScroll, Math.max(0, nextScroll));
          event.preventDefault();
        }
      }}>
      {error && <p role="alert" className="application-reminder__error">{error}</p>}
      <p id={bodyId}>{message.body}</p>
      <p className="application-reminder__action-help">
        {message.is_test
          ? "Snooze or Dismiss closes this preview only; it does not change reminder policy or your session."
          : currentTiming === undefined ? "Reading the current reminder timing…"
          : currentTiming === null ? "Current reminder timing is unavailable. Snooze and Dismiss delays could not be confirmed."
          : currentTiming.snooze_minutes === null
            ? `At opening: Snooze unavailable. Dismiss/Close acknowledges, waits ${currentTiming.current_dismiss_minutes} min; ${futureDismissTiming}`
            : `At opening: Snooze ${currentTiming.snooze_minutes} min; Dismiss step stays. Dismiss/Close acknowledges, waits ${currentTiming.current_dismiss_minutes} min; later Dismiss uses next delay, last repeats.`}
      </p>
      {message.elapsed_secs !== null && message.limit_secs !== null && <p className="application-reminder__values">
        {Math.floor(message.elapsed_secs / 60)}m {message.elapsed_secs % 60}s / {Math.floor(message.limit_secs / 60)}m {message.limit_secs % 60}s
      </p>}
    </div>
    <footer className={`application-reminder__actions${onReconnect && !retrySave ? " application-reminder__actions--reconnect" : ""}`}>
      {onReconnect && !retrySave && <button type="button" className="btn application-reminder__reconnect"
        disabled={busy} onClick={onReconnect}>Retry connection</button>}
      {retrySave ? <button type="button" className="btn" disabled={busy} onClick={() => onAction("retry")}>Retry saving</button>
        : closeAckRetry && onCloseRetry ? <button type="button" className="btn" disabled={busy} onClick={onCloseRetry}>Retry dismissal</button> : <>
        <button type="button" className="btn" disabled={busy} onClick={() => onAction("snooze")}>Snooze</button>
        <button type="button" className="btn" disabled={busy} onClick={() => onAction("dismiss")}>Dismiss</button>
      </>}
      <span role="status">{busy ? "Saving…" : message.is_test ? "Preview only" : "MoveUp reminder"}</span>
    </footer>
  </section>;
}
