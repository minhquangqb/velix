//! App settings exposed to the shell UI and the tray popup.

use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_autostart::ManagerExt;

use crate::{tray, webviews};

/// Emitted after any settings change so every local surface (shell UI and tray
/// popup) re-themes and re-renders its toggles without polling.
pub const SETTINGS_EVENT: &str = "velix://settings";

use crate::config::{Settings, Theme};
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.0.lock().unwrap().config.settings.clone()
}

fn update<R: Runtime, F: FnOnce(&mut Settings)>(
    app: &AppHandle<R>,
    apply: F,
) -> Result<Settings, String> {
    let settings = {
        let state = app.state::<AppState>();
        let mut store = state.0.lock().unwrap();
        apply(&mut store.config.settings);
        store.save()?;
        store.config.settings.clone()
    };

    // Targeted, not broadcast: remote platform webviews must not see settings.
    let _ = app.emit_to(webviews::UI_LABEL, SETTINGS_EVENT, &settings);
    let _ = app.emit_to(tray::POPUP_WINDOW, SETTINGS_EVENT, &settings);
    Ok(settings)
}

#[tauri::command]
pub async fn set_theme<R: Runtime>(app: AppHandle<R>, theme: Theme) -> Result<Settings, String> {
    update(&app, |s| s.theme = theme)
}

#[tauri::command]
pub async fn set_quiet<R: Runtime>(app: AppHandle<R>, quiet: bool) -> Result<Settings, String> {
    update(&app, |s| s.quiet = quiet)
}

#[tauri::command]
pub async fn set_close_to_tray<R: Runtime>(
    app: AppHandle<R>,
    close_to_tray: bool,
) -> Result<Settings, String> {
    update(&app, |s| s.close_to_tray = close_to_tray)
}

#[tauri::command]
pub async fn set_autostart<R: Runtime>(
    app: AppHandle<R>,
    autostart: bool,
) -> Result<Settings, String> {
    let manager = app.autolaunch();
    if autostart {
        manager.enable().map_err(|e| e.to_string())?;
    } else {
        manager.disable().map_err(|e| e.to_string())?;
    }
    update(&app, |s| s.autostart = autostart)
}

/// Realigns the OS autostart entry with the stored setting at boot — the user
/// may have removed it outside the app.
pub fn sync_autostart<R: Runtime>(app: &AppHandle<R>) {
    let wanted = {
        let state = app.state::<AppState>();
        let autostart = state.0.lock().unwrap().config.settings.autostart;
        autostart
    };

    let manager = app.autolaunch();
    let actual = manager.is_enabled().unwrap_or(false);
    if wanted && !actual {
        let _ = manager.enable();
    } else if !wanted && actual {
        let _ = manager.disable();
    }
}
