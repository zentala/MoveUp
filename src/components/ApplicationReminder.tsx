import { useEffect, useId } from "react";
import { AlertTriangle, CheckCircle2, Info, X, CircleAlert } from "lucide-react";
import { UiIcon } from "./UiIcon";
import type { AppReminderSnapshot } from "@/types";
import "./application-reminder.css";

type Message = NonNullable<AppReminderSnapshot["active"]>;
const categories = {
  info: { label: "Information", icon: Info }, warning: { label: "Warning", icon: AlertTriangle },
  urgent: { label: "Action needed", icon: CircleAlert }, success: { label: "Progress", icon: CheckCircle2 },
};

export function ApplicationReminder({ message, busy, error, retrySave, onAction, onReconnect }: {
  message: Message; busy: boolean; error: string | null; retrySave: boolean;
  onAction: (action: "dismiss" | "snooze" | "retry") => void;
  onReconnect?: () => void;
}) {
  const headingId = useId(), bodyId = useId();
  const category = categories[message.category];
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
        disabled={busy || retrySave} onClick={() => onAction("dismiss")}><UiIcon icon={X} /></button>
      <h1 id={headingId}>{message.title}</h1>
    </header>
    <div className="application-reminder__body" role="region" aria-label="Reminder details" tabIndex={0}>
      {error && <p role="alert" className="application-reminder__error">{error}</p>}
      <p id={bodyId}>{message.body}</p>
      {message.elapsed_secs !== null && message.limit_secs !== null && <p className="application-reminder__values">
        {Math.floor(message.elapsed_secs / 60)}m {message.elapsed_secs % 60}s / {Math.floor(message.limit_secs / 60)}m {message.limit_secs % 60}s
      </p>}
    </div>
    <footer className={`application-reminder__actions${onReconnect && !retrySave ? " application-reminder__actions--reconnect" : ""}`}>
      {onReconnect && !retrySave && <button type="button" className="btn application-reminder__reconnect"
        disabled={busy} onClick={onReconnect}>Retry connection</button>}
      {retrySave ? <button type="button" className="btn" disabled={busy} onClick={() => onAction("retry")}>Retry saving</button> : <>
        <button type="button" className="btn" disabled={busy} onClick={() => onAction("snooze")}>Snooze</button>
        <button type="button" className="btn" disabled={busy} onClick={() => onAction("dismiss")}>Dismiss</button>
      </>}
      <span role="status">{busy ? "Saving…" : message.is_test ? "Preview only" : "MoveUp reminder"}</span>
    </footer>
  </section>;
}
