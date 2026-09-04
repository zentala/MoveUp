import { useEffect, useState } from "react";
import type { ChangeEvent, FC } from "react";

interface AutostartToggleProps {
  appName: string;
}

/** Controls the app's Tauri-managed login startup setting. */
export const AutostartToggle: FC<AutostartToggleProps> = ({ appName }) => {
  const [enabled, setEnabled] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadAutostartStatus();
  }, []);

  async function loadAutostartStatus() {
    try {
      const { isEnabled } = await import("@tauri-apps/plugin-autostart");
      setEnabled(await isEnabled());
    } catch (cause) {
      setError(String(cause));
    } finally {
      setLoading(false);
    }
  }

  async function handleToggle(event: ChangeEvent<HTMLInputElement>) {
    const checked = event.target.checked;
    setError(null);
    try {
      const { disable, enable } = await import("@tauri-apps/plugin-autostart");
      if (checked) await enable();
      else await disable();
      setEnabled(checked);
    } catch (cause) {
      setError(String(cause));
    }
  }

  return (
    <div>
      <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
        {loading ? <span>…</span> : <input type="checkbox" checked={enabled} onChange={handleToggle} />}
        <span>Launch {appName} on login</span>
      </label>
      {error && <span style={{ color: "#f44", fontSize: 12 }}>{error}</span>}
    </div>
  );
};
