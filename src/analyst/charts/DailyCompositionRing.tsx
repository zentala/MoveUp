import { useId } from "react";
import type { TimelineLabel, dayComposition } from "@/timeline/intervals";
import { timelineColors } from "@/timeline/colors";

export type DailyComposition = ReturnType<typeof dayComposition>;
export interface DailyCompositionRingProps {
  date: string;
  composition: DailyComposition | null;
  loading?: boolean;
}

const LABELS: readonly TimelineLabel[] = ["Sitting", "Standing", "Walking", "Away", "Unknown", "Future"];
const seconds = (ms: number) => (ms / 1000).toFixed(3);

function describe(date: string, composition: DailyComposition): string {
  const entries = LABELS.map((label) => {
    const ms = composition.totals[label];
    const pct = composition.dayMs === 0 ? 0 : 100 * ms / composition.dayMs;
    return `${label} ${pct.toFixed(2)} percent, ${seconds(ms)} seconds`;
  });
  const confirmed = LABELS.slice(0, 4).some((label) => composition.totals[label] > 0);
  const empty = !confirmed && composition.totals.Future === 0 ? "No observations recorded. " : "";
  return `${date}, full local calendar day denominator ${seconds(composition.dayMs)} seconds. ${empty}${entries.join(". ")}.`;
}

export function DailyCompositionRing({ date, composition, loading = false }: DailyCompositionRingProps) {
  const patternId = `unknown-hatch-${useId().replace(/:/g, "")}`;
  if (!composition) {
    return <span className="day-composition-ring day-composition-ring--empty" data-testid={`day-ring-${date}`} role="img"
      aria-label={loading ? `${date}, composition loading; no duration asserted` : `${date}, composition unavailable; elapsed coverage unknown`}>
      <span aria-hidden="true">{loading ? "…" : "?"}</span>
    </span>;
  }

  const circumference = 2 * Math.PI * 20;
  return <svg className="day-composition-ring" data-testid={`day-ring-${date}`} width={48} height={48} viewBox="0 0 48 48"
    role="img" aria-label={describe(date, composition)}>
    <defs><pattern id={patternId} width={4} height={4} patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <rect width={4} height={4} fill={timelineColors.Unknown} />
      <line x1={0} y1={0} x2={0} y2={4} stroke="var(--ink-tertiary, #a8a8a8)" strokeWidth={1} />
    </pattern></defs>
    <circle cx={24} cy={24} r={20} fill="none" stroke="var(--rail-subtle, #393939)" strokeWidth={1} aria-hidden="true" />
    {LABELS.map((label, index) => {
      const valueMs = composition.totals[label];
      const offsetMs = LABELS.slice(0, index).reduce((total, previousLabel) => total + composition.totals[previousLabel], 0);
      const segmentLength = circumference * valueMs / composition.dayMs;
      const segmentOffset = circumference * offsetMs / composition.dayMs;
      if (valueMs <= 0 || segmentLength <= 0) return null;
      return <circle key={label} data-state={label} cx={24} cy={24} r={20} fill="none"
        stroke={label === "Unknown" ? `url(#${patternId})` : timelineColors[label]}
        strokeWidth={label === "Future" ? 2 : 6}
        strokeDasharray={`${segmentLength} ${circumference - segmentLength}`}
        strokeDashoffset={-segmentOffset} strokeLinecap="butt" transform="rotate(-90 24 24)" aria-hidden="true" />;
    })}
    <text x={24} y={24} textAnchor="middle" dominantBaseline="central" className="day-composition-ring__date" aria-hidden="true">
      {date.slice(8, 10)}
    </text>
  </svg>;
}
