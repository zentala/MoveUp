/**
 * useExponentialPoll — self-rearming polling hook with exponential backoff
 * on failure and an initial kick delay.
 *
 * The caller's `fetch` function must return a result whose shape we can
 * inspect for failure via the `isFailure` predicate. On success, the next
 * tick fires after `successIntervalMs`. On failure, the next tick uses
 * `backoffLadderMs[Math.min(failures - 1, ladder.length - 1)]`. On a
 * "terminal" condition (e.g. auth revoked) the schedule halts.
 *
 * Concurrency: the hook serializes ticks via a single setTimeout —
 * there is never more than one outstanding scheduled call. Manual
 * out-of-band calls are the caller's responsibility (the hook does not
 * own a ref-guard around `fetch`).
 *
 * Lifetime: cancels its timer on unmount and ignores any in-flight
 * result that arrives after unmount (via a `cancelled` closure flag).
 */
import { useEffect } from "react";

export interface ExponentialPollOptions<T> {
  /** Delay before the first invocation. */
  initialDelayMs: number;
  /** Steady-state interval on success. */
  successIntervalMs: number;
  /** Backoff ladder used when `isFailure(result)` returns true. */
  backoffLadderMs: number[];
  /** Predicate distinguishing failure from success. */
  isFailure: (result: T) => boolean;
  /**
   * Predicate signaling a *terminal* failure that should HALT polling
   * (e.g. token revoked — retrying without re-consent is pointless).
   * Defaults to "never halt".
   */
  isTerminal?: (result: T) => boolean;
}

export function nextDelay(
  failures: number,
  successMs: number,
  ladder: number[],
): number {
  if (failures === 0) return successMs;
  const idx = Math.min(failures - 1, ladder.length - 1);
  return ladder[idx];
}

export function useExponentialPoll<T>(
  fetch: () => Promise<T>,
  opts: ExponentialPollOptions<T>,
): void {
  // Disable the exhaustive-deps lint for this closure-based scheduler;
  // the inputs are stable per-render and re-running on each render
  // would defeat the purpose of self-rearming.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | null = null;
    let failures = 0;

    const tick = async () => {
      if (cancelled) return;
      let result: T;
      try {
        result = await fetch();
      } catch (_e) {
        // Caller is expected to map exceptions into their result type;
        // treat unexpected throws as a failure to be safe.
        failures += 1;
        if (!cancelled) timer = setTimeout(tick, nextDelay(failures, opts.successIntervalMs, opts.backoffLadderMs));
        return;
      }
      if (cancelled) return;
      if (opts.isTerminal?.(result)) {
        // Halt — caller must re-mount to resume.
        return;
      }
      if (opts.isFailure(result)) {
        failures += 1;
      } else {
        failures = 0;
      }
      timer = setTimeout(tick, nextDelay(failures, opts.successIntervalMs, opts.backoffLadderMs));
    };

    timer = setTimeout(tick, opts.initialDelayMs);

    return () => {
      cancelled = true;
      if (timer !== null) clearTimeout(timer);
    };
  }, []);
}
