/**
 * TodayStats.tsx — displays today's total sitting and standing durations.
 *
 * Fetches data from SQLite via getTodaySummary() on mount and refreshes
 * every 60 s. Also ensures the DB schema is initialised by calling getDb()
 * on first render.
 */
import { useEffect, useState } from "react";
import type { FC } from "react";
import { getDb, getTodaySummary } from "@/db";
import { formatDurationShort } from "@/utils/format";

const REFRESH_INTERVAL_MS = 60_000;

/** Today's summary shape returned by getTodaySummary. */
interface TodaySummary {
  sitting_secs: number;
  standing_secs: number;
}

/**
 * Renders today's total sitting and standing times,
 * auto-refreshing every 60 seconds.
 */
const TodayStats: FC = () => {
  const [summary, setSummary] = useState<TodaySummary | null>(null);

  useEffect(() => {
    // Ensure schema is initialised before any queries run.
    getDb().catch(console.error);

    function fetchSummary() {
      getTodaySummary().then(setSummary).catch(console.error);
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
