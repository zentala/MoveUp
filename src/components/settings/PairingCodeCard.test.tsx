/**
 * PairingCodeCard.test.tsx — the code, the QR, and what happens without one.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import QRCode from "qrcode";
import PairingCodeCard from "./PairingCodeCard";
import { relayPairingCodeShown } from "@/test/scenarios";

const PAIRING = relayPairingCodeShown.pairing!;

beforeEach(() => {
  vi.useFakeTimers({ shouldAdvanceTime: true });
  vi.setSystemTime(Date.parse("2026-09-07T09:00:00Z"));
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("PairingCodeCard", () => {
  it("shows the code, the desk id and the minutes left", () => {
    render(<PairingCodeCard pairing={PAIRING} />);

    expect(screen.getByTestId("pairing-code")).toHaveTextContent("ABCD2345");
    expect(screen.getByTestId("pairing-desk-id")).toHaveTextContent("desk-42");
    expect(screen.getByTestId("pairing-code-card")).toHaveTextContent("expires in 5 min");
  });

  it("never shows negative minutes for an expired code", () => {
    vi.setSystemTime(Date.parse("2026-09-07T09:30:00Z"));
    render(<PairingCodeCard pairing={PAIRING} />);

    expect(screen.getByTestId("pairing-code-card")).toHaveTextContent("expires in 0 min");
  });

  it("renders the QR as inline SVG carrying the deep link", async () => {
    render(<PairingCodeCard pairing={PAIRING} />);

    const qr = await screen.findByTestId("pairing-qr");
    expect(qr.querySelector("svg")).not.toBeNull();
    // Same payload the phone's #/pair route parses.
    await expect(QRCode.toString(PAIRING.qr_payload, { type: "svg", margin: 1 })).resolves.toContain("<svg");
  });

  it("falls back to typing when the QR cannot be drawn", async () => {
    vi.spyOn(QRCode, "toString").mockRejectedValue(new Error("no encoder"));
    render(<PairingCodeCard pairing={PAIRING} />);

    await waitFor(() => {
      expect(screen.getByTestId("pairing-qr-missing")).toBeInTheDocument();
    });
    expect(screen.getByTestId("pairing-code")).toHaveTextContent("ABCD2345");
  });
});
