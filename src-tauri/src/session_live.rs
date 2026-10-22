//! session_live.rs — Live value computation for SessionManager.
//!
//! These methods compute real-time values by combining committed counters
//! with elapsed time from active timestamps. Used by snapshot() and UI.

use chrono::{DateTime, Duration, Utc};
use log::warn;

use crate::session_types::*;
use crate::session_manager::SessionManager;

impl SessionManager {
    /// Returns the last actual observation that can support a live projection.
    /// A gap threshold detects missing coverage; it does not extend it.
    pub(crate) fn known_projection_end(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let span = self.history_span_started?;
        if now < span {
            return None;
        }
        let last = self.history_last_reading.into_iter()
            .chain(self.state.last_tick_ts).max()?;
        Some(now.min(last))
    }

    /// Returns known elapsed time after the later of the timer anchor and the
    /// currently confirmed history prefix. A resumed sample cannot revive an
    /// older timer anchor until its coherent sequence establishes a new prefix.
    pub(crate) fn known_elapsed_since(&self, started: DateTime<Utc>, now: DateTime<Utc>) -> i64 {
        let start = self.history_span_started
            .map(|span| started.max(span))
            .unwrap_or(started);
        let end = self.known_projection_end(now).unwrap_or(start);
        (end - start).num_seconds().max(0)
    }

    /// Whether current state may still be used for time-based decisions.
    pub(crate) fn observation_is_fresh_at(&self, now: DateTime<Utc>) -> bool {
        let Some(last) = self.history_last_reading.into_iter()
            .chain(self.state.last_tick_ts).max() else { return false; };
        self.history_span_started.is_some()
            && !self.pending_after_gap
            && now >= last
            && now - last <= Duration::seconds(SLEEP_GAP_THRESHOLD_SECS)
    }

    /// Commits the observed prefix through `boundary`, closing history and raw
    /// posture counters together. The caller may reopen the same posture at a
    /// known boundary; unknown time is never included by this operation.
    pub(crate) fn close_observed_boundary(
        &mut self,
        boundary: DateTime<Utc>,
    ) -> Option<CompletedSession> {
        // A requested calendar/transition boundary can fall after the latest
        // sensor sample. Close only through the latest actual observation.
        let last_observation = self.history_last_reading.into_iter()
            .chain(self.state.last_tick_ts).max()?;
        let end = boundary.min(last_observation);
        if end < self.history_span_started.unwrap_or(end) {
            return None;
        }
        let completed = self.observed_span_at(end);
        match self.state.state {
            DeskState::Sitting => {
                if self.state.sitting_started.is_some_and(|started| started <= end) {
                    let started = self.state.sitting_started.take().unwrap();
                    let elapsed = (end - started).num_seconds().max(0);
                    self.state.sitting_seconds += elapsed;
                    self.state.sitting_seconds_total += elapsed;
                    self.state.last_sitting_secs = elapsed;
                }
            }
            DeskState::Standing | DeskState::Walking => {
                if self.state.standing_bout_started.is_some_and(|started| started <= end) {
                    let started = self.state.standing_bout_started.take().unwrap();
                    self.state.standing_seconds += (end - started).num_seconds().max(0);
                }
            }
            DeskState::Away => {}
        }
        if self.history_span_started.is_some_and(|started| started <= end) {
            self.history_span_started = Some(end);
        }
        completed
    }

    /// Starts observation and raw-counter anchors at an explicit known instant.
    pub(crate) fn open_observed_boundary(&mut self, at: DateTime<Utc>) {
        self.history_span_started = Some(at);
        match self.state.state {
            DeskState::Sitting => self.state.sitting_started = Some(at),
            DeskState::Standing => {
                self.state.standing_bout_started = Some(at);
                self.state.standing_session_started = Some(at);
                self.state.standing_session_secs = 0;
                self.state.lap_bonus_awarded_for_lap = 0;
            }
            DeskState::Walking | DeskState::Away => {}
        }
    }

