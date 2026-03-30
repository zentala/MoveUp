//! Signal types for the SmartDesk communication architecture.
//!
//! Defines the discrete signals that `CommunicationPolicy` emits for each UI channel
//! (tray icon, overlay bar, popup window, and notifications). Callers collect these
//! into a [`Signals`] struct and hand them to the respective renderers.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TraySignal
// ---------------------------------------------------------------------------

/// Desired visual state for the system-tray icon / tooltip colour dot.
///
/// - `None`  — neutral state; use the default (grey) dot.
/// - `Yellow` — approaching the session limit; yellow dot.
/// - `Red`   — session limit reached; red dot.
/// - `Blink` — animated blink with an optional reason string (e.g. "alert").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraySignal {
    /// No special colouring; default neutral dot.
    None,
    /// Approaching session limit — show yellow dot.
    Yellow,
    /// Session limit reached — show red dot.
    Red,
    /// Animated blink with a reason label (used for escalating alerts).
    Blink(String),
}

// ---------------------------------------------------------------------------
// OverlaySignal
// ---------------------------------------------------------------------------

/// Desired visual state for the top-of-screen overlay progress bar.
///
/// Each variant carries the data the renderer needs to draw the bar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverlaySignal {
    /// Hide the overlay bar entirely.
    Hidden,
    /// Show a neutral (grey) bar — used while standing or in an away state.
    /// `progress` is 0.0–1.0 (current fill level, carried over for continuity).
    Neutral {
        /// Current fill level (0.0–1.0).
        progress: f32,
    },
    /// Normal sitting progress — bar colour shifts green → yellow → red.
    /// `lap` counts completed 40-min sessions today; `flash` triggers a brief
    /// white flash animation when a lap resets.
    Progress {
        /// Fill level (0.0–1.0).
        progress: f32,
        /// Number of completed sessions (laps) so far today.
        lap: u32,
        /// Trigger a flash animation on lap reset.
        flash: bool,
    },
    /// Warning state — force yellow regardless of progress value.
    Yellow {
        /// Fill level (0.0–1.0).
        progress: f32,
    },
    /// Alert state — force red regardless of progress value.
    Red {
        /// Fill level (0.0–1.0).
        progress: f32,
    },
    /// Pulsing red — used for escalated alerts requiring immediate attention.
    PulseRed {
        /// Fill level (0.0–1.0).
        progress: f32,
    },
}

// ---------------------------------------------------------------------------
// PopupSignal
// ---------------------------------------------------------------------------

/// Desired visual theme for the floating popup window.
///
/// The popup renders session stats and KPIs; this signal controls its colour
/// scheme / urgency level rather than its content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PopupSignal {
    /// Default appearance — no urgency.
    Neutral,
    /// Warning appearance — approaching session limit.
    Yellow,
    /// Alert appearance — session limit exceeded.
    Red,
    /// Muted/inactive appearance — user is away or standing.
    Gray,
}

// ---------------------------------------------------------------------------
// NotifySignal
// ---------------------------------------------------------------------------

/// A one-shot notification to fire.
///
/// `None` signals (via `Option<NotifySignal>` on [`Signals`]) mean "do not
/// fire a notification this cycle".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotifySignal {
    /// Show a Windows toast notification with the given body text.
    Toast(String),
    /// Show an in-app alert popup with the given message.
    Popup(String),
}

// ---------------------------------------------------------------------------
// Signals
// ---------------------------------------------------------------------------

/// Aggregated per-cycle output from `CommunicationPolicy`.
///
/// One `Signals` value is produced per sensor/session update.  Each renderer
/// reads the relevant field and updates its UI element accordingly.
///
/// `notify` is `None` in the common case (no notification needed this cycle).
#[derive(Debug, Clone)]
pub struct Signals {
    /// Target state for the system-tray icon.
    pub tray: TraySignal,
    /// Target state for the overlay progress bar.
    pub overlay: OverlaySignal,
    /// Target theme for the floating popup window.
    pub popup: PopupSignal,
    /// Optional one-shot notification to fire this cycle.
    pub notify: Option<NotifySignal>,
}

impl Default for Signals {
    fn default() -> Self {
        Self {
            tray: TraySignal::None,
            overlay: OverlaySignal::Hidden,
            popup: PopupSignal::Neutral,
            notify: None,
        }
    }
}
