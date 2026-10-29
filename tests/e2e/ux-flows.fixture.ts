/** Browser-only public IPC fixture around the real App; no native writes. */
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import type { AppReminderSnapshot, LastBreakCreditFacts, MetricSnapshot, SessionRow, SessionStateDto } from '../../src/types';

const scenario = new URLSearchParams(location.search).get('settings-fixture') ?? 'normal';
const welcomeEntry = location.pathname.endsWith('/welcome.html');
const calls: { command: string; args: unknown }[] = [];
const blocked: string[] = [];
const settings = {
  sitting_mm: 750, standing_mm: 1050, desk_thickness_mm: 30,
  active_widget: 'one-bar', timeline_skin: 'semantic', show_welcome_on_startup: true,
  show_activity_status: true, telemetry_enabled: false, telemetry_device_id: '',
  notify_webhook_enabled: false, notify_webhook_url: null, voice_ai_model: null,
  relay_enabled: false, relay_url: 'https://relay.desk.zentala.io', remote_lan_enabled: true,
};
const effectiveLimits = {
  sitting_secs: 2400, standing_secs: 60, standing_target_secs: 900, standing_max_secs: 5400,
  break_min_secs: 60, break_credit_multiplier: 2, day_break_min_secs: 21600,
  posture_balance_min_sitting_secs: 21600, max_continuous_computer_secs: 3600, computer_break_reset_secs: 300,
};
let activeProfiles = {
  communication_id: scenario === 'runtime-profile' ? 'custom-communication' : 'default',
  ergonomic_id: scenario === 'runtime-profile' ? 'custom-live' : 'standard',
  effective_limits: { ...effectiveLimits },
  session_limit_secs: scenario === 'runtime-profile' ? 3300 : 2400,
  stand_limit_secs: 900,
  notifications_enabled: true,
  dismiss_durations_mins: scenario === 'runtime-profile' ? [0, 11, 23] : [5, 15, 30],
  current_dismiss_minutes: scenario === 'runtime-profile' ? 0 : 5,
  snooze_minutes: scenario === 'runtime-profile' ? null : 5,
  tone_shift_after_dismisses: 2,
  notify_cooldowns_secs: [0, 45, 90],
};
const session: SessionStateDto = {
  state: 'Sitting', desk_height_cm: 72, sitting_seconds: 600, standing_seconds: 1200,
  break_seconds: 0, session_limit_secs: 2400, stand_limit_secs: 900, position_changes: 7,
  limit_used_secs: 600, daily_score: 12, standing_session_secs: 0, secs_since_last_break: 600,
  continuous_computer_secs: 600, longest_computer_session_secs: 1800, sitting_seconds_total: 3600,
  idle_secs: 0, away_bout_secs: 0, max_continuous_computer_secs: 3600,
};
const noReadingSession: SessionStateDto = {
  ...session,
  desk_height_cm: 0,
  live_observation: {
    span: null,
    last_observation_at: null,
    snapshot_at: '2026-10-29T00:00:00.000Z',
    quality: null,
    reason: 'no_observation',
  },
};
const knownObservationThroughMs = Date.now() - 2 * 60_000;
const knownSittingSession: SessionStateDto = {
  ...session,
  live_observation: {
    span: { state: 'Sitting', started_at: new Date(knownObservationThroughMs - 10 * 60_000).toISOString(), observed_through: new Date(knownObservationThroughMs).toISOString() },
    last_observation_at: new Date(knownObservationThroughMs).toISOString(),
    snapshot_at: new Date(knownObservationThroughMs + 1000).toISOString(),
    quality: 'known',
    reason: null,
  },
};
const inferredAwaySession: SessionStateDto = {
  ...noReadingSession,
  state: 'Away',
  idle_secs: 600,
  away_bout_secs: 600,
};
const staleSession: SessionStateDto = {
  ...session,
  live_observation: {
    span: { state: 'Sitting', started_at: '2026-10-29T09:30:00Z', observed_through: '2026-10-29T09:40:00Z' },
    last_observation_at: '2026-10-29T09:40:00Z',
    snapshot_at: '2026-10-29T10:00:00Z',
    quality: null,
    reason: 'stale',
  },
};
const breakCreditFacts: LastBreakCreditFacts[] = [
  {
    break_started_at: '2026-10-29T10:00:00Z', observed_break_secs: 300,
    minimum_break_secs: 60, multiplier: 2, requested_credit_secs: 600,
    prior_applied_credit_secs: 0, sitting_before_secs: 480,
    applied_credit_secs: 480, sitting_after_secs: 0,
    applied_at: '2026-10-29T10:05:00Z', no_credit_reason: null,
  },
  {
    break_started_at: '2026-10-29T10:00:00Z', observed_break_secs: 600,
    minimum_break_secs: 60, multiplier: 1, requested_credit_secs: 120,
    prior_applied_credit_secs: 480, sitting_before_secs: 720,
    applied_credit_secs: 120, sitting_after_secs: 600,
    applied_at: '2026-10-29T10:10:00Z', no_credit_reason: null,
  },
  {
    break_started_at: '2026-10-29T10:00:00Z', observed_break_secs: 600,
    minimum_break_secs: 60, multiplier: 1, requested_credit_secs: 120,
    prior_applied_credit_secs: 600, sitting_before_secs: 600,
    applied_credit_secs: 0, sitting_after_secs: 600,
    applied_at: '2026-10-29T10:11:00Z', no_credit_reason: 'already_credited_prefix',
  },
];
const breakCreditSession: SessionStateDto = {
  ...session, limit_used_secs: 3120, last_break_credit_facts: breakCreditFacts,
};
const popupFullSession: SessionStateDto = {
  ...knownSittingSession,
  last_break_credit_facts: breakCreditFacts.slice(1, 2),
};
const popupOverdueSession: SessionStateDto = {
  ...popupFullSession,
  sitting_seconds: 2558,
  limit_used_secs: 2558,
};
const popupOverdueDayStart = new Date(knownObservationThroughMs);
popupOverdueDayStart.setHours(0, 0, 0, 0);
const popupOverdueOvernightSitting: SessionRow = {
  start: new Date(popupOverdueDayStart.getTime() - 3 * 60_000).toISOString(),
  end: new Date(popupOverdueDayStart.getTime() + 48 * 60_000).toISOString(),
  state: 'Sitting',
  duration_secs: 51 * 60,
  break_credit: null,
  history_kind: 'individual',
};
const popupOverduePreviousStanding: SessionRow = {
  start: new Date(knownObservationThroughMs - 40 * 60_000).toISOString(),
  end: new Date(knownObservationThroughMs - 20 * 60_000).toISOString(),
  state: 'Standing',
  duration_secs: 20 * 60,
  break_credit: null,
  history_kind: 'individual',
};
const popupOverduePreviousAway: SessionRow = {
  start: new Date(knownObservationThroughMs - 17 * 60_000).toISOString(),
  end: new Date(knownObservationThroughMs - 14 * 60_000).toISOString(),
  state: 'Away',
  duration_secs: 180,
  break_credit: null,
  history_kind: 'individual',
};
const popupOverduePreviousSitting: SessionRow = {
  start: new Date(knownObservationThroughMs - 14 * 60_000).toISOString(),
  end: new Date(knownObservationThroughMs - 10 * 60_000).toISOString(),
  state: 'Sitting',
  duration_secs: 240,
  break_credit: null,
  history_kind: 'individual',
};
const popupFullMetrics: MetricSnapshot[] = [
  { id: 'standing_pct', label: 'Standing', result: { value: .42, display: '42%', level: 'green', is_personal_best: false } },
  { id: 'position_rate', label: 'Posture changes', result: { value: 1.2, display: '1.2/h', level: 'green', is_personal_best: false } },
  { id: 'hourly_breaks', label: 'Hourly breaks', result: { value: 3, display: '3/4', level: 'yellow', is_personal_best: false } },
  { id: 'longest_session', label: 'Longest screen session', result: { value: 1800, display: '30m', level: 'green', is_personal_best: false } },
];
let loadAttempts = 0;
let failNextLoad = false;
let saveAttempts = 0;
let welcomeOpenAttempts = 0;
let rejectSave: (() => void) | null = null;
let dashboardRefreshShouldFail = false;
let rejectDashboardRefresh: (() => void) | null = null;
let relayStatusRejected = false;
const pendingRelayStatusRejects: (() => void)[] = [];
let clientId: string | null = null;
let flow: string | null = null;
let failShareRelease = scenario === 'share-close-error';
let failReminderAck = scenario === 'reminder-x-ack-fail';
let failReminderHide = scenario === 'reminder-x-hide-fail';
let reminderHideAttempts = 0;
let resolvePendingReminderHide: (() => void) | null = null;
let revision = 0;
const reminderMessage: NonNullable<AppReminderSnapshot['active']> = {
  id: 'ux-reminder-x:1', source: 'policy', reason: 'sitting_limit', category: 'urgent',
  title: 'Time to stand up', body: 'Your configured reminder is ready.', is_test: false,
  elapsed_secs: 600, limit_secs: 1200,
};
const reminder: AppReminderSnapshot = {
  session_id: 'ux-settings-fixture', revision, active: scenario.startsWith('reminder-x') ? reminderMessage : null, pending: null,
  protected_flow: null, last_retirement: null, presentation_error: null, client_id: null,
};
const copyReminder = (): AppReminderSnapshot => ({ ...reminder, revision, client_id: clientId, protected_flow: flow });

