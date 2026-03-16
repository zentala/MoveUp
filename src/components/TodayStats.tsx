/**
 * TodayStats.tsx — displays today's total sitting and standing durations.
 *
 * Fetches data from Rust backend via get_today_summary command on mount
 * and refreshes every 60 s.
 */
import { useEffect, useState } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { formatDurationShort } from "@/utils/format";

const REFRESH_INTERVAL_MS = 60_000;

/** Today's summary shape returned by get_today_summary. */
interface TodaySummary {
  sitting_secs: number;
  standing_secs: number;
  sessions: Array<{
    id: number;
    started_at: string;
    ended_at: string | null;
    state: string;
    duration_seconds: number | null;
  }>;
}

/**
 * Renders today's total sitting and standing times,
 * auto-refreshing every 60 seconds.
 */
const TodayStats: FC = () => {
  const [summary, setSummary] = useState<TodaySummary | null>(null);

  useEffect(() => {
    function fetchSummary() {
      invoke<TodaySummary>("get_today_summary")
        .then(setSummary)
        .catch(console.error);
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
