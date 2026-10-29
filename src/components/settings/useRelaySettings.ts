/**
 * useRelaySettings.ts — the IPC half of Settings → More → Remote access (E022-T10).
 *
 * Kept out of the components so each of them stays a render function: the
 * section owns no data of its own, it owns four buttons that call in here.
 */
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PairingCode } from "@/generated/PairingCode";
import type { RelayStatus } from "@/generated/RelayStatus";
import type { Viewer } from "@/generated/Viewer";

/** States in which a registration exists, so viewers are worth asking for. */
const REGISTERED: RelayStatus["state"][] = ["connecting", "online", "unentitled", "revoked", "replaced", "error"];
const RELAY_STATES: RelayStatus["state"][] = ["disabled", ...REGISTERED];

export interface RelaySettings {
  status: RelayStatus | null;
  viewers: Viewer[];
  viewerState: "unknown" | "loading" | "loaded" | "error" | "not-applicable";
  pairing: PairingCode | null;
  error: string | null;
  statusError: "status" | "viewers" | null;
  loading: boolean;
  busy: boolean;
  refresh: () => Promise<void>;
  register: (licenseKey: string) => Promise<void>;
  startPairing: () => Promise<void>;
  revoke: (viewerId: string) => Promise<void>;
  disable: () => Promise<void>;
}

/** Loads relay status plus paired devices, and drives the four actions. */
export function useRelaySettings(): RelaySettings {
  const [status, setStatus] = useState<RelayStatus | null>(null);
  const [viewers, setViewers] = useState<Viewer[]>([]);
  const [viewerState, setViewerState] = useState<RelaySettings["viewerState"]>("unknown");
  const [pairing, setPairing] = useState<PairingCode | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [statusError, setStatusError] = useState<RelaySettings["statusError"]>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    setStatusError(null);
    // A previous status is not current evidence while its replacement is in flight.
    setStatus(null);
    setViewers([]);
    setViewerState("unknown");
    let next: RelayStatus;
    try {
      next = await invoke<RelayStatus>("get_relay_status");
      if (!next || !RELAY_STATES.includes(next.state)) throw new Error("invalid relay status");
    } catch {
      setStatusError("status");
      setLoading(false);
      return;
    }
    setStatus(next);
    if (!REGISTERED.includes(next.state)) {
      // An unregistered desk has no viewers endpoint to answer; asking would
      // surface "not registered" as an error the user cannot act on.
      setViewerState("not-applicable");
      setLoading(false);
      return;
    }
    setViewerState("loading");
    try {
      const nextViewers = await invoke<Viewer[]>("relay_list_viewers");
      setViewers(nextViewers);
      setViewerState("loaded");
    } catch {
      setViewers([]);
      setViewerState("error");
      setStatusError("viewers");
    } finally {
      setLoading(false);
    }
  }, []);

  const run = useCallback(
    async (action: () => Promise<void>) => {
      setBusy(true);
      setError(null);
      try {
        await action();
        await refresh();
      } catch (e) {
        setError(String(e));
      } finally {
        setBusy(false);
      }
    },
    [refresh],
  );

  // The first read goes straight to `refresh`, not through `run`: `run` flips
  // `busy` synchronously, and a mount is not a user action to guard against.
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return {
    status,
    viewers,
    viewerState,
    pairing,
    error,
    statusError,
    loading,
    busy,
    refresh,
    register: (licenseKey) =>
      run(async () => {
        await invoke("relay_register", { licenseKey });
      }),
    startPairing: () =>
      run(async () => {
        setPairing(await invoke<PairingCode>("relay_start_pairing"));
      }),
    revoke: (viewerId) =>
      run(async () => {
        await invoke("relay_revoke_viewer", { viewerId });
      }),
    disable: () =>
      run(async () => {
        await invoke("relay_disable");
        setPairing(null);
      }),
  };
}
