import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import type { TimelineInterval, dayComposition } from "@/timeline/intervals";
import { dayComposition as composeDay } from "@/timeline/intervals";
import { useLocalDayBoundsRange } from "../hooks/useLocalDayBoundsRange";
import { DateNavigatorColumn } from "./DateNavigatorColumn";
import { enumerateDays, localIsoDate } from "./timeline-utils";
import "./day-composition.css";

export interface DateNavigatorProps {
  range: { from: string; to: string };
  selectedDay: string;
  intervals: readonly TimelineInterval[];
  panelId: string;
  busy?: boolean;
  error?: string | null;
  nowMs: number;
  onSelectDay: (dateLocal: string) => void;
}

type Composition = ReturnType<typeof dayComposition>;
interface CompositionState { byDay: Map<string, Composition>; error: string | null }

export function DateNavigator({ range, selectedDay, intervals, panelId, busy = false, error = null,
  nowMs, onSelectDay }: DateNavigatorProps) {
  const days = useMemo(() => enumerateDays(range.from, range.to), [range.from, range.to]);
  const boundsQuery = useLocalDayBoundsRange(range.from, range.to);
  const [focusedDay, setFocusedDay] = useState(selectedDay);
  const buttonRefs = useRef(new Map<string, HTMLButtonElement>());
  const tabsRef = useRef<HTMLDivElement>(null);
  const today = localIsoDate(new Date(nowMs));
  const compositions = useMemo<CompositionState>(() => {
    if (boundsQuery.status !== "ready") return { byDay: new Map(), error: null };
    try {
      const byDay = new Map<string, Composition>();
      for (const bounds of boundsQuery.data) {
        const startMs = Date.parse(bounds.start_utc);
        const endMs = Date.parse(bounds.end_utc);
        if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
          throw new RangeError(`Invalid query-local bounds for ${bounds.date_local}`);
        }
        byDay.set(bounds.date_local, composeDay(intervals, { startMs, endMs }, nowMs));
      }
      return { byDay, error: null };
    } catch (cause) {
      return { byDay: new Map(), error: cause instanceof Error ? cause.message : "Invalid interval data" };
    }
  }, [boundsQuery, intervals, nowMs]);

  const revealSelectedDay = useCallback(() => {
    const tabs = tabsRef.current;
    const button = buttonRefs.current.get(selectedDay);
    if (!tabs || !button || tabs.clientWidth <= 0) return;
    const tabsRect = tabs.getBoundingClientRect();
    const buttonRect = button.getBoundingClientRect();
    if (buttonRect.left < tabsRect.left) {
      tabs.scrollLeft = Math.max(0, tabs.scrollLeft + buttonRect.left - tabsRect.left);
    } else if (buttonRect.right > tabsRect.right) {
      tabs.scrollLeft = Math.max(0, tabs.scrollLeft + buttonRect.right - tabsRect.right);
    }
  }, [selectedDay]);

  useEffect(() => {
    setFocusedDay(selectedDay);
    revealSelectedDay();
  }, [selectedDay, range.from, range.to, revealSelectedDay]);

  useEffect(() => {
    const tabs = tabsRef.current;
    if (!tabs || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(revealSelectedDay);
    observer.observe(tabs);
    return () => observer.disconnect();
  }, [revealSelectedDay]);

  const focusDay = (day: string) => {
    setFocusedDay(day);
    buttonRefs.current.get(day)?.focus();
  };
  const onKeyDown = (index: number, event: KeyboardEvent<HTMLButtonElement>) => {
    let target: string | undefined;
    if (event.key === "ArrowLeft") target = days[Math.max(0, index - 1)];
    if (event.key === "ArrowRight") target = days[Math.min(days.length - 1, index + 1)];
    if (event.key === "Home") target = days[0];
    if (event.key === "End") target = days[days.length - 1];
    if (target) { event.preventDefault(); focusDay(target); }
  };
  const boundsLoading = boundsQuery.status === "loading";
  const visibleError = error ?? (boundsQuery.status === "error" ? boundsQuery.error : compositions.error);
  const loading = busy || boundsLoading;

  return <section className="day-composition" data-testid="date-navigator">
    <div ref={tabsRef} role="tablist" aria-label="Days in selected range" aria-orientation="horizontal"
      className="day-composition__tabs" data-testid="day-cols">
      {days.map((date, index) => <DateNavigatorColumn key={date} date={date}
        composition={loading || visibleError ? null : compositions.byDay.get(date) ?? null} active={date === selectedDay} today={date === today}
        loading={loading} error={visibleError} tabIndex={focusedDay === date ? 0 : -1}
        id={`analyst-day-${date}`} panelId={panelId}
        buttonRef={(button) => { if (button) buttonRefs.current.set(date, button); else buttonRefs.current.delete(date); }}
        onClick={() => onSelectDay(date)} onFocus={() => setFocusedDay(date)}
        onKeyDown={(event) => onKeyDown(index, event)} />)}
    </div>
    <p className="day-composition__basis" role="note">
      Calendar bounds use the current query-local timezone; historical source timezone is unknown. Unknown is uncovered elapsed time. Future is after now. Snapshot markers are instants and do not add duration.
    </p>
    {loading ? <p className="day-composition__status" role="status">Loading calendar bounds and observations…</p> : null}
    {visibleError ? <p className="day-composition__status" role="alert">Daily composition unavailable: {visibleError}</p> : null}
  </section>;
}
