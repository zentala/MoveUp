/**
 * WelcomePopup.tsx — first-launch setup guidance in the dedicated welcome window.
 */
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppReminderSnapshot } from "../types";

/** Welcome popup shown on first app launch. */
export function WelcomePopup() {
  const [dontShowAgain, setDontShowAgain] = useState(false);
  const [notificationFeedback, setNotificationFeedback] = useState<{ role: "status" | "alert"; message: string } | null>(null);
  const [testingNotification, setTestingNotification] = useState(false);
  const [openingMoveUp, setOpeningMoveUp] = useState(false);

  const handleTestNotification = async () => {
    if (testingNotification) return;
    setTestingNotification(true);
    setNotificationFeedback({ role: "status", message: "Preparing test preview…" });
    try {
      const snapshot = await invoke<AppReminderSnapshot>("trigger_test_notification");
      if (snapshot?.pending?.is_test) {
        setNotificationFeedback({
          role: "status",
          message: "Test preview queued in the main window. Return to status to view it.",
        });
      } else {
        setNotificationFeedback({
          role: "status",
          message: "Test preview sent to the main window.",
        });
      }
    } catch (e) {
      setNotificationFeedback({
        role: "alert",
        message: `Could not start test preview: ${e instanceof Error ? e.message : String(e)}`,
      });
    } finally {
      setTestingNotification(false);
    }
  };

  const handleOpenMoveUp = async () => {
    if (openingMoveUp) return;
    setOpeningMoveUp(true);
    setNotificationFeedback(null);
    try {
      await invoke("dismiss_welcome", { dontShowAgain });
    } catch (error) {
      const code = error instanceof Error ? error.message : String(error);
      const messages: Record<string, string> = {
        welcome_main_window_unavailable: "MoveUp could not be opened. Try again.",
        welcome_main_window_show_failed: "MoveUp could not be shown. Try again.",
        welcome_main_window_focus_failed: "MoveUp could not be focused. Try again.",
        welcome_preference_store_unavailable: "Could not save the Welcome preference. Try again.",
        welcome_preference_save_failed: "Could not save the Welcome preference. Try again.",
        welcome_close_failed: "MoveUp opened, but Welcome could not close. Try again.",
      };
      setNotificationFeedback({ role: "alert", message: messages[code] ?? "Could not open MoveUp. Try again." });
    } finally {
      setOpeningMoveUp(false);
    }
  };

  return (
    <main
      className="welcome-screen"
      data-testid="welcome-shell"
      data-tauri-drag-region
      style={{ height: "100vh", overflow: "hidden" }}
    >
      <header className="welcome-screen__header">
        <p className="welcome-screen__eyebrow">MOVEUP · FIRST RUN</p>
        <h1>Your first reading</h1>
        <p className="welcome-screen__lead">
          Move the desk and confirm that the height reading changes.
        </p>
      </header>

      <section
        className="welcome-screen__content"
        data-testid="welcome-content"
        aria-label="First steps"
        style={{ flex: "1 1 auto", minHeight: 0, overflowX: "hidden", overflowY: "auto" }}
      >
        <ol className="welcome-screen__steps">
          <li>
            <span className="welcome-screen__step-number" aria-hidden="true">01</span>
            <div>
              <h2>Check the assembled sensor</h2>
              <p>USB controller detection alone does not confirm that the assembled sensor sends readings.</p>
            </div>
          </li>
          <li>
            <span className="welcome-screen__step-number" aria-hidden="true">02</span>
            <div>
              <h2>Confirm a live reading</h2>
              <p>The height in the main window should change as the desk moves. Without a reading, the state remains unknown.</p>
            </div>
          </li>
          <li>
            <span className="welcome-screen__step-number" aria-hidden="true">03</span>
            <div>
              <h2>Calibrate and understand your day</h2>
              <p>In Settings → Calibr., enter the sitting and standing heights. Sitting balance includes break credit; history shows recorded elapsed time. Posture is inferred from readings, not USB detection.</p>
            </div>
          </li>
        </ol>
        <p className="welcome-screen__help-note">
          If no reading appears, try a short data-capable cable connected directly to a USB port. More sensor help is in Settings.
        </p>
      </section>

      <footer className="welcome-screen__footer" data-testid="welcome-footer" style={{ flexShrink: 0 }}>
        <p
          className="welcome-screen__feedback"
          {...(notificationFeedback ? { role: notificationFeedback.role } : {})}
        >
          {notificationFeedback?.message ?? "Reminder tests open in the main MoveUp window."}
        </p>
        <label className="welcome-screen__checkbox">
          <input
            type="checkbox"
            checked={dontShowAgain}
            onChange={(e) => setDontShowAgain(e.target.checked)}
          />
          Don't show this at next launch
        </label>
        <div className="welcome-screen__actions">
          <button
            className="welcome-screen__test"
            type="button"
            onClick={handleTestNotification}
            disabled={testingNotification}
          >
            Test reminder
          </button>
          <button className="welcome-screen__primary" type="button" onClick={handleOpenMoveUp} disabled={openingMoveUp}>
            {openingMoveUp ? "Opening MoveUp…" : "Open MoveUp"}
          </button>
        </div>
      </footer>
    </main>
  );
}
