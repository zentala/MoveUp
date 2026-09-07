/**
 * useRelaySettings.ts — the IPC half of Settings → Remote (E022-T10).
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

export interface RelaySettings {
  status: RelayStatus | null;
  viewers: Viewer[];
  pairing: PairingCode | null;
  error: string | null;
  busy: boolean;
  register: (licenseKey: string) => Promise<void>;
  startPairing: () => Promise<void>;
  revoke: (viewerId: string) => Promise<void>;
  disable: () => Promise<void>;
}

/** Loads relay status plus paired devices, and drives the four actions. */
export function useRelaySettings(): RelaySettings {
  const [status, setStatus] = useState<RelayStatus | null>(null);
  const [viewers, setViewers] = useState<Viewer[]>([]);
  const [pairing, setPairing] = useState<PairingCode | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const next = await invoke<RelayStatus>("get_relay_status");
    setStatus(next);
    // An unregistered desk has no viewers endpoint to answer; asking would
    // surface "not registered" as an error the user cannot act on.
    setViewers(REGISTERED.includes(next.state) ? await invoke<Viewer[]>("relay_list_viewers") : []);
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
    refresh().catch((e) => setError(String(e)));
  }, [refresh]);

  return {
    status,
    viewers,
    pairing,
    error,
    busy,
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
