/**
 * WelcomePopup.tsx — First-launch welcome / onboarding popup.
 *
 * Shown in a separate Tauri WebviewWindow (label: "welcome").
 * Introduces the app, explains the overlay bar and notifications,
 * and lets the user dismiss with an optional "don't show again" checkbox.
 */
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const STYLES = {
  container: {
    fontFamily: "'Segoe UI', Tahoma, Geneva, Verdana, sans-serif",
    padding: "28px 32px",
    color: "#e0e0e0",
    background: "#1a1a2e",
    height: "100vh",
    boxSizing: "border-box" as const,
    display: "flex",
    flexDirection: "column" as const,
    gap: "12px",
    overflow: "auto",
    userSelect: "none" as const,
  },
  heading: {
    fontSize: "22px",
    fontWeight: 600,
    margin: "0 0 4px 0",
    color: "#ffffff",
  },
  intro: {
    fontSize: "14px",
    lineHeight: "1.6",
    margin: 0,
    color: "#c0c0d0",
  },
  featureList: {
    listStyle: "none",
    padding: 0,
    margin: "4px 0",
    display: "flex",
    flexDirection: "column" as const,
    gap: "8px",
  },
  featureItem: {
    fontSize: "13px",
    lineHeight: "1.5",
    color: "#d0d0e0",
  },
  divider: {
    border: "none",
    borderTop: "1px solid #2a2a4a",
    margin: "4px 0",
  },
  hint: {
    fontSize: "12px",
    color: "#808090",
    margin: 0,
  },
  checkboxLabel: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    fontSize: "13px",
    color: "#a0a0b0",
    cursor: "pointer",
  },
  footer: {
    marginTop: "auto",
    display: "flex",
    flexDirection: "column" as const,
    gap: "8px",
  },
  btnPrimary: {
    padding: "10px 20px",
    fontSize: "15px",
    fontWeight: 600,
    border: "none",
    borderRadius: "6px",
    cursor: "pointer",
    background: "#4a6cf7",
    color: "#ffffff",
    transition: "background 0.15s",
  },
  btnSecondary: {
    padding: "8px 16px",
    fontSize: "13px",
    border: "1px solid #3a3a5a",
    borderRadius: "6px",
    cursor: "pointer",
    background: "transparent",
    color: "#a0a0c0",
    transition: "background 0.15s",
  },
} as const;

/** Welcome popup shown on first app launch. */
export function WelcomePopup() {
  const [dontShowAgain, setDontShowAgain] = useState(false);
  const [notifStatus, setNotifStatus] = useState<string | null>(null);

  const handleTestNotification = async () => {
    try {
      await invoke("trigger_test_notification");
      setNotifStatus("Sent!");
    } catch (e) {
      setNotifStatus(String(e));
    }
  };

  const handleDismiss = async () => {
    try {
      await invoke("dismiss_welcome", { dontShowAgain });
    } catch (e) {
      console.error("Failed to dismiss welcome:", e);
    }
  };

  return (
    <div data-tauri-drag-region style={STYLES.container}>
      <h2 style={STYLES.heading}>
        {"👋"} Cześć! Jestem Twoim osobistym asystentem
        biurkowym.
      </h2>

      <p style={STYLES.intro}>
        Pomagam Ci zadbać o ciało podczas pracy &mdash;
        {" żebyś"} się częściej ruszał i nie
        zapominał o przerwach na ruch.
      </p>

      <p style={{ ...STYLES.intro, fontWeight: 500 }}>Oto jak działam:</p>

      <ul style={STYLES.featureList}>
        <li style={STYLES.featureItem}>
          {"🟢"} Pasek na górze ekranu pokazuje, jak
          długo siedzisz. Kolor zmienia się: zielony &rarr;
          {" żółty"} &rarr; czerwony.
        </li>
        <li style={STYLES.featureItem}>
          {"🔔"} Gdy przesiedzisz za długo, delikatnie
          Cię przypomnę.
        </li>
        <li style={STYLES.featureItem}>
          {"🏆"} Gdy wstajesz &mdash; nagradzam Cię
          złotym paskiem i punktami.
        </li>
      </ul>

      <hr style={STYLES.divider} />

      <p style={STYLES.hint}>
        Możesz przeciągnąć to okienko w dowolne miejsce na
        ekranie. Kliknij ikonę w zasobniku systemowym, aby mnie
        ukryć lub pokazać.
      </p>

      <label style={STYLES.checkboxLabel}>
        <input
          type="checkbox"
          checked={dontShowAgain}
          onChange={(e) => setDontShowAgain(e.target.checked)}
        />
        Nie pokazuj przy następnym uruchomieniu
      </label>

      <div style={STYLES.footer}>
        <button
          style={STYLES.btnSecondary}
          onClick={handleTestNotification}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = "#2a2a4a";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = "transparent";
          }}
        >
          {"🔔"} Przetestuj powiadomienia
          {notifStatus ? ` (${notifStatus})` : ""}
        </button>
        <button
          style={STYLES.btnPrimary}
          onClick={handleDismiss}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = "#3a5ce7";
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = "#4a6cf7";
          }}
        >
          Gotowy! Zaczynamy! {"✨"}
        </button>
      </div>
    </div>
  );
}
