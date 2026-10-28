//! Wires `desk:state-changed` + `desk:distance` events to tray tooltip,
//! overlay progress bar, and [`CommunicationPolicy`] signal engine.

use chrono::Utc;
use tauri::{AppHandle, Listener, Manager};

use crate::{
    colors::color_for_progress,
    commands::AppState,
    communication_policy::PolicyInput,
    communication_types::NotifySignal,
    notify_webhook::{Notification, WebhookNotifier},
    desk_events::{
        DESK_DEVICE_CONNECTED, DESK_DEVICE_LOST, DESK_DEVICE_MISSING, DESK_DISTANCE,
        DESK_STATE_CHANGED,
    },
    remote_display_state,
    session::{DeskState, StateChangedPayload},
    tray,
    tray_signal_exec,
    ws_broadcaster::{self, DisplayEvent},
};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Registers `desk:state-changed` and `desk:distance` event listeners.
/// Call once from `lib.rs` setup.
pub fn setup(app: &AppHandle) {
    let handle = app.clone();
    app.listen(DESK_STATE_CHANGED, move |event| {
        if let Ok(payload) = serde_json::from_str::<StateChangedPayload>(event.payload()) {
            on_state_changed(&handle, &payload);
        }
    });

    let handle2 = app.clone();
    app.listen(DESK_DISTANCE, move |_event| {
        update_from_policy(&handle2);
    });

    let handle3 = app.clone();
    app.listen(DESK_DEVICE_LOST, move |_event| {
        let _ = tray::update_tray(&handle3, "Desk \u{2014} sensor disconnected", DeskState::Away, 0.0);
    });

    let handle4 = app.clone();
    app.listen(DESK_DEVICE_MISSING, move |_event| {
        let _ = tray::update_tray(&handle4, "Desk \u{2014} no sensor found", DeskState::Away, 0.0);
    });

    let handle5 = app.clone();
    app.listen(DESK_DEVICE_CONNECTED, move |_event| {
        let app_state = handle5.state::<AppState>();
        app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).on_sensor_connected();
    });
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Reacts to a state-change event: notify policy of position change, then evaluate.
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    let app_state = app.state::<AppState>();

    app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()).on_position_changed();

    let snapshot = app_state.session.lock().unwrap_or_else(|e| e.into_inner()).snapshot();
    let overlay = app_state.overlay.clone();

    let progress = if snapshot.session_limit_secs > 0 {
        payload.limit_used_secs as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    let label = build_tooltip_label(
        payload.desk_height_cm,
        &payload.state,
        payload.limit_used_secs,
        payload.standing_seconds,
        payload.break_seconds,
        snapshot.daily_score,
    );
    let _ = tray::update_tray(app, &label, payload.state.clone(), progress);

    if payload.state == DeskState::Sitting {
        let (r, g, b, _) = color_for_progress(progress);
        overlay.clear_standing();
        overlay.update(progress, (r, g, b));
        overlay.show();
    } else {
        overlay.clear_standing();
        overlay.hide();
    }

    ws_broadcaster::broadcast_event(
        &app_state.ws_tx,
        &DisplayEvent::StateChanged(
            serde_json::to_value(payload).unwrap_or_default(),
        ),
    );

    refresh_today_cache(app);
}

