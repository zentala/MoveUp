/**
 * KpiDonut.tsx — One donut KPI card: an arc filled to `valuePct` with two
 * stacked SVG text nodes in the middle.
 *
 * Built on Recharts' `PieChart` (see ADR 016). The chart is given explicit
 * pixel dimensions rather than a `ResponsiveContainer` so it also renders
 * under jsdom, where a container has zero measured size.
 */
import { Cell, Label, Pie, PieChart } from "recharts";
import { chartColors } from "./chart-utils";

export interface KpiDonutProps {
  /** Caption under the donut. */
  label: string;
  /** Arc fill, 0..1. Values outside the range are clamped. */
  valuePct: number;
  /** Large text in the middle — usually the headline number. */
  centerPrimary: string;
  /** Smaller text under it — usually what the number represents. */
  centerSecondary?: string;
  /** Arc colour. Defaults to the standing green. */
  color?: string;
  /** Outer diameter in pixels. */
  size?: number;
  /** Disable the arc's entry/transition animation (tests, reduced motion). */
  animate?: boolean;
}

const TRACK_COLOR = "#222230";

function clamp01(v: number): number {
  if (!Number.isFinite(v)) return 0;
  return Math.min(1, Math.max(0, v));
}

interface CenterLabelProps {
  viewBox?: { cx?: number; cy?: number };
  primary: string;
  secondary?: string;
}

function CenterLabel({ viewBox, primary, secondary }: CenterLabelProps) {
  const cx = viewBox?.cx ?? 0;
  const cy = viewBox?.cy ?? 0;
  return (
    <g>
      <text
        x={cx}
        y={secondary ? cy - 4 : cy}
        textAnchor="middle"
        dominantBaseline="central"
        fill={chartColors.text}
        fontSize={22}
        fontWeight={600}
      >
        {primary}
      </text>
      {secondary ? (
        <text
          x={cx}
          y={cy + 16}
          textAnchor="middle"
          dominantBaseline="central"
          fill={chartColors.subtext}
          fontSize={11}
        >
          {secondary}
        </text>
      ) : null}
    </g>
  );
}

export function KpiDonut({
  label,
  valuePct,
  centerPrimary,
  centerSecondary,
  color = chartColors.standing,
  size = 132,
  animate = true,
}: KpiDonutProps) {
  const pct = clamp01(valuePct);
  const data = [
    { name: label, value: pct },
    { name: "remainder", value: 1 - pct },
  ];
  const outerRadius = size / 2 - 2;
  const innerRadius = outerRadius - Math.max(8, Math.round(size * 0.09));

  return (
    <div
      data-testid="kpi-donut"
      data-label={label}
      data-pct={pct.toFixed(4)}
      style={{ display: "flex", flexDirection: "column", alignItems: "center" }}
    >
      <PieChart width={size} height={size} role="img" aria-label={`${label}: ${centerPrimary}`}>
        <Pie
          data={data}
          dataKey="value"
          cx="50%"
          cy="50%"
          innerRadius={innerRadius}
          outerRadius={outerRadius}
          startAngle={90}
          endAngle={-270}
          stroke="none"
          isAnimationActive={animate}
          animationDuration={450}
        >
          <Cell fill={color} />
          <Cell fill={TRACK_COLOR} />
          <Label
            position="center"
            content={
              <CenterLabel primary={centerPrimary} secondary={centerSecondary} />
            }
          />
        </Pie>
      </PieChart>
      <div
        style={{
          marginTop: 4,
          fontSize: 10,
          color: chartColors.subtext,
          textTransform: "uppercase",
          letterSpacing: 0.5,
          textAlign: "center",
        }}
      >
        {label}
      </div>
    </div>
  );
}
