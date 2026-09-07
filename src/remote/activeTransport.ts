/**
 * activeTransport.ts — who currently owns the wire to the desk (E022-T09).
 *
 * `useRemoteDesk` builds exactly one transport per page load and keeps it
 * private, which is right for the data path: nothing else should be able to
 * reconnect or close it. But two things outside that hook need to know which
 * wire is up — the layout, to decide whether the controls exist at all, and
 * `RemoteControls`, to send a command. Passing the transport down would mean
 * widening `useWidgetData`'s contract for one consumer.
 *
 * So `selectTransport()` publishes what it picked here, and readers subscribe.
 * This is a registry of one, not a second source of transports: nothing in
 * this file constructs, connects or closes anything.
 */
import { useSyncExternalStore } from "react";
import type { Transport } from "./transports/types";

let active: Transport | null = null;
const listeners = new Set<() => void>();

/** Publishes the transport this page load is using. */
export function setActiveTransport(transport: Transport | null): void {
  active = transport;
  for (const listener of listeners) listener();
}

/** The transport in use, or `null` before one has been selected. */
export function getActiveTransport(): Transport | null {
  return active;
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Re-renders the caller whenever the active transport changes. */
export function useActiveTransport(): Transport | null {
  return useSyncExternalStore(subscribe, getActiveTransport, getActiveTransport);
}

/**
 * Whether this page may send commands at all.
 *
 * False on the LAN wire (unauthenticated, read-only by design) and false
 * before any transport exists — which is not the same fact, but has the same
 * answer: no controls.
 */
export function useControlCapability(): boolean {
  return useActiveTransport()?.capabilities.control ?? false;
}
