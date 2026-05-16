/**
 * DateRangePicker.tsx — Two native date inputs for from/to bounds.
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
};

export function DateRangePicker({ value, onChange }: DateRangePickerProps) {
  return (
    <div
      style={{ display: "flex", gap: 8, alignItems: "center" }}
      data-testid="date-range-picker"
    >
      <label style={{ fontSize: 11, color: chartColors.subtext }}>
        From
        <input
          type="date"
          value={value.from}
          style={{ ...inputStyle, marginLeft: 6 }}
          onChange={(e) => onChange({ ...value, from: e.target.value })}
        />
      </label>
      <label style={{ fontSize: 11, color: chartColors.subtext }}>
        To
        <input
          type="date"
          value={value.to}
          style={{ ...inputStyle, marginLeft: 6 }}
          onChange={(e) => onChange({ ...value, to: e.target.value })}
        />
      </label>
    </div>
  );
}
