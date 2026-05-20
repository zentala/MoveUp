/**
 * DateNavigatorColumn.tsx — One clickable day column in DateNavigator.
 *
 * Renders 4 mini bars (Sit / Stand / Walk / Away) on a shared Y scale,
 * then the weekday name + day-of-month below as the clickable tab label.
 */
import { chartColors } from "./chart-utils";
import {
  dayOfMonthLabel,
  isWeekend,
  shortDayName,
  type DayStateTotals,
} from "./timeline-utils";

export const Y_AXIS_MIN_CAP_SECS = 8 * 3600;

const STATE_COLOR = {
  sit: chartColors.sitting,
  stand: chartColors.standing,
  walk: chartColors.walking,
  away: chartColors.away,
} as const;

export interface DateNavigatorColumnProps {
  date: string;
  totals: DayStateTotals;
  active: boolean;
  isToday: boolean;
  /** Y-axis cap in seconds (parent computes max across days). */
  yAxisCapSecs?: number;
  onClick: () => void;
}

function barHeight(secs: number, cap: number): string {
  return `${Math.min(100, (secs / cap) * 100)}%`;
}

function bar(color: string, height: string): React.CSSProperties {
  return {
    flex: 1,
    minHeight: 2,
    background: color,
    borderRadius: "1px 1px 0 0",
    height,
  };
}

export function DateNavigatorColumn({
  date,
  totals,
  active,
  isToday,
  yAxisCapSecs = Y_AXIS_MIN_CAP_SECS,
  onClick,
}: DateNavigatorColumnProps) {
  const we = isWeekend(date);
  const cap = yAxisCapSecs;
  const labelColor = active
    ? chartColors.primary
    : we
      ? "#555"
      : chartColors.subtext;
  const numColor = active ? chartColors.primary : we ? "#666" : chartColors.text;

  return (
    <button
      type="button"
      onClick={onClick}
      data-testid={`day-${date}`}
      aria-pressed={active}
      title={`${shortDayName(date)} ${date} — click to focus`}
      style={{
        position: "relative",
        display: "flex",
        flexDirection: "column",
        background: active ? "rgba(218,165,32,0.06)" : "transparent",
        border: "none",
        padding: "4px 4px 0",
        borderRadius: 6,
        cursor: "pointer",
        color: "inherit",
        fontFamily: "inherit",
      }}
    >
      {isToday ? (
        <span
          data-testid="today-stamp"
          style={{
            position: "absolute",
            top: -16,
            left: "50%",
            transform: "translateX(-50%)",
            fontSize: 10,
            color: chartColors.primary,
            letterSpacing: "0.04em",
            whiteSpace: "nowrap",
            textTransform: "uppercase",
          }}
        >
          today
        </span>
      ) : null}
      <div
        data-testid="mini-bars"
        style={{
          flex: 1,
          display: "flex",
          alignItems: "flex-end",
          justifyContent: "space-between",
          gap: 2,
          paddingBottom: 6,
          borderBottom: `1px solid ${chartColors.gridline}`,
          height: 110,
        }}
      >
        <div style={bar(STATE_COLOR.sit, barHeight(totals.sit, cap))} />
        <div style={bar(STATE_COLOR.stand, barHeight(totals.stand, cap))} />
        <div style={bar(STATE_COLOR.walk, barHeight(totals.walk, cap))} />
        <div style={bar(STATE_COLOR.away, barHeight(totals.away, cap))} />
      </div>
      <div style={{ paddingTop: 6, textAlign: "center" }}>
        <div
          style={{
            fontSize: 9,
            color: labelColor,
            letterSpacing: "0.12em",
            textTransform: "uppercase",
          }}
        >
          {shortDayName(date)}
        </div>
        <div
          style={{
            fontSize: 13,
            color: numColor,
            fontWeight: 600,
            marginTop: 3,
            fontVariantNumeric: "tabular-nums",
          }}
        >
          {dayOfMonthLabel(date)}
        </div>
      </div>
    </button>
  );
}

export { STATE_COLOR };
