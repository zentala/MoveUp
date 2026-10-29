/**
 * NotificationsSection.tsx — effective communication-profile settings.
 */
import { useContext, useEffect, useRef, useState, type FC } from "react";
import ProfileSelector from "./ProfileSelector";
import { AppReminderContext, type ReminderCategory } from "@/hooks/useAppReminder";
import "../application-reminder.css";

const NotificationsSection: FC = () => {
  const reminder = useContext(AppReminderContext);
  const [category, setCategory] = useState<ReminderCategory>("info");
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const statusRef = useRef<HTMLParagraphElement>(null);
  useEffect(() => {
    // This follows an explicit Test click inside the existing Settings scroller;
    // it never focuses a window or changes the settings draft.
    if (status) statusRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [status]);
  async function preview() {
    if (!reminder || busy) return;
    setBusy(true); setStatus(null);
    try {
      const snapshot = await reminder.preview(category);
      setStatus(snapshot.protected_flow || snapshot.pending
        ? "Test queued. Return to status to view it."
        : "Test reminder ready. Your session is unchanged.");
    } catch (cause) {
      setStatus(String(cause).includes("Rate limited")
        ? "Wait 60 seconds between tests (all appearances)."
        : "Could not prepare the test reminder. Please retry.");
    } finally { setBusy(false); }
  }
  return <>
    <ProfileSelector type="communication" label="Communication Profile" />
    <section className="reminder-preview" aria-label="Test reminder">
      <div className="reminder-preview__controls">
        <label htmlFor="reminder-appearance">Test appearance</label>
        <select id="reminder-appearance" value={category} onChange={event => setCategory(event.target.value as ReminderCategory)}>
          <option value="info">Information</option><option value="warning">Warning</option>
          <option value="urgent">Urgent</option><option value="success">Success</option>
        </select>
        <button type="button" className="btn" disabled={!reminder || busy} onClick={() => { void preview(); }}>Test reminder</button>
      </div>
      {status && <p ref={statusRef} role="status">{status}</p>}
    </section>
  </>;
};

export default NotificationsSection;
