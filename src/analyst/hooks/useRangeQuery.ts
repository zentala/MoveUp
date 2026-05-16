/**
 * useRangeQuery.ts — Generic Tauri range query hook.
 *
 * Calls `invoke(command, { from, to })` whenever the range changes, with a
 * configurable debounce. Surfaces a discriminated state: loading → ready | error.
 * Never throws. Used by the analyst Explorer hooks to pull range-scoped history.
 */
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export type RangeQueryState<T> =
  | { status: "loading"; data: null }
  | { status: "ready"; data: T[] }
  | { status: "error"; data: null; error: string };

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
 * date input.
 */
export function useRangeQuery<T>(args: UseRangeQueryArgs): RangeQueryState<T> {
  const { command, from, to, debounceMs = 250, skip = false } = args;
  const [state, setState] = useState<RangeQueryState<T>>({
    status: "loading",
    data: null,
  });

  useEffect(() => {
    if (skip) return;
    let cancelled = false;
    setState({ status: "loading", data: null });
    const handle = setTimeout(() => {
      invoke<T[]>(command, { from, to })
        .then((data) => {
          if (cancelled) return;
          setState({ status: "ready", data });
        })
        .catch((err: unknown) => {
          if (cancelled) return;
          const message = err instanceof Error ? err.message : String(err);
          setState({ status: "error", data: null, error: message });
        });
    }, debounceMs);
    return () => {
      cancelled = true;
      clearTimeout(handle);
    };
  }, [command, from, to, debounceMs, skip]);

  return state;
}
