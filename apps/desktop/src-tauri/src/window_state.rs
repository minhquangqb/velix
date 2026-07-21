//! Remembers the main window's geometry across runs.
//!
//! Geometry is captured in memory on every move/resize and only written to disk
//! when the app quits — the window can be hidden to tray for hours, and polling
//! it at exit would read a stale or off-screen position.

use tauri::{AppHandle, Manager, Runtime, Window};

use crate::AppState;

/// Copies the window's current geometry into the in-memory config.
pub fn capture<R: Runtime>(window: &Window<R>) {
    let maximized = window.is_maximized().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);

    let state = window.app_handle().state::<AppState>();
    let mut store = state.0.lock().unwrap();
    store.config.window.maximized = maximized;

    // A maximized or minimized window reports its transient bounds, not the
    // size to restore to, so keep the last normal geometry instead.
    if maximized || minimized {
        return;
    }
    let (Ok(size), Ok(position), Ok(scale)) = (
        window.inner_size(),
        window.outer_position(),
        window.scale_factor(),
    ) else {
        return;
    };
    let size = size.to_logical::<f64>(scale);
    let position = position.to_logical::<f64>(scale);

    store.config.window.width = size.width;
    store.config.window.height = size.height;
    store.config.window.x = Some(position.x);
    store.config.window.y = Some(position.y);
}

/// Captures the main window's geometry and flushes the config to disk.
pub fn persist<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_window(crate::webviews::MAIN_WINDOW) {
        capture(&window);
    }
    let state = app.state::<AppState>();
    let store = state.0.lock().unwrap();
    let _ = store.save();
}
