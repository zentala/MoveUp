/**
 * NotificationsSection.tsx — notification toggle settings section.
 */
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";

interface NotificationsSectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
}

const NotificationsSection: FC<NotificationsSectionProps> = ({
  settings,
  onChange,
}) => (
  <div className="settings-panel__section">
    <h3 className="settings-panel__section-title">Notifications</h3>

    <div className="settings-panel__toggle-group">
      <label className="settings-panel__toggle-label">
        <input
          type="checkbox"
          checked={settings.notify_inactivity}
          onChange={(e) =>
            onChange({ ...settings, notify_inactivity: e.target.checked })
          }
          className="settings-panel__checkbox"
        />
        <span>Alert if no position change for 90 min</span>
      </label>
    </div>

    <div className="settings-panel__toggle-group">
      <label className="settings-panel__toggle-label">
        <input
          type="checkbox"
          checked={settings.notify_daily_posture_balance}
          onChange={(e) =>
            onChange({
              ...settings,
              notify_daily_posture_balance: e.target.checked,
            })
          }
          className="settings-panel__checkbox"
        />
        <span>Alert if sitting dominates today</span>
      </label>
    </div>

    <div className="settings-panel__toggle-group">
      <label className="settings-panel__toggle-label">
        <input
          type="checkbox"
          checked={settings.notify_praise_halfway}
          onChange={(e) =>
            onChange({ ...settings, notify_praise_halfway: e.target.checked })
          }
          className="settings-panel__checkbox"
        />
        <span>Praise when halfway through standing goal</span>
      </label>
    </div>
  </div>
);

export default NotificationsSection;
