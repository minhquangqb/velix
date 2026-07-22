mod bridge;
mod config;
mod profiles;
mod settings;
mod tray;
mod webviews;
mod window_ctl;
mod window_state;

use std::sync::Mutex;

use tauri::{
    webview::WebviewBuilder, window::WindowBuilder, LogicalPosition, LogicalSize, Manager,
    RunEvent, WebviewUrl, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

use config::ConfigStore;

pub struct AppState(pub Mutex<ConfigStore>);

/// Passed by the autostart entry so a boot-time launch goes straight to tray.
const AUTOSTART_FLAG: &str = "--autostart";

fn launched_by_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_FLAG)
}

/// `Ctrl+Shift+V` brings the window back — the shortcut the tray popup advertises.
/// In debug builds `Ctrl+Shift+I` opens devtools on the platform webview on
/// screen; it has to be a global shortcut because that webview owns the
/// keyboard while it is focused, so a key handler in the shell never sees it.
fn global_shortcut_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let toggle = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyV);
    let devtools = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyI);
    tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts([toggle, devtools])
        .expect("invalid global shortcut")
        .with_handler(move |app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if shortcut == &toggle {
                tray::show_main(app);
            } else if shortcut == &devtools {
                #[cfg(debug_assertions)]
                webviews::open_devtools(app);
            }
        })
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_FLAG]),
        ))
        .plugin(global_shortcut_plugin())
        .plugin(bridge::init())
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            app.manage(AppState(Mutex::new(ConfigStore::load(config_path))));
            app.manage(bridge::BridgeState::default());
            app.manage(webviews::ActiveWebview::default());
            app.manage(tray::TrayState::default());
            settings::sync_autostart(app.handle());

            let saved = {
                let state = app.state::<AppState>();
                let store = state.0.lock().unwrap();
                store.config.window.clone()
            };

            // Frameless: the design draws its own title bar. Window controls and
            // the drag region live in `window_ctl.rs` / `TitleBar.vue`.
            let mut builder = WindowBuilder::new(app, webviews::MAIN_WINDOW)
                .title("Velix")
                .decorations(false)
                .inner_size(saved.width, saved.height)
                .min_inner_size(860.0, 640.0)
                .visible(!launched_by_autostart());
            builder = match (saved.x, saved.y) {
                (Some(x), Some(y)) => builder.position(x, y),
                _ => builder.center(),
            };
            let window = builder.build()?;
            if saved.maximized {
                window.maximize()?;
            }

            // Size from the window's actual bounds, not the saved ones: after a
            // maximize the two differ, and the shell UI must span the window.
            let inner = window
                .inner_size()?
                .to_logical::<f64>(window.scale_factor()?);
            window.add_child(
                WebviewBuilder::new(webviews::UI_LABEL, WebviewUrl::App(Default::default())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(inner.width, inner.height),
            )?;

            tray::build_popup(app.handle())?;
            tray::build(app.handle())?;

            let handle = window.clone();
            window.on_window_event(move |event| match event {
                WindowEvent::Resized(_) => {
                    webviews::relayout(&handle);
                    window_state::capture(&handle);
                    // Maximising arrives as a resize; the title bar glyph follows.
                    window_ctl::publish(&handle);
                }
                WindowEvent::Moved(_) => window_state::capture(&handle),
                WindowEvent::CloseRequested { api, .. } => {
                    let close_to_tray = {
                        let state = handle.app_handle().state::<AppState>();
                        let store = state.0.lock().unwrap();
                        store.config.settings.close_to_tray
                    };
                    // Hiding keeps every platform session alive; quitting is an
                    // explicit action from the tray popup.
                    if close_to_tray && !tray::is_quitting() {
                        api.prevent_close();
                        window_state::capture(&handle);
                        // Tell the active page it is now backgrounded, so it
                        // notifies for messages that arrive while in the tray.
                        webviews::set_active_hidden(handle.app_handle(), true);
                        let _ = handle.hide();
                    }
                }
                _ => {}
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profiles::list_profiles,
            profiles::create_profile,
            profiles::rename_profile,
            profiles::set_profile_muted,
            profiles::delete_profile,
            webviews::open_webview,
            webviews::focus_webview,
            webviews::hide_webviews,
            webviews::close_webview,
            webviews::webview_back,
            webviews::webview_forward,
            webviews::webview_reload,
            webviews::webview_set_zoom,
            window_ctl::window_is_maximized,
            window_ctl::window_minimize,
            window_ctl::window_toggle_maximize,
            window_ctl::window_close,
            window_ctl::window_start_drag,
            settings::get_settings,
            settings::set_theme,
            settings::set_quiet,
            settings::set_close_to_tray,
            settings::set_autostart,
            settings::set_sidebar_collapsed,
            tray::show_main_window,
            tray::open_settings,
            tray::open_account,
            tray::hide_tray_popup,
            tray::list_unread,
            tray::quit_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            window_state::persist(handle);
        }
    });
}
