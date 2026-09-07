//! ws_broadcaster.rs — Broadcast channel for remote display clients.
//!
//! Creates a tokio broadcast channel that fans out desk events to all
//! connected WebSocket clients. Event producers send to the channel,
//! WS clients receive from it.

use serde::Serialize;
use tokio::sync::broadcast;

use crate::db::TodaySummary;
use crate::health_models::HealthView;
use crate::metrics::MetricSnapshot;
use crate::session::SessionStateDto;

/// Full state sent to remote display clients on every tick (~1/s).
/// Includes session state + KPI metrics + today's session list + health.
#[derive(Clone, Debug, Serialize)]
pub struct RemoteDisplayState {
    pub session: SessionStateDto,
    pub metrics: Vec<MetricSnapshot>,
    pub today: TodaySummary,
    /// Merged health view (E021-T03). Carried in the snapshot rather than
    /// fetched separately so the phone — which has no Tauri IPC — gets steps
    /// and heart rate on the same stream as everything else.
    pub health: HealthView,
}

/// Message types sent to remote display clients.
#[derive(Clone, Debug, Serialize)]
/// The `rename` values below are the same wire names as the constants in
/// [`crate::desk_events`], but `serde` attributes need literals, so they cannot
/// reference them. `desk_events`'s tests assert the two stay equal.
#[serde(tag = "event", content = "payload")]
#[allow(dead_code)] // Heartbeat variant used for serialization schema completeness
pub enum DisplayEvent {
    /// Full state snapshot (sent on connect + every ~1s).
    #[serde(rename = "snapshot")]
    Snapshot(RemoteDisplayState),

    /// State transition (sitting -> standing, etc.).
    #[serde(rename = "desk:state-changed")]
    StateChanged(serde_json::Value),

    /// Device connected to serial port.
    #[serde(rename = "desk:device-connected")]
    DeviceConnected { port: String },

    /// Device lost (serial disconnected).
    #[serde(rename = "desk:device-lost")]
    DeviceLost,

    /// Daily counter reset (midnight).
    #[serde(rename = "desk:daily-reset")]
    DailyReset,

    /// Keep-alive ping.
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// Creates the broadcast channel with capacity for 64 messages.
/// Returns the sender — clone it for each producer, subscribe for each consumer.
pub fn create_channel() -> broadcast::Sender<String> {
    let (tx, _) = broadcast::channel::<String>(64);
    tx
}

/// Serializes a [`DisplayEvent`] and sends it to the broadcast channel.
/// Silently drops if no receivers are listening (fire-and-forget).
pub fn broadcast_event(tx: &broadcast::Sender<String>, event: &DisplayEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        let _ = tx.send(json); // Ignore SendError (no receivers = ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_event_serializes_heartbeat() {
        let tx = create_channel();
        let mut rx = tx.subscribe();
        broadcast_event(&tx, &DisplayEvent::Heartbeat);
        let msg = rx.try_recv().expect("should receive message");
        assert!(msg.contains("heartbeat"));
    }

    #[test]
    fn multiple_receivers_get_same_message() {
        let tx = create_channel();
        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();
        broadcast_event(&tx, &DisplayEvent::DailyReset);
        let m1 = rx1.try_recv().expect("rx1 should receive");
        let m2 = rx2.try_recv().expect("rx2 should receive");
        assert_eq!(m1, m2);
    }

    #[test]
    fn no_panic_when_no_receivers() {
        let tx = create_channel();
        // Should not panic — fire-and-forget
        broadcast_event(&tx, &DisplayEvent::DeviceLost);
    }

    #[test]
    fn lagged_receiver_does_not_block_sender() {
        let tx = create_channel();
        let _rx = tx.subscribe(); // slow receiver never reads
        // Send more than channel capacity (64)
        for i in 0..70 {
            broadcast_event(
                &tx,
                &DisplayEvent::DeviceConnected {
                    port: format!("COM{}", i),
                },
            );
        }
        // Sender should not block or panic
    }

    #[test]
    fn snapshot_serialization_includes_all_fields() {
        let state = RemoteDisplayState {
            session: SessionStateDto {
                state: crate::session::DeskState::Sitting,
                sitting_seconds: 100,
                standing_seconds: 50,
                break_seconds: 0,
                session_limit_secs: 2700,
                stand_limit_secs: 600,
                desk_height_cm: 72.0,
                position_changes: 3,
                limit_used_secs: 100,
                daily_score: 10.0,
                standing_session_secs: 0,
                secs_since_last_break: 100,
                continuous_computer_secs: 150,
                longest_computer_session_secs: 150,
                sitting_seconds_total: 100,
                idle_secs: 0,
                away_bout_secs: 0,
                max_continuous_computer_secs: 3600,
            },
            metrics: vec![],
            today: TodaySummary {
                sitting_secs: 100,
                standing_secs: 50,
                yesterday_sitting_secs: 0,
                yesterday_standing_secs: 0,
                position_changes: 3,
                sessions: vec![],
            },
            health: HealthView::unconfigured(),
        };
        let event = DisplayEvent::Snapshot(state);
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"event\":\"snapshot\""));
        assert!(json.contains("\"session\""));
        assert!(json.contains("\"metrics\""));
        assert!(json.contains("\"today\""));
        assert!(json.contains("\"health\""));
    }

    #[test]
    fn state_changed_serialization() {
        let val = serde_json::json!({"state": "Standing", "sitting_seconds": 120});
        let event = DisplayEvent::StateChanged(val);
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("desk:state-changed"));
    }
}
