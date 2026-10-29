//! notification_service.rs — Central notification routing module.
//!
//! Executes notification decisions already resolved by CommunicationPolicy
//! through the configured toast and popup side-effect channels.

use log::info;

use crate::alert_popup::AlertPopup;
use crate::communication_types::{NotifySignal, ReminderDraft};
use crate::event_logger::EventLogger;
use crate::session_types::NotificationEvent;

/// A notification intent with title, body, and the originating event type.
#[derive(Debug, Clone)]
pub struct NotificationIntent {
    pub event: NotificationEvent,
    pub title: String,
    pub body: String,
    pub(crate) channels: Vec<NotificationChannel>,
    pub(crate) reminder: Option<ReminderDraft>,
}

/// Routes notifications through the configured backend.
///
/// Backends:
/// - `"toast"` — native OS toast via `tauri-plugin-notification`
/// - `"popup"` — WinAPI always-on-top popup via [`AlertPopup`]
/// - `"both"` — fires both toast and popup
pub struct NotificationService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotificationChannel {
    Main,
}

/// Resolves a configured backend to the channels the executor will call.
/// The caller that owns policy supplies the backend; this function only routes it.
pub(crate) fn channels_for_backend(backend: &str) -> Vec<NotificationChannel> {
    match backend {
        "toast" | "popup" | "both" => vec![NotificationChannel::Main],
        _ => Vec::new(),
    }
}

impl NotificationService {
    /// Builds notification intents from session events and context.
    ///
    /// Translates raw `NotificationEvent` variants into human-readable
    /// title/body pairs. Context values enrich messages where appropriate.
    ///
    /// Both counters must be RAW daily totals (`sitting_seconds_total`,
    /// `standing_seconds`). The PostureBalance message states one against the
    /// other, so feeding it the credited `sitting_seconds` would report a
    /// ratio the engine never computed (E015-T03).
    pub fn build_intents(
        events: &[NotificationEvent],
        sitting_secs_total: i64,
        standing_secs: i64,
    ) -> Vec<NotificationIntent> {
        let mut intents = Vec::new();
        for event in events {
            let (title, body) = match event {
                NotificationEvent::Inactivity => (
                    "No position change in 60 minutes".to_string(),
                    "Time to move.".to_string(),
                ),
                NotificationEvent::PostureBalance => {
                    let sit_h = sitting_secs_total / 3600;
                    let sit_m = (sitting_secs_total % 3600) / 60;
                    let stand_h = standing_secs / 3600;
                    let stand_m = (standing_secs % 3600) / 60;
                    info!(
                        "Posture balance: sitting {}h{}m vs standing {}h{}m",
                        sit_h, sit_m, stand_h, stand_m
                    );
                    (
                        format!(
                            "Sitting {}h {}m vs standing {}h {}m",
                            sit_h, sit_m, stand_h, stand_m
                        ),
                        "Consider standing for a while.".to_string(),
                    )
                }
                NotificationEvent::Praise => (
                    "Halfway through your standing goal!".to_string(),
                    "Keep it up.".to_string(),
                ),
                NotificationEvent::StandLimitReached => continue,
                NotificationEvent::StandingTargetReached => (
                    "Standing target reached!".to_string(),
                    "Great break! You stood for the full target duration.".to_string(),
                ),
            };
            intents.push(NotificationIntent {
                event: event.clone(),
                title,
                body,
                channels: Vec::new(),
                reminder: None,
            });
        }
        intents
    }

    /// Creates a sit-limit alert intent.
    pub fn sit_limit_intent() -> NotificationIntent {
        NotificationIntent {
            event: NotificationEvent::Inactivity, // reusing closest variant
            title: "Time to stand up!".to_string(),
            body: "You've been sitting for 40 minutes. Take a break.".to_string(),
            channels: Vec::new(),
            reminder: None,
        }
    }

    /// Creates a stand-limit alert intent.
    pub fn stand_limit_intent() -> NotificationIntent {
        NotificationIntent {
            event: NotificationEvent::StandLimitReached,
            title: "You've been standing a while".to_string(),
            body: "Ready to sit down for a bit?".to_string(),
            channels: Vec::new(),
            reminder: None,
        }
    }

    /// Creates a praise-halfway intent.
    pub fn praise_halfway_intent() -> NotificationIntent {
        NotificationIntent {
            event: NotificationEvent::Praise,
            title: "Halfway through your standing goal!".to_string(),
            body: "Keep it up.".to_string(),
            channels: Vec::new(),
            reminder: None,
        }
    }

    /// Routes a list of intents through the configured backend.
    ///
    /// `notification_backend` decides where to send:
    /// - `"toast"` — native OS notification only
    /// - `"popup"` — WinAPI popup only
    /// - `"both"` — both toast and popup
    pub fn dispatch(
        app: &tauri::AppHandle,
        intents: &[NotificationIntent],
        event_logger: &EventLogger,
        _alert_popup: &std::sync::Arc<std::sync::Mutex<AlertPopup>>,
    ) {
        Self::dispatch_reminders_with(
            intents,
            |line| event_logger.log(line),
            |channels, _, _, draft| {
                if channels.contains(&NotificationChannel::Main) {
                    if let Some(draft) = draft { crate::commands_reminders::publish(app, draft.clone(), crate::app_reminders::ReminderSource::Session); }
                }
            },
        );
    }

    /// Runs the production intent loop with injectable terminal side effects.
    /// Policy still supplies the resolved intents and channels.
    pub(crate) fn dispatch_with(
        intents: &[NotificationIntent],
        log: impl FnMut(&str),
        mut execute: impl FnMut(&[NotificationChannel], &str, &str),
    ) {
        Self::dispatch_reminders_with(intents, log, |channels, title, body, _| execute(channels, title, body));
    }

