/**
 * DateRangePicker.tsx — Two native date inputs for from/to bounds,
 * with a "Last 7 days" reset shortcut.
 */
import { chartColors } from "./charts/chart-utils";

export interface DateRange {
  from: string;
  to: string;
}

export interface DateRangePickerProps {
  value: DateRange;
  onChange: (next: DateRange) => void;
}

const inputStyle: React.CSSProperties = {
  background: "#11111c",
  border: "1px solid #2a2a3a",
  color: chartColors.text,
  padding: "4px 8px",
  borderRadius: 4,
  fontSize: 12,
  fontFamily: "inherit",
  colorScheme: "dark",
};

const resetButtonStyle: React.CSSProperties = {
  background: "transparent",
  border: `1px solid ${chartColors.gridline}`,
  color: chartColors.subtext,
  padding: "4px 10px",
  borderRadius: 4,
  fontSize: 11,
  cursor: "pointer",
  fontFamily: "inherit",
};

function isoDate(d: Date): string {
  return d.toISOString().slice(0, 10);
}

/** Build the canonical "last 7 days" range ending today (local). */
export function defaultRange(now: Date = new Date()): DateRange {
  const to = isoDate(now);
  const fromDate = new Date(now);
  fromDate.setDate(fromDate.getDate() - 6);
  return { from: isoDate(fromDate), to };
}

export function DateRangePicker({ value, onChange }: DateRangePickerProps) {
  const handleReset = () => onChange(defaultRange());
  return (
    <div
      style={{ display: "flex", gap: 8, alignItems: "center" }}
      data-testid="date-range-picker"
    >
      <label style={{ fontSize: 11, color: chartColors.subtext }} lang="en">
        From
        <input
          type="date"
          lang="en"
          value={value.from}
          style={{ ...inputStyle, marginLeft: 6 }}
          onChange={(e) => onChange({ ...value, from: e.target.value })}
        />
      </label>
      <label style={{ fontSize: 11, color: chartColors.subtext }} lang="en">
        To
        <input
          type="date"
          lang="en"
          value={value.to}
          style={{ ...inputStyle, marginLeft: 6 }}
          onChange={(e) => onChange({ ...value, to: e.target.value })}
        />
      </label>
      <button
        type="button"
        style={resetButtonStyle}
        onClick={handleReset}
        title="Reset to the last 7 days"
      >
        Last 7 days
      </button>
    </div>
  );
}
