//! Canonical names of the Tauri events the backend emits and listens to.
//!
//! Every `app.emit(...)` / `app.listen(...)` call site uses a constant from
//! this module instead of a string literal, so a renamed event is a compile
//! error rather than a listener that silently never fires. The mirrored
//! frontend copy lives in `src/events.ts`; `src/events.test.ts` asserts the
//! two files agree.
//!
//! [`crate::ws_broadcaster::DisplayEvent`] is a separate, already-typed
//! representation of the WebSocket channel. Its `#[serde(rename = "...")]`
//! attributes need literals, so they cannot reference these constants — a
//! test below pins them to the same values instead.

/// Raw distance reading from the sensor, emitted on every serial line.
pub const DESK_DISTANCE: &str = "desk:distance";

/// Desk state transition (sitting → standing, etc.).
pub const DESK_STATE_CHANGED: &str = "desk:state-changed";

/// Sensor found and the reader loop started.
pub const DESK_DEVICE_CONNECTED: &str = "desk:device-connected";

/// Sensor was connected and then disappeared.
pub const DESK_DEVICE_LOST: &str = "desk:device-lost";

/// A scan finished without finding a sensor.
pub const DESK_DEVICE_MISSING: &str = "desk:device-missing";

/// The firmware reported an error line.
pub const DESK_SENSOR_ERROR: &str = "desk:sensor-error";

/// Daily counters were reset at midnight.
pub const DESK_DAILY_RESET: &str = "desk:daily-reset";

/// Tray asked the frontend to show the widget view.
pub const DESK_SHOW_WIDGET: &str = "desk:show-widget";

/// Tray asked the frontend to show the settings view.
pub const DESK_SHOW_SETTINGS: &str = "desk:show-settings";

/// Popup colour scheme changed (neutral / yellow / red / gray).
pub const DESK_POPUP_THEME: &str = "desk:popup-theme";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws_broadcaster::DisplayEvent;

    #[test]
    fn e019_t03_desk_events_values_are_stable() {
        assert_eq!(DESK_DISTANCE, "desk:distance");
        assert_eq!(DESK_STATE_CHANGED, "desk:state-changed");
        assert_eq!(DESK_DEVICE_CONNECTED, "desk:device-connected");
        assert_eq!(DESK_DEVICE_LOST, "desk:device-lost");
        assert_eq!(DESK_DEVICE_MISSING, "desk:device-missing");
        assert_eq!(DESK_SENSOR_ERROR, "desk:sensor-error");
        assert_eq!(DESK_DAILY_RESET, "desk:daily-reset");
        assert_eq!(DESK_SHOW_WIDGET, "desk:show-widget");
        assert_eq!(DESK_SHOW_SETTINGS, "desk:show-settings");
        assert_eq!(DESK_POPUP_THEME, "desk:popup-theme");
    }

    #[test]
    fn e019_t03_desk_events_all_names_are_namespaced_and_unique() {
        let all = [
            DESK_DISTANCE,
            DESK_STATE_CHANGED,
            DESK_DEVICE_CONNECTED,
            DESK_DEVICE_LOST,
            DESK_DEVICE_MISSING,
            DESK_SENSOR_ERROR,
            DESK_DAILY_RESET,
            DESK_SHOW_WIDGET,
            DESK_SHOW_SETTINGS,
            DESK_POPUP_THEME,
        ];
        for name in all {
            assert!(name.starts_with("desk:"), "{name} is not namespaced");
        }
        let mut sorted = all.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "duplicate event name in desk_events");
    }

    /// `DisplayEvent`'s serde renames must stay equal to these constants —
    /// they are the same wire names read by remote clients, but the attribute
    /// cannot reference a const, so the two are pinned here instead.
    #[test]
    fn e019_t03_desk_events_match_display_event_wire_names() {
        let cases: Vec<(DisplayEvent, &str)> = vec![
            (
                DisplayEvent::StateChanged(serde_json::json!({})),
                DESK_STATE_CHANGED,
            ),
            (
                DisplayEvent::DeviceConnected { port: "COM3".into() },
                DESK_DEVICE_CONNECTED,
            ),
            (DisplayEvent::DeviceLost, DESK_DEVICE_LOST),
            (DisplayEvent::DailyReset, DESK_DAILY_RESET),
        ];
        for (event, expected) in cases {
            let json = serde_json::to_value(&event).expect("should serialize");
            assert_eq!(
                json.get("event").and_then(|v| v.as_str()),
                Some(expected),
                "DisplayEvent wire name drifted from desk_events"
            );
        }
    }
}
