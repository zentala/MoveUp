/** Browser-only synthetic IPC. Import before the real application bootstrap. */
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { SessionStateDto, MetricSnapshot, AppReminderSnapshot } from '../../src/types';

const scenario = new URLSearchParams(location.search).get('share-fixture') ?? 'normal';
const stress = scenario === 'stress';
const empty = scenario === 'empty';
const session: SessionStateDto = {
  state: 'Sitting', desk_height_cm: 72, sitting_seconds: 600,
  standing_seconds: stress ? 99999999 : 1200, break_seconds: 0,
  session_limit_secs: 2400, stand_limit_secs: 900,
  position_changes: stress ? 123456789 : 7, limit_used_secs: 600,
  daily_score: stress ? -123456789 : 12.4, standing_session_secs: 0,
  secs_since_last_break: 600, continuous_computer_secs: 600,
  longest_computer_session_secs: 1800, sitting_seconds_total: 3600,
  idle_secs: 0, away_bout_secs: 0, max_continuous_computer_secs: 3600,
};
const metrics: MetricSnapshot[] = empty ? [] : [
  { id: 'standing_pct', label: 'Standing', result: { value: .42, display: '42%', level: 'green', is_personal_best: false } },
  { id: 'longest_session', label: 'Longest session', result: { value: 1800, display: stress ? '123456h 59m' : '30m', level: 'yellow', is_personal_best: false } },
];
mockWindows('main');
let reminder: AppReminderSnapshot = { session_id: 'share-fixture', revision: 0, active: null, pending: null,
  protected_flow: null, last_retirement: null, presentation_error: null, client_id: null };
mockIPC((command, raw) => {
  const args = raw as Record<string, unknown>;
  switch (command) {
    case 'get_app_reminder_snapshot': return { ...reminder };
    case 'claim_app_reminder_client':
      if (args.previousClientId !== reminder.client_id) throw new Error('stale_client');
      reminder = { ...reminder, revision: reminder.revision + 1, client_id: args.clientId as string };
      return { ...reminder };
    case 'set_app_reminder_flow':
      if (args.clientId !== reminder.client_id) throw new Error('stale_client_update');
      reminder = { ...reminder, revision: reminder.revision + 1, protected_flow: args.flow as AppReminderSnapshot['protected_flow'] };
      return { ...reminder };
    case 'get_dashboard_state': return { session, metrics };
    case 'get_today_summary': return {
      sitting_secs: session.sitting_seconds_total, standing_secs: session.standing_seconds,
      yesterday_sitting_secs: 0, yesterday_standing_secs: 0,
      position_changes: session.position_changes, sessions: [],
    };
    case 'get_settings': return { timeline_skin: 'semantic' };
    case 'get_connected_port': return 'SYNTHETIC';
    case 'get_health_today': case 'refresh_health_now': return { configured: false, sources: [], error: null };
    case 'start_auto_connect': case 'plugin:log|log': return null;
    case 'get_overlay_state': return null;
    default: throw new Error(`Unexpected synthetic IPC: ${command}`);
  }
}, { shouldMockEvents: true });

// Capture rather than execute clipboard writes and external navigation.
Object.assign(window, { shareTest: { copied: '', opened: [] as string[] } });
Object.defineProperty(navigator, 'clipboard', { configurable: true, value: {
  writeText: async (text: string) => { window.shareTest.copied = text; },
} });
window.open = (url) => { window.shareTest.opened.push(String(url)); return null; };
window.WebSocket = class { constructor() { throw new Error('Display transport forbidden in Share fixture'); } } as typeof WebSocket;

declare global {
  interface Window { shareTest: { copied: string; opened: string[] } }
}
await import('../../src/main');
