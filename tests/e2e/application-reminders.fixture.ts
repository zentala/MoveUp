/** Public pre-entry mocks around the REAL App. No native input, history or profile writes. */
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import type { AppReminderSnapshot, SessionStateDto } from '../../src/types';
type Category = NonNullable<AppReminderSnapshot['active']>['category'];
const scenario = new URLSearchParams(location.search).get('reminder-fixture') ?? 'normal';
let communicationId = 'default';
let currentDismissMinutes = scenario === 'reminder-snooze-unavailable' ? 0 : 5;
let snoozeMinutes: number | null = scenario === 'reminder-snooze-unavailable' ? null : 5;
const saved = sessionStorage.getItem('reminder-fixture-state');
let state: AppReminderSnapshot = saved ? JSON.parse(saved).state : { session_id: 'synthetic-reminder', revision: 0,
  active: null, pending: null, protected_flow: null, last_retirement: null, presentation_error: null, client_id: null };
let nextId = saved ? JSON.parse(saved).nextId : 0;
let applied = saved ? JSON.parse(saved).applied : 0;
let reserved: { id: string; action: string } | null = saved ? JSON.parse(saved).reserved : null;
let sequence = 0, failSave = false, raceSent = false, failedClaim = false;
let lastPreview = -60;
let clearPresentationOnNextFlow = false;
const calls: { command: string; args: unknown }[] = [], blocked: string[] = [];
const copy = () => structuredClone(state);
const save = () => sessionStorage.setItem('reminder-fixture-state', JSON.stringify({ state, nextId, applied, reserved }));
async function announce() { save(); await emit('desk:app-reminder', copy()); }
async function publish(category: Category = 'urgent', test = false, stress = false, reason: 'sitting_limit' | 'praise' = 'sitting_limit') {
  const entry: NonNullable<AppReminderSnapshot['active']> = {
    id: `${state.session_id}:${++nextId}`, source: test ? 'test' : reason === 'praise' ? 'session' : 'policy', reason: test ? 'preview' : reason,
    category, title: test ? 'Test reminder' : reason === 'praise' ? 'Halfway through your standing goal!' : 'Time to change position',
    body: test ? 'This is a test reminder. Your ergonomic session is unchanged.' : stress
      ? 'Your configured reminder is ready. Read this policy-supported explanation before choosing your next action. '.repeat(16)
      : reason === 'praise' ? 'Keep it up.' : 'Your configured sitting reminder is ready. Take a moment to change position.',
    is_test: test, elapsed_secs: test ? null : reason === 'praise' ? 450 : 319, limit_secs: test ? null : reason === 'praise' ? 900 : 1200,
  };
  if (state.protected_flow || state.active) state.pending = entry;
  else state.active = entry;
  state.revision++; await announce(); return copy();
}
const session: SessionStateDto = { state: 'Sitting', desk_height_cm: 72, sitting_seconds: 319, standing_seconds: 1200,
  break_seconds: 0, session_limit_secs: 1200, stand_limit_secs: 900, position_changes: 7, limit_used_secs: 319,
  daily_score: 12, standing_session_secs: 0, secs_since_last_break: 319, continuous_computer_secs: 319,
  longest_computer_session_secs: 1800, sitting_seconds_total: 3600, idle_secs: 0, away_bout_secs: 0, max_continuous_computer_secs: 3600 };

