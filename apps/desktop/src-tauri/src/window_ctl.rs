//! Window controls for the frameless main window.
//!
//! The design uses `decorations: false`, so minimise/maximise/close and the
//! title-bar drag are the shell UI's job. They go through Rust rather than
//! `@tauri-apps/api/window` because the shell runs in a *child* webview: the
//! JS API would need window permissions granted to a surface that also hosts
//! remote content, and the drag region attribute is not wired up for children.

use tauri::{AppHandle, Emitter, Manager, Runtime, Window};

use crate::webviews;

/// Emitted whenever the maximised state may have changed, so the title bar can
/// swap the restore/maximise glyph without polling.
pub const WINDOW_EVENT: &str = "velix://window";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowStatus {
    pub maximized: bool,
}

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Result<Window<R>, String> {
    app.get_window(webviews::MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())
}

/// Pushes the current maximised state to the shell UI only — a broadcast would
/// also reach the remote platform webviews.
pub fn publish<R: Runtime>(window: &Window<R>) {
    let status = WindowStatus {
        maximized: window.is_maximized().unwrap_or(false),
    };
    let _ = window
        .app_handle()
        .emit_to(webviews::UI_LABEL, WINDOW_EVENT, status);
}

#[tauri::command]
pub async fn window_is_maximized<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    main_window(&app)?.is_maximized().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_minimize<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    main_window(&app)?.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn window_toggle_maximize<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let window = main_window(&app)?;
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Closes through the normal path, so `close_to_tray` still decides whether
/// this hides the window or quits.
#[tauri::command]
pub async fn window_close<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    main_window(&app)?.close().map_err(|e| e.to_string())
}

/// Begins an OS-driven window drag; called from the title bar's mousedown.
#[tauri::command]
pub async fn window_start_drag<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    main_window(&app)?
        .start_dragging()
        .map_err(|e| e.to_string())
}
