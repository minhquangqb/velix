//! Bridge between remote platform pages and native notifications.
//!
//! Remote webviews get IPC access to exactly two commands, `notify` and
//! `set_badge`, exposed as the inlined `velix` plugin. Access is granted per
//! webview at runtime (see [`grant_remote_access`]) and scoped to that
//! webview's own origin, so one platform page cannot act for another.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use serde::Serialize;
use tauri::ipc::CapabilityBuilder;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Emitter, Manager, Runtime, Webview};
use tauri_plugin_notification::NotificationExt;

use crate::config::Profile;
use crate::{tray, webviews, AppState};

/// Emitted to the shell UI and the tray popup whenever unread counts change.
pub const UNREAD_EVENT: &str = "velix://unread";

/// Polyfill injected into every platform webview.
///
/// Runs on every top-level navigation (WebView2 re-injects per document), so it
/// guards against double-installing. `Notification` and `navigator.setAppBadge`
/// are forwarded to Rust; `requestPermission` resolves to "granted" so pages
/// never show the in-page permission prompt they cannot satisfy here.
pub const INJECT_SCRIPT: &str = r#"
(function () {
  if (window.__VELIX_BRIDGE__) return;
  Object.defineProperty(window, '__VELIX_BRIDGE__', { value: true });

  function invoke(cmd, args) {
    try {
      var internals = window.__TAURI_INTERNALS__;
      if (internals && internals.invoke) {
        return Promise.resolve(internals.invoke('plugin:velix|' + cmd, args)).catch(function () {});
      }
    } catch (e) {}
    return Promise.resolve();
  }

  function send(title, options) {
    options = options || {};
    invoke('notify', {
      title: title == null ? '' : String(title),
      body: options.body == null ? '' : String(options.body)
    });
  }

  function VelixNotification(title, options) {
    options = options || {};
    this.title = title;
    this.body = options.body || '';
    this.tag = options.tag || '';
    this.icon = options.icon || '';
    this.data = options.data;
    this.onclick = null;
    this.onclose = null;
    this.onerror = null;
    this.onshow = null;
    send(title, options);
  }
  VelixNotification.prototype.close = function () {};
  VelixNotification.prototype.addEventListener = function () {};
  VelixNotification.prototype.removeEventListener = function () {};
  VelixNotification.prototype.dispatchEvent = function () { return false; };
  VelixNotification.permission = 'granted';
  VelixNotification.maxActions = 2;
  VelixNotification.requestPermission = function (cb) {
    if (typeof cb === 'function') { try { cb('granted'); } catch (e) {} }
    return Promise.resolve('granted');
  };

  try {
    Object.defineProperty(window, 'Notification', {
      configurable: true,
      writable: true,
      value: VelixNotification
    });
  } catch (e) {}

  // Service-worker notifications are the other common path (Messenger uses it).
  try {
    if (window.ServiceWorkerRegistration && window.ServiceWorkerRegistration.prototype) {
      window.ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
        send(title, options);
        return Promise.resolve();
      };
      window.ServiceWorkerRegistration.prototype.getNotifications = function () {
        return Promise.resolve([]);
      };
    }
  } catch (e) {}

  try {
    navigator.setAppBadge = function (count) {
      var n = Number(count);
      invoke('set_badge', { count: isFinite(n) && n > 0 ? Math.floor(n) : 0 });
      return Promise.resolve();
    };
    navigator.clearAppBadge = function () {
      invoke('set_badge', { count: 0 });
      return Promise.resolve();
    };
  } catch (e) {}
})();
"#;