if (scenario !== 'remote') {
  mockWindows(scenario === 'other-window' ? 'analyst' : 'main');
  mockIPC(async (command, raw) => {
    const args = raw as Record<string, unknown>;
    calls.push({ command, args });
    switch (command) {
      case 'get_app_reminder_snapshot': {
        const old = copy();
        if (scenario === 'race' && !raceSent) {
          raceSent = true; await publish('warning');
          await new Promise(resolve => setTimeout(resolve, 50)); return old;
        }
        return old;
      }
      case 'claim_app_reminder_client':
        if (scenario === 'claim-race' && !failedClaim) {
          failedClaim = true; state.client_id = 'intervening-owner'; state.revision++; throw new Error('stale_client');
        }
        if (args.previousClientId !== state.client_id && args.clientId !== state.client_id) throw new Error('stale_client');
        state.client_id = args.clientId as string; sequence = 0; state.revision++; save(); return copy();
      case 'set_app_reminder_flow':
        if (args.clientId !== state.client_id || (args.sequence as number) <= sequence) throw new Error('stale_client_update');
        sequence = args.sequence as number; state.protected_flow = args.flow as AppReminderSnapshot['protected_flow'];
        if (clearPresentationOnNextFlow) { state.presentation_error = null; clearPresentationOnNextFlow = false; }
        if (!state.protected_flow && !state.active) { state.active = state.pending; state.pending = null; }
        state.revision++; await announce(); return copy();
      case 'acknowledge_app_reminder': {
        if (args.clientId !== state.client_id || args.id !== state.active?.id || args.revision !== state.revision) throw new Error('stale_reminder');
        if (reserved && (reserved.id !== args.id || (args.action !== 'retry' && reserved.action !== args.action))) throw new Error('acknowledgement_in_progress');
        if (!reserved && args.action === 'retry') throw new Error('no_pending_persistence');
        if (!state.active.is_test) {
          if (!reserved) { applied++; reserved = { id: state.active.id, action: args.action as string }; }
          if (failSave) { failSave = false; state.presentation_error = 'reminder_persistence_failed'; state.revision++; await announce(); throw new Error('reminder_persistence_failed'); }
        }
        state.last_retirement = { id: state.active.id, reason: 'acknowledged' };
        state.active = state.pending; state.pending = null; state.presentation_error = null; reserved = null;
        state.revision++; await announce(); return copy();
      }
      case 'trigger_test_notification': {
        const now = Math.floor(performance.now() / 1000);
        if (now - lastPreview < 60) throw new Error('Rate limited: wait before testing again');
        lastPreview = now;
        return publish((args?.category as Category) ?? 'info', true);
      }
      case 'get_dashboard_state': return { session, metrics: [] };
      case 'get_today_summary': return { sitting_secs: 3600, standing_secs: 1200, yesterday_sitting_secs: 0,
        yesterday_standing_secs: 0, position_changes: 7, sessions: [] };
      case 'get_settings': return { sitting_mm: 750, standing_mm: 1050, desk_thickness_mm: 30, timeline_skin: 'semantic' };
      case 'list_communication_profiles': return scenario === 'reminder-delay-return' ? [
        { id: 'default', name: 'Default', description: 'Configured reminders', path: 'SYNTHETIC' },
        { id: 'long-delay', name: 'Long delay', description: 'Configured reminders', path: 'SYNTHETIC' },
      ] : [{ id: communicationId, name: 'Current', description: 'Configured reminders', path: 'SYNTHETIC' }];
      case 'list_ergonomic_profiles': return [{ id: 'standard', name: 'Standard', description: 'Configured limits', path: 'SYNTHETIC' }];
      case 'switch_communication_profile':
        communicationId = args.id as string;
        currentDismissMinutes = communicationId === 'long-delay' ? 23 : 5;
        snoozeMinutes = currentDismissMinutes;
        return null;
      case 'get_active_profiles': return {
        communication_id: communicationId, ergonomic_id: 'standard', effective_limits: { sitting_secs: 1200, standing_secs: 1200,
          standing_target_secs: 900, standing_max_secs: 1800, break_min_secs: 60, break_credit_multiplier: 1,
          day_break_min_secs: 0, posture_balance_min_sitting_secs: 3600, max_continuous_computer_secs: 0, computer_break_reset_secs: 0 },
        session_limit_secs: 1200, stand_limit_secs: 900, notifications_enabled: true,
        dismiss_durations_mins: [currentDismissMinutes, 23], current_dismiss_minutes: currentDismissMinutes,
        snooze_minutes: snoozeMinutes, tone_shift_after_dismisses: 3, notify_cooldowns_secs: [60],
      };
      case 'get_connected_port': return 'SYNTHETIC';
      case 'get_health_today': case 'refresh_health_now': return { configured: false, sources: [], error: null };
      case 'start_auto_connect': case 'plugin:log|log': case 'get_overlay_state': return null;
      default: blocked.push(command); throw new Error(`Unexpected synthetic IPC: ${command}`);
    }
  }, { shouldMockEvents: true });
}
Object.assign(window, { reminderTest: {
  calls, blocked, publish, snapshot: copy, failNextSave: () => { failSave = true; }, applied: () => applied,
  late: (snapshot: AppReminderSnapshot) => emit('desk:app-reminder', snapshot),
  presentationFailure: async () => {
    state.presentation_error = 'main_show_not_visible'; state.revision++; clearPresentationOnNextFlow = true; await announce();
  },
  praise: () => publish('success', false, false, 'praise'),
  allowNextPreview: () => { lastPreview = -60; }, // Synthetic clock advance; never available in product.
} });
Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async () => {} } });
window.open = () => { throw new Error('External navigation forbidden in reminder fixture'); };
window.WebSocket = class {
  readyState = 0; close() {} send() { throw new Error('Display writes forbidden'); }
  addEventListener() {} removeEventListener() {}
} as unknown as typeof WebSocket;
declare global { interface Window { reminderTest: {
  calls: { command: string; args: unknown }[]; blocked: string[];
  publish: (category?: Category, test?: boolean, stress?: boolean) => Promise<AppReminderSnapshot>;
  snapshot: () => AppReminderSnapshot; failNextSave: () => void; applied: () => number;
  late: (snapshot: AppReminderSnapshot) => Promise<void>; allowNextPreview: () => void;
  presentationFailure: () => Promise<void>; praise: () => Promise<AppReminderSnapshot>;
} } }
await import('../../src/main');
