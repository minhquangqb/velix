use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub plugin_id: String,
    pub name: String,
    /// Suppresses this account's native notifications; its badge keeps counting.
    #[serde(default)]
    pub muted: bool,
}

/// Last known geometry of the main window, restored on the next launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub width: f64,
    pub height: f64,
    /// `None` until the window has been moved, so the first launch stays centered.
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// Which palette the shell renders in. `System` follows the OS light/dark setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default)]
    pub theme: Theme,
    /// Suppresses native notifications; unread badges keep updating.
    #[serde(default)]
    pub quiet: bool,
    /// Closing the main window hides it to tray instead of quitting.
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub autostart: bool,
    /// Account list hidden to give the workspace more room; restored on launch.
    #[serde(default)]
    pub sidebar_collapsed: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            quiet: false,
            close_to_tray: true,
            autostart: false,
            sidebar_collapsed: false,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub window: WindowState,
    #[serde(default)]
    pub settings: Settings,
}

/// JSON-backed app config. All mutations go through this store so the file on
/// disk always mirrors in-memory state.
pub struct ConfigStore {
    path: PathBuf,
    pub config: Config,
}

impl ConfigStore {
    /// Loads the config from disk; a missing or unreadable file yields defaults.
    pub fn load(path: PathBuf) -> Self {
        let config = fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        Self { path, config }
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(&self.config).map_err(|e| e.to_string())?;
        fs::write(&self.path, raw).map_err(|e| e.to_string())
    }
}
