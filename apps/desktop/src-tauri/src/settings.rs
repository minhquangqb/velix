//! App settings exposed to the shell UI and the tray popup.

use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_autostart::ManagerExt;

use crate::config::Settings;
use crate::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.0.lock().unwrap().config.settings.clone()
}

fn update<R: Runtime, F: FnOnce(&mut Settings)>(
    app: &AppHandle<R>,
    apply: F,
) -> Result<Settings, String> {
    let state = app.state::<AppState>();
    let mut store = state.0.lock().unwrap();
    apply(&mut store.config.settings);
    store.save()?;
    Ok(store.config.settings.clone())
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