    pub(crate) fn dispatch_reminders_with(
        intents: &[NotificationIntent],
        mut log: impl FnMut(&str),
        mut execute: impl FnMut(&[NotificationChannel], &str, &str, Option<&ReminderDraft>),
    ) {
        for intent in intents {
            let log_tag = notification_log_tag(&intent.event);
            log(&format!("NOTIF {}", log_tag));
            execute(
                &intent.channels,
                &intent.title,
                &intent.body,
                intent.reminder.as_ref(),
            );
        }
    }

    /// Adds policy-selected channels to intents for the executor.
    pub(crate) fn resolve_channels(intents: &mut [NotificationIntent], backend: &str) {
        let channels = channels_for_backend(backend);
        for intent in intents {
            intent.channels = channels.clone();
        }
    }

    /// Executes a channel already selected by `CommunicationPolicy`.
    /// This is the common side-effect path for per-tick policy signals.
    pub(crate) fn dispatch_policy_signal(
        app: &tauri::AppHandle,
        signal: &NotifySignal,
        draft: Option<&ReminderDraft>,
        event_logger: Option<&EventLogger>,
        _alert_popup: &std::sync::Arc<std::sync::Mutex<AlertPopup>>,
    ) {
        Self::dispatch_policy_signal_with(
            signal,
            draft,
            || {
                if let Some(event_logger) = event_logger {
                    event_logger.log("NOTIF policy");
                }
            },
            |NotificationChannel::Main, _, _, draft| {
                let draft = draft.expect("final effect selection requires typed policy metadata");
                crate::commands_reminders::publish(app, draft.clone(), crate::app_reminders::ReminderSource::Policy);
            },
        );
    }

    /// Runs the policy-selected log and channel effects. Production supplies
    /// the event logger and platform executor; tests count the same effects.
    pub(crate) fn dispatch_policy_signal_with(
        signal: &NotifySignal,
        draft: Option<&ReminderDraft>,
        mut log: impl FnMut(),
        mut execute_channel: impl FnMut(NotificationChannel, &str, &str, Option<&ReminderDraft>),
    ) {
        let Some(draft) = draft else {
            log::error!("Policy notice has no typed reminder metadata; no presentation effect executed");
            return;
        };
        let body = match signal {
            NotifySignal::Toast(body) | NotifySignal::Popup(body) => body.as_str(),
        };
        log();
        execute_channel(NotificationChannel::Main, "MoveUp", body, Some(draft));
    }

    #[cfg(test)]
    fn execute_channels_with(
        channels: &[NotificationChannel],
        title: &str,
        body: &str,
        mut execute: impl FnMut(&NotificationChannel, &str, &str),
    ) {
        for channel in channels {
            execute(channel, title, body);
        }
    }
}

/// Maps a notification event to a short log tag for the event log.
pub(crate) fn notification_log_tag(event: &NotificationEvent) -> &'static str {
    match event {
        NotificationEvent::Inactivity => "inactivity",
        NotificationEvent::PostureBalance => "posture_balance",
        NotificationEvent::Praise => "praise",
        NotificationEvent::StandLimitReached => "stand_limit",
        NotificationEvent::StandingTargetReached => "standing_target_reached",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_intents_from_empty_events() {
        let intents = NotificationService::build_intents(&[], 0, 0);
        assert!(intents.is_empty());
    }

    #[test]
    fn build_intents_inactivity() {
        let events = vec![NotificationEvent::Inactivity];
        let intents = NotificationService::build_intents(&events, 3600, 0);
        assert_eq!(intents.len(), 1);
        assert!(intents[0].title.contains("60 minutes"));
    }

    #[test]
    fn build_intents_skips_stand_limit_reached() {
        let events = vec![NotificationEvent::StandLimitReached];
        let intents = NotificationService::build_intents(&events, 0, 0);
        assert!(intents.is_empty());
    }

    #[test]
    fn build_intents_multiple_events() {
        let events = vec![
            NotificationEvent::Inactivity,
            NotificationEvent::PostureBalance,
            NotificationEvent::StandingTargetReached,
        ];
        let intents = NotificationService::build_intents(&events, 7200, 1800);
        assert_eq!(intents.len(), 3);
    }

    #[test]
    fn sit_limit_intent_has_correct_title() {
        let intent = NotificationService::sit_limit_intent();
        assert!(intent.title.contains("stand up"));
    }

    #[test]
    fn stand_limit_intent_has_correct_title() {
        let intent = NotificationService::stand_limit_intent();
        assert!(intent.title.contains("standing"));
    }

    #[test]
    fn praise_halfway_intent_has_correct_title() {
        let intent = NotificationService::praise_halfway_intent();
        assert!(intent.title.contains("Halfway"));
    }

    #[test]
    fn log_tag_mapping() {
        assert_eq!(notification_log_tag(&NotificationEvent::Inactivity), "inactivity");
        assert_eq!(notification_log_tag(&NotificationEvent::PostureBalance), "posture_balance");
        assert_eq!(notification_log_tag(&NotificationEvent::Praise), "praise");
        assert_eq!(notification_log_tag(&NotificationEvent::StandLimitReached), "stand_limit");
        assert_eq!(
            notification_log_tag(&NotificationEvent::StandingTargetReached),
            "standing_target_reached"
        );
    }

    #[test]
    fn dispatch_seam_executes_only_policy_selected_channels() {
        let selected = vec![NotificationChannel::Main];
        let mut calls = Vec::new();
        NotificationService::execute_channels_with(&selected, "title", "body", |channel, title, body| {
            calls.push((*channel, title.to_string(), body.to_string()));
        });
        assert_eq!(calls, vec![(NotificationChannel::Main, "title".into(), "body".into())]);
    }
}
