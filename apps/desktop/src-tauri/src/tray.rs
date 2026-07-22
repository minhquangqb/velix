//! System tray icon plus the custom popup that replaces the native tray menu.
//!
//! The design calls for a toggle, keyboard hints and per-account unread badges,
//! none of which a native menu can render, so the menu is a real (frameless,
//! always-on-top) window showing the same Vue app at `?view=tray`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::image::Image;
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, Runtime, WebviewUrl, WindowEvent,
};

use crate::{bridge, webviews};

pub const POPUP_WINDOW: &str = "tray";
pub const TRAY_ID: &str = "velix-tray";

/// Panel geometry. Kept in sync with `apps/desktop/src/tray/TrayMenu.vue`; the
/// window is sized in Rust because Rust owns the unread list that drives height.
const PANEL_WIDTH: f64 = 280.0;
/// Transparent gutter around the panel so its CSS drop shadow is not clipped.
const SHADOW_MARGIN: f64 = 16.0;
const PANEL_PADDING: f64 = 6.0;
const ROW_OPEN: f64 = 40.0;
const ROW_ACCOUNT: f64 = 38.0;
const ROW_ACTION: f64 = 34.0;
const ROW_SECTION_LABEL: f64 = 23.0;
const DIVIDER: f64 = 11.0;
/// Beyond this the account list scrolls instead of growing the window.
const MAX_VISIBLE_ACCOUNTS: usize = 6;
/// Gap between the tray icon and the panel edge.
const ANCHOR_GAP: f64 = 8.0;

/// Set while quitting so the main window's close handler stops hiding to tray.
static QUITTING: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct TrayState {
    /// Physical rect of the tray icon from the last click, used to anchor the popup.
    anchor: Mutex<Option<(f64, f64, f64, f64)>>,
}

pub fn is_quitting() -> bool {
    QUITTING.load(Ordering::SeqCst)
}

fn panel_height(accounts: usize) -> f64 {
    let mut height = PANEL_PADDING * 2.0 + ROW_OPEN + DIVIDER;
    if accounts > 0 {
        let visible = accounts.min(MAX_VISIBLE_ACCOUNTS) as f64;
        height += ROW_SECTION_LABEL + visible * ROW_ACCOUNT + DIVIDER;
    }
    // Quiet toggle + Settings, then a divider before Quit.
    height += ROW_ACTION * 2.0 + DIVIDER + ROW_ACTION;
    height
}

/// Draws a red dot into the icon's lower-right corner as the unread indicator.
/// Tray icons are far too small for legible digits — the popup carries counts.
fn with_unread_dot(base: &Image<'_>) -> Image<'static> {
    let (width, height) = (base.width(), base.height());
    let mut rgba = base.rgba().to_vec();

    let radius = (width.min(height) as f64 / 3.2).max(3.0);
    let center_x = width as f64 - radius - 1.0;
    let center_y = height as f64 - radius - 1.0;
    let (dot_r, dot_g, dot_b) = (242.0, 85.0, 90.0); // #F2555A, matches the design

    for y in 0..height {
        for x in 0..width {
            let dx = x as f64 + 0.5 - center_x;
            let dy = y as f64 + 0.5 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            // Feather the last pixel so the dot is not visibly jagged.
            let coverage = (radius - distance).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }

            let index = ((y * width + x) * 4) as usize;
            let blend = |dst: u8, src: f64| -> u8 {
                (src * coverage + dst as f64 * (1.0 - coverage)).round() as u8
            };
            rgba[index] = blend(rgba[index], dot_r);
            rgba[index + 1] = blend(rgba[index + 1], dot_g);
            rgba[index + 2] = blend(rgba[index + 2], dot_b);
            rgba[index + 3] = blend(rgba[index + 3], 255.0);
        }
    }

    Image::new_owned(rgba, width, height)
}

pub fn set_unread_total<R: Runtime>(app: &AppHandle<R>, total: u32) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let Some(base) = app.default_window_icon().cloned() else {
        return;
    };

    let icon = if total > 0 {
        with_unread_dot(&base)
    } else {
        base
    };
    let _ = tray.set_icon(Some(icon));
    let _ = tray.set_tooltip(Some(&if total > 0 {
        format!("Velix — {total} tin chưa đọc")
    } else {
        "Velix".to_string()
    }));
}

