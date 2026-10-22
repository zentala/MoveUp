/** DailyScoreTrajectory.tsx — recorded signed profile-score samples over time. */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors, scaleLinear } from "./chart-utils";
import {
  formatScore,
  scoreObservations,
  sourceLocalTimeLabel,
  type ScoreObservation,
} from "./score-derivations";

export interface DailyScoreTrajectoryProps {
  data: SnapshotRow[];
  width?: number;
  height?: number;
}

function isPlottable(sample: ScoreObservation): sample is ScoreObservation & {
  atMs: number;
  score: number;
  localDate: string;
  timezoneOffsetSecs: number;
} {
  return sample.atMs !== null && sample.score !== null &&
    sample.localDate !== null && sample.timezoneOffsetSecs !== null;
}

function representativeTicks(samples: readonly (ScoreObservation & {
  atMs: number;
  score: number;
  localDate: string;
  timezoneOffsetSecs: number;
})[]): typeof samples[number][] {
  if (samples.length <= 5) return [...samples];
  const indices = [0, 0.25, 0.5, 0.75, 1].map((fraction) =>
    Math.round(fraction * (samples.length - 1)));
  return Array.from(new Set(indices), (index) => samples[index]);
}

function axisTimeLabel(sample: ScoreObservation): string {
  if (!sample.localDate) return "Unknown";
  const label = sourceLocalTimeLabel(sample);
  const clock = label.match(/ (\d{2}:\d{2}):\d{2} UTC/);
  return clock ? `${sample.localDate.slice(5)} ${clock[1]}` : "Unknown";
}

export function DailyScoreTrajectory({
  data,
  width = 720,
  height = 220,
}: DailyScoreTrajectoryProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  const innerH = height - m.top - m.bottom;
  const summary = scoreObservations(data);
  const samples = summary.observations.filter(isPlottable);
  const values = samples.map((sample) => sample.score);

  let minScore = values.length ? Math.min(0, ...values) : -1;
  let maxScore = values.length ? Math.max(0, ...values) : 1;
  if (minScore === maxScore) {
    minScore -= 1;
    maxScore += 1;
  }
  const y = scaleLinear(minScore, maxScore, innerH, 0);

  let minTime = samples.length ? samples[0].atMs : 0;
  let maxTime = samples.length ? samples[samples.length - 1].atMs : 1;
  if (minTime === maxTime) {
    minTime -= 30_000;
    maxTime += 30_000;
  }
  const x = scaleLinear(minTime, maxTime, 0, innerW);
  const scoreTicks = [0, 0.25, 0.5, 0.75, 1].map((fraction) =>
    minScore + fraction * (maxScore - minScore));
  const timeTicks = representativeTicks(samples);

  return (
    <ChartCard
      title="Daily score trajectory"
      subtitle="Recorded signed profile points; gaps are unobserved."
      help="Each point is a recorded daily score from the active ergonomic profile. Scores can be negative or exceed 100; they are not a percentage or health measure. Daily and long-break resets appear only through recorded score values—the snapshot has no reset-reason field. Points are not joined or interpolated. Local labels use the date and UTC offset stored with each sample."
    >
      <p className="score-coverage" aria-live="polite">
        {summary.recorded} recorded samples · {summary.unknown} unknown
      </p>
      {samples.length === 0 && (
        <p className="score-empty">
          {data.length === 0
            ? "No score samples recorded"
            : "No score samples can be plotted with recorded local time"}
        </p>
      )}
      <svg
        width={width}
        height={height}
        role="img"
        aria-label="Daily score trajectory with signed recorded samples"
      >
        <g transform={`translate(${m.left},${m.top})`}>
          {scoreTicks.map((score, index) => {
            const yy = y(score);
            return (
              <g key={index} transform={`translate(0,${yy})`}>
                <line
                  x1={0}
                  x2={innerW}
                  stroke={score === 0 ? chartColors.axis : chartColors.gridline}
                  strokeDasharray={score === 0 ? undefined : "3 4"}
                />
                <text x={-6} y={4} fontSize={10} textAnchor="end" fill={chartColors.axisText}>
                  {formatScore(score)}
                </text>
              </g>
            );
          })}
          {timeTicks.map((sample, index) => (
            <g key={`${sample.atMs}-${index}`} transform={`translate(${x(sample.atMs)},${innerH})`}>
              <line y2={4} stroke={chartColors.axis} />
              <text y={16} fontSize={10} fill={chartColors.axisText} textAnchor="middle">
                {axisTimeLabel(sample)}
              </text>
            </g>
          ))}
          {samples.map((sample, index) => {
            const label = `Recorded score ${formatScore(sample.score)} at ${sourceLocalTimeLabel(sample)}`;
            return (
              <circle
                key={`${sample.atMs}-${index}`}
                className="score-sample"
                cx={x(sample.atMs)}
                cy={y(sample.score)}
                r={2.8}
                fill={chartColors.score}
                aria-label={label}
                data-score={sample.score}
                data-at-ms={sample.atMs}
              >
                <title>{label}</title>
              </circle>
            );
          })}
          <text x={-m.left + 4} y={-2} fontSize={10} fill={chartColors.axisText}>
            score points
          </text>
        </g>
      </svg>
    </ChartCard>
  );
}
