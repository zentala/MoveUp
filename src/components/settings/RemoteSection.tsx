/**
 * RemoteSection.tsx — Settings → More → Remote access (E022-T10).
 *
 * Registration, pairing and revocation for the cloud relay, plus the toggle
 * that keeps the LAN display alive. The relay is opt-in and the LAN stays on
 * by default: turning one transport on must never take the other away.
 *
 * Everything here reads `RelayStatus.state` rather than the `enabled` flag —
 * "switched on" and "actually connected" are different facts, and the user
 * needs the second one.
 */
import { useState } from "react";
import type { FC } from "react";
import type { RelayState } from "@/generated/RelayState";
import type { DeskSettings } from "./SettingsTypes";
import PairedDevicesList from "./PairedDevicesList";
import PairingCodeCard from "./PairingCodeCard";
import { useRelaySettings } from "./useRelaySettings";

interface RemoteSectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
}

/** What each relay state means to somebody who has never read the protocol. */
const STATE_LABEL: Record<RelayState, string> = {
  disabled: "Off — this desk is not registered",
  connecting: "Connecting…",
  online: "Online — your phone can reach this desk",
  unentitled: "Licence expired or not valid for this desk",
  revoked: "This desk was removed from the licence",
  replaced: "Another desk took over this registration",
  error: "Cannot reach the relay",
};

/** Relay registration, pairing, paired phones, and the LAN toggle. */
const RemoteSection: FC<RemoteSectionProps> = ({ settings, onChange }) => {
  const relay = useRelaySettings();
  const [licenseKey, setLicenseKey] = useState("");
  const state = relay.status?.state ?? "disabled";

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Remote access</h3>

      <p className="settings-panel__label" data-testid="relay-state">{STATE_LABEL[state]}</p>
      {relay.status?.last_error && (
        <p className="settings-panel__hint" data-testid="relay-last-error">{relay.status.last_error}</p>
      )}

      {state === "disabled" ? (
        <div className="settings-panel__field">
          <label className="settings-panel__label" htmlFor="relay-license-key">Licence key</label>
          <input
            id="relay-license-key"
            className="settings-panel__input"
            value={licenseKey}
            disabled={relay.busy}
            onChange={(e) => setLicenseKey(e.target.value)}
          />
          <button
            className="btn"
            disabled={relay.busy || licenseKey.trim() === ""}
            onClick={() => relay.register(licenseKey)}
          >
            Enable remote access
          </button>
        </div>
      ) : (
        <div className="settings-panel__field">
          <button className="btn" disabled={relay.busy || state !== "online"} onClick={relay.startPairing}>
            Pair a phone
          </button>
          <button className="btn btn--secondary" disabled={relay.busy} onClick={relay.disable}>
            Turn off and forget this desk
          </button>
        </div>
      )}

      {relay.pairing && <PairingCodeCard pairing={relay.pairing} />}

      {state !== "disabled" && (
        <div className="settings-panel__field">
          <h4 className="settings-panel__label">Paired phones</h4>
          <PairedDevicesList viewers={relay.viewers} busy={relay.busy} onRevoke={relay.revoke} />
        </div>
      )}

      <div className="settings-panel__field">
        <label className="settings-panel__toggle-row">
          <input
            type="checkbox"
            checked={settings.remote_lan_enabled}
            onChange={(e) => onChange({ ...settings, remote_lan_enabled: e.target.checked })}
          />
          <span className="settings-panel__label">Serve the display on this network too</span>
        </label>
      </div>

      {relay.error && <p className="error-banner" data-testid="relay-error">{relay.error}</p>}
    </div>
  );
};

export default RemoteSection;
