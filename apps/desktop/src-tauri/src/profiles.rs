use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Runtime, State};

use crate::config::Profile;
use crate::{webviews, AppState};

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Vec<Profile> {
    state.0.lock().unwrap().config.profiles.clone()
}

#[tauri::command]
pub fn create_profile(
    state: State<'_, AppState>,
    plugin_id: String,
    name: String,
) -> Result<Profile, String> {
    webviews::validate_id(&plugin_id)?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("profile name must not be empty".to_string());
    }

    let id = format!(
        "p{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis()
    );
    let profile = Profile {
        id,
        plugin_id,
        name,
        muted: false,
    };

    let mut store = state.0.lock().unwrap();
    store.config.profiles.push(profile.clone());
    store.save()?;
    Ok(profile)
}

/// Applies `edit` to the profile with `id` and persists the config.
fn update<F: FnOnce(&mut Profile)>(
    state: &State<'_, AppState>,
    id: &str,
    edit: F,
) -> Result<Profile, String> {
    let mut store = state.0.lock().unwrap();
    let Some(profile) = store.config.profiles.iter_mut().find(|p| p.id == id) else {
        return Err(format!("profile not found: {id}"));
    };
    edit(profile);
    let updated = profile.clone();
    store.save()?;
    Ok(updated)
}

#[tauri::command]
pub fn rename_profile(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Profile, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("profile name must not be empty".to_string());
    }
    update(&state, &id, |p| p.name = name)
}

#[tauri::command]
pub fn set_profile_muted(
    state: State<'_, AppState>,
    id: String,
    muted: bool,
) -> Result<Profile, String> {
    update(&state, &id, |p| p.muted = muted)
}

// Async because it may close a webview (see the note in webviews.rs).
#[tauri::command]
pub async fn delete_profile<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let removed = {
        let mut store = state.0.lock().unwrap();
        let Some(index) = store.config.profiles.iter().position(|p| p.id == id) else {
            return Err(format!("profile not found: {id}"));
        };
        let removed = store.config.profiles.remove(index);
        // Otherwise the next launch would try to reopen an account that is gone.
        if store.config.last_profile_id.as_deref() == Some(id.as_str()) {
            store.config.last_profile_id = None;
        }
        store.save()?;
        removed
    };

    let label = webviews::webview_label(&removed.plugin_id, &removed.id);
    let _ = webviews::close_by_label(&app, &label);

    // Best effort: WebView2 may still hold locks right after close; leftover
    // dirs are harmless since the profile is already gone from the config.
    if let Ok(dir) = webviews::profile_data_dir(&app, &removed.plugin_id, &removed.id) {
        let _ = fs::remove_dir_all(dir);
    }
    Ok(())
}
