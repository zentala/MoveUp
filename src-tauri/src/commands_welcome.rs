//! commands_welcome.rs — Welcome popup window commands.

use tauri::{Manager, WebviewWindowBuilder, WebviewUrl};
use crate::config::AppConfig;

fn with_welcome_window<T>(
    existing: Option<T>,
    create: impl FnOnce() -> Result<T, String>,
    show: impl FnOnce(&T) -> Result<(), String>,
    after_show: impl FnOnce(&T) -> Result<(), String>,
) -> Result<T, String> {
    let window = match existing {
        Some(window) => window,
        None => create()?,
    };
    show(&window)?;
    after_show(&window)?;
    Ok(window)
}

fn create_welcome_window(app: &tauri::AppHandle) -> Result<tauri::WebviewWindow, String> {
    WebviewWindowBuilder::new(app, "welcome", WebviewUrl::App("welcome.html".into()))
        .title("MoveUp \u{2014} First run")
        .inner_size(480.0, 420.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .decorations(true)
        .build()
        .map_err(|e| format!("Failed to create welcome window: {}", e))
}

fn show_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| format!("Failed to show welcome window: {}", e))
}

/// Creates and shows the welcome popup without explicitly taking focus (startup path).
pub fn show_welcome_window(app: &tauri::AppHandle) -> Result<(), String> {
    with_welcome_window(
        app.get_webview_window("welcome"),
        || create_welcome_window(app),
        show_window,
        |_| Ok(()),
    ).map(|_| ())
}

fn activation_failure(show_succeeded: bool, focus_succeeded: bool) -> Option<&'static str> {
    if !show_succeeded { Some("welcome_main_window_show_failed") }
    else if !focus_succeeded { Some("welcome_main_window_focus_failed") }
    else { None }
}

fn require_preference_store<T>(dont_show_again: bool, store: Option<T>) -> Result<Option<T>, &'static str> {
    if dont_show_again && store.is_none() { Err("welcome_preference_store_unavailable") }
    else { Ok(store) }
}

fn complete_welcome_open(
    dont_show_again: bool,
    activate: impl FnOnce() -> Result<(), &'static str>,
    persist_preference: impl FnOnce() -> Result<(), &'static str>,
    close_welcome: impl FnOnce() -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    activate()?;
    if dont_show_again { persist_preference()?; }
    close_welcome()
}

/// Opens the existing main window from Welcome, then closes Welcome. If
/// `dont_show_again` is true, persists that preference before closing Welcome.
#[tauri::command]
pub fn dismiss_welcome(dont_show_again: bool, app: tauri::AppHandle) -> Result<(), String> {
    use crate::ui_diagnostics_types::WindowLabel;
    use tauri::Manager;

    complete_welcome_open(
        dont_show_again,
        || {
            let main = app.get_webview_window("main").ok_or("welcome_main_window_unavailable")?;
            let activation = crate::ui_diagnostics::show_and_focus(&app, &main, WindowLabel::Main);
            match activation_failure(activation.show.is_ok(), activation.focus.is_ok()) {
                Some(error) => Err(error),
                None => Ok(()),
            }
        },
        || {
            let store = require_preference_store(
                dont_show_again,
                app.try_state::<crate::app_store::AppStore<tauri::Wry>>(),
            )?;
            let Some(store) = store else { return Err("welcome_preference_store_unavailable"); };
            let mut config = AppConfig::load(store.inner());
            config.show_welcome_on_startup = false;
            config.save(store.inner()).map_err(|_| "welcome_preference_save_failed")
        },
        || match app.get_webview_window("welcome") {
            Some(win) => win.close().map_err(|_| "welcome_close_failed"),
            None => Ok(()),
        },
    ).map_err(str::to_string)
}