    /// Computes live standing seconds: accumulated + current standing bout elapsed.
    /// Only counts actual Standing time, not Away time.
    pub(crate) fn get_live_standing_seconds(&self, now: DateTime<Utc>) -> i64 {
        let base = self.state.standing_seconds;
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_bout_started {
                return base + self.known_elapsed_since(started, now);
            }
        }
        base
    }

    /// Seconds elapsed since the last position change — uncredited.
    ///
    /// Debug-tab diagnostic only (E015 decision D2). Never use it for a
    /// timer, progress bar, colour band or notification; those read
    /// `limit_used_secs`, the credited counter. Returns 0 when no position
    /// change has been recorded yet.
    pub(crate) fn get_secs_since_last_break(&self, now: DateTime<Utc>) -> i64 {
        match self.state.last_position_change_at {
            Some(changed_at) => (now - changed_at).num_seconds().max(0),
            None => 0,
        }
    }

    /// Computes live sitting seconds: committed + elapsed since sitting_started.
    pub(crate) fn get_live_sitting_seconds(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                return self.state.sitting_seconds + self.known_elapsed_since(started, now);
            }
        }
        self.state.sitting_seconds
    }

    /// Computes live raw sitting seconds (never reduced by break credit).
    /// Used by standing_pct metric for accurate KPI calculation.
    pub(crate) fn get_live_sitting_seconds_total(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Sitting {
            if let Some(started) = self.state.sitting_started {
                return self.state.sitting_seconds_total + self.known_elapsed_since(started, now);
            }
        }
        self.state.sitting_seconds_total
    }

    /// Computes live standing session seconds from timestamp.
    pub(crate) fn get_live_standing_session_secs(&self, now: DateTime<Utc>) -> i64 {
        if self.state.state == DeskState::Standing {
            if let Some(started) = self.state.standing_session_started {
                if self.known_projection_end(now).is_none() {
                    return self.state.standing_session_secs;
                }
                return self.state.standing_session_secs
                    + self.known_elapsed_since(started, now);
            }
        }
        self.state.standing_session_secs
    }

    /// Builds an exact history interval, independent of credited counters.
    pub(crate) fn observed_span_at(&self, end: DateTime<Utc>) -> Option<CompletedSession> {
        let started = self.history_span_started?;
        if end <= started {
            return None;
        }
        Some(CompletedSession {
            started_at: started.to_rfc3339(),
            ended_at: end.to_rfc3339(),
            duration_secs: (end - started).num_seconds(),
        })
    }

    pub(crate) fn queue_history(&mut self, state: DeskState, span: CompletedSession,
        credit: Option<BreakCredit>) {
        if !self.pending_history.iter().any(|pending|
            pending.state == state && pending.span.started_at == span.started_at
                && pending.span.ended_at == span.ended_at) {
            self.pending_history.push(PendingHistory { state, span, credit });
        }
    }

    pub(crate) fn queue_interrupted_history(&mut self) {
        for (state, span) in std::mem::take(&mut self.interrupted_history) {
            self.queue_history(state, span, None);
        }
    }

    /// Close the shutdown tail once; repeated flushes cannot overlap that tail.
    pub(crate) fn queue_shutdown_history(&mut self, now: DateTime<Utc>) {
        self.queue_interrupted_history();
        let observation_backed = self.history_span_started.is_some();
        if let Some(span) = self.flush_current_session_at(now) {
            if let Ok(end) = DateTime::parse_from_rfc3339(&span.ended_at) {
                let end = end.with_timezone(&Utc);
                // Snapshot the raw and credited counter tail at the same
                // instant as the shutdown row. Keep the anchors at this
                // boundary if shutdown is cancelled and this process resumes;
                // restore clears all live anchors so stopped time stays unknown.
                if self.state.state == DeskState::Sitting {
                    if let Some(started) = self.state.sitting_started.filter(|started| *started <= end) {
                        let elapsed = (end - started).num_seconds().max(0)
                            .min(span.duration_secs);
                        self.state.sitting_seconds += elapsed;
                        self.state.sitting_seconds_total += elapsed;
                        self.state.sitting_started = Some(end);
                    }
                } else if self.state.state.is_standing_like() {
                    if let Some(started) = self.state.standing_bout_started.filter(|started| *started <= end) {
                        self.state.standing_seconds += (end - started).num_seconds().max(0)
                            .min(span.duration_secs);
                        self.state.standing_bout_started = Some(end);
                    }
                }
                if self.state.state == DeskState::Standing {
                    self.state.standing_session_secs =
                        self.get_live_standing_session_secs(end);
                }
                if self.state.state != DeskState::Sitting
                    && self.state.break_started.is_some()
                {
                    let observed_break_secs = self.get_live_break_seconds(end);
                    self.observed_break_secs = observed_break_secs;
                    self.state.break_seconds = observed_break_secs;
                }
                if observation_backed {
                    self.history_span_started = Some(end);
                }
            }
            self.queue_history(self.current_state(), span, None);
        }
    }

    /// Snapshots the current in-progress session as a CompletedSession for DB persistence.
    /// Does NOT modify state — safe to call on shutdown without side effects.
    /// Returns `None` if no session is currently active.
    /// Observed spans use consistent endpoints; legacy counter-only callers
    /// retain the old `MAX_REASONABLE_SESSION_SECS` cap.
    pub fn flush_current_session(&self) -> Option<CompletedSession> {
        self.flush_current_session_at(Utc::now())
    }

    /// Clock-injected shutdown snapshot. A stale last reading bounds observed
    /// coverage; a fresh reading does not extend coverage without another
    /// observation. The unobserved tail is never relabeled as Away.
    pub fn flush_current_session_at(&self, now: DateTime<Utc>) -> Option<CompletedSession> {
        if self.history_span_started.is_some() {
            let end = self.known_projection_end(now)?;
            return self.observed_span_at(end);
        }
        // Compatibility for callers that seed counter anchors directly. A
        // restored snapshot clears them, so it cannot duplicate prior history.
        let started = match self.state.state {
            DeskState::Sitting => self.state.sitting_started,
            DeskState::Standing | DeskState::Walking => self.state.standing_bout_started,
            _ => None,
        }?;
        let raw_elapsed = (now - started).num_seconds().max(0);
        let elapsed = if raw_elapsed > MAX_REASONABLE_SESSION_SECS {
            warn!(
                "Session capped at {}s (was {}s — likely sleep gap)",
                MAX_REASONABLE_SESSION_SECS, raw_elapsed
            );
            MAX_REASONABLE_SESSION_SECS
        } else {
            raw_elapsed
        };
        Some(CompletedSession {
            started_at: started.to_rfc3339(),
            ended_at: now.to_rfc3339(),
            duration_secs: elapsed,
        })
    }
}
