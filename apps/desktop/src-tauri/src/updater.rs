//! Self-update against the signed release feed.
//!
//! The whole flow lives in Rust rather than the updater plugin's JS API: the
//! shell runs in a child webview, and keeping every IPC surface as one of our
//! own typed commands is what lets the ACL stay closed to the remote platform
//! webviews (see `bridge.rs`). Progress reaches the shell as `velix://update`
//! events, emitted to the `ui` webview only — never broadcast.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::webviews;

/// Carries every phase of the update flow to the shell.
pub const UPDATE_EVENT: &str = "velix://update";

/// Grace period before the first automatic check, so a launch spends its first
/// seconds opening the user's account rather than on a network round trip.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(20);
/// Quiet enough not to matter for an app that stays open for days.
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// Set while a download is running so a second click cannot start a parallel one.
static INSTALLING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// Version offered by the feed.
    pub version: String,
    /// Version running right now, so the UI can show both sides of the upgrade.
    pub current_version: String,
    /// Release notes, as written in the GitHub release body.
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// Mirrors the update flow as a discriminated union — `phase` is the tag on the
/// TypeScript side too, so the shell renders one branch per state.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum UpdateStatus {
    Checking,
    UpToDate,
    Available {
        info: UpdateInfo,
    },
    Downloading {
        received: u64,
        /// Absent when the server sends no content-length.
        total: Option<u64>,
    },
    Installing,
    Failed {
        message: String,
    },
}

fn publish<R: Runtime>(app: &AppHandle<R>, status: UpdateStatus) {
    let _ = app.emit_to(webviews::UI_LABEL, UPDATE_EVENT, status);
}

fn describe<R: Runtime>(app: &AppHandle<R>, update: &Update) -> UpdateInfo {
    UpdateInfo {
        version: update.version.clone(),
        current_version: app.package_info().version.to_string(),
        notes: update.body.clone(),
        date: update.date.map(|date| date.to_string()),
    }
}

async fn fetch<R: Runtime>(app: &AppHandle<R>) -> Result<Option<Update>, String> {
    app.updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())
}

/// The version this build reports — shown in Settings next to the check button.
#[tauri::command]
pub fn app_version<R: Runtime>(app: AppHandle<R>) -> String {
    app.package_info().version.to_string()
}

/// Asks the feed for a newer build. Also emits the outcome, so an automatic
/// check and a manual one leave the shell in exactly the same state.
#[tauri::command]
pub async fn check_update<R: Runtime>(app: AppHandle<R>) -> Result<Option<UpdateInfo>, String> {
    publish(&app, UpdateStatus::Checking);
    match fetch(&app).await {
        Ok(Some(update)) => {
            let info = describe(&app, &update);
            publish(&app, UpdateStatus::Available { info: info.clone() });
            Ok(Some(info))
        }
        Ok(None) => {
            publish(&app, UpdateStatus::UpToDate);
            Ok(None)
        }
        Err(message) => {
            publish(
                &app,
                UpdateStatus::Failed {
                    message: message.clone(),
                },
            );
            Err(message)
        }
    }
}

/// Downloads and installs the pending update, then restarts into it.
///
/// The update is re-fetched rather than parked in app state between the check
/// and the click: `Update` is not `Sync`, and one extra request costs far less
/// than holding it across an arbitrary user pause.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if INSTALLING.swap(true, Ordering::SeqCst) {
        return Err("đang tải bản cập nhật rồi".to_string());
    }
    let result = download(&app).await;
    INSTALLING.store(false, Ordering::SeqCst);

    match result {
        // Unreachable in practice: the installer replaces this process.
        Ok(()) => Ok(()),
        Err(message) => {
            publish(
                &app,
                UpdateStatus::Failed {
                    message: message.clone(),
                },
            );
            Err(message)
        }
    }
}

async fn download<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let Some(update) = fetch(app).await? else {
        publish(app, UpdateStatus::UpToDate);
        return Ok(());
    };

    let mut received: u64 = 0;
    // A chunk callback fires hundreds of times; only forward it when the whole
    // percent changes, so the progress bar animates without flooding the IPC.
    let mut last_percent = u64::MAX;
    let progress_app = app.clone();
    update
        .download_and_install(
            move |chunk: usize, total: Option<u64>| {
                received += chunk as u64;
                let percent = total
                    .filter(|t| *t > 0)
                    .map(|t| received * 100 / t)
                    .unwrap_or(u64::MAX);
                if percent != last_percent {
                    last_percent = percent;
                    publish(&progress_app, UpdateStatus::Downloading { received, total });
                }
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    publish(app, UpdateStatus::Installing);
    // Windows hands control to the NSIS installer, which needs this process
    // gone before it can overwrite the binary. `restart` never returns.
    app.restart()
}

/// Polls the feed in the background so the user learns about a new build without
/// asking. Deliberately silent: it only emits `Available`, and the shell shows
/// that as a dot on the settings button — no toast, no dialog, no modal.
///
/// Release builds only. A debug build's version never trails the feed, and the
/// request would just be noise while developing.
#[cfg(not(debug_assertions))]
pub fn start_auto_check<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK_DELAY);
        loop {
            tauri::async_runtime::block_on(async {
                if let Ok(Some(update)) = fetch(&app).await {
                    let info = describe(&app, &update);
                    publish(&app, UpdateStatus::Available { info });
                }
            });
            std::thread::sleep(CHECK_INTERVAL);
        }
    });
}

#[cfg(debug_assertions)]
pub fn start_auto_check<R: Runtime>(_app: AppHandle<R>) {
    let _ = (FIRST_CHECK_DELAY, CHECK_INTERVAL);
}
