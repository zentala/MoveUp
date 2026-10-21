//! serial_periodic_reset.rs — Responsibility units behind the serial loop's
//! periodic tick and reading handler.
//!
//! [`crate::serial_periodic`] keeps only the two orchestrators called from
//! `serial.rs` (`check_periodic`, `handle_reading`); every step they take that
//! does not need an [`AppHandle`] lives here, one function per responsibility:
//! daily reset + telemetry, per-minute snapshot logging, notification-intent
//! collection, completed-session persistence, and the event-log line formats.
//!
//! Splitting on the `AppHandle` boundary is what makes these testable — a
//! Tauri app handle cannot be built in a unit test, so anything that keeps one
//! stays in the orchestrator.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, NaiveDate, Utc};
use log::error;

use crate::communication_profile::CommunicationProfile;
use crate::config::AppConfig;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::event_logger::EventLogger;
use crate::metrics::MetricEngine;
use crate::notification_service::{NotificationIntent, NotificationService};
use crate::session::{BreakCredit, CompletedSession, DeskState, SessionManager};
use crate::snapshot_logger::SnapshotLogger;

/// Runs the daily rollover, sending telemetry first so it captures the full day.
///
/// Returns `true` when counters were actually reset this tick. The telemetry
/// send is gated by config and is a no-op when opted out.
pub fn run_daily_reset(session: &Arc<Mutex<SessionManager>>, config: &AppConfig) -> bool {
    let mut sess = session.lock().unwrap();
    if sess.needs_daily_reset() {
        crate::telemetry::send_telemetry_if_enabled(config, &sess);
    }
    sess.check_daily_reset()
}

/// Rolls over a changed local day before the current reading changes posture.
/// The explicit instant/date/boundary keep the adapter deterministic, while
/// the engine decides whether the reading extends known observation coverage.
pub(crate) fn run_daily_reset_before_reading(
    session: &Arc<Mutex<SessionManager>>,
    config: &AppConfig,
    now: DateTime<Utc>,
    today_local: NaiveDate,
    day_boundary: DateTime<Utc>,
) -> bool {
    let mut sess = session.lock().unwrap_or_else(|error| error.into_inner());
    if sess.last_reset_date >= today_local {
        return false;
    }
    let last_observation = sess.history_last_reading.into_iter()
        .chain(sess.state.last_tick_ts).max();
    if last_observation.is_some_and(|last| now < last) {
        return false;
    }
    crate::telemetry::send_telemetry_if_enabled(config, &sess);
    sess.check_daily_reset_at_observation(now, today_local, day_boundary)
}

/// Writes the per-minute JSON snapshot with computed KPI metrics.
///
/// Metrics are computed from RAW live totals, not the credited counter — the
/// KPI layer compares raw against raw (E015).
pub fn log_minute_snapshot(
    session: &Arc<Mutex<SessionManager>>,
    snapshot_logger: &SnapshotLogger,
    port_name: &str,
    ergo: &ErgonomicProfile,
) {
    let sess = session.lock().unwrap();
    let now = chrono::Utc::now();
    let snapshot = sess.snapshot();
    let mut raw = sess.state.clone();
    raw.sitting_seconds_total = sess.get_live_sitting_seconds_total(now);
    raw.standing_seconds = sess.get_live_standing_seconds(now);
    let metrics = MetricEngine::with_defaults().compute_all(&raw, ergo);
    snapshot_logger.log_snapshot(
        &snapshot,
        true,
        Some(port_name.to_string()),
        crate::APP_VERSION,
        metrics,
    );
}

/// What one periodic notification check produced.
pub struct NotificationTick {
    /// True when the session raised at least one notification event, even if it
    /// yielded no dispatchable intent (`StandLimitReached` is deliberately
    /// intent-less). Persistence keys off this, not off `intents` — a flag that
    /// flipped must survive a restart whether or not anything was shown.
    pub events_fired: bool,
    /// Intents ready for [`NotificationService::dispatch`].
    pub intents: Vec<NotificationIntent>,
}

