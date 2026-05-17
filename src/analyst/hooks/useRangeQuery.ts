/**
 * useRangeQuery.ts — Generic Tauri range query hook.
 *
 * Calls `invoke(command, { from, to })` whenever the range changes, with a
 * configurable debounce. Surfaces a discriminated state: loading → ready | error.
 * Plus a `refetch` thunk a parent can wire to a "Refresh" button.
 * Never throws. Used by the analyst Explorer hooks to pull range-scoped history.
 */
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface BaseFields {
  /** Re-trigger the invoke. Skips the debounce — fires immediately. */
  refetch: () => void;
  /** Wall-clock time of the most recent `ready` result. `null` until first success. */
  lastRefreshed: number | null;
}

type RawRangeState<T> =
  | { status: "loading"; data: null }
  | { status: "ready"; data: T[] }
  | { status: "error"; data: null; error: string };

export type RangeQueryState<T> = RawRangeState<T> & BaseFields;

export interface UseRangeQueryArgs {
  command: string;
  from: string;
  to: string;
  debounceMs?: number;
  /** When true, the hook stays in `loading` and never invokes. */
  skip?: boolean;
}

/**
 * Generic range-scoped invoke hook. Debounces range changes by `debounceMs`
 * (default 250 ms) to avoid spamming the backend while the user types into a
 * date input. Manual refetch via the returned thunk skips the debounce.
 */
export function useRangeQuery<T>(args: UseRangeQueryArgs): RangeQueryState<T> {
  const { command, from, to, debounceMs = 250, skip = false } = args;
  const [nonce, setNonce] = useState(0);
  const [state, setState] = useState<RawRangeState<T>>({
    status: "loading",
    data: null,
  });
  const [lastRefreshed, setLastRefreshed] = useState<number | null>(null);

  const refetch = useCallback(() => setNonce((n) => n + 1), []);

  useEffect(() => {
    if (skip) return;
    let cancelled = false;
    setState({ status: "loading", data: null });
    const debounce = nonce > 0 ? 0 : debounceMs;
    const handle = setTimeout(() => {
      invoke<T[]>(command, { from, to })
        .then((data) => {
          if (cancelled) return;
          setState({ status: "ready", data });
          setLastRefreshed(Date.now());
        })
        .catch((err: unknown) => {
          if (cancelled) return;
          const message = err instanceof Error ? err.message : String(err);
          setState({ status: "error", data: null, error: message });
        });
    }, debounce);
    return () => {
      cancelled = true;
      clearTimeout(handle);
    };
  }, [command, from, to, debounceMs, skip, nonce]);

  return { ...state, refetch, lastRefreshed } as RangeQueryState<T>;
}