#[derive(Default)]
pub struct BridgeState {
    /// webview label -> unread count
    unread: Mutex<HashMap<String, u32>>,
    /// webview labels already granted a runtime capability
    granted: Mutex<HashSet<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadEntry {
    pub label: String,
    pub plugin_id: String,
    pub profile_id: String,
    pub name: String,
    pub count: u32,
}

fn profile_for_label(profiles: &[Profile], label: &str) -> Option<Profile> {
    profiles
        .iter()
        .find(|p| webviews::webview_label(&p.plugin_id, &p.id) == label)
        .cloned()
}

/// Unread counts joined with profile metadata, highest first so the tray popup
/// leads with the noisiest account.
pub fn unread_entries<R: Runtime>(app: &AppHandle<R>) -> Vec<UnreadEntry> {
    let state = app.state::<AppState>();
    let profiles = state.0.lock().unwrap().config.profiles.clone();
    let unread = app.state::<BridgeState>();
    let unread = unread.unread.lock().unwrap();

    let mut entries: Vec<UnreadEntry> = unread
        .iter()
        .filter(|(_, &count)| count > 0)
        .filter_map(|(label, &count)| {
            let profile = profile_for_label(&profiles, label)?;
            Some(UnreadEntry {
                label: label.clone(),
                plugin_id: profile.plugin_id,
                profile_id: profile.id,
                name: profile.name,
                count,
            })
        })
        .collect();
    entries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    entries
}

/// Refreshes the tray badge and pushes counts to the shell UI and tray popup.
pub fn publish<R: Runtime>(app: &AppHandle<R>) {
    let entries = unread_entries(app);
    let total: u32 = entries.iter().map(|e| e.count).sum();
    tray::set_unread_total(app, total);

    // Targeted rather than broadcast: a plain `emit` would also reach the remote
    // platform webviews, handing them the user's account list.
    let _ = app.emit_to(webviews::UI_LABEL, UNREAD_EVENT, &entries);
    let _ = app.emit_to(tray::POPUP_WINDOW, UNREAD_EVENT, &entries);
}

fn set_unread<R: Runtime>(app: &AppHandle<R>, label: &str, count: u32) {
    let state = app.state::<BridgeState>();
    let mut unread = state.unread.lock().unwrap();
    if count == 0 {
        unread.remove(label);
    } else {
        unread.insert(label.to_string(), count);
    }
}

fn bump_unread<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let state = app.state::<BridgeState>();
    let mut unread = state.unread.lock().unwrap();
    *unread.entry(label.to_string()).or_insert(0) += 1;
}

/// Clears a webview's unread count — called when the user actually looks at it.
pub fn clear_unread<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let had = {
        let state = app.state::<BridgeState>();
        let mut unread = state.unread.lock().unwrap();
        unread.remove(label).is_some()
    };
    if had {
        publish(app);
    }
}

/// Drops all bridge state for a webview that is going away.
pub fn forget<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let had = {
        let state = app.state::<BridgeState>();
        state.granted.lock().unwrap().remove(label);
        let removed = state.unread.lock().unwrap().remove(label).is_some();
        removed
    };
    if had {
        publish(app);
    }
}

fn remote_pattern(url: &tauri::Url) -> Option<String> {
    let scheme = url.scheme();
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{scheme}://{host}:{port}/*"),
        None => format!("{scheme}://{host}/*"),
    })
}

/// Lets one platform webview call the `velix` plugin, but only while it is on
/// its own origin. Navigating away (OAuth, an external link) drops the grant,
/// which is why the pattern is host-scoped rather than a blanket wildcard.
pub fn grant_remote_access<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    url: &tauri::Url,
) -> Result<(), String> {
    let state = app.state::<BridgeState>();
    if !state.granted.lock().unwrap().insert(label.to_string()) {
        return Ok(()); // already granted in this run
    }

    let pattern = remote_pattern(url).ok_or_else(|| format!("url has no host: {url}"))?;
    let capability = CapabilityBuilder::new(format!("velix-remote-{label}"))
        .webview(label)
        .local(false)
        .remote(pattern)
        .permission("velix:default");

    app.add_capability(capability).map_err(|e| {
        state.granted.lock().unwrap().remove(label);
        e.to_string()
    })
}

#[tauri::command]
pub async fn notify<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    title: String,
    body: String,
) -> Result<(), String> {
    let label = webview.label().to_string();
    let (quiet, profile) = {
        let state = app.state::<AppState>();
        let store = state.0.lock().unwrap();
        (
            store.config.settings.quiet,
            profile_for_label(&store.config.profiles, &label),
        )
    };
    // Unknown webview: nothing to attribute the notification to, so drop it.
    let Some(profile) = profile else {
        return Ok(());
    };

    // Count regardless of quiet mode — quiet silences the toast, not the badge.
    bump_unread(&app, &label);
    publish(&app);

    if !quiet {
        let heading = if title.trim().is_empty() {
            profile.name
        } else {
            format!("{} · {}", profile.name, title)
        };
        app.notification()
            .builder()
            .title(heading)
            .body(body)
            .show()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Authoritative unread count from `navigator.setAppBadge`; overrides the
/// running tally kept by [`notify`] for pages that report badges themselves.
#[tauri::command]
pub async fn set_badge<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    count: u32,
) -> Result<(), String> {
    set_unread(&app, webview.label(), count);
    publish(&app);
    Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("velix")
        .invoke_handler(tauri::generate_handler![notify, set_badge])
        .build()
}
