use tauri::{
    webview::WebviewBuilder, window::WindowBuilder, LogicalPosition, LogicalSize, WebviewUrl,
};

const SIDEBAR_WIDTH: f64 = 260.0;
const WINDOW_WIDTH: f64 = 1200.0;
const WINDOW_HEIGHT: f64 = 800.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = WindowBuilder::new(app, "main")
                .title("Velix")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .min_inner_size(800.0, 600.0)
                .build()?;

            window.add_child(
                WebviewBuilder::new("ui", WebviewUrl::App(Default::default())).auto_resize(),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_WIDTH, WINDOW_HEIGHT),
            )?;

            // Temporary spike until the workspace UI lands: load Messenger by default
            // so multi-webview + performance can be measured on Windows/WebView2.
            window.add_child(
                WebviewBuilder::new(
                    "messenger",
                    WebviewUrl::External("https://www.messenger.com".parse()?),
                )
                .auto_resize(),
                LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
                LogicalSize::new(WINDOW_WIDTH - SIDEBAR_WIDTH, WINDOW_HEIGHT),
            )?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
