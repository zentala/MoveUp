//! Signal-parsing and signal-building helpers for [`CommunicationPolicy`].
//!
//! Pure string → enum conversions and stateless signal builders extracted from
//! the main policy module to keep file sizes under 250 lines.

use crate::communication_profile::{BaselineConfig, CommunicationProfile, MessageConfig};
use crate::communication_types::{NotifySignal, OverlaySignal, PopupSignal, Signals, TraySignal};
use crate::session_types::DeskState;

// ── Signal parsers ────────────────────────────────────────────────────────────

/// Parse a tray signal name into a [`TraySignal`].
pub(crate) fn parse_tray_signal(s: &str) -> TraySignal {
    match s {
        "none" | "" => TraySignal::None,
        "yellow" => TraySignal::Yellow,
        "red" => TraySignal::Red,
        other => {
            log::warn!("Unrecognized tray signal '{}' in profile — treating as blink pattern", other);
            TraySignal::Blink(other.to_string())
        }
    }
}

/// Parse an overlay signal name into an [`OverlaySignal`] with the given progress.
pub(crate) fn parse_overlay_signal(s: &str, progress: f32) -> OverlaySignal {
    match s {
        "hidden" | "" => OverlaySignal::Hidden,
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

/// Build the inactive-state signals (Away or Walking).
pub(crate) fn inactive_signals() -> Signals {
    Signals {
        tray: TraySignal::None,
        overlay: OverlaySignal::Hidden,
        popup: PopupSignal::Neutral,
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
                overlay: OverlaySignal::Neutral { progress: 0.0 },
                popup: parse_popup_signal(&bl.popup),
                notify: None,
            }
        }
        DeskState::Standing => {
            let bl = &baseline.standing;
            Signals {
                tray: parse_tray_signal(&bl.tray),
                overlay: OverlaySignal::Progress { progress: lap_progress, lap, flash: lap_flash },
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
            if is_firm {
                "You really need to stand up now.".to_string()
            } else {
                msgs.sitting_overdue_popup.clone()
            }
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
