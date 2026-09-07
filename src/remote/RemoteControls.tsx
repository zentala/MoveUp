/**
 * RemoteControls.tsx — the three writes a phone is allowed (E022-T09).
 *
 * Rendered only when the transport reports `capabilities.control`, which is
 * true on the relay and false on the LAN socket — the LAN path is
 * unauthenticated and stays read-only (D4). The gate lives here as well as in
 * the layout, because "which wire am I on" is a transport fact and this is
 * the component that would send the command.
 *
 * Profile names are the built-ins listed in `CLAUDE.md`. The phone cannot
 * enumerate profiles — that list comes from a Tauri command the browser does
 * not have — so a custom profile is switchable from the desktop only.
 */
import { specFor } from "./protocol";
import { useActiveTransport } from "./activeTransport";
import { useRemoteCommand } from "./useRemoteCommand";
import type { Transport } from "./transports/types";
import "./remote.css";

const ERGONOMIC_PROFILES = ["standard", "strict", "relaxed", "demo"];
const COMMUNICATION_PROFILES = ["default", "aggressive", "gentle", "silent", "demo"];
/** How much one tap moves a limit. */
const STEP_MIN = 5;

const LIMIT_BOUNDS = specFor("set_limits")?.ints ?? [];
const boundsOf = (name: string) =>
  LIMIT_BOUNDS.find((arg) => arg.name === name) ?? { min: 1, max: 240 };

const clamp = (value: number, name: string) => {
  const { min, max } = boundsOf(name);
  return Math.min(max, Math.max(min, value));
};

export interface RemoteControlsProps {
  /** Current sit limit in minutes — the base for the ± buttons. */
  sitMin: number;
  /** Current stand target in minutes. */
  standMin: number;
  /** Transport override; defaults to the active one. */
  transport?: Transport | null;
}

/** Alert dismissal, limit nudges and profile switching, from the phone. */
export function RemoteControls({ sitMin, standMin, transport }: RemoteControlsProps) {
  const active = useActiveTransport();
  const wire = transport !== undefined ? transport : active;
  const { pending, error, send } = useRemoteCommand(wire);

  if (!wire?.capabilities.control) return null;

  const busy = pending !== null;
  const limitButton = (label: string, arg: string, value: number) => (
    <button
      className="rc__btn"
      data-testid={`rc-${label}`}
      type="button"
      disabled={busy}
      onClick={() => void send(label, "set_limits", { [arg]: clamp(value, arg) })}
    >
      {pending === label ? "…" : label}
    </button>
  );

  return (
    <section className="rc" data-testid="remote-controls">
      <button
        className="rc__btn rc__btn--wide"
        data-testid="rc-ack"
        type="button"
        disabled={busy}
        onClick={() => void send("ack", "ack_alert", {})}
      >
        {pending === "ack" ? "Dismissing…" : "Dismiss alert"}
      </button>

      <div className="rc__row">
        <span className="rc__label">Sit {sitMin} min</span>
        {limitButton("sit-", "sit_min", sitMin - STEP_MIN)}
        {limitButton("sit+", "sit_min", sitMin + STEP_MIN)}
      </div>

      <div className="rc__row">
        <span className="rc__label">Stand {standMin} min</span>
        {limitButton("stand-", "stand_min", standMin - STEP_MIN)}
        {limitButton("stand+", "stand_min", standMin + STEP_MIN)}
      </div>

      <div className="rc__row">
        <label className="rc__label" htmlFor="rc-ergonomic">
          Ergonomic
        </label>
        <select
          id="rc-ergonomic"
          className="rc__select"
          data-testid="rc-ergonomic"
          value=""
          disabled={busy}
          onChange={(e) =>
            void send("ergonomic", "switch_profile", {
              kind: "ergonomic",
              name: e.target.value,
            })
          }
        >
          <option value="">Switch…</option>
          {ERGONOMIC_PROFILES.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      </div>

      <div className="rc__row">
        <label className="rc__label" htmlFor="rc-communication">
          Nudges
        </label>
        <select
          id="rc-communication"
          className="rc__select"
          data-testid="rc-communication"
          value=""
          disabled={busy}
          onChange={(e) =>
            void send("communication", "switch_profile", {
              kind: "communication",
              name: e.target.value,
            })
          }
        >
          <option value="">Switch…</option>
          {COMMUNICATION_PROFILES.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
      </div>

      {error && (
        <p className="rc__error" data-testid="rc-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}

export default RemoteControls;
