//! notify_webhook.rs — pushes desk notifications to ntfy or any JSON webhook.
//!
//! A Windows toast only reaches the machine the app runs on. This module
//! mirrors the same notification to a URL the user controls, so a phone or a
//! watch can show it too.
//!
//! ## Wire format
//! One `POST <url>` with `Content-Type: application/json` and the body
//! `{"title": …, "message": …, "priority": …, "tags": [...]}`. That is
//! [ntfy](https://docs.ntfy.sh/publish/#publish-as-json)'s publish-as-JSON
//! shape, posted to the topic URL itself (`https://ntfy.sh/<topic>`), so the
//! same request works against ntfy and against any generic webhook receiver
//! that reads those four fields.
//!
//! ## Delivery model
//! Delivery is fire-and-forget: [`WebhookNotifier::send`] spawns the request
//! and returns immediately, so a slow or dead receiver can never stall the
//! ~1 Hz policy tick that produced the notification. Each attempt has a 5 s
//! timeout and one retry on a 5xx or a timeout — a receiver that answers 4xx
//! is misconfigured, not flaky, so that case is not retried.
//!
//! ## Configuration
//! The URL comes from `AppConfig.notify_webhook_url`, falling back to the
//! `DESK_NOTIFY_WEBHOOK_URL` environment variable. Either way the toggle
//! `AppConfig.notify_webhook_enabled` gates it: off means no request is ever
//! made. The URL may embed a token, so it is never logged.

use serde::Serialize;
use std::time::Duration;

/// Environment fallback for the webhook URL.
pub const WEBHOOK_URL_ENV: &str = "DESK_NOTIFY_WEBHOOK_URL";

/// Per-attempt timeout. Two attempts means at most ~10 s in the worst case,
/// all of it off the caller's thread.
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(5);

/// ntfy priority for a sit-limit alert (4 = high, 5 = max/urgent).
pub const PRIORITY_ALERT: u8 = 4;

/// The JSON body sent to the webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: u8,
    pub tags: Vec<String>,
}

impl Notification {
    /// A sit/stand-limit alert — the notification class that mirrors the
    /// Windows toast raised by `NotifySignal::Toast`.
    pub fn alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            priority: PRIORITY_ALERT,
            tags: vec!["chair".to_string()],
        }
    }
}

/// Why a delivery attempt failed. Retryable failures are transport-level or
/// server-side; a rejected request is the receiver telling us we are wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookError {
    /// Timed out, connection refused, DNS failure — worth one retry.
    Transport,
    /// Receiver answered 5xx — worth one retry.
    Server(u16),
    /// Receiver answered 4xx — a retry would fail identically.
    Rejected(u16),
}

/// A configured webhook endpoint.
pub struct WebhookNotifier {
    url: String,
    http: reqwest::Client,
    timeout: Duration,
}

impl WebhookNotifier {
    /// Resolves the notifier from the saved config, falling back to
    /// [`WEBHOOK_URL_ENV`].
    ///
    /// Returns `None` — meaning "not configured", never an error — when the
    /// toggle is off, when neither source holds a URL, or when the URL that
    /// is there is blank.
    pub fn from_env_or_config(enabled: bool, config_url: Option<&str>) -> Option<Self> {
        if !enabled {
            return None;
        }
        let url = config_url
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .map(str::to_string)
            .or_else(|| {
                std::env::var(WEBHOOK_URL_ENV)
                    .ok()
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty())
            })?;
        Some(Self::at_url(url))
    }

    /// Builds a notifier pointed at an explicit URL.
    pub fn at_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            http: reqwest::Client::new(),
            timeout: ATTEMPT_TIMEOUT,
        }
    }

    /// Shortens the per-attempt timeout — used by tests so the timeout path
    /// does not cost ten real seconds.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Fires the notification and returns immediately.
    ///
    /// Callers must never await webhook delivery: this runs on the ~1 Hz
    /// policy tick, and a receiver that hangs would otherwise hang the tray.
    pub fn send(self, notification: Notification) {
        tauri::async_runtime::spawn(async move {
            if let Err(e) = self.deliver(&notification).await {
                log::warn!("notify_webhook: delivery failed: {e:?}");
            }
        });
    }

    /// One delivery, with a single retry on a transport failure or a 5xx.
    pub(crate) async fn deliver(&self, notification: &Notification) -> Result<(), WebhookError> {
        match self.attempt(notification).await {
            Ok(()) => Ok(()),
            Err(WebhookError::Rejected(code)) => Err(WebhookError::Rejected(code)),
            Err(_) => self.attempt(notification).await,
        }
    }

    /// A single POST. The URL is never logged — it can carry a token.
    async fn attempt(&self, notification: &Notification) -> Result<(), WebhookError> {
        let response = self
            .http
            .post(&self.url)
            .timeout(self.timeout)
            .json(notification)
            .send()
            .await
            .map_err(|_| WebhookError::Transport)?;

        let status = response.status();
        if status.is_success() || status.is_redirection() {
            return Ok(());
        }
        if status.is_client_error() {
            return Err(WebhookError::Rejected(status.as_u16()));
        }
        Err(WebhookError::Server(status.as_u16()))
    }
}

// Tests live in notify_webhook_tests.rs.
