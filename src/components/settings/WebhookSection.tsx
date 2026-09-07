/**
 * WebhookSection.tsx — mirror desk alerts to a phone or watch.
 *
 * A Windows toast only reaches this machine. Pointing this at an ntfy topic
 * (or any endpoint that reads `{title, message, priority, tags}`) makes the
 * same alert show up wherever the user subscribed.
 */
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";

interface WebhookSectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
}

/** Webhook toggle plus the URL it posts to. */
const WebhookSection: FC<WebhookSectionProps> = ({ settings, onChange }) => (
  <div className="settings-panel__section">
    <h3 className="settings-panel__section-title">Phone notifications</h3>

    <div className="settings-panel__field">
      <label className="settings-panel__toggle-row">
        <input
          type="checkbox"
          checked={settings.notify_webhook_enabled}
          onChange={(e) =>
            onChange({ ...settings, notify_webhook_enabled: e.target.checked })
          }
        />
        <span className="settings-panel__label">Send alerts to a webhook</span>
      </label>
    </div>

    <div className="settings-panel__field">
      <label className="settings-panel__label" htmlFor="notify-webhook-url">
        Webhook URL
      </label>
      <input
        id="notify-webhook-url"
        type="url"
        className="settings-panel__input"
        placeholder="https://ntfy.sh/your-topic"
        value={settings.notify_webhook_url ?? ""}
        disabled={!settings.notify_webhook_enabled}
        onChange={(e) =>
          onChange({ ...settings, notify_webhook_url: e.target.value })
        }
      />
    </div>

    <p className="settings-panel__hint">
      Leave empty to use the DESK_NOTIFY_WEBHOOK_URL environment variable.
    </p>
  </div>
);

export default WebhookSection;