/// Evaluates the session's notification conditions and turns them into intents.
///
/// Dispatch stays with the caller: this function only decides *what* would be
/// said, so it can be exercised without a notification backend.
///
/// Both counters handed to the intent builder are RAW daily totals — the
/// PostureBalance message states sitting against standing, so a credited
/// sitting value would report a ratio the engine never computed (E015-T03).
pub fn collect_notification_intents(
    session: &Arc<Mutex<SessionManager>>,
    comm: &CommunicationProfile,
) -> NotificationTick {
    let mut sess = session.lock().unwrap();
    let events = sess.check_notification_conditions(comm);
    if events.is_empty() {
        return NotificationTick {
            events_fired: false,
            intents: Vec::new(),
        };
    }
    let snap = sess.snapshot();
    NotificationTick {
        events_fired: true,
        intents: NotificationService::build_intents(
            &events,
            snap.sitting_seconds_total,
            snap.standing_seconds,
        ),
    }
}

/// Saves a span that ended this tick to the sessions table.
///
/// A break span carries the credit it earned; a sitting span carries none and
/// the column stays NULL for it. A failed insert is returned so the queued span
/// remains available for a later retry.
pub fn persist_completed_session(
    db: &Arc<Mutex<Option<rusqlite::Connection>>>,
    completed: &CompletedSession,
    state_before: &DeskState,
    credit: Option<&BreakCredit>,
) -> bool {
    let db_lock = db.lock().unwrap();
    let Some(ref conn) = *db_lock else {
        return false;
    };
    if let Err(e) = crate::db_sessions::insert_session_once(
        conn,
        &completed.started_at,
        &completed.ended_at,
        &format!("{:?}", state_before),
        completed.duration_secs,
        credit,
    ) {
        error!("Failed to save completed session: {}", e);
        return false;
    }
    true
}

/// Persist pending rows in order, acknowledging each only after its SQLite
/// statement succeeds. The callback records rows this attempt actually inserted;
/// deduplicated rows are not owned by the caller and must never be rolled back.
pub(crate) fn persist_pending_history(
    session: &mut SessionManager,
    conn: &rusqlite::Connection,
    mut on_insert: impl FnMut(&crate::session_types::PendingHistory, i64),
) -> Result<(), rusqlite::Error> {
    session.queue_interrupted_history();
    while let Some(pending) = session.pending_history.first().cloned() {
        let inserted_id = crate::db_sessions::insert_session_once(
            conn,
            &pending.span.started_at,
            &pending.span.ended_at,
            &format!("{:?}", pending.state),
            pending.span.duration_secs,
            pending.credit.as_ref(),
        )?;
        if let Some(id) = inserted_id {
            on_insert(&pending, id);
        }
        session.pending_history.remove(0);
    }
    Ok(())
}

/// Formats the `STATE` event-log line for a state transition.
pub fn format_state_change_line(
    before: &DeskState,
    after: &DeskState,
    height_cm: f32,
    idle_secs: u64,
) -> String {
    format!(
        "STATE {:?}\u{2192}{:?} h={:.0}cm idle={}s",
        before, after, height_cm, idle_secs
    )
}

/// Formats the `CREDIT` event-log line for a break credit applied this tick.
pub fn format_credit_line(credit: &BreakCredit, duration_secs: i64, sitting_secs: i64) -> String {
    format!(
        "CREDIT {:?} dur={}s sitting={}",
        credit, duration_secs, sitting_secs
    )
}

/// Logs the break credit earned by a break that ended this tick.
///
/// Also drains the one-shot `day_break_applied` flag, emitting its own line so
/// a day-length break is distinguishable in the log from an ordinary one.
pub fn log_break_credit(
    session: &Arc<Mutex<SessionManager>>,
    event_logger: &EventLogger,
    credit: &BreakCredit,
    duration_secs: i64,
) {
    let mut sess = session.lock().unwrap();
    let sitting = sess.snapshot().sitting_seconds;
    event_logger.log(&format_credit_line(credit, duration_secs, sitting));
    if sess.day_break_applied {
        event_logger.log(&format!("CREDIT day_break dur={}s", duration_secs));
        sess.day_break_applied = false;
    }
}
