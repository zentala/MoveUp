/**
 * useHealth.ts — one health view for both transports.
 *
 * Desktop (Tauri): `invoke("get_health_today")` once on mount, then the
 * shared exponential-poll ladder drives `refresh_health_now`.
 *
 * Remote display (browser): the phone has no IPC, so the health view rides
 * along in the `RemoteDisplayState` snapshot the WS stream already pushes
 * every ~1 s. `useRemoteDesk` publishes each snapshot's `health` here and
 * this hook replays it — no polling, no second network path.
 *
 * The publish/subscribe seam is module-level for the same reason
 * `subscribeVoiceAck` is: the WS connection is owned by whoever calls
 * `useRemoteDesk` (App, via `useDeskAuto`), while the widget rendering the
 * view sits elsewhere in the tree and cannot be handed the hook's options.
 */
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { useExponentialPoll } from "@/hooks/useExponentialPoll";
import type { HealthView } from "@/generated/HealthView";

const INITIAL_KICK_DELAY_MS = 500;
const SUCCESS_INTERVAL_MS = 5 * 60 * 1000;
const BACKOFF_LADDER_MS = [
  60 * 1000,
  2 * 60 * 1000,
  5 * 60 * 1000,
  10 * 60 * 1000,
  30 * 60 * 1000,
];

/** Whether we run inside Tauri (desktop) rather than a browser (remote display). */
export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
}

type HealthListener = (view: HealthView) => void;

const remoteListeners = new Set<HealthListener>();
/**
 * Last health view seen on the remote stream, replayed to late subscribers.
 *
 * Without it a widget mounted after the first snapshot would show "loading"
 * until the next one — indistinguishable from a dead stream.
 */
let lastRemoteHealth: HealthView | null = null;

/** Publishes the health slice of a remote snapshot to every subscriber. */
export function publishRemoteHealth(view: HealthView): void {
  lastRemoteHealth = view;
  for (const listener of remoteListeners) listener(view);
}

/** Subscribes to remote health snapshots. Returns the unsubscribe function. */
export function subscribeRemoteHealth(listener: HealthListener): () => void {
  remoteListeners.add(listener);
  return () => {
    remoteListeners.delete(listener);
  };
}

/** Current remote health view; the `useSyncExternalStore` read side. */
function getRemoteHealth(): HealthView | null {
  return lastRemoteHealth;
}

/** Drops the replay cache. Test-only seam; production never needs it. */
export function resetRemoteHealth(): void {
  lastRemoteHealth = null;
}

export interface UseHealthResult {
  /** null until the first view arrives from either transport. */
  view: HealthView | null;
  /** True while a manual or scheduled refresh is in flight (desktop only). */
  refreshing: boolean;
  /** No-op in remote mode, where the desktop owns every outbound call. */
  refresh: () => Promise<HealthView | null>;
  /** False in remote mode — the phone must not drive the upstream API. */
  canRefresh: boolean;
}

/** Reads today's health view for whichever transport this build runs on. */
export function useHealth(): UseHealthResult {
  const isTauri = isTauriRuntime();
  const [ipcView, setView] = useState<HealthView | null>(null);
  // Remote mode reads the published snapshot as an external store: no
  // effect-driven setState, and no window where a snapshot published
  // between render and subscribe would be missed.
  const remoteView = useSyncExternalStore(
    subscribeRemoteHealth,
    getRemoteHealth,
    getRemoteHealth,
  );
  const view = isTauri ? ipcView : remoteView;
  const [refreshing, setRefreshing] = useState(false);
  const inflightRef = useRef(false);

  const refresh = useCallback(async (): Promise<HealthView | null> => {
    if (!isTauri) return null;
    if (inflightRef.current) return null;
    inflightRef.current = true;
    setRefreshing(true);
    try {
      const v = await invoke<HealthView>("refresh_health_now");
      setView(v);
      return v;
    } catch (e) {
      const v: HealthView = {
        configured: true,
        snapshot: null,
        error_kind: "transient",
        error_message: String(e),
      };
      setView(v);
      return v;
    } finally {
      inflightRef.current = false;
      setRefreshing(false);
    }
  }, [isTauri]);

  // Desktop: warm the cached view once; the poll hook owns every refresh after.
  useEffect(() => {
    if (!isTauri) return;
    let cancelled = false;
    void (async () => {
      try {
        const v = await invoke<HealthView>("get_health_today");
        if (!cancelled) setView(v);
      } catch (e) {
        if (cancelled) return;
        setView({
          configured: true,
          snapshot: null,
          error_kind: "transient",
          error_message: String(e),
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [isTauri]);

  useExponentialPoll<HealthView | null>(refresh, {
    initialDelayMs: INITIAL_KICK_DELAY_MS,
    successIntervalMs: SUCCESS_INTERVAL_MS,
    backoffLadderMs: BACKOFF_LADDER_MS,
    isFailure: (v) => v?.error_kind === "transient",
    // Halt on auth_revoked — retrying without re-consent is pointless — and
    // in remote mode, where `refresh` is a no-op and re-arming the timer
    // would burn a wake-up on the phone for nothing.
    isTerminal: (v) => !isTauri || v?.error_kind === "auth_revoked",
  });

  return { view, refreshing, refresh, canRefresh: isTauri };
}
