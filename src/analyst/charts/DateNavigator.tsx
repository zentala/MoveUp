/**
 * DateNavigator.tsx — Range header + day tabs for the Daily Timeline feature.
 *
 * Replaces the standalone DateRangePicker on the Explorer tab. Renders 14
 * (or however many days are in range) grouped mini bar charts side by side,
 * each acting as a clickable tab that drives the TimelineDetail strip below.
 *
 * Sub-components live in sibling files to respect the 250-line-per-file cap:
 *   - DateNavigatorHeader: range pills + preset chips
 *   - DateNavigatorColumn: single day column (mini bars + tab label)
 */
import { useMemo } from "react";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { chartColors } from "./chart-utils";
import { DateNavigatorColumn, STATE_COLOR } from "./DateNavigatorColumn";
import { DateNavigatorHeader, type DateNavRange } from "./DateNavigatorHeader";
import { aggregateDayTotals, enumerateDays } from "./timeline-utils";

export type { DateNavRange };

export interface DateNavigatorProps {
  range: DateNavRange;
  onRangeChange: (r: DateNavRange) => void;
  selectedDay: string;
  onSelectDay: (date: string) => void;
  snapshots: SnapshotRow[];
  /** Date considered "today" for the TODAY pill. Defaults to local now. */
  today?: string;
}

function todayLocal(): string {
  return new Date().toISOString().slice(0, 10);
}

const LABELS: ReadonlyArray<["sit" | "stand" | "walk" | "away", string]> = [
  ["sit", "Sit"],
  ["stand", "Stand"],
  ["walk", "Walk"],
  ["away", "Away"],
];

export function DateNavigator({
  range,
  onRangeChange,
  selectedDay,
  onSelectDay,
  snapshots,
  today = todayLocal(),
}: DateNavigatorProps) {
  const days = useMemo(() => enumerateDays(range.from, range.to), [range.from, range.to]);
  const totalsMap = useMemo(() => aggregateDayTotals(snapshots), [snapshots]);

  return (
    <div
      data-testid="date-navigator"
      style={{
        background: chartColors.card,
        border: `1px solid ${chartColors.gridline}`,
        borderRadius: 8,
        padding: "14px 18px 16px",
      }}
    >
      <DateNavigatorHeader range={range} onRangeChange={onRangeChange} today={today} />

      <div style={{ display: "grid", gridTemplateColumns: "40px 1fr", gap: 14 }}>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            justifyContent: "space-between",
            paddingBottom: 36,
            height: 150,
            textAlign: "right",
            fontSize: 9,
            color: "#555",
            fontVariantNumeric: "tabular-nums",
            letterSpacing: "0.08em",
          }}
        >
          <span>8h</span>
          <span>4h</span>
          <span>0h</span>
        </div>
        <div
          data-testid="day-cols"
          style={{
            display: "grid",
            gridTemplateColumns: `repeat(${days.length || 1}, 1fr)`,
            gap: 5,
            position: "relative",
            height: 150,
          }}
        >
          {days.map((d) => (
            <DateNavigatorColumn
              key={d}
              date={d}
              totals={
                totalsMap.get(d) ?? {
                  dateLocal: d,
                  sit: 0,
                  stand: 0,
                  walk: 0,
                  away: 0,
                  total: 0,
                }
              }
              active={d === selectedDay}
              isToday={d === today}
              onClick={() => onSelectDay(d)}
            />
          ))}
        </div>
      </div>

      <div
        style={{
          display: "flex",
          gap: 14,
          marginTop: 18,
          paddingTop: 12,
          borderTop: `1px solid ${chartColors.gridline}`,
          fontSize: 11,
          color: chartColors.subtext,
          alignItems: "center",
        }}
      >
        {LABELS.map(([k, label]) => (
          <span key={k} style={{ display: "inline-flex", alignItems: "center", gap: 6 }}>
            <span
              style={{
                width: 9,
                height: 9,
                borderRadius: 2,
                background: STATE_COLOR[k],
                display: "inline-block",
              }}
            />
            {label}
          </span>
        ))}
        <span style={{ marginLeft: "auto", fontSize: 11, fontStyle: "italic" }}>
          click a day to focus the timeline below
        </span>
      </div>
    </div>
  );
}
