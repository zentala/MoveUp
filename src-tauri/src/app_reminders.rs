//! Bounded main-window presentation state. No ergonomic thresholds or native calls.
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::communication_types::{ReminderCategory, ReminderDraft, ReminderReason, RetirementReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum ProtectedFlow { Settings, Share }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum ReminderSource { Policy, Session, Test }

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AppReminderMessage {
    pub id: String,
    #[cfg_attr(test, ts(inline))]
    pub source: ReminderSource,
    #[cfg_attr(test, ts(inline))]
    pub reason: ReminderReason,
    #[cfg_attr(test, ts(inline))]
    pub category: ReminderCategory,
    pub title: String,
    pub body: String,
    pub is_test: bool,
    pub elapsed_secs: Option<i64>,
    pub limit_secs: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ReminderRetirement {
    pub id: String,
    #[cfg_attr(test, ts(inline))]
    pub reason: RetirementReason,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export_to = "../../src/generated/"))]
pub struct AppReminderSnapshot {
    pub session_id: String,
    pub revision: u32,
    #[cfg_attr(test, ts(inline))]
    pub active: Option<AppReminderMessage>,
    #[cfg_attr(test, ts(inline))]
    pub pending: Option<AppReminderMessage>,
    #[cfg_attr(test, ts(inline))]
    pub protected_flow: Option<ProtectedFlow>,
    #[cfg_attr(test, ts(inline))]
    pub last_retirement: Option<ReminderRetirement>,
    pub presentation_error: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Clone)]
pub(crate) struct Entry {
    pub message: AppReminderMessage,
    pub draft: ReminderDraft,
    pub day: chrono::NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderAction { Dismiss, Snooze, Retry }

#[derive(Clone)]
pub(crate) struct Acknowledgement {
    pub id: String,
    pub action: ReminderAction,
    pub applied: bool,
    pub busy: bool,
}

pub(crate) struct Reminders {
    pub snapshot: AppReminderSnapshot,
    pub active: Option<Entry>,
    pub pending: Option<Entry>,
    pub acknowledgement: Option<Acknowledgement>,
    next_id: u64,
    client_sequence: u32,
    last_occurrences: [u64; 3],
    presented_id: Option<String>,
}

pub struct AppReminderState(pub(crate) Mutex<Reminders>);

impl Default for AppReminderState {
    fn default() -> Self {
        Self(Mutex::new(Reminders {
            snapshot: AppReminderSnapshot {
                session_id: uuid::Uuid::new_v4().to_string(), revision: 0,
                active: None, pending: None, protected_flow: None, last_retirement: None,
                presentation_error: None, client_id: None,
            }, active: None, pending: None, acknowledgement: None, next_id: 0, client_sequence: 0,
            last_occurrences: [0; 3],
            presented_id: None,
        }))
    }
}

impl Reminders {
    pub fn snapshot(&self) -> AppReminderSnapshot { self.snapshot.clone() }

    fn changed(&mut self) {
        self.snapshot.revision = self.snapshot.revision.checked_add(1).expect("reminder revision exhausted");
        self.snapshot.active = self.active.as_ref().map(|entry| entry.message.clone());
        self.snapshot.pending = self.pending.as_ref().map(|entry| entry.message.clone());
    }

    pub fn claim(&mut self, previous: Option<&str>, client: &str) -> Result<(), String> {
        if client.is_empty() || client.len() > 128 { return Err("invalid_client".into()); }
        if self.snapshot.client_id.as_deref() == Some(client) { return Ok(()); }
        if self.snapshot.client_id.as_deref() != previous { return Err("stale_client".into()); }
        self.snapshot.client_id = Some(client.into());
        self.client_sequence = 0;
        // Retain protected edits until the new owner explicitly reports its actual flow.
        self.changed();
        Ok(())
    }

    pub fn set_flow(&mut self, client: &str, sequence: u32, flow: Option<ProtectedFlow>) -> Result<(), String> {
        if self.snapshot.client_id.as_deref() != Some(client) || sequence <= self.client_sequence {
            return Err("stale_client_update".into());
        }
        self.client_sequence = sequence;
        self.snapshot.protected_flow = flow;
        // Protected views mask the current message; they never move unread or
        // reserved current state into the replaceable pending slot.
        if flow.is_none() && self.active.is_none() {
            self.active = self.pending.take();
        }
        self.changed();
        Ok(())
    }

