/**
 * TodayStats.tsx — displays today's total sitting and standing durations.
 *
 * Fetches data from SQLite via getTodaySummary() on mount and refreshes
 * every 60 s. Also ensures the DB schema is initialised by calling getDb()
 * on first render.
 */
import { useEffect, useState } from "react";
import type { FC } from "react";
import { listen } from "@tauri-apps/api/event";
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

    // Also refresh when desk state changes (session just ended).
    let unlisten: (() => void) | undefined;
    listen("desk:state-changed", fetchSummary).then((fn) => {
      unlisten = fn;
    });

    return () => {
      clearInterval(id);
      unlisten?.();
    };
  }, []);

  if (!summary) {
    return <div className="today-stats today-stats--loading">loading…</div>;
  }

  return (
    <div className="today-stats">
      <div className="today-stats__item">
        <span className="today-stats__value">{formatDurationShort(summary.sitting_secs)}</span>
        <span className="today-stats__label">sitting</span>
      </div>
      <div className="today-stats__item">
        <span className="today-stats__value">{formatDurationShort(summary.standing_secs)}</span>
        <span className="today-stats__label">standing</span>
      </div>
    </div>
  );
};

export default TodayStats;
