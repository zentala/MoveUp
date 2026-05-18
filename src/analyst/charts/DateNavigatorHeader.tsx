/**
 * DateNavigatorHeader.tsx — Range pills + preset chips for DateNavigator.
 *
 * Two native date inputs bound to the active range, plus three preset
 * buttons (7d / 14d / 30d) that snap the range relative to `today`.
 */
import { chartColors } from "./chart-utils";
import { localIsoDate } from "./timeline-utils";

export interface DateNavRange {
  from: string;
  to: string;
}

export interface DateNavigatorHeaderProps {
  range: DateNavRange;
  onRangeChange: (r: DateNavRange) => void;
  today: string;
}

const PRESETS = [
  { id: "7d", label: "7d", days: 7 },
  { id: "14d", label: "14d", days: 14 },
  { id: "30d", label: "30d", days: 30 },
] as const;

function shiftDateBy(date: string, days: number): string {
  const d = new Date(`${date}T00:00:00`);
  d.setDate(d.getDate() + days);
  return localIsoDate(d);
}

function rangeDays(from: string, to: string): number {
  const a = new Date(`${from}T00:00:00`).getTime();
  const b = new Date(`${to}T00:00:00`).getTime();
  return Math.round((b - a) / 86_400_000) + 1;
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
  today,
}: DateNavigatorHeaderProps) {
  const activePresetDays = rangeDays(range.from, range.to);
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
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
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
        <input
          type="date"
          value={range.from}
          data-testid="range-from"
          onChange={(e) => onRangeChange({ ...range, from: e.target.value })}
          style={dateInputStyle}
        />
        <span style={{ color: chartColors.subtext }}>→</span>
        <input
          type="date"
          value={range.to}
          data-testid="range-to"
          onChange={(e) => onRangeChange({ ...range, to: e.target.value })}
          style={dateInputStyle}
        />
      </div>
      <div
        style={{
          display: "inline-flex",
          gap: 2,
          padding: 2,
          border: `1px solid ${chartColors.gridline}`,
          background: chartColors.background,
          borderRadius: 6,
        }}
      >
        {PRESETS.map((p) => {
          const on = p.days === activePresetDays;
          return (
            <button
              key={p.id}
              type="button"
              data-testid={`preset-${p.id}`}
              onClick={() =>
                onRangeChange({
                  from: shiftDateBy(today, -(p.days - 1)),
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
