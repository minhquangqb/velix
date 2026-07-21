use std::fs;
use std::path::PathBuf;

use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl,
    Window,
};

use crate::{bridge, AppState};

pub const UI_LABEL: &str = "ui";
pub const MAIN_WINDOW: &str = "main";

/// Chrome geometry, load-bearing: these numbers carve out the hole in the shell
/// UI that platform webviews are positioned into, so they must match the widths
/// in `PlatformRail.vue` / `AccountList.vue` and the height of `TitleBar.vue`.
pub const RAIL_WIDTH: f64 = 68.0;
pub const ACCOUNTS_WIDTH: f64 = 248.0;
pub const SIDEBAR_WIDTH: f64 = RAIL_WIDTH + ACCOUNTS_WIDTH;
pub const TOPBAR_HEIGHT: f64 = 44.0;

/// Only [a-zA-Z0-9-_] so ids stay safe as webview labels and path segments.
pub fn validate_id(id: &str) -> Result<(), String> {
    if !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err(format!("invalid id: {id:?}"))
    }
}

pub fn webview_label(plugin_id: &str, profile_id: &str) -> String {
    format!("{plugin_id}-{profile_id}")
}

pub fn profile_data_dir<R: Runtime>(
    app: &AppHandle<R>,
    plugin_id: &str,
    profile_id: &str,
) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join("profiles").join(plugin_id).join(profile_id))
}

fn workspace_bounds<R: Runtime>(
    window: &Window<R>,
) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let size = window
        .inner_size()?
        .to_logical::<f64>(window.scale_factor()?);
    Ok((
        LogicalPosition::new(SIDEBAR_WIDTH, TOPBAR_HEIGHT),
        LogicalSize::new(
            (size.width - SIDEBAR_WIDTH).max(0.0),
            (size.height - TOPBAR_HEIGHT).max(0.0),
        ),
    ))
}

/// Resizes the shell UI to fill the window and repositions platform webviews
/// into the workspace area. Both are explicit because `auto_resize` scales
/// proportionally from the size a webview was created at, which drifts the
/// fixed sidebar edge and is simply wrong when the window starts maximized.
pub fn relayout<R: Runtime>(window: &Window<R>) {
    let Ok((position, size)) = workspace_bounds(window) else {
        return;
    };
    let full = window
        .inner_size()
        .ok()
        .zip(window.scale_factor().ok())
        .map(|(size, scale)| size.to_logical::<f64>(scale));

    for webview in window.webviews() {
        if webview.label() == UI_LABEL {
            if let Some(full) = full {
                let _ = webview.set_position(LogicalPosition::new(0.0, 0.0));
                let _ = webview.set_size(LogicalSize::new(full.width, full.height));
            }
            continue;
        }
        let _ = webview.set_position(position);
        let _ = webview.set_size(size);
    }
}

fn show_only<R: Runtime>(window: &Window<R>, label: &str) -> Result<(), String> {
    for webview in window.webviews() {
        if webview.label() == UI_LABEL {
            continue;
        }
        if webview.label() == label {
            webview.show().map_err(|e| e.to_string())?;
        } else {
            let _ = webview.hide();
        }
    }
    // The user is now looking at it, so its notifications are no longer unread.
    bridge::clear_unread(window.app_handle(), label);
    Ok(())
}

fn main_window<R: Runtime>(app: &AppHandle<R>) -> Result<Window<R>, String> {
    app.get_window(MAIN_WINDOW)
        .ok_or_else(|| "main window not found".to_string())
}

pub fn close_by_label<R: Runtime>(app: &AppHandle<R>, label: &str) -> Result<(), String> {
    let window = main_window(app)?;
    if let Some(webview) = window.webviews().into_iter().find(|w| w.label() == label) {
        webview.close().map_err(|e| e.to_string())?;
    }
    bridge::forget(app, label);
    Ok(())
}

// Webview-touching commands must be async: a sync command would block the main
// thread while webview creation is dispatched to it — deadlock on Windows (wry#583).
#[tauri::command]
pub async fn open_webview<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    plugin_id: String,
    profile_id: String,
    url: String,
) -> Result<String, String> {
    validate_id(&plugin_id)?;
    validate_id(&profile_id)?;

    let known = state
        .0
        .lock()
        .unwrap()
        .config
        .profiles
        .iter()
        .any(|p| p.id == profile_id && p.plugin_id == plugin_id);
    if !known {
        return Err(format!("unknown profile: {plugin_id}/{profile_id}"));
    }

    let window = main_window(&app)?;
    let label = webview_label(&plugin_id, &profile_id);

    if !window.webviews().iter().any(|w| w.label() == label) {
        let parsed = url
            .parse::<tauri::Url>()
            .map_err(|e| format!("invalid url: {e}"))?;
        let data_dir = profile_data_dir(&app, &plugin_id, &profile_id)?;
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        // Must precede the webview: the page starts calling the bridge as soon
        // as it loads, and a capability added later would not cover those calls.
        bridge::grant_remote_access(&app, &label, &parsed)?;

        let (position, size) = workspace_bounds(&window).map_err(|e| e.to_string())?;
        window
            .add_child(
                WebviewBuilder::new(&label, WebviewUrl::External(parsed))
                    .data_directory(data_dir)
                    .initialization_script(bridge::INJECT_SCRIPT),
                position,
                size,
            )
            .map_err(|e| e.to_string())?;
    }

    show_only(&window, &label)?;
    Ok(label)
}

#[tauri::command]
pub async fn focus_webview<R: Runtime>(app: AppHandle<R>, label: String) -> Result<(), String> {
    let window = main_window(&app)?;
    if !window.webviews().iter().any(|w| w.label() == label) {
        return Err(format!("webview not found: {label}"));
    }
    show_only(&window, &label)
}

/// Hides every platform webview without closing it, so the shell can draw a
/// full-window surface (settings, wizards) over the workspace area. Native
/// child webviews always paint above the shell UI, so hiding is the only way to
/// get them out of the way; the sessions they hold stay alive.
#[tauri::command]
pub async fn hide_webviews<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let window = main_window(&app)?;
    for webview in window.webviews() {
        if webview.label() != UI_LABEL {
            let _ = webview.hide();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn close_webview<R: Runtime>(app: AppHandle<R>, label: String) -> Result<(), String> {
    if label == UI_LABEL {
        return Err("cannot close the ui webview".to_string());
    }
    close_by_label(&app, &label)
}
