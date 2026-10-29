/**
 * DateNavigatorHeader.tsx — Range pills + preset chips for DateNavigator.
 *
 * Two native date inputs bound to the active range, plus three preset
 * buttons (7d / 14d / 30d) that snap the range relative to `today`.
 */
import { chartColors } from "./chart-utils";
import { shiftDay } from "./timeline-utils";

export interface DateNavRange {
  from: string;
  to: string;
}

export interface DateNavigatorHeaderProps {
  range: DateNavRange;
  onRangeChange: (r: DateNavRange) => void;
  onToday: () => void;
  today: string;
}

const PRESETS = [
  { id: "7d", label: "7d", days: 7 },
  { id: "14d", label: "14d", days: 14 },
  { id: "30d", label: "30d", days: 30 },
] as const;

/** Ensure from ≤ to by swapping if necessary. */
function normaliseRange(r: DateNavRange): DateNavRange {
  return r.from <= r.to ? r : { from: r.to, to: r.from };
}

const dateInputStyle: React.CSSProperties = {
  background: chartColors.background,
  color: chartColors.text,
  border: `1px solid ${chartColors.gridline}`,
  borderRadius: 4,
  padding: "4px 8px",
  fontSize: 12,
  fontFamily: "inherit",
  colorScheme: "dark",
};

export function DateNavigatorHeader({
  range,
  onRangeChange,
  onToday,
  today,
}: DateNavigatorHeaderProps) {
  const activePreset = PRESETS.find((preset) =>
    range.from === shiftDay(today, -(preset.days - 1)) && range.to === today,
  )?.id;
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        gap: 12,
        marginBottom: 22,
        flexWrap: "wrap",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
        <span
          style={{
            fontSize: 10,
            letterSpacing: "0.2em",
            color: chartColors.subtext,
            textTransform: "uppercase",
          }}
        >
          Range
        </span>
        <label htmlFor="analyst-range-from" style={{ color: chartColors.subtext, fontSize: 11 }}>From</label>
        <input
          id="analyst-range-from"
          type="date"
          value={range.from}
          data-testid="range-from"
          onChange={(e) => onRangeChange(normaliseRange({ ...range, from: e.target.value }))}
          style={dateInputStyle}
        />
        <span style={{ color: chartColors.subtext }}>→</span>
        <label htmlFor="analyst-range-to" style={{ color: chartColors.subtext, fontSize: 11 }}>To</label>
        <input
          id="analyst-range-to"
          type="date"
          value={range.to}
          data-testid="range-to"
          onChange={(e) => onRangeChange(normaliseRange({ ...range, to: e.target.value }))}
          style={dateInputStyle}
        />
      </div>
      <div
        style={{
          display: "inline-flex",
          alignItems: "center",
          gap: 2,
          padding: 2,
          border: `1px solid ${chartColors.gridline}`,
          background: chartColors.background,
          borderRadius: 6,
        }}
      >
        <button
          type="button"
          data-testid="range-today"
          onClick={onToday}
          aria-label="Today"
          style={{
            background: "transparent",
            color: chartColors.text,
            border: "none",
            padding: "4px 10px",
            minHeight: 28,
            fontSize: 11,
            fontFamily: "inherit",
            borderRadius: 4,
            cursor: "pointer",
          }}
        >
          Today
        </button>
        {PRESETS.map((p) => {
          const on = p.id === activePreset;
          return (
            <button
              key={p.id}
              type="button"
              data-testid={`preset-${p.id}`}
              aria-pressed={on}
              onClick={() =>
                onRangeChange({
                  from: shiftDay(today, -(p.days - 1)),
                  to: today,
                })
              }
              style={{
                background: on ? chartColors.text : "transparent",
                color: on ? chartColors.card : chartColors.subtext,
                border: "none",
                padding: "4px 10px",
                fontSize: 11,
                fontFamily: "inherit",
                borderRadius: 4,
                cursor: "pointer",
              }}
            >
              {p.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}