/// Positions the popup above the tray icon, right-aligned to it, and clamped to
/// the monitor's work area. Falls back to below the icon when the taskbar is on
/// top, which is where a naive "always above" would push it off-screen.
fn place_popup<R: Runtime>(app: &AppHandle<R>, window: &tauri::Window<R>) -> tauri::Result<()> {
    let accounts = bridge::unread_entries(app).len();
    let scale = window.scale_factor()?;
    let logical = LogicalSize::new(
        PANEL_WIDTH + SHADOW_MARGIN * 2.0,
        panel_height(accounts) + SHADOW_MARGIN * 2.0,
    );
    window.set_size(logical)?;

    let size = logical.to_physical::<f64>(scale);
    let margin = SHADOW_MARGIN * scale;

    let state = app.state::<TrayState>();
    let anchor = *state.anchor.lock().unwrap();
    let Some((tray_x, tray_y, tray_w, _tray_h)) = anchor else {
        return Ok(());
    };

    let mut x = tray_x + tray_w - size.width + margin;
    let mut y = tray_y - ANCHOR_GAP * scale - size.height + margin;

    if let Ok(Some(monitor)) = app.monitor_from_point(tray_x, tray_y) {
        let pos = monitor.position();
        let msize = monitor.size();
        let (left, top) = (pos.x as f64, pos.y as f64);
        let right = left + msize.width as f64;
        let bottom = top + msize.height as f64;

        x = x.clamp(
            left - margin,
            (right - size.width + margin).max(left - margin),
        );
        // Tray on a top-edge taskbar: drop the panel below the icon instead.
        if y < top {
            y = tray_y + _tray_h + ANCHOR_GAP * scale - margin;
        }
        y = y.min(bottom - size.height + margin);
    }

    window.set_position(PhysicalPosition::new(x, y))
}

fn popup<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::WebviewWindow<R>> {
    app.get_webview_window(POPUP_WINDOW)
}

async fn toggle_popup<R: Runtime>(app: AppHandle<R>) {
    let Some(window) = popup(&app) else { return };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    let parent = window.as_ref().window();
    let _ = place_popup(&app, &parent);
    let _ = window.show();
    let _ = window.set_focus();
    bridge::publish(&app);
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Velix")
        // No menu: the popup window below is the menu.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            let (TrayIconEvent::Click { rect, .. } | TrayIconEvent::DoubleClick { rect, .. }) =
                event
            else {
                return;
            };
            let app = tray.app_handle().clone();
            let scale = app
                .get_webview_window(POPUP_WINDOW)
                .and_then(|w| w.scale_factor().ok())
                .unwrap_or(1.0);
            let position = rect.position.to_physical::<f64>(scale);
            let size = rect.size.to_physical::<f64>(scale);
            *app.state::<TrayState>().anchor.lock().unwrap() =
                Some((position.x, position.y, size.width, size.height));

            // Showing a webview must not block the event thread (wry#583).
            tauri::async_runtime::spawn(toggle_popup(app));
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)
}

/// Creates the popup window up front, hidden, so the first tray click does not
/// pay webview startup cost.
pub fn build_popup<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = tauri::WebviewWindowBuilder::new(
        app,
        POPUP_WINDOW,
        WebviewUrl::App("index.html?view=tray".into()),
    )
    .title("Velix")
    .inner_size(
        PANEL_WIDTH + SHADOW_MARGIN * 2.0,
        panel_height(0) + SHADOW_MARGIN * 2.0,
    )
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .visible(false)
    .build()?;

    // Dismiss like a real menu: any click elsewhere takes focus away.
    let handle = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            let _ = handle.hide();
        }
    });
    Ok(())
}

pub fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_window(webviews::MAIN_WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        // Back on screen: the active page is foreground again.
        webviews::set_active_hidden(app, false);
    }
}

#[tauri::command]
pub async fn show_main_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = popup(&app) {
        let _ = window.hide();
    }
    show_main(&app);
    Ok(())
}

/// Emitted to the shell UI to switch its view; the tray popup is a separate
/// window and cannot reach the shell's store any other way.
pub const NAVIGATE_EVENT: &str = "velix://navigate";

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Navigate {
    pub view: String,
    /// Set for `view: "account"`, naming the account to bring forward.
    pub profile_id: Option<String>,
}

fn navigate<R: Runtime>(
    app: &AppHandle<R>,
    view: &str,
    profile_id: Option<String>,
) -> Result<(), String> {
    if let Some(window) = popup(app) {
        let _ = window.hide();
    }
    show_main(app);
    app.emit_to(
        webviews::UI_LABEL,
        NAVIGATE_EVENT,
        Navigate {
            view: view.to_string(),
            profile_id,
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_settings<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    navigate(&app, "settings", None)
}

/// Brings an account forward from the tray popup. The shell drives the actual
/// webview switch so its stores stay in step — showing the webview from here
/// would leave the sidebar highlighting the wrong account.
#[tauri::command]
pub async fn open_account<R: Runtime>(app: AppHandle<R>, profile_id: String) -> Result<(), String> {
    navigate(&app, "account", Some(profile_id))
}

#[tauri::command]
pub async fn hide_tray_popup<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = popup(&app) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_unread<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<bridge::UnreadEntry>, String> {
    Ok(bridge::unread_entries(&app))
}

#[tauri::command]
pub async fn quit_app<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    QUITTING.store(true, Ordering::SeqCst);
    crate::window_state::persist(&app);
    app.exit(0);
    Ok(())
}