mockWindows(welcomeEntry ? 'welcome' : 'main');
mockIPC(async (command, raw) => {
  const args = raw as Record<string, unknown>;
  calls.push({ command, args });
  switch (command) {
    case 'get_app_reminder_snapshot': return copyReminder();
    case 'claim_app_reminder_client':
      if (args.previousClientId !== clientId && args.clientId !== clientId) throw new Error('stale_client');
      clientId = args.clientId as string; revision++; return copyReminder();
    case 'set_app_reminder_flow':
      if (args.clientId !== clientId) throw new Error('stale_client_update');
      if (args.flow === null && flow === 'share' && failShareRelease) {
        failShareRelease = false;
        throw new Error('synthetic Share release failure');
      }
      flow = args.flow as string | null; revision++; reminder.protected_flow = flow as AppReminderSnapshot['protected_flow'];
      if (!flow && !reminder.active && reminder.pending) { reminder.active = reminder.pending; reminder.pending = null; }
      return copyReminder();
    case 'acknowledge_app_reminder':
      if (args.clientId !== clientId || args.id !== reminder.active?.id) throw new Error('stale_reminder_action');
      if (args.action === 'dismiss' && failReminderAck) {
        failReminderAck = false;
        reminder.presentation_error = 'reminder_persistence_failed';
        throw new Error('reminder_persistence_failed');
      }
      if (args.action !== 'dismiss' && args.action !== 'retry' && args.action !== 'snooze') throw new Error('invalid_reminder_action');
      reminder.active = null; reminder.presentation_error = null; revision++;
      return copyReminder();
    case 'get_dashboard_state': {
      if (scenario === 'view-refresh-error' && dashboardRefreshShouldFail) {
        return new Promise((_, reject) => {
          rejectDashboardRefresh = () => reject(new Error('synthetic dashboard refresh failure'));
        });
      }
      const dashboardSession = scenario === 'no-reading' ? noReadingSession
        : scenario === 'inferred-away' ? inferredAwaySession
        : scenario === 'stale-observation' ? staleSession
        : scenario === 'activity' || scenario === 'activity-save-fail' ? knownSittingSession
        : scenario === 'known-sitting' ? knownSittingSession
        : scenario === 'view-refresh-error' ? knownSittingSession
        : scenario === 'popup-full' ? popupFullSession
        : scenario === 'popup-overdue' ? popupOverdueSession
        : scenario === 'break-credit' ? breakCreditSession
        : session;
      return { session: dashboardSession, metrics: scenario === 'popup-full' || scenario === 'popup-overdue' ? popupFullMetrics : [] };
    }
    case 'get_today_summary': return {
      sitting_secs: scenario === 'popup-overdue' ? 62 * 60 : 3600,
      standing_secs: 20 * 60,
      yesterday_sitting_secs: scenario === 'popup-overdue' ? 3 * 60 : 0,
      yesterday_standing_secs: 0,
      position_changes: 7,
      sessions: scenario === 'popup-overdue'
        ? [popupOverdueOvernightSitting, popupOverduePreviousStanding, popupOverduePreviousAway, popupOverduePreviousSitting]
        : [] };
    case 'get_settings':
      loadAttempts++;
      if (failNextLoad) { failNextLoad = false; throw new Error('settings store unavailable'); }
      if (scenario === 'invalid-both') return { ...settings, sitting_mm: 390, standing_mm: 1500 };
      if (scenario === 'invalid') return { ...settings, sitting_mm: 900, standing_mm: 900 };
      return settings;
    case 'get_relay_status':
      if (scenario === 'relay-status-error' && !relayStatusRejected) {
        return new Promise((_, reject) => {
          pendingRelayStatusRejects.push(() => reject(new Error('synthetic relay status IPC failure')));
        });
      }
      return { enabled: false, state: 'disabled', since: 0, viewers_online: 0, last_error: null };
    case 'relay_list_viewers': return [];
    case 'save_settings':
      saveAttempts++;
      if (scenario === 'save-fail' || scenario === 'activity-save-fail') {
        return new Promise((_, reject) => { rejectSave = () => reject(new Error('settings write failed')); });
      }
      if (args.config && typeof args.config === 'object') Object.assign(settings, args.config);
      return null;
    case 'trigger_test_notification':
      if (scenario === 'welcome-error') throw new Error('Rate limited: wait before testing again');
      {
        const preview: NonNullable<AppReminderSnapshot['active']> = {
          id: 'ux-welcome-fixture:1', source: 'test', reason: 'preview', category: 'info',
          title: 'Test reminder', body: 'This is a test reminder. Your ergonomic session is unchanged.',
          is_test: true, elapsed_secs: null, limit_secs: null,
        };
        if (scenario === 'welcome-queued') reminder.pending = preview;
        else reminder.active = preview;
        revision++;
        return copyReminder();
      }
    case 'dismiss_welcome':
      if (scenario === 'welcome-open-error' && welcomeOpenAttempts++ === 0) throw new Error('welcome_main_window_focus_failed');
      if (scenario === 'welcome-preference-error' && welcomeOpenAttempts++ === 0 && args.dontShowAgain === true) {
        throw new Error('welcome_preference_store_unavailable');
      }
      return null;
    case 'list_communication_profiles': return [
      { id: 'default', name: 'Default', description: 'Default', path: 'SYNTHETIC' },
      { id: 'custom-communication', name: 'Custom communication', description: 'Custom live policy', path: 'SYNTHETIC' },
    ];
    case 'list_ergonomic_profiles': return [
      { id: 'standard', name: 'Standard', description: 'Standard', path: 'SYNTHETIC' },
      { id: 'custom-live', name: 'Custom live', description: 'Custom live limits', path: 'SYNTHETIC' },
    ];
    case 'switch_ergonomic_profile':
      activeProfiles = {
        ...activeProfiles,
        ergonomic_id: args.id as string,
        session_limit_secs: args.id === 'standard' ? 2400 : 3300,
      };
      return null;
    case 'switch_communication_profile':
      activeProfiles = { ...activeProfiles, communication_id: args.id as string };
      return null;
    case 'get_active_profiles': return structuredClone(activeProfiles);
    case 'get_connected_port': return scenario === 'no-reading' ? null : 'SYNTHETIC';
    case 'open_analyst_window': return null;
    case 'hide_main_window':
      if (scenario === 'reminder-x-hide-pending' && reminderHideAttempts++ === 0) {
        return new Promise(resolve => { resolvePendingReminderHide = () => resolve(null); });
      }
      if (scenario === 'reminder-x-hide-fail' && failReminderHide) {
        failReminderHide = false;
        throw new Error('Synthetic main hide failure');
      }
      if (!welcomeEntry && scenario !== 'hide-main-fail') return null;
      if (scenario === 'hide-main-fail') throw new Error('Synthetic main hide failure');
      blocked.push(command); throw new Error('Only main window may be hidden through this command');
    case 'get_health_today': case 'refresh_health_now': return { configured: false, sources: [], error: null };
    case 'get_overlay_state': case 'start_auto_connect': case 'plugin:log|log': return null;
    case 'plugin:autostart|is_enabled': return false;
    default: blocked.push(command); throw new Error(`Unexpected synthetic IPC: ${command}`);
  }
}, { shouldMockEvents: true });

