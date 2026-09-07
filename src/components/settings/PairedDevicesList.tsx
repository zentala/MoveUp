/**
 * PairedDevicesList.tsx — the phones allowed to watch this desk (E022-T10).
 *
 * An empty list says so in words. Rendering nothing would be the same pixels
 * as "still loading" and as "the request failed", and those are three
 * different facts.
 */
import type { FC } from "react";
import type { Viewer } from "@/generated/Viewer";

interface PairedDevicesListProps {
  viewers: Viewer[];
  /** Disabled while another relay call is in flight. */
  busy: boolean;
  onRevoke: (viewerId: string) => void;
}

/** Local date, or a dash when the phone has never connected. */
function lastSeen(viewer: Viewer): string {
  if (!viewer.last_seen) return "never connected";
  const at = new Date(viewer.last_seen);
  return Number.isNaN(at.getTime()) ? "unknown" : at.toLocaleString();
}

/** One row per paired phone, each with its own Remove button. */
const PairedDevicesList: FC<PairedDevicesListProps> = ({ viewers, busy, onRevoke }) => {
  if (viewers.length === 0) {
    return (
      <p className="settings-panel__hint" data-testid="paired-devices-empty">
        No phones paired yet.
      </p>
    );
  }

  return (
    <ul className="settings-panel__list" data-testid="paired-devices-list">
      {viewers.map((viewer) => (
        <li key={viewer.viewer_id} className="settings-panel__list-row">
          <span className="settings-panel__label">{viewer.device_name}</span>
          <span className="settings-panel__hint">
            {viewer.online ? "online" : `last seen ${lastSeen(viewer)}`}
          </span>
          <button
            className="btn btn--secondary"
            disabled={busy}
            onClick={() => onRevoke(viewer.viewer_id)}
            aria-label={`Remove ${viewer.device_name}`}
          >
            Remove
          </button>
        </li>
      ))}
    </ul>
  );
};

export default PairedDevicesList;
