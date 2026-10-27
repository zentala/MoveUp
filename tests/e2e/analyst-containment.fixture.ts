/** Synthetic public IPC only; install before the actual App/bootstrap imports. */
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { SessionStateDto } from '../../src/types';
import type { SessionRow } from '../../src/generated/SessionRow';

const session: SessionStateDto = {
  state: 'Standing', desk_height_cm: 108, sitting_seconds: 600,
  standing_seconds: 1200, break_seconds: 0, session_limit_secs: 2400,
  stand_limit_secs: 900, position_changes: 7, limit_used_secs: 600,
  daily_score: 12, standing_session_secs: 0, secs_since_last_break: 600,
  continuous_computer_secs: 600, longest_computer_session_secs: 1800,
  sitting_seconds_total: 3600, idle_secs: 0, away_bout_secs: 0,
  max_continuous_computer_secs: 3600,
};

// Anonymized dates/times, same individual Sitting-containing-Away conflict shape.
const rows: SessionRow[] = [
  { start: '2026-05-16T08:00:00+00:00', end: '2026-05-16T08:18:14+00:00', state: 'Sitting',
    duration_secs: 1094, break_credit: null, history_kind: 'individual' },
  { start: '2026-05-16T08:02:01+00:00', end: '2026-05-16T08:04:22+00:00', state: 'Away',
    duration_secs: 141, break_credit: 'partial', history_kind: 'individual' },
  { start: '2026-05-17T09:00:00+00:00', end: '2026-05-17T09:20:00+00:00', state: 'Standing',
    duration_secs: 1200, break_credit: 'full', history_kind: 'individual' },
];

Object.assign(window, { analystContainmentTest: {
  boot: crypto.randomUUID(), calls: [] as string[], blocked: [] as string[],
} });
localStorage.setItem('desk:analyst:range', JSON.stringify({ from: '2026-05-16', to: '2026-05-17' }));
localStorage.removeItem('desk:analyst:open-day');
mockWindows('analyst');
mockIPC(async (command, payload) => {
  window.analystContainmentTest.calls.push(command);
  switch (command) {
    case 'get_dashboard_state': return { session, metrics: [] };
    case 'get_snapshots_range': case 'get_events_range': return [];
    case 'get_sessions_range': {
      // Exercise actual asynchronous range hooks, not a substituted React hook.
      await new Promise(resolve => setTimeout(resolve, 120));
      const { from, to } = payload as { from: string; to: string };
      return rows.filter(row => row.start.slice(0, 10) >= from && row.start.slice(0, 10) <= to);
    }
    case 'get_local_day_bounds_range': {
      const { from, to } = payload as { from: string; to: string };
      const bounds = [];
      for (let at = Date.parse(`${from}T00:00:00Z`); at <= Date.parse(`${to}T00:00:00Z`); at += 86400000) {
        const day = new Date(at).toISOString().slice(0, 10);
        bounds.push({ date_local: day, start_utc: new Date(at).toISOString(),
          end_utc: new Date(at + 86400000).toISOString(), zone_basis: 'current-query-local',
          historical_source_zone: 'unknown', hour_ticks: Array.from({ length: 24 }, (_, hour) => ({
            at_utc: new Date(at + hour * 3600000).toISOString(),
            wall_time: `${String(hour).padStart(2, '0')}:00`, utc_offset_secs: 0,
          })),
        });
      }
      return bounds;
    }
    case 'plugin:log|log': return null; // In-memory synthetic logging only.
    default:
      window.analystContainmentTest.blocked.push(command);
      throw new Error(`Native IPC forbidden in Analyst fixture: ${command}`);
  }
}, { shouldMockEvents: true });

window.open = () => { throw new Error('External navigation forbidden in Analyst fixture'); };
window.WebSocket = class { constructor() { throw new Error('Display transport forbidden in Analyst fixture'); } } as typeof WebSocket;

declare global {
  interface Window {
    analystContainmentTest: { boot: string; calls: string[]; blocked: string[] };
  }
}
await import('../../src/main');
