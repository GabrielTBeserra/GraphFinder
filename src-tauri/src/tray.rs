//! System tray integration and main window close behavior.
//!
//! Closing the main window can either quit the app or hide it to the system
//! tray, which keeps the embedded MCP HTTP server reachable for AI agents.
//! The choice is persisted in `settings.json` via `tauri-plugin-store`. While
//! no choice has been made, the frontend is asked once through the
//! `close-requested` event.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, Window, WindowEvent};
use tauri_plugin_store::StoreExt;

const MAIN_WINDOW: &str = "main";
const SETTINGS_STORE: &str = "settings.json";
const CLOSE_BEHAVIOR_KEY: &str = "closeBehavior";
const CLOSE_REQUESTED_EVENT: &str = "close-requested";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseBehavior {
    Tray,
    Quit,
}

#[derive(Default)]
pub struct TrayState {
    /// Set once the frontend listens for `close-requested`. Until then the
    /// close button keeps quitting, so a window whose UI has not loaded (or
    /// has crashed) can never become impossible to close.
    prompt_ready: AtomicBool,
    /// Set when the window was hidden to the tray, so the startup reveal
    /// fallback does not bring it back on screen.
    hidden_by_user: AtomicBool,
}

impl TrayState {
    pub fn hidden_by_user(&self) -> bool {
        self.hidden_by_user.load(Ordering::Relaxed)
    }
}

fn read_close_behavior<R: Runtime>(app: &AppHandle<R>) -> Option<CloseBehavior> {
    let store = app.store(SETTINGS_STORE).ok()?;
    store
        .get(CLOSE_BEHAVIOR_KEY)
        .and_then(|value| serde_json::from_value(value).ok())
}

fn write_close_behavior<R: Runtime>(
    app: &AppHandle<R>,
    behavior: CloseBehavior,
) -> Result<(), String> {
    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    let value = serde_json::to_value(behavior).map_err(|e| e.to_string())?;
    store.set(CLOSE_BEHAVIOR_KEY, value);
    store.save().map_err(|e| e.to_string())
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
        app.state::<TrayState>()
            .hidden_by_user
            .store(true, Ordering::Relaxed);
        let _ = window.hide();
    }
}

#[cfg(desktop)]
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let show = MenuItem::with_id(app, "show", "Show GraphFinder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("GraphFinder")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

pub fn handle_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    if window.label() != MAIN_WINDOW {
        return;
    }

    let app = window.app_handle();
    match read_close_behavior(app) {
        Some(CloseBehavior::Tray) => {
            api.prevent_close();
            hide_main_window(app);
        }
        Some(CloseBehavior::Quit) => {}
        None => {
            if app
                .state::<TrayState>()
                .prompt_ready
                .load(Ordering::Relaxed)
            {
                api.prevent_close();
                let _ = app.emit(CLOSE_REQUESTED_EVENT, ());
            }
        }
    }
}

#[tauri::command]
pub fn get_close_behavior(app: AppHandle) -> Option<CloseBehavior> {
    read_close_behavior(&app)
}

#[tauri::command]
pub fn set_close_behavior(app: AppHandle, behavior: CloseBehavior) -> Result<(), String> {
    write_close_behavior(&app, behavior)
}

/// Called by the frontend once its `close-requested` listener is registered.
#[tauri::command]
pub fn close_prompt_ready(state: State<'_, TrayState>) {
    state.prompt_ready.store(true, Ordering::Relaxed);
}

/// Persists the choice made in the first-close dialog and applies it.
#[tauri::command]
pub fn resolve_close_prompt(app: AppHandle, behavior: CloseBehavior) -> Result<(), String> {
    let persisted = write_close_behavior(&app, behavior);
    match behavior {
        CloseBehavior::Tray => hide_main_window(&app),
        CloseBehavior::Quit => app.exit(0),
    }
    persisted
}
