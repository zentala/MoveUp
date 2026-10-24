//! Sensor-independent periodic snapshots of the current observed engine state.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager};
use tauri::Runtime;

const CHECKPOINT_INTERVAL: Duration = Duration::from_secs(60);
const FAILURE_NOTICE_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[derive(Default)]
pub struct CheckpointWorker {
    cancel: Arc<(Mutex<bool>, Condvar)>,
    running: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl CheckpointWorker {
    pub fn start(&self, interval: Duration, task: impl Fn() + Send + 'static) -> bool {
        let mut thread_slot = self.thread.lock().unwrap_or_else(|error| error.into_inner());
        if self.running.load(Ordering::Acquire) {
            return false;
        }
        if let Some(previous) = thread_slot.take() {
            let _ = previous.join();
        }
        let (lock, _wake) = &*self.cancel;
        *lock.lock().unwrap_or_else(|error| error.into_inner()) = false;
        self.running.store(true, Ordering::Release);

        let cancel = self.cancel.clone();
        let running = self.running.clone();
        *thread_slot = Some(thread::spawn(move || {
            let _running_lifetime = RunningReset(running.clone());
            loop {
                let (lock, wake) = &*cancel;
                let cancelled = lock.lock().unwrap_or_else(|error| error.into_inner());
                let (cancelled, _) = wake
                    .wait_timeout_while(cancelled, interval, |cancelled| !*cancelled)
                    .unwrap_or_else(|error| error.into_inner());
                if *cancelled {
                    break;
                }
                drop(cancelled);
                task();
            }
        }));
        true
    }

    pub fn stop(&self) -> Result<(), String> {
        let (lock, wake) = &*self.cancel;
        *lock.lock().unwrap_or_else(|error| error.into_inner()) = true;
        wake.notify_all();
        let thread = self.thread.lock().unwrap_or_else(|error| error.into_inner()).take();
        if let Some(thread) = thread {
            thread.join().map_err(|_| "Persistence checkpoint worker panicked".to_string())?;
        }
        self.running.store(false, Ordering::Release);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
}

struct RunningReset(Arc<AtomicBool>);

impl Drop for RunningReset {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub fn start(app: &AppHandle) {
    let Some(lifecycle) = app.try_state::<Arc<crate::app_lifecycle::AppLifecycle>>() else {
        log::error!("Cannot start persistence checkpoint: lifecycle state is missing");
        return;
    };
    let lifecycle = lifecycle.inner().clone();
    let checkpoint_app = app.clone();
    if lifecycle.checkpoint_worker.start(CHECKPOINT_INTERVAL, move || {
        checkpoint_once(&checkpoint_app);
    }) {
        lifecycle.event("persistence_checkpoint_started interval_seconds=60");
    }
}

pub fn stop(app: &AppHandle) -> Result<(), String> {
    let Some(lifecycle) = app.try_state::<Arc<crate::app_lifecycle::AppLifecycle>>() else {
        return Ok(());
    };
    lifecycle.checkpoint_worker.stop()
}

fn checkpoint_once(app: &AppHandle) {
    // Capture the paired wall and monotonic anchors once for this snapshot.
    let wall_now = chrono::Utc::now();
    let monotonic_now = Instant::now();
    let result = crate::session_persistence::save_via_app_at(
        app,
        wall_now,
        monotonic_now,
    );
    // save_via_app_at owns result status, event logging and notification;
    // reporting here as well would duplicate both failure and recovery events.
    let _ = result;
}

pub(crate) fn record_result<R: Runtime>(
    app: &AppHandle<R>,
    result: &Result<(), String>,
    at: chrono::DateTime<chrono::Utc>,
) {
    record_result_inner(app, result, at, false);
}

pub(crate) fn record_checkpoint_result<R: Runtime>(
    app: &AppHandle<R>,
    result: &Result<(), String>,
    at: chrono::DateTime<chrono::Utc>,
) {
    record_result_inner(app, result, at, true);
}

fn record_result_inner<R: Runtime>(
    app: &AppHandle<R>,
    result: &Result<(), String>,
    at: chrono::DateTime<chrono::Utc>,
    periodic_checkpoint: bool,
) {
    let Some(lifecycle) = app.try_state::<Arc<crate::app_lifecycle::AppLifecycle>>() else {
        return;
    };
    let lifecycle = lifecycle.inner();
    let notification = {
        let mut status = lifecycle.checkpoint_status.lock().unwrap_or_else(|error| error.into_inner());
        match result {
            Ok(()) => {
                status.last_success_utc = Some(at.to_rfc3339());
                if status.last_error.take().is_some() {
                    Some(("recovered".to_string(), "Session saving has recovered.".to_string()))
                } else {
                    None
                }
            }
            Err(error) => {
                status.last_error = Some(error.clone());
                Some((error.clone(), format!(
                    "MoveUp could not save session state. Check access to the app data folder. {error}"
                )))
            }
        }
    };

    match result {
        Ok(()) => {
            if notification.is_some() {
                lifecycle.event(&format!("persistence_checkpoint_recovered at={}", at.to_rfc3339()));
            } else if periodic_checkpoint {
                lifecycle.event(&format!("persistence_checkpoint_succeeded at={}", at.to_rfc3339()));
            }
        }
        Err(error) => {
            log::error!("Persistence checkpoint failed: {error}");
            lifecycle.event(&format!("persistence_checkpoint_failed error={error}"));
        }
    }

    if let Some((key, body)) = notification {
        if should_notify(&key) {
            let title = if key == "recovered" { "MoveUp session saving recovered" } else { "MoveUp could not save session state" };
            crate::notify::show(title, &body);
        }
    }
}

static LAST_NOTIFICATION: Mutex<Option<(Instant, String)>> = Mutex::new(None);

fn should_notify(key: &str) -> bool {
    let now = Instant::now();
    let mut last = LAST_NOTIFICATION.lock().unwrap_or_else(|error| error.into_inner());
    let allowed = last.as_ref().map_or(true, |(at, previous)| {
        previous != key || now.duration_since(*at) >= FAILURE_NOTICE_INTERVAL
    });
    if allowed {
        *last = Some((now, key.to_string()));
    }
    allowed
}