    pub fn publish(&mut self, draft: ReminderDraft, source: ReminderSource, day: chrono::NaiveDate) {
        let producer = match source { ReminderSource::Policy => 0, ReminderSource::Session => 1, ReminderSource::Test => 2 };
        // Dedupe an actual occurrence, never all identical messages for the day.
        if draft.occurrence <= self.last_occurrences[producer] { return; }
        self.last_occurrences[producer] = draft.occurrence;
        self.next_id += 1;
        let entry = Entry { message: AppReminderMessage {
            id: format!("{}:{}", self.snapshot.session_id, self.next_id), source,
            reason: draft.reason, category: draft.category, title: draft.title.clone(),
            body: draft.body.clone(), is_test: source == ReminderSource::Test,
            elapsed_secs: draft.elapsed_secs, limit_secs: draft.limit_secs,
        }, draft, day };
        // Never replace an in-flight acknowledgement, or hide an unread current message.
        if self.snapshot.protected_flow.is_some() || self.active.is_some() {
            if let Some(old) = self.pending.replace(entry) {
                self.snapshot.last_retirement = Some(ReminderRetirement { id: old.message.id, reason: RetirementReason::Replaced });
            }
        } else { self.active = Some(entry); }
        self.changed();
    }

    pub fn reconcile(&mut self, mut obsolete: impl FnMut(&Entry) -> Option<RetirementReason>) {
        let ack_id = self.acknowledgement.as_ref().map(|ack| ack.id.as_str());
        let mut changed = false;
        for slot in [&mut self.active, &mut self.pending] {
            let reason = slot.as_ref().filter(|entry| !entry.message.is_test && Some(entry.message.id.as_str()) != ack_id)
                .and_then(&mut obsolete);
            if let Some(reason) = reason {
                let entry = slot.take().unwrap();
                self.snapshot.last_retirement = Some(ReminderRetirement { id: entry.message.id, reason });
                changed = true;
            }
        }
        if self.active.is_none() && self.snapshot.protected_flow.is_none() && self.pending.is_some() {
            self.active = self.pending.take(); changed = true;
        }
        if changed { self.changed(); }
    }

    pub fn begin_ack(&mut self, client: &str, id: &str, revision: u32, action: ReminderAction) -> Result<(bool, bool), String> {
        if self.snapshot.client_id.as_deref() != Some(client) || self.snapshot.revision != revision {
            return Err("stale_reminder".into());
        }
        let entry = self.active.as_ref().filter(|entry| entry.message.id == id)
            .ok_or("stale_reminder")?;
        if let Some(ack) = &mut self.acknowledgement {
            if ack.id != id || ack.busy || (action != ReminderAction::Retry && ack.action != action) {
                return Err("acknowledgement_in_progress".into());
            }
            // Generic Retry is server-owned: it may only persist an already applied
            // reservation, never create or guess a Dismiss/Snooze after remount.
            if action == ReminderAction::Retry && !ack.applied { return Err("no_pending_persistence".into()); }
            ack.busy = true;
            return Ok((entry.message.is_test, !ack.applied));
        }
        if action == ReminderAction::Retry { return Err("no_pending_persistence".into()); }
        self.acknowledgement = Some(Acknowledgement { id: id.into(), action, applied: false, busy: true });
        Ok((entry.message.is_test, true))
    }

    pub fn ack_applied(&mut self, id: &str) {
        if let Some(ack) = self.acknowledgement.as_mut().filter(|ack| ack.id == id) { ack.applied = true; }
    }

    pub fn finish_ack(&mut self, id: &str, result: Result<(), String>) -> Result<(), String> {
        if self.acknowledgement.as_ref().map(|ack| ack.id.as_str()) != Some(id) { return Err("stale_reminder".into()); }
        if let Err(error) = result {
            self.acknowledgement.as_mut().unwrap().busy = false;
            self.snapshot.presentation_error = Some(error.clone()); self.changed();
            return Err(error);
        }
        self.acknowledgement = None;
        let mut removed = false;
        for slot in [&mut self.active, &mut self.pending] {
            if slot.as_ref().is_some_and(|entry| entry.message.id == id) { *slot = None; removed = true; }
        }
        if removed {
            self.snapshot.last_retirement = Some(ReminderRetirement { id: id.into(), reason: RetirementReason::Acknowledged });
            if self.snapshot.protected_flow.is_none() && self.active.is_none() { self.active = self.pending.take(); }
        }
        self.snapshot.presentation_error = None; self.changed();
        Ok(())
    }

    pub fn record_presentation(&mut self, revision: u32, result: Result<(), String>) {
        if self.snapshot.revision != revision { return; }
        if result.is_ok() && self.snapshot.protected_flow.is_none() {
            self.presented_id = self.active.as_ref().map(|entry| entry.message.id.clone());
        }
        // Native success cannot conceal an outstanding acknowledgement write failure.
        if self.acknowledgement.is_some() { return; }
        let error = result.err();
        if self.snapshot.presentation_error != error {
            self.snapshot.presentation_error = error; self.changed();
        }
    }

    pub fn retain_on_focus_loss(&self) -> bool {
        self.snapshot.protected_flow.is_none() && self.active.as_ref()
            .is_some_and(|entry| self.presented_id.as_deref() == Some(entry.message.id.as_str()))
    }
}
