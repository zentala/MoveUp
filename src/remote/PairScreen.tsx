/**
 * PairScreen.tsx — `#/pair` on the phone (E022-T09).
 *
 * Two ways in, one form. The QR on the desktop encodes
 * `/app/#/pair?d=<desk_id>&c=<code>`, which prefills the fields and submits
 * on its own — scanning a code and then typing it would be theatre. Without
 * those parameters the same form is filled by hand, because a phone that
 * cannot scan still has to be able to pair.
 *
 * On success the pairing is stored and the page reloads into the dashboard:
 * the transport is chosen once, at mount, so swapping wires is a reload by
 * design (see `transports/index.ts`).
 */
import { useCallback, useEffect, useRef, useState } from "react";
import {
  PairFailed,
  defaultDeviceName,
  defaultRelayUrl,
  pairViewer,
  parsePairLink,
} from "./pairing";
import { clearRelayRecord, loadRelayRecord, saveRelayRecord } from "./storage";
import "./remote.css";

/** Turns a relay error code into something a person can act on. */
function messageFor(error: PairFailed): string {
  switch (error.code) {
    case "bad_code":
      return "That code is not right. Check it on the PC and try again.";
    case "code_expired":
      return "That code has expired. Generate a new one on the PC.";
    case "pairing_locked":
      return error.retryAfterSecs
        ? `Too many wrong codes. Try again in ${Math.ceil(error.retryAfterSecs / 60)} min.`
        : error.message;
    case "viewer_limit":
      return "This desk has no free phone slots. Remove one in Settings → Remote.";
    default:
      return error.message;
  }
}

/** Codes after which the entered code is worthless and the field is cleared. */
const CLEARS_CODE = new Set(["bad_code", "code_expired"]);

export interface PairScreenProps {
  /** Overrides `window.location.hash` — tests and the mockup gallery. */
  hash?: string;
  /** Runs after a successful pairing. Default: reload into the dashboard. */
  onPaired?: () => void;
  /** Runs after the stored pairing is dropped. Default: reload. */
  onForget?: () => void;
}

function reloadIntoDashboard(): void {
  window.location.hash = "";
  window.location.reload();
}

/** Pairing form for the phone viewer. */
export function PairScreen({ hash, onPaired, onForget }: PairScreenProps = {}) {
  // Parsed once: the hash may change while the form is open (the user edits
  // the URL, a router pushes) and re-prefilling half-typed fields would be
  // worse than ignoring it.
  const [link] = useState(() => parsePairLink(hash ?? window.location.hash));
  const [deskId, setDeskId] = useState(link.deskId);
  const [code, setCode] = useState(link.code);
  const [deviceName, setDeviceName] = useState(defaultDeviceName);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [paired, setPaired] = useState(() => loadRelayRecord() !== null);
  const autoSubmitted = useRef(false);

  const submit = useCallback(
    async (nextDeskId: string, nextCode: string) => {
      if (!nextDeskId.trim() || !nextCode.trim()) {
        setError("Enter the desk ID and the pairing code.");
        return;
      }
      setBusy(true);
      setError(null);
      try {
        const record = await pairViewer({
          relayUrl: link.relayUrl ?? defaultRelayUrl(),
          deskId: nextDeskId.trim(),
          code: nextCode.trim().toUpperCase(),
          deviceName: deviceName.trim() || defaultDeviceName(),
        });
        saveRelayRecord(record);
        setPaired(true);
        (onPaired ?? reloadIntoDashboard)();
      } catch (e) {
        const failure =
          e instanceof PairFailed ? e : new PairFailed("unknown", "Pairing failed");
        setError(messageFor(failure));
        if (CLEARS_CODE.has(failure.code)) setCode("");
      } finally {
        setBusy(false);
      }
    },
    [deviceName, link.relayUrl, onPaired],
  );

  // A complete deep link is a decision the user already made by scanning.
  useEffect(() => {
    if (autoSubmitted.current || !link.deskId || !link.code) return;
    autoSubmitted.current = true;
    void submit(link.deskId, link.code);
  }, [link.deskId, link.code, submit]);

  const forget = () => {
    clearRelayRecord();
    setPaired(false);
    (onForget ?? reloadIntoDashboard)();
  };

  return (
    <main className="pair" data-testid="pair-screen">
      <h1 className="pair__title">Pair with your desk</h1>
      <p className="pair__hint">
        On the PC: Settings → Remote → <strong>Pair a phone</strong>.
      </p>

      <form
        className="pair__form"
        data-testid="pair-form"
        onSubmit={(e) => {
          e.preventDefault();
          void submit(deskId, code);
        }}
      >
        <label className="pair__label" htmlFor="pair-desk">
          Desk ID
        </label>
        <input
          id="pair-desk"
          className="pair__input"
          data-testid="pair-desk"
          value={deskId}
          autoComplete="off"
          onChange={(e) => setDeskId(e.target.value)}
        />

        <label className="pair__label" htmlFor="pair-code">
          Pairing code
        </label>
        <input
          id="pair-code"
          className="pair__input pair__input--code"
          data-testid="pair-code"
          value={code}
          autoComplete="off"
          placeholder="8 characters"
          onChange={(e) => setCode(e.target.value)}
        />

        <label className="pair__label" htmlFor="pair-device">
          This device
        </label>
        <input
          id="pair-device"
          className="pair__input"
          data-testid="pair-device"
          value={deviceName}
          onChange={(e) => setDeviceName(e.target.value)}
        />

        <button className="pair__btn" data-testid="pair-submit" type="submit" disabled={busy}>
          {busy ? "Pairing…" : "Pair"}
        </button>
      </form>

      {error && (
        <p className="pair__error" data-testid="pair-error" role="alert">
          {error}
        </p>
      )}

      {paired && (
        <button
          className="pair__btn pair__btn--ghost"
          data-testid="pair-forget"
          type="button"
          onClick={forget}
        >
          Forget this desk
        </button>
      )}
    </main>
  );
}

export default PairScreen;
