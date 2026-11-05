import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { ApplicationReminder } from "../../src/components/ApplicationReminder";
import { useAppReminder } from "../../src/hooks/useAppReminder";
import "../../src/styles/globals.css";

declare global { interface Window { __REMINDER_NATIVE_ROLE__: "input" | "reminder" } }

function InputTarget() {
  const [round, setRound] = useState(1);
  const [phase, setPhase] = useState("baseline");
  const [status, setStatus] = useState("Type disposable text, then record this phase.");
  const receipts = useRef({ keys: 0, pointers: 0, inputEvents: 0, length: 0 });
  async function record() {
    try {
      await invoke("fixture_input_receipt", { round, phase, ...receipts.current });
      receipts.current = { keys: 0, pointers: 0, inputEvents: 0, length: 0 };
      setStatus(`Recorded round ${round} ${phase}.`);
    } catch { setStatus("Receipt refused. Keep this owned window foreground and provide real keyboard and pointer input."); }
  }
  return <main style={{ padding: 16 }} onPointerDown={event => {
    if (event.isTrusted) receipts.current.pointers++;
  }}>
    <h1>Isolated input target</h1>
    <p>This is a separate process with a private profile. Use only disposable text.</p>
    <p>For each round: record baseline, keep typing while the reminder arrives, then record during and after before clicking its X.</p>
    <label>Round <select value={round} onChange={event => setRound(Number(event.target.value))}>
      <option value={1}>1</option><option value={2}>2</option>
    </select></label>{" "}
    <label>Phase <select value={phase} onChange={event => setPhase(event.target.value)}>
      <option>baseline</option><option>during</option><option>after</option>
    </select></label>
    <p><textarea aria-label="Disposable input" maxLength={128} rows={4} onInput={event => {
      if (event.isTrusted) {
        receipts.current.inputEvents++;
        receipts.current.length = event.currentTarget.value.length;
      }
    }} onKeyDown={event => {
      if (event.isTrusted && event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey) receipts.current.keys++;
    }} /></p>
    <button type="button" onClick={() => void record()}>Record phase</button>{" "}
    <button type="button" onClick={() => void invoke("fixture_quit")}>Quit input target</button>
    <p role="status">{status}</p>
  </main>;
}

function ReminderHost() {
  const reminder = useAppReminder(true);
  const [fixtureError, setFixtureError] = useState<string | null>(null);
  const [closedAttempt, setClosedAttempt] = useState<{ trustedPointer: boolean } | null>(null);
  const verifying = useRef(false);
  const pointerX = useRef(false);
  useEffect(() => {
    if (reminder.ready) void invoke("fixture_frontend_ready").catch(() => setFixtureError("Fixture readiness refused."));
  }, [reminder.ready]);
  async function close() {
    const trustedPointer = pointerX.current;
    pointerX.current = false;
    await reminder.closeReminder(); // Actual hook -> production ACK -> Store -> guarded native hide.
    setClosedAttempt({ trustedPointer });
  }
  useEffect(() => {
    // This runs after a committed render, not in close()'s pre-ACK closure.
    // No requestAnimationFrame: a genuinely hidden native WebView may throttle it.
    if (!closedAttempt || reminder.busy || verifying.current) return;
    const snapshot = reminder.snapshot;
    const renderedAlerts = document.querySelectorAll('[role="alert"]').length;
    if (!snapshot || reminder.closeError || reminder.error || fixtureError || renderedAlerts) {
      setFixtureError("Settled reminder UI is not clear; this close is not accepted.");
      setClosedAttempt(null);
      return;
    }
    verifying.current = true;
    void invoke("fixture_verify_close", { trustedPointer: closedAttempt.trustedPointer, hook: {
      sessionId: snapshot.session_id, clientId: snapshot.client_id, revision: snapshot.revision,
      ready: reminder.ready, busy: reminder.busy, closeError: !!reminder.closeError, actionError: !!reminder.error,
      active: !!snapshot.active, pending: !!snapshot.pending, protected: !!snapshot.protected_flow, renderedAlerts,
    } }).catch(() => setFixtureError("Native/disk/duplicate verification failed. Keep the fixture evidence for review."))
      .finally(() => { verifying.current = false; setClosedAttempt(null); });
  }, [closedAttempt, reminder.busy, reminder.closeError, reminder.error, reminder.snapshot, reminder.ready, fixtureError]);
  return <main style={{ height: "100vh", overflow: "auto" }} onPointerDownCapture={event => {
    if (event.isTrusted && event.target instanceof Element && event.target.closest('[aria-label="Close reminder"]')) pointerX.current = true;
  }}>
    {reminder.snapshot?.active ? <ApplicationReminder message={reminder.snapshot.active}
      busy={reminder.busy} error={reminder.error} retrySave={reminder.retrySave}
      closeAckRetry={reminder.closeAckRetry} onClose={() => void close()}
      onCloseRetry={() => void reminder.retryCloseAcknowledgement()}
      onAction={action => void reminder.acknowledge(action)} />
      : <p>Waiting for the next real policy reminder. Tracking uses disposable synthetic state.</p>}
    {reminder.closeError && <p role="alert">{reminder.closeError} <button onClick={() => void reminder.retryClose()}>Retry close</button></p>}
    {fixtureError && <p role="alert">{fixtureError}</p>}
  </main>;
}

const root = document.getElementById("root");
if (!root) throw new Error("fixture_root_missing");
createRoot(root).render(window.__REMINDER_NATIVE_ROLE__ === "input" ? <InputTarget /> : <ReminderHost />);