/// Evaluates CommunicationPolicy each tick (~1/s) and executes returned signals.
fn update_from_policy(app: &AppHandle) {
    let app_state = app.state::<AppState>();

    let dismissed = app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).take_user_dismissed();
    apply_user_dismissal_at(
        &mut app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner()),
        dismissed,
        std::time::Instant::now(),
    );

    let is_connected = app_state.conn.connected_port.lock().unwrap_or_else(|e| e.into_inner()).is_some();

    let session = app_state.session.lock().unwrap_or_else(|e| e.into_inner());
    // Dismiss alert popup when user stands (popup is no longer relevant).
    if session.state.state == DeskState::Standing {
        app_state.alert_popup.lock().unwrap_or_else(|e| e.into_inner()).dismiss();
    }
    let snapshot = session.snapshot();
    let stand_limit_fired = session.stand_alert_fired;
    // Every policy-facing derived value comes from the engine (E020-T05); this
    // adapter contributes only the sensor-connectivity fact the engine lacks.
    let input: PolicyInput = session.policy_input(Utc::now(), is_connected);
    drop(session);

    // Cached read, not a merge: this runs every ~1 s on the tray thread and
    // must not await source I/O. `try_state` because a test harness may build
    // an app without the health aggregator managed — and "no aggregator" then
    // renders as the setup hint, not as zeroed metrics.
    let health = app
        .try_state::<crate::health_source::HealthState>()
        .map(|state| state.last_view())
        .unwrap_or_else(crate::health_models::HealthView::unconfigured);

    remote_display_state::broadcast(
        &app_state.ws_tx,
        &app_state.session,
        &app_state.comm_policy,
        &app_state.today_cache,
        health,
    );
    update_tooltip(app, &snapshot);

    let facts = app_state.session.lock().unwrap_or_else(|error| error.into_inner()).notification_facts(Utc::now());
    let (signals, reminder, reminders_changed) = {
        let mut policy = app_state.comm_policy.lock().unwrap_or_else(|e| e.into_inner());
        let signals = policy.evaluate_with_stand_limit_fired(
            &input,
            snapshot.session_limit_secs,
            snapshot.standing_session_secs,
            stand_limit_fired,
        );
        // Only short pure state reconciliation under policy -> presentation state.
        // No caller ever acquires policy while retaining the presentation mutex.
        let mut reminders_changed = false;
        if let Some(reminders) = app.try_state::<crate::app_reminders::AppReminderState>() {
            let mut guard = reminders.0.lock().unwrap_or_else(|error| error.into_inner());
            let before = guard.snapshot().revision;
            guard.reconcile(|entry|
                policy.reminder_obsolete_reason(&entry.draft, entry.day, &input, &facts,
                    snapshot.session_limit_secs, snapshot.standing_session_secs, std::time::Instant::now()));
            reminders_changed = before != guard.snapshot().revision;
        }
        let reminder = policy.take_reminder().map(|mut draft| {
            draft.observation_day = Some(facts.now.with_timezone(&chrono::Local).date_naive()); draft
        });
        (signals, reminder, reminders_changed)
    };
    // Event/native work only after policy and presentation mutexes are released.
    if reminders_changed {
        crate::commands_reminders::announce(app);
    }

    if !signals.accepted_events.is_empty() {
        let standing = snapshot.standing_session_secs;
        let mut session = app_state.session.lock().unwrap_or_else(|e| e.into_inner());
        session.record_notification_events(&signals.accepted_events);
        if signals.accepted_events.iter().any(|event| matches!(event, crate::session_types::NotificationEvent::StandLimitReached)) {
            if let Some(loggers) = app.try_state::<crate::Loggers>() {
                loggers.event.log(&format!("ALERT stand_limit standing={}s", standing));
            }
        }
    }

    tray_signal_exec::execute_tray(&signals.tray, app, &snapshot);

    let overlay = app_state.overlay.clone();
    tray_signal_exec::execute_overlay(&signals.overlay, &overlay, &snapshot);
    tray_signal_exec::execute_popup(&signals.popup, app);

    let event_logger = app.try_state::<crate::Loggers>().map(|loggers| loggers.event.clone());
    dispatch_notification_with(
        signals.notify.as_ref(),
        |notify| tray_signal_exec::execute_notify(app, notify, reminder.as_ref(), &app_state, event_logger.as_deref()),
        |notify| push_to_webhook(&app_state, notify),
    );
    if notification_state_needs_save(
        dismissed,
        signals.notify.is_some(),
        !signals.accepted_events.is_empty(),
    ) {
        let session = app_state.session.lock().unwrap_or_else(|e| e.into_inner());
        crate::session_persistence::save_via_app(app, &session);
    }
}

/// Applies the popup adapter's consumed dismissal signal to the policy.
/// Kept as the production seam exercised by the disposable Store regression.
pub(crate) fn apply_user_dismissal_at(
    policy: &mut crate::communication_policy::CommunicationPolicy,
    dismissed: bool,
    now: std::time::Instant,
) -> bool {
    if dismissed { policy.dismiss_at(now); }
    dismissed
}

/// Whether this tray tick must checkpoint policy state, including a dismissal
/// when no subsequent notification or accepted session event was produced.
pub(crate) fn notification_state_needs_save(
    dismissed: bool,
    has_notification: bool,
    has_accepted_events: bool,
) -> bool {
    dismissed || has_notification || has_accepted_events
}

/// Mirrors a toast to the user's webhook so a phone or watch sees it too.
///
/// Only [`NotifySignal::Toast`] is mirrored: `Popup` is the on-screen
/// escalation of a toast that already went out, and pushing both would send
/// the same alert twice. Sending is fire-and-forget — see
/// [`crate::notify_webhook`] — so a dead receiver cannot stall this tick.
fn push_to_webhook(app_state: &AppState, notify: &NotifySignal) {
    let NotifySignal::Toast(message) = notify else {
        return;
    };
    let config = app_state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default();
    if let Some(notifier) = WebhookNotifier::from_env_or_config(
        config.notify_webhook_enabled,
        config.notify_webhook_url.as_deref(),
    ) {
        notifier.send(Notification::alert("MoveUp", message));
    }
}

/// Runs the production notification path once for a policy result. A missing
/// result has no channel, log, or webhook effects; only toasts are mirrored.
pub(crate) fn dispatch_notification_with(
    notify: Option<&NotifySignal>,
    mut dispatch: impl FnMut(&NotifySignal),
    mut webhook: impl FnMut(&NotifySignal),
) {
    let Some(notify) = notify else { return };
    dispatch(notify);
    if matches!(notify, NotifySignal::Toast(_)) {
        webhook(notify);
    }
}

/// Updates tray tooltip from a session snapshot (called every ~1s).
fn update_tooltip(app: &AppHandle, snapshot: &crate::session::SessionStateDto) {
    let label = build_tooltip_label(
        snapshot.desk_height_cm,
        &snapshot.state,
        snapshot.sitting_seconds,
        snapshot.standing_seconds,
        snapshot.break_seconds,
        snapshot.daily_score,
    );
    let _ = tray::update_tray_tooltip(app, &label);
}

use crate::tray_helpers::{build_tooltip_label, refresh_today_cache};

// Tests moved to tray_controller_tests.rs
