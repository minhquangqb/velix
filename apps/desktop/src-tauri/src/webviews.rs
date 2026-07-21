use std::fs;
use std::path::PathBuf;

use tauri::{
    webview::WebviewBuilder, AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl,
    Window,
};

use crate::AppState;

pub const UI_LABEL: &str = "ui";
pub const MAIN_WINDOW: &str = "main";
pub const SIDEBAR_WIDTH: f64 = 260.0;

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
        LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
        LogicalSize::new((size.width - SIDEBAR_WIDTH).max(0.0), size.height),
    ))
}

/// Repositions platform webviews into the workspace area. Needed because
/// `auto_resize` scales proportionally and would drift the fixed sidebar edge.
pub fn relayout<R: Runtime>(window: &Window<R>) {
    let Ok((position, size)) = workspace_bounds(window) else {
        return;
    };
    for webview in window.webviews() {
        if webview.label() == UI_LABEL {
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
    Ok(())
}

#[tauri::command]
pub fn open_webview<R: Runtime>(
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

        let (position, size) = workspace_bounds(&window).map_err(|e| e.to_string())?;
        window
            .add_child(
                WebviewBuilder::new(&label, WebviewUrl::External(parsed)).data_directory(data_dir),
                position,
                size,
            )
            .map_err(|e| e.to_string())?;
    }

    show_only(&window, &label)?;
    Ok(label)
}

#[tauri::command]
pub fn focus_webview<R: Runtime>(app: AppHandle<R>, label: String) -> Result<(), String> {
    let window = main_window(&app)?;
    if !window.webviews().iter().any(|w| w.label() == label) {
        return Err(format!("webview not found: {label}"));
    }
    show_only(&window, &label)
}

#[tauri::command]
pub fn close_webview<R: Runtime>(app: AppHandle<R>, label: String) -> Result<(), String> {
    if label == UI_LABEL {
        return Err("cannot close the ui webview".to_string());
    }
    close_by_label(&app, &label)
}