Object.assign(window, {
  uxSettingsTest: {
    calls, blocked, saveAttempts: () => saveAttempts,
    storedActivityPreference: () => settings.show_activity_status,
    failNextDashboardRefresh: () => { dashboardRefreshShouldFail = true; },
    dashboardRefreshPending: () => rejectDashboardRefresh !== null,
    rejectDashboardRefresh: () => {
      if (!rejectDashboardRefresh) throw new Error('no pending synthetic dashboard refresh');
      const reject = rejectDashboardRefresh;
      rejectDashboardRefresh = null;
      reject();
    },
    rejectNextSettingsLoad: () => { failNextLoad = true; },
    rejectPendingSave: () => { if (!rejectSave) throw new Error('no pending synthetic save'); rejectSave(); rejectSave = null; },
    rejectRelayStatus: () => {
      if (pendingRelayStatusRejects.length === 0) throw new Error('no pending synthetic relay status');
      relayStatusRejected = true;
      for (const reject of pendingRelayStatusRejects.splice(0)) reject();
    },
    publishReminder: async () => {
      const message: NonNullable<AppReminderSnapshot['active']> = {
        id: 'ux-settings-fixture:1', source: 'policy', reason: 'sitting_limit', category: 'warning',
        title: 'Time to change position', body: 'Your configured reminder is ready.', is_test: false,
        elapsed_secs: 600, limit_secs: 1200,
      };
      if (flow) reminder.pending = message; else reminder.active = message;
      revision++; await emit('desk:app-reminder', copyReminder());
    },
    finishPendingReminderHide: () => {
      if (!resolvePendingReminderHide) throw new Error('no pending synthetic Reminder X hide');
      const resolve = resolvePendingReminderHide;
      resolvePendingReminderHide = null;
      resolve();
    },
  },
});
Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: async () => {} } });
window.open = () => { throw new Error('External navigation forbidden in settings fixture'); };
window.WebSocket = class { constructor() { throw new Error('Display transport forbidden in settings fixture'); } } as typeof WebSocket;

declare global {
  interface Window {
    uxSettingsTest: {
      calls: { command: string; args: unknown }[];
      blocked: string[];
      saveAttempts: () => number;
      storedActivityPreference: () => boolean;
      failNextDashboardRefresh: () => void;
      dashboardRefreshPending: () => boolean;
      rejectDashboardRefresh: () => void;
      rejectNextSettingsLoad: () => void;
      rejectPendingSave: () => void;
      rejectRelayStatus: () => void;
      publishReminder: () => Promise<void>;
      finishPendingReminderHide: () => void;
    };
  }
}

if (welcomeEntry) await import('../../src/welcome');
else await import('../../src/main');
