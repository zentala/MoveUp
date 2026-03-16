/**
 * TodayStats.tsx — displays today's total sitting and standing durations.
 *
 * Fetches data from `get_today_summary()` on mount and refreshes every 60 s.
 */
import { useEffect, useState } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TodaySummaryDto } from "@/types";
import { formatDurationShort } from "@/utils/format";

const REFRESH_INTERVAL_MS = 60_000;

/**
 * Renders today's total sitting and standing times,
 * auto-refreshing every 60 seconds.
 */
const TodayStats: FC = () => {
  const [summary, setSummary] = useState<TodaySummaryDto | null>(null);

  useEffect(() => {
    function fetchSummary() {
      invoke<TodaySummaryDto>("get_today_summary")
        .then(setSummary)
        .catch(console.error);
    }

    fetchSummary();
    const id = setInterval(fetchSummary, REFRESH_INTERVAL_MS);
    return () => clearInterval(id);
  }, []);

  if (!summary) {
    return <div className="today-stats today-stats--loading">Loading today's stats…</div>;
  }

  return (
    <div className="today-stats">
      <span className="today-stats__label">Today:</span>
      <span className="today-stats__item">
        sat <strong>{formatDurationShort(summary.sitting_secs)}</strong>
      </span>
      <span className="today-stats__sep">/</span>
      <span className="today-stats__item">
        stood <strong>{formatDurationShort(summary.standing_secs)}</strong>
      </span>
    </div>
  );
};

export default TodayStats;