/// Re-opens the welcome popup (callable from settings).
#[tauri::command]
pub fn show_welcome(app: tauri::AppHandle) -> Result<(), String> {
    with_welcome_window(
        app.get_webview_window("welcome"),
        || create_welcome_window(&app),
        show_window,
        |window| window.set_focus().map_err(|e| format!("Failed to focus welcome window: {}", e)),
    ).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::{activation_failure, complete_welcome_open, require_preference_store, with_welcome_window};
    use std::cell::RefCell;

    #[test]
    fn open_moveup_requires_successful_show_and_focus() {
        assert_eq!(activation_failure(true, true), None);
        assert_eq!(activation_failure(false, true), Some("welcome_main_window_show_failed"));
        assert_eq!(activation_failure(true, false), Some("welcome_main_window_focus_failed"));
        assert_eq!(activation_failure(false, false), Some("welcome_main_window_show_failed"));
    }

    #[test]
    fn requested_welcome_preference_requires_the_existing_store() {
        assert_eq!(require_preference_store(true, None::<()>), Err("welcome_preference_store_unavailable"));
        assert_eq!(require_preference_store(false, None::<()>), Ok(None));
        assert_eq!(require_preference_store(true, Some(())), Ok(Some(())));
    }

    #[test]
    fn activation_failure_stops_before_preference_save_and_welcome_close() {
        let calls = RefCell::new(Vec::new());
        let result = complete_welcome_open(
            true,
            || { calls.borrow_mut().push("activate"); Err("welcome_main_window_focus_failed") },
            || { calls.borrow_mut().push("persist"); Ok(()) },
            || { calls.borrow_mut().push("close"); Ok(()) },
        );
        assert_eq!(result, Err("welcome_main_window_focus_failed"));
        assert_eq!(*calls.borrow(), ["activate"]);
    }

    #[test]
    fn missing_preference_store_keeps_welcome_open_and_retryable() {
        let calls = RefCell::new(Vec::new());
        let result = complete_welcome_open(
            true,
            || { calls.borrow_mut().push("activate"); Ok(()) },
            || {
                calls.borrow_mut().push("persist");
                require_preference_store(true, None::<()>).map(|_| ())
            },
            || { calls.borrow_mut().push("close"); Ok(()) },
        );
        assert_eq!(result, Err("welcome_preference_store_unavailable"));
        assert_eq!(*calls.borrow(), ["activate", "persist"]);
    }

    #[test]
    fn successful_open_persists_only_when_requested_then_closes_welcome() {
        let calls = RefCell::new(Vec::new());
        let result = complete_welcome_open(
            false,
            || { calls.borrow_mut().push("activate"); Ok(()) },
            || { calls.borrow_mut().push("persist"); Ok(()) },
            || { calls.borrow_mut().push("close"); Ok(()) },
        );
        assert_eq!(result, Ok(()));
        assert_eq!(*calls.borrow(), ["activate", "close"]);
    }

    #[test]
    fn startup_shows_an_existing_hidden_welcome_without_explicit_focus() {
        let calls = RefCell::new(Vec::new());
        let created = RefCell::new(false);
        let result = with_welcome_window(
            Some("existing welcome"),
            || { *created.borrow_mut() = true; Ok("new welcome") },
            |_| { calls.borrow_mut().push("show"); Ok(()) },
            |_| Ok(()),
        );
        assert_eq!(result, Ok("existing welcome"));
        assert!(!*created.borrow());
        assert_eq!(*calls.borrow(), ["show"]);
    }

    #[test]
    fn manual_reopen_shows_then_focuses_and_propagates_activation_errors() {
        let calls = RefCell::new(Vec::new());
        let result = with_welcome_window(
            Some("existing welcome"),
            || panic!("must reuse the configured window"),
            |_| { calls.borrow_mut().push("show"); Ok(()) },
            |_| { calls.borrow_mut().push("focus"); Ok(()) },
        );
        assert_eq!(result, Ok("existing welcome"));
        assert_eq!(*calls.borrow(), ["show", "focus"]);

        let result = with_welcome_window(
            Some("existing welcome"),
            || panic!("must reuse the configured window"),
            |_| Err("Failed to show welcome window: synthetic show failure".to_owned()),
            |_| panic!("focus must not run after show failure"),
        );
        assert_eq!(result, Err("Failed to show welcome window: synthetic show failure".to_owned()));

        let result = with_welcome_window(
            Some("existing welcome"),
            || panic!("must reuse the configured window"),
            |_| Ok(()),
            |_| Err("Failed to focus welcome window: synthetic focus failure".to_owned()),
        );
        assert_eq!(result, Err("Failed to focus welcome window: synthetic focus failure".to_owned()));
    }
}
