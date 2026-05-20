//! Signal-parsing and signal-building helpers for [`CommunicationPolicy`].
//!
//! Pure string → enum conversions and stateless signal builders extracted from
//! the main policy module to keep file sizes under 250 lines.

use crate::communication_profile::{BaselineConfig, ChannelConfig, CommunicationProfile, MessageConfig};
use crate::communication_types::{NotifySignal, OverlaySignal, PopupSignal, Signals, TraySignal};
use crate::session_types::DeskState;

// ── Signal parsers ────────────────────────────────────────────────────────────

/// Parse a tray signal name into a [`TraySignal`].
///
/// `blink_*` names are valid blink-pattern references resolved later by
/// `execute_tray_blink`; they are not "unrecognized". Anything else falls
/// through to the warn branch — but only logged at debug level to avoid
/// drowning the event log at the policy's tick rate (the warning used to
/// fire >10Hz on `blink_red`, which triggered a downstream Arc/Rc UB
/// during heavy log churn).
pub(crate) fn parse_tray_signal(s: &str) -> TraySignal {
    match s {
        "none" | "" => TraySignal::None,
        "yellow" => TraySignal::Yellow,
        "red" => TraySignal::Red,
        other if other.starts_with("blink_") => TraySignal::Blink(other.to_string()),
        other => {
            log::debug!("Unrecognized tray signal '{}' — treating as blink pattern", other);
            TraySignal::Blink(other.to_string())
        }
    }
}

/// Parse an overlay signal name into an [`OverlaySignal`] with the given progress.
pub(crate) fn parse_overlay_signal(s: &str, progress: f32) -> OverlaySignal {
    match s {
        "hidden" | "none" | "" => OverlaySignal::Hidden,
        "neutral" => OverlaySignal::Neutral { progress },
        "yellow" => OverlaySignal::Yellow { progress },
        "red" => OverlaySignal::Red { progress },
        "pulse_red" => OverlaySignal::PulseRed { progress },
        other => {
            log::warn!("Unrecognized overlay signal '{}' in profile — defaulting to Neutral", other);
            OverlaySignal::Neutral { progress }
        }
    }
}

/// Parse a popup signal name into a [`PopupSignal`].
pub(crate) fn parse_popup_signal(s: &str) -> PopupSignal {
    match s {
        "neutral" | "none" | "" => PopupSignal::Neutral,
        "yellow" => PopupSignal::Yellow,
        "red" => PopupSignal::Red,
        "gray" => PopupSignal::Gray,
        other => {
            log::warn!("Unrecognized popup signal '{}' in profile — defaulting to Neutral", other);
            PopupSignal::Neutral
        }
    }
}

/// Build a [`NotifySignal`] from a backend name and message text.
pub(crate) fn make_notify_signal(notify_type: &str, message: &str) -> NotifySignal {
    match notify_type {
        "popup" => NotifySignal::Popup(message.to_string()),
        _ => NotifySignal::Toast(message.to_string()),
    }
}

// ── Stateless signal builders ─────────────────────────────────────────────────

/// Build the inactive-state signals (Away or Walking) from profile config.
pub(crate) fn inactive_signals(inactive: &ChannelConfig) -> Signals {
    Signals {
        tray: parse_tray_signal(&inactive.tray),
        overlay: parse_overlay_signal(&inactive.overlay, 0.0),
        popup: parse_popup_signal(&inactive.popup_header),
        notify: None,
    }
}

/// Build baseline signals for a sitting or standing state.
pub(crate) fn baseline_signals(
    state: &DeskState,
    baseline: &BaselineConfig,
    lap_progress: f32,
    lap: u32,
    lap_flash: bool,
) -> Signals {
    match state {
        DeskState::Sitting => {
            let bl = &baseline.sitting;
            Signals {
                tray: parse_tray_signal(&bl.tray),
                overlay: parse_overlay_signal(&bl.overlay, 0.0),
                popup: parse_popup_signal(&bl.popup),
                notify: None,
            }
        }
        DeskState::Standing => {
            let bl = &baseline.standing;
            // "progress" needs lap/flash params that parse_overlay_signal can't carry
            let overlay = match bl.overlay.as_str() {
                "progress" => OverlaySignal::Progress { progress: lap_progress, lap, flash: lap_flash },
                other => parse_overlay_signal(other, lap_progress),
            };
            Signals {
                tray: parse_tray_signal(&bl.tray),
                overlay,
                popup: parse_popup_signal(&bl.popup),
                notify: None,
            }
        }
        _ => Signals::default(),
    }
}

