/**
 * PairingCodeCard.tsx — the code a phone is paired with (E022-T10).
 *
 * The QR is rendered as inline SVG rather than a canvas data URL: it is the
 * one `qrcode` output that needs no browser canvas, so it draws identically
 * in the app, in the mockup gallery and under jsdom.
 *
 * Both halves are always shown. A phone that cannot scan has to be pairable
 * by typing the desk id and the code, which is why neither is hidden behind
 * the other.
 */
import { useEffect, useState } from "react";
import type { FC } from "react";
import QRCode from "qrcode";
import type { PairingCode } from "@/generated/PairingCode";

interface PairingCodeCardProps {
  pairing: PairingCode;
}

/** Minutes left, floored, never negative. */
function minutesLeft(expiresAt: string): number {
  const ms = Date.parse(expiresAt) - Date.now();
  return Number.isNaN(ms) ? 0 : Math.max(0, Math.floor(ms / 60_000));
}

/** Code, desk id, expiry and the QR that carries all three. */
const PairingCodeCard: FC<PairingCodeCardProps> = ({ pairing }) => {
  const [svg, setSvg] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    QRCode.toString(pairing.qr_payload, { type: "svg", margin: 1 })
      .then((markup) => { if (live) setSvg(markup); })
      .catch(() => { if (live) setSvg(null); });
    return () => { live = false; };
  }, [pairing.qr_payload]);

  const mins = minutesLeft(pairing.expires_at);

  return (
    <div className="settings-panel__field" data-testid="pairing-code-card">
      <p className="settings-panel__label">Scan this on the phone, or type it in:</p>

      <p data-testid="pairing-code" className="settings-panel__code">{pairing.code}</p>
      <p className="settings-panel__hint">
        Desk ID <span data-testid="pairing-desk-id">{pairing.desk_id}</span> · expires in {mins} min
      </p>

      {svg ? (
        <div data-testid="pairing-qr" aria-label="Pairing QR code" dangerouslySetInnerHTML={{ __html: svg }} />
      ) : (
        <p className="settings-panel__hint" data-testid="pairing-qr-missing">
          QR code unavailable — enter the desk ID and code by hand.
        </p>
      )}
    </div>
  );
};

export default PairingCodeCard;
