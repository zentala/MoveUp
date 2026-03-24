//! notification_service.rs — Central notification routing module.
//!
//! Collects notification intents from SessionManager and other sources,
//! routes them through the configured backend (toast, popup, or both),
//! and manages suppression gates centrally.

use log::info;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::alert_popup::AlertPopup;
use crate::config::AppConfig;
use crate::event_logger::EventLogger;
use crate::session_types::NotificationEvent;

/// A notification intent with title, body, and the originating event type.
#[derive(Debug, Clone)]
pub struct NotificationIntent {
    pub event: NotificationEvent,
    pub title: String,
    pub body: String,
}

/// Routes notifications through the configured backend.
///
/// Backends:
/// - `"toast"` — native OS toast via `tauri-plugin-notification`
/// - `"popup"` — WinAPI always-on-top popup via [`AlertPopup`]
/// - `"both"` — fires both toast and popup
pub struct NotificationService;

impl NotificationService {
    /// Builds notification intents from session events and context.
    ///
    /// Translates raw `NotificationEvent` variants into human-readable
    /// title/body pairs. Context values (sitting/standing seconds) are
    /// used to enrich messages where appropriate.
    pub fn build_intents(
        events: &[NotificationEvent],
        sitting_secs: i64,
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
                    let ratio = if standing_secs > 0 {
                        sitting_secs as f32 / standing_secs as f32
                    } else {
                        f32::INFINITY
                    };
                    info!("Posture balance ratio: {:.1}", ratio);
                    (
                        "You've been sitting most of today".to_string(),
                        "Consider standing for a while.".to_string(),
                    )
                }
                NotificationEvent::Praise => (
                    "Halfway through your standing goal!".to_string(),
                    "Keep it up.".to_string(),
                ),
                NotificationEvent::StandLimitReached => {
                    // Currently a no-op event; handled by should_stand_alert().
                    continue;
                }
                NotificationEvent::StandingTargetReached => (
                    "Standing target reached!".to_string(),
                    "Great break! You stood for the full target duration.".to_string(),
                ),
            };
            intents.push(NotificationIntent {
                event: event.clone(),
                title,
                body,
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
        }
    }

    /// Creates a stand-limit alert intent.
    pub fn stand_limit_intent() -> NotificationIntent {
        NotificationIntent {
            event: NotificationEvent::StandLimitReached,
            title: "You've been standing a while".to_string(),
            body: "Ready to sit down for a bit?".to_string(),
        }
    }

    /// Creates a praise-halfway intent.
    pub fn praise_halfway_intent() -> NotificationIntent {
        NotificationIntent {
            event: NotificationEvent::Praise,
            title: "Halfway through your standing goal!".to_string(),
            body: "Keep it up.".to_string(),
        }
    }

    /// Routes a list of intents through the configured backend.
    ///
    /// Reads `config.notification_backend` to decide where to send:
    /// - `"toast"` — native OS notification only
    /// - `"popup"` — WinAPI popup only
    /// - `"both"` — both toast and popup
    pub fn dispatch(
        intents: &[NotificationIntent],
        app: &AppHandle,
        config: &AppConfig,
        event_logger: &EventLogger,
        alert_popup: &std::sync::Arc<std::sync::Mutex<AlertPopup>>,
    ) {
        let backend = config.notification_backend.as_str();
        let use_toast = backend == "toast" || backend == "both";
        let use_popup = backend == "popup" || backend == "both";

        for intent in intents {
            let log_tag = notification_log_tag(&intent.event);
            event_logger.log(&format!("NOTIF {}", log_tag));

            if use_toast {
                let _ = app
                    .notification()
                    .builder()
                    .title(&intent.title)
                    .body(&intent.body)
                    .show();
            }

            if use_popup {
                let msg = format!("{}\n\n{}", intent.title, intent.body);
                alert_popup.lock().unwrap().show(msg);
            }
        }
    }
}

/// Maps a notification event to a short log tag for the event log.
fn notification_log_tag(event: &NotificationEvent) -> &'static str {
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
}