/// Select the notification message for the current state and notify type.
pub(crate) fn message_for_state(
    state: &DeskState,
    notify_type: &str,
    msgs: &MessageConfig,
    is_firm: bool,
) -> String {
    match (state, notify_type) {
        (DeskState::Sitting, "toast") => msgs.sitting_limit_toast.clone(),
        (DeskState::Sitting, "popup") => {
            if is_firm { msgs.sitting_firm_popup.clone() } else { msgs.sitting_overdue_popup.clone() }
        }
        (DeskState::Standing, "toast") => msgs.standing_limit_toast.clone(),
        (DeskState::Standing, "popup") => msgs.standing_overdue_popup.clone(),
        _ => msgs.sitting_limit_toast.clone(),
    }
}

/// Build the disconnected signals, firing a one-shot notification if requested.
///
/// Returns `(Signals, bool)` where the bool is `true` when a notification was produced
/// (caller must set `disconnect_notified = true`).
pub(crate) fn disconnected_signals(
    profile: &CommunicationProfile,
    already_notified: bool,
) -> (Signals, bool) {
    let (notify, fired) = if !already_notified {
        let notify_type = profile.disconnected.notify_once.clone().unwrap_or_default();
        let msg = profile.messages.sensor_disconnected.clone();
        (Some(make_notify_signal(&notify_type, &msg)), true)
    } else {
        (None, false)
    };
    let signals = Signals {
        tray: parse_tray_signal(&profile.disconnected.tray),
        overlay: OverlaySignal::Hidden,
        popup: PopupSignal::Gray,
        notify,
    };
    (signals, fired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::communication_types::{OverlaySignal, PopupSignal, TraySignal};

    #[test]
    fn parse_tray_signal_known_variants() {
        assert!(matches!(parse_tray_signal("none"), TraySignal::None));
        assert!(matches!(parse_tray_signal(""), TraySignal::None));
        assert!(matches!(parse_tray_signal("yellow"), TraySignal::Yellow));
        assert!(matches!(parse_tray_signal("red"), TraySignal::Red));
    }

    #[test]
    fn parse_tray_signal_unknown_is_blink() {
        assert!(matches!(parse_tray_signal("blink_red"), TraySignal::Blink(_)));
        assert!(matches!(parse_tray_signal("blink_gray"), TraySignal::Blink(_)));
    }

    #[test]
    fn parse_overlay_signal_known_variants() {
        assert!(matches!(parse_overlay_signal("hidden", 0.0), OverlaySignal::Hidden));
        assert!(matches!(parse_overlay_signal("none", 0.0), OverlaySignal::Hidden));
        assert!(matches!(parse_overlay_signal("", 0.0), OverlaySignal::Hidden));
        assert!(matches!(parse_overlay_signal("neutral", 0.5), OverlaySignal::Neutral { .. }));
        assert!(matches!(parse_overlay_signal("yellow", 0.5), OverlaySignal::Yellow { .. }));
        assert!(matches!(parse_overlay_signal("red", 0.5), OverlaySignal::Red { .. }));
        assert!(matches!(parse_overlay_signal("pulse_red", 0.5), OverlaySignal::PulseRed { .. }));
    }

    #[test]
    fn parse_overlay_signal_unknown_defaults_to_neutral() {
        assert!(matches!(parse_overlay_signal("bogus", 0.5), OverlaySignal::Neutral { .. }));
    }

    #[test]
    fn parse_popup_signal_neutral_variants() {
        assert!(matches!(parse_popup_signal("neutral"), PopupSignal::Neutral));
        assert!(matches!(parse_popup_signal("none"), PopupSignal::Neutral));
        assert!(matches!(parse_popup_signal(""), PopupSignal::Neutral));
    }

    #[test]
    fn parse_popup_signal_known_variants() {
        assert!(matches!(parse_popup_signal("yellow"), PopupSignal::Yellow));
        assert!(matches!(parse_popup_signal("red"), PopupSignal::Red));
        assert!(matches!(parse_popup_signal("gray"), PopupSignal::Gray));
    }

    #[test]
    fn parse_popup_signal_unknown_defaults_to_neutral() {
        assert!(matches!(parse_popup_signal("bogus"), PopupSignal::Neutral));
    }
}
