/**
 * useTimelineNav.ts — selectedDay state for the Daily Timeline feature.
 *
 * Responsibilities:
 *   1. Hold the current `selectedDay` and clamp it to the active range.
 *   2. Live mode: at 02:00 local time auto-advance selectedDay to the new
 *      calendar day, but only if the user hasn't navigated manually in the
 *      configurable cooldown window (default 60 minutes).
 *   3. Expose `next`, `prev`, `goTo`, and a `manualSince` timestamp so the
 *      consumer can show "live" vs "browsing history" status.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { localIsoDate, shiftDay } from "../charts/timeline-utils";

export interface UseTimelineNavOptions {
  rangeFrom: string;
  rangeTo: string;
  /**
   * Hour-of-day (local) when live mode flips `selectedDay` forward. Defaults
   * to 2, matching the locked-in "02:00 flip" decision from the design spec.
   */
  flipHour?: number;
  /**
   * After a manual nav action, the auto-advance is paused for this many
   * milliseconds. Default 60 min.
   */
  manualCooldownMs?: number;
  /** Override "now" for tests. */
  nowMs?: () => number;
}

export interface TimelineNav {
  selectedDay: string;
  /** True if live mode (no manual nav recently AND selectedDay is up-to-date). */
  isLive: boolean;
  /** Timestamp (epoch ms) of last manual nav, or null if never. */
  manualSince: number | null;
  next: () => void;
  prev: () => void;
  goTo: (dateLocal: string) => void;
  goLive: () => void;
}

const DAY_MS = 86_400_000;
const DEFAULT_COOLDOWN_MS = 60 * 60 * 1000;
const DEFAULT_FLIP_HOUR = 2;

function clampDate(d: string, from: string, to: string): string {
  if (d < from) return from;
  if (d > to) return to;
  return d;
}

/**
 * What "today" should be in live mode, given the flip-hour rule. Before
 * `flipHour` (e.g. 02:00) we still consider yesterday's date "today" so the
 * label doesn't snap forward at midnight.
 */
export function liveSelectedDay(now: Date, flipHour: number): string {
  const out = new Date(now.getTime());
  if (out.getHours() < flipHour) {
    out.setDate(out.getDate() - 1);
  }
  return localIsoDate(out);
}

export function useTimelineNav(opts: UseTimelineNavOptions): TimelineNav {
  const {
    rangeFrom,
    rangeTo,
    flipHour = DEFAULT_FLIP_HOUR,
    manualCooldownMs = DEFAULT_COOLDOWN_MS,
    nowMs = Date.now,
  } = opts;

  const liveDayNow = liveSelectedDay(new Date(nowMs()), flipHour);
  const initial = clampDate(liveDayNow, rangeFrom, rangeTo);
  const [selectedDay, setSelectedDay] = useState<string>(initial);
  const [manualSince, setManualSince] = useState<number | null>(null);

  // Re-clamp when the active range changes from underneath us.
  useEffect(() => {
    setSelectedDay((prev) => clampDate(prev, rangeFrom, rangeTo));
  }, [rangeFrom, rangeTo]);

  // Tick once per minute to detect the live-mode flip.
  const tickRef = useRef<number | null>(null);
  useEffect(() => {
    const check = () => {
      const cooled = manualSince === null || nowMs() - manualSince > manualCooldownMs;
      if (!cooled) return;
      const target = clampDate(liveSelectedDay(new Date(nowMs()), flipHour), rangeFrom, rangeTo);
      setSelectedDay((prev) => (prev === target ? prev : target));
    };
    check();
    tickRef.current = window.setInterval(check, 60_000);
    return () => {
      if (tickRef.current !== null) window.clearInterval(tickRef.current);
    };
  }, [rangeFrom, rangeTo, flipHour, manualCooldownMs, manualSince, nowMs]);

  const markManual = useCallback(() => {
    setManualSince(nowMs());
  }, [nowMs]);

  const next = useCallback(() => {
    setSelectedDay((cur) => clampDate(shiftDay(cur, 1), rangeFrom, rangeTo));
    markManual();
  }, [rangeFrom, rangeTo, markManual]);

  const prev = useCallback(() => {
    setSelectedDay((cur) => clampDate(shiftDay(cur, -1), rangeFrom, rangeTo));
    markManual();
  }, [rangeFrom, rangeTo, markManual]);

  const goTo = useCallback(
    (date: string) => {
      setSelectedDay(clampDate(date, rangeFrom, rangeTo));
      markManual();
    },
    [rangeFrom, rangeTo, markManual],
  );

  const goLive = useCallback(() => {
    setManualSince(null);
    setSelectedDay(clampDate(liveSelectedDay(new Date(nowMs()), flipHour), rangeFrom, rangeTo));
  }, [rangeFrom, rangeTo, flipHour, nowMs]);

  const cooled = manualSince === null || nowMs() - manualSince > manualCooldownMs;
  const liveDay = clampDate(liveSelectedDay(new Date(nowMs()), flipHour), rangeFrom, rangeTo);
  const isLive = cooled && selectedDay === liveDay;

  return { selectedDay, isLive, manualSince, next, prev, goTo, goLive };
}

/** Days between two YYYY-MM-DD dates (b - a). Stable across DST. */
export function daysBetween(a: string, b: string): number {
  const da = new Date(`${a}T00:00:00Z`).getTime();
  const db = new Date(`${b}T00:00:00Z`).getTime();
  return Math.round((db - da) / DAY_MS);
}
