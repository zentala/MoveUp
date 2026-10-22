import type { KeyboardEvent, Ref } from "react";
import { DailyCompositionRing, type DailyComposition } from "./DailyCompositionRing";
import { dayOfMonthLabel, shortDayName } from "./timeline-utils";

export interface DateNavigatorColumnProps {
  date: string;
  composition: DailyComposition | null;
  active: boolean;
  today: boolean;
  loading: boolean;
  error: string | null;
  tabIndex: 0 | -1;
  id: string;
  panelId: string;
  buttonRef: Ref<HTMLButtonElement>;
  onClick: () => void;
  onFocus: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLButtonElement>) => void;
}

function coverage(composition: DailyComposition): string {
  if (composition.dayMs <= 0) return "Unknown";
  const knownMs = composition.dayMs - composition.totals.Unknown - composition.totals.Future;
  return `${(100 * knownMs / composition.dayMs).toFixed(1)}%`;
}

export function DateNavigatorColumn({ date, composition, active, today, loading, error, tabIndex, id, panelId,
  buttonRef, onClick, onFocus, onKeyDown }: DateNavigatorColumnProps) {
  const summary = loading ? "Composition loading; no duration asserted"
    : error ? `Composition unavailable; elapsed coverage unknown: ${error}`
      : composition ? coverage(composition) : "Composition unavailable; elapsed coverage unknown";
  return <button type="button" role="tab" id={id} aria-controls={panelId} aria-selected={active}
    aria-label={`${shortDayName(date)} ${date}${today ? ", today" : ""}; ${summary}` }
    aria-busy={loading || undefined} tabIndex={tabIndex} ref={buttonRef} onClick={onClick} onFocus={onFocus}
    onKeyDown={onKeyDown} data-testid={`day-${date}`} className={`day-tab${active ? " day-tab--selected" : ""}`}>
    <span className="day-tab__weekday">{shortDayName(date)}</span>
    <DailyCompositionRing date={date} composition={composition} loading={loading} />
    <span className="day-tab__date">{dayOfMonthLabel(date)}</span>
    <span className="day-tab__coverage day-tab__coverage--reserved">
      {composition ? `${coverage(composition)} recorded coverage` : loading ? "Loading" : "Unknown"}
    </span>
    {today ? <span className="day-tab__today">Today</span> : null}
  </button>;
}
