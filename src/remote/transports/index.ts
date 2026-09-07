/**
 * transports/index.ts — public surface plus the one selection rule (E022-T08).
 */
import { loadRelayRecord } from "../storage";
import { LanTransport } from "./lan";
import { RelayTransport } from "./relay";
import type { Transport } from "./types";

export * from "./types";
export { LanTransport } from "./lan";
export type { LanTransportOptions } from "./lan";
export { RelayTransport, relayWsUrl } from "./relay";
export type { RelayTransportOptions } from "./relay";

/**
 * Picks the wire for this page load.
 *
 * A stored pairing wins: a phone that has been paired is off-LAN more often
 * than not, and the relay path degrades gracefully (it shows the desk as
 * offline) where the LAN path just never connects. With no pairing there is
 * nothing to authenticate with, so the only thing left is the LAN socket —
 * `#/pair` (T09) is what creates the record in the first place.
 */
export function selectTransport(): Transport {
  const record = loadRelayRecord();
  return record ? new RelayTransport(record) : new LanTransport();
}
