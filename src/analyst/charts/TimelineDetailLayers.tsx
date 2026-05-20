/**
 * TimelineDetailLayers.tsx — SVG layers rendered inside TimelineDetail.
 *
 * Split out of TimelineDetail.tsx to keep that file under the project's
 * 250-line cap. Each component receives pre-computed data (no math here);
 * the parent computes scale + hour ticks + day boundaries once and passes
 * them down.
 */
import { chartColors } from "./chart-utils";
import type { HourTick } from "./timeline-segments";
import { msToPx, type TimelineScale } from "./timeline-utils";

export interface HourTickLayerProps {
  ticks: HourTick[];
  scale: TimelineScale;
  stripHeight: number;
}

export function HourTickLayer({ ticks, scale, stripHeight }: HourTickLayerProps) {
  return (
    <g data-testid="hour-ticks">
      {ticks.map((tick) => {
        const x = msToPx(tick.ms, scale);
        return (
          <g key={tick.ms}>
            <line
              x1={x}
              x2={x}
              y1={stripHeight - 14}
              y2={stripHeight - 6}
              stroke={chartColors.gridline}
              strokeWidth={1}
            />
            {tick.isLabel ? (
              <text
                x={x}
                y={stripHeight - 1}
                fontSize={9}
                fill={chartColors.subtext}
                textAnchor="middle"
              >
                {String(tick.hour).padStart(2, "0")}:00
              </text>
            ) : null}
          </g>
        );
      })}
    </g>
  );
}

export interface DayDividerLayerProps {
  dividers: number[];
  scale: TimelineScale;
  stripHeight: number;
}

export function DayDividerLayer({ dividers, scale, stripHeight }: DayDividerLayerProps) {
  return (
    <>
      {dividers.map((t) => {
        const x = msToPx(t, scale);
        return (
          <g key={t}>
            <line
              x1={x}
              x2={x}
              y1={6}
              y2={stripHeight - 6}
              stroke={chartColors.gridline}
              strokeWidth={1}
            />
            <text x={x + 4} y={14} fontSize={9} fill={chartColors.subtext}>
              {new Date(t).toISOString().slice(5, 10)}
            </text>
          </g>
        );
      })}
    </>
  );
}

export interface NowIndicatorProps {
  x: number;
  stripHeight: number;
}

export function NowIndicator({ x, stripHeight }: NowIndicatorProps) {
  return (
    <g>
      <line
        x1={x}
        x2={x}
        y1={0}
        y2={stripHeight}
        stroke={chartColors.primary}
        strokeWidth={1.5}
      />
      <rect
        x={x - 18}
        y={stripHeight - 14}
        width={36}
        height={12}
        fill={chartColors.primary}
      />
      <text
        x={x}
        y={stripHeight - 5}
        fontSize={9}
        fill={chartColors.card}
        textAnchor="middle"
        fontWeight={600}
      >
        NOW
      </text>
    </g>
  );
}
