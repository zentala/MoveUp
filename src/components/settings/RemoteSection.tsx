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
  disabled: "Relay is off",
  connecting: "Connecting to the cloud relay…",
  online: "Connected to the cloud relay",
  unentitled: "Relay licence expired or is not valid for this desk",
  revoked: "This desk's relay registration was removed",
  replaced: "Another desk took over this relay registration",
  error: "Cloud relay unavailable; retrying in the background",
};

/** Relay registration, pairing, paired phones, and the LAN toggle. */
const RemoteSection: FC<RemoteSectionProps> = ({ settings, onChange }) => {
  const relay = useRelaySettings();
  const [licenseKey, setLicenseKey] = useState("");
  const state = relay.status?.state;

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Remote access</h3>

      <h4 className="settings-panel__label">Cloud relay</h4>
      <p className="settings-panel__hint">
        A separate Pro route: this desk connects out to the relay. This status
        reports the desk's relay connection, not whether a phone is connected.
      </p>
      {relay.status ? (
        <p className="settings-panel__label" data-testid="relay-state">
          {STATE_LABEL[relay.status.state]}
        </p>
      ) : relay.statusError === "status" ? (
        <p className="settings-panel__label" data-testid="relay-state" role="status">
          Relay status unavailable
        </p>
      ) : relay.loading ? (
        <p className="settings-panel__label" data-testid="relay-state" role="status">
          Checking relay status…
        </p>
      ) : null}
      {relay.status && relay.loading && (
        <p className="settings-panel__hint" data-testid="relay-refreshing" role="status">
          Refreshing relay details…
        </p>
      )}
      {relay.status?.last_error && (
        <p className="settings-panel__hint" data-testid="relay-last-error">
          Relay detail: {relay.status.last_error}
        </p>
      )}

      {relay.statusError && (
        <div className="error-banner" data-testid="relay-status-error" role="alert">
          <span>
            {relay.statusError === "status"
              ? "Could not load relay status. The relay state is unknown."
              : "Could not refresh paired phones. The relay status above is still available."}
          </span>
          <button
            className="btn btn--secondary"
            disabled={relay.loading}
            onClick={() => void relay.refresh()}
          >
            {relay.loading ? "Retrying…" : "Retry relay status"}
          </button>
        </div>
      )}

      {relay.status && relay.statusError !== "status" && state === "disabled" ? (
        <div className="settings-panel__field">
          <label className="settings-panel__label" htmlFor="relay-license-key">
            Licence key
          </label>
          <input
            id="relay-license-key"
            type="password"
            autoComplete="off"
            className="settings-panel__input"
            value={licenseKey}
            disabled={relay.busy || relay.loading}
            onChange={(e) => setLicenseKey(e.target.value)}
          />
          <button
            className="btn"
            disabled={relay.busy || relay.loading || licenseKey.trim() === ""}
            onClick={() => relay.register(licenseKey)}
          >
            Enable remote access
          </button>
        </div>
      ) : relay.status && relay.statusError !== "status" ? (
        <div className="settings-panel__field">
          <button
            className="btn"
            disabled={relay.busy || relay.loading || state !== "online"}
            onClick={relay.startPairing}
          >
            Pair a phone
          </button>
          <button
            className="btn btn--secondary"
            disabled={relay.busy || relay.loading}
            onClick={relay.disable}
          >
            Turn off and forget this desk
          </button>
        </div>
      ) : null}

      {relay.pairing && relay.status && relay.statusError !== "status" && (
        <PairingCodeCard pairing={relay.pairing} />
      )}

      {relay.status && relay.statusError !== "status" && state !== "disabled" && (
        <div className="settings-panel__field">
          <h4 className="settings-panel__label">Paired phones</h4>
          {relay.viewerState === "loading" ? (
            <p className="settings-panel__hint" data-testid="paired-devices-loading" role="status">
              Loading paired phones…
            </p>
          ) : relay.viewerState === "error" ? (
            <p className="settings-panel__hint" data-testid="paired-devices-unavailable">
              Paired phones are unavailable until the list loads successfully.
            </p>
          ) : relay.viewerState === "loaded" ? (
            <PairedDevicesList viewers={relay.viewers} busy={relay.busy} onRevoke={relay.revoke} />
          ) : null}
        </div>
      )}

      <div className="settings-panel__field">
        <h4 className="settings-panel__label">LAN display · same network</h4>
        <p className="settings-panel__hint">
          This toggle enables the same-network display path. When its server is
          listening, devices on that network may view the dashboard without
          sign-in. MoveUp does not report whether the server is listening or
          which adapter address to use, and cannot confirm firewall or phone
          reachability.
        </p>
        <label className="settings-panel__toggle-row">
          <input
            type="checkbox"
            checked={settings.remote_lan_enabled}
            onChange={(e) => onChange({ ...settings, remote_lan_enabled: e.target.checked })}
          />
          <span className="settings-panel__label">Serve the display on this network</span>
        </label>
      </div>

      {relay.error && (
        <p className="error-banner" data-testid="relay-error" role="alert">
          {relay.error}
        </p>
      )}
    </div>
  );
};

export default RemoteSection;
