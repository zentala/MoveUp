//! Signal execution — maps CommunicationPolicy signals to UI side-effects.
//!
//! Extracted from `tray_controller.rs` to keep it under 250 lines.

use tauri::AppHandle;

use crate::{
    colors::{color_for_progress, color_for_standing},
    commands::AppState,
    communication_types::{NotifySignal, OverlaySignal, PopupSignal, TraySignal},
    session::DeskState,
    tray,
};

/// Maps a [`TraySignal`] to a tray icon update.
pub(crate) fn execute_tray(
    signal: &TraySignal,
    app: &AppHandle,
    snapshot: &crate::session::SessionStateDto,
) {
    let progress = if snapshot.session_limit_secs > 0 {
        snapshot.sitting_seconds as f32 / snapshot.session_limit_secs as f32
    } else {
        0.0
    };

    match signal {
        TraySignal::None => {
            let _ = tray::update_tray(app, "", DeskState::Away, 0.0);
        }
        TraySignal::Yellow => {
            let _ = tray::update_tray(app, "", DeskState::Sitting, 0.7);
        }
        TraySignal::Red | TraySignal::Blink(_) => {
            let _ = tray::update_tray(app, "", DeskState::Sitting, progress.max(0.86));
        }
    }
}

/// Maps an [`OverlaySignal`] to overlay renderer calls.
pub(crate) fn execute_overlay(
    signal: &OverlaySignal,
    overlay: &crate::overlay_renderer::OverlayRenderer,
    snapshot: &crate::session::SessionStateDto,
) {
    match signal {
        OverlaySignal::Hidden => {
            overlay.hide();
        }
        OverlaySignal::Neutral { progress } => {
            let (r, g, b, _) = color_for_progress(*progress);
            overlay.set_variant(0);
            overlay.update(*progress, (r, g, b));
            overlay.show();
        }
        OverlaySignal::Yellow { progress } => {
            overlay.set_variant(0);
            overlay.update(*progress, (255, 193, 7));
            overlay.show();
        }
        OverlaySignal::Red { progress } => {
            overlay.set_variant(0);
            overlay.update(*progress, (244, 67, 54));
            overlay.show();
        }
        OverlaySignal::PulseRed { progress } => {
            overlay.set_variant(2);
            overlay.update(*progress, (244, 67, 54));
            overlay.show();
        }
        OverlaySignal::Progress { progress, lap, flash } => {
            let (r, g, b) = color_for_standing(*progress);
            overlay.update(*progress, (r, g, b));
            overlay.update_standing(*progress, *lap);
            overlay.show();
            if *flash {
                let target = snapshot.stand_limit_secs;
                if target > 0 {
                    let session_lap = (snapshot.break_seconds / target) as u32;
                    overlay.maybe_flash_lap(session_lap);
                }
            }
        }
    }
}

/// Emits a `desk:popup-theme` event so the frontend can update the popup colour scheme.
///
/// The frontend floating window listens for this event and applies the
/// appropriate CSS class / theme. `Neutral` resets to the default appearance.
pub(crate) fn execute_popup(signal: &PopupSignal, app: &AppHandle) {
    use tauri::Emitter;
    let theme = match signal {
        PopupSignal::Neutral => "neutral",
        PopupSignal::Yellow => "yellow",
        PopupSignal::Red => "red",
        PopupSignal::Gray => "gray",
    };
    let _ = app.emit("desk:popup-theme", theme);
}

/// Dispatches a [`NotifySignal`] to the appropriate notification backend.
pub(crate) fn execute_notify(signal: &NotifySignal, app_state: &AppState) {
    match signal {
        NotifySignal::Toast(msg) => {
            crate::notify::show("Smart Desk", msg);
        }
        NotifySignal::Popup(msg) => {
            app_state.alert_popup.lock().unwrap().show(msg.clone());
        }
    }
}
