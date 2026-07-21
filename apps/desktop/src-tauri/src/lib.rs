mod config;
mod profiles;
mod webviews;

use std::sync::Mutex;

use tauri::{
    webview::WebviewBuilder, window::WindowBuilder, LogicalPosition, LogicalSize, Manager,
    WebviewUrl, WindowEvent,
};

use config::ConfigStore;

const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;

pub struct AppState(pub Mutex<ConfigStore>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            app.manage(AppState(Mutex::new(ConfigStore::load(config_path))));

            let window = WindowBuilder::new(app, webviews::MAIN_WINDOW)
                .title("Velix")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .min_inner_size(800.0, 600.0)
                .build()?;

            // The shell UI spans the whole window; platform webviews are placed
            // over its workspace area, so full-size auto_resize is correct here.
            window.add_child(
                WebviewBuilder::new(webviews::UI_LABEL, WebviewUrl::App(Default::default()))
                    .auto_resize(),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            )?;

            let handle = window.clone();
            window.on_window_event(move |event| {
                if matches!(event, WindowEvent::Resized(_)) {
                    webviews::relayout(&handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profiles::list_profiles,
            profiles::create_profile,
            profiles::delete_profile,
            webviews::open_webview,
            webviews::focus_webview,
            webviews::close_webview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
