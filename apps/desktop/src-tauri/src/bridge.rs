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

/// Polyfill injected into every platform webview. Re-runs on every top-level
/// navigation (WebView2 re-injects per document), so it guards against
/// double-installing. It does four jobs:
///
/// 1. Forward the page's own notifications (`Notification`,
///    `ServiceWorkerRegistration.showNotification`) to native toasts — these
///    carry the real sender and message.
/// 2. Spoof `document.visibilityState`/`hidden` so a hidden webview looks
///    hidden to the page (WebView2 always reports it visible). Chat sites
///    suppress their own notifications while "visible", so without this a
///    backgrounded account stays silent. Rust drives it via `__velixSetHidden`.
/// 3. Track the unread count in the tab title (`(3) Messenger`) for the badge,
///    and fire a generic toast as a fallback when the page produced none.
/// 4. Open cross-site links in the system browser instead of letting them
///    navigate the webview away — which would drop its origin-scoped IPC grant.
pub const INJECT_SCRIPT: &str = r#"
(function () {
  if (window.__VELIX_BRIDGE__) return;
  Object.defineProperty(window, '__VELIX_BRIDGE__', { value: true });

  var stats = { ctor: 0, sw: 0, badge: 0, title: 0, sent: 0, failed: 0 };
  var lastError = null;
  // Timestamp of the last real (content-bearing) notification, used to suppress
  // the generic title-based fallback toast when the page already toasted.
  var lastRealNotifyMs = 0;
  var pendingToast = null;

  function invoke(cmd, args) {
    try {
      var internals = window.__TAURI_INTERNALS__;
      if (internals && internals.invoke) {
        return Promise.resolve(internals.invoke('plugin:velix|' + cmd, args)).then(
          function (value) { stats.sent++; return value; },
          function (error) {
            // Swallowed so a blocked call cannot break the page, but recorded:
            // an ACL rejection and an API the page never calls look identical
            // from the outside otherwise.
            stats.failed++; lastError = String(error); return undefined;
          }
        );
      }
      lastError = '__TAURI_INTERNALS__.invoke missing';
    } catch (e) { lastError = String(e); }
    stats.failed++;
    return Promise.resolve();
  }

  // A real notification from the page: it has content, so it wins over the
  // generic title fallback. No count — the title watcher owns the badge.
  function send(title, options) {
    options = options || {};
    lastRealNotifyMs = Date.now();
    if (pendingToast) { clearTimeout(pendingToast); pendingToast = null; }
    invoke('notify', {
      title: title == null ? '' : String(title),
      body: options.body == null ? '' : String(options.body)
    });
  }

  // --- Diagnostics (dev bring-up; strip before release, Phase 5) -----------
  window.__velixProbe = function () {
    var report = {
      bridgeInstalled: true,
      tauriInternals: !!(window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke),
      notificationPermission: window.Notification ? window.Notification.permission : 'no api',
      badgeApi: typeof navigator.setAppBadge,
      documentTitle: document.title,
      titleCount: titleCount(),
      spoofedHidden: velixHidden,
      visibilityState: document.visibilityState,
      hasFocus: document.hasFocus(),
      calls: stats,
      lastError: lastError
    };
    console.log('[velix probe]\n' + JSON.stringify(report, null, 2));
    return report;
  };
  window.__velixTest = function () {
    return invoke('notify', { title: 'Velix', body: 'Test tu devtools' });
  };

  // --- Native notifications from the page's own APIs -----------------------
  function VelixNotification(title, options) {
    options = options || {};
    this.title = title;
    this.body = options.body || '';
    this.tag = options.tag || '';
    this.icon = options.icon || '';
    this.data = options.data;
    this.onclick = null; this.onclose = null; this.onerror = null; this.onshow = null;
    stats.ctor++;
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
      configurable: true, writable: true, value: VelixNotification
    });
  } catch (e) {}

  try {
    if (window.ServiceWorkerRegistration && window.ServiceWorkerRegistration.prototype) {
      window.ServiceWorkerRegistration.prototype.showNotification = function (title, options) {
        stats.sw++;
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
      stats.badge++;
      var n = Number(count);
      invoke('set_badge', { count: isFinite(n) && n > 0 ? Math.floor(n) : 0 });
      return Promise.resolve();
    };
    navigator.clearAppBadge = function () {
      stats.badge++;
      invoke('set_badge', { count: 0 });
      return Promise.resolve();
    };
  } catch (e) {}

  // --- Visibility spoof ----------------------------------------------------
  // WebView2 reports a hidden child webview as visible, so chat sites think the
  // user is watching and never notify. Rust calls __velixSetHidden(true) when
  // the webview is off screen; the page then behaves as a background tab and
  // fires its own (content-bearing) notifications, caught above.
  var velixHidden = false;
  window.__velixSetHidden = function (hidden) {
    hidden = !!hidden;
    if (hidden === velixHidden) return;
    velixHidden = hidden;
    try { document.dispatchEvent(new Event('visibilitychange')); } catch (e) {}
    try { window.dispatchEvent(new Event(hidden ? 'blur' : 'focus')); } catch (e) {}
  };
  try {
    var visGetter = function () { return velixHidden ? 'hidden' : 'visible'; };
    var hidGetter = function () { return velixHidden; };
    Object.defineProperty(document, 'visibilityState', { configurable: true, get: visGetter });
    Object.defineProperty(document, 'hidden', { configurable: true, get: hidGetter });
    Object.defineProperty(document, 'webkitVisibilityState', { configurable: true, get: visGetter });
    Object.defineProperty(document, 'webkitHidden', { configurable: true, get: hidGetter });
  } catch (e) {}
  try {
    var realHasFocus = document.hasFocus.bind(document);
    document.hasFocus = function () { return velixHidden ? false : realHasFocus(); };
  } catch (e) {}

  // --- Unread count from the tab title ------------------------------------
  function titleCount() {
    var m = /\((\d+)\+?\)/.exec(document.title || '');
    return m ? parseInt(m[1], 10) : 0;
  }

  var lastTitleCount = -1;

  function onTitle() {
    var count = titleCount();
    if (count === lastTitleCount) return;
    var prev = lastTitleCount;
    lastTitleCount = count;
    stats.title++;

    invoke('set_badge', { count: count }); // badge is instant and authoritative

    // A rise while the user is not looking is a new message. Defer the generic
    // toast: if the page fires its own (content-bearing) one first, send()
    // cancels this, and the user gets the richer notification instead.
    if (prev >= 0 && count > prev && !document.hasFocus()) {
      if (pendingToast) clearTimeout(pendingToast);
      pendingToast = setTimeout(function () {
        pendingToast = null;
        if (Date.now() - lastRealNotifyMs < 1500) return;
        var body = count > 1 ? ('Bạn có ' + count + ' tin nhắn mới') : 'Bạn có tin nhắn mới';
        invoke('notify', { title: '', body: body, count: count });
      }, 1200);
    }
  }

  try {
    var titleEl = document.querySelector('title');
    if (titleEl) {
      new MutationObserver(onTitle).observe(titleEl, {
        childList: true, characterData: true, subtree: true
      });
    }
  } catch (e) {}
  // Backstop: the <title> node can be replaced wholesale, orphaning the
  // observer, and a background tab may batch mutations. A slow poll never misses.
  setInterval(onTitle, 4000);
  onTitle();

  // --- External links ------------------------------------------------------
  // A message link to a third-party site must open in the system browser, not
  // hijack the platform webview (which would drop its origin-scoped IPC grant
  // and strand the user with no way back). "External" = a different registrable
  // domain than the current page; same-site links navigate normally.
  function registrable(host) {
    var parts = (host || '').split('.');
    return parts.slice(-2).join('.');
  }
  function externalUrl(href) {
    try {
      var u = new URL(href, location.href);
      if (u.protocol !== 'http:' && u.protocol !== 'https:') return null;
      return registrable(u.hostname) === registrable(location.hostname) ? null : u.href;
    } catch (e) { return null; }
  }
  document.addEventListener('click', function (e) {
    if (e.defaultPrevented || e.button !== 0) return;
    var a = e.target && e.target.closest ? e.target.closest('a[href]') : null;
    if (!a) return;
    var ext = externalUrl(a.getAttribute('href'));
    if (ext) {
      e.preventDefault();
      e.stopPropagation();
      invoke('open_external', { url: ext });
    }
  }, true);
  var realOpen = window.open;
  window.open = function (url) {
    var ext = url ? externalUrl(String(url)) : null;
    if (ext) { invoke('open_external', { url: ext }); return null; }
    return realOpen ? realOpen.apply(window, arguments) : null;
  };
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

/// Fires a native toast for a platform page and, when the page reports one,
/// updates its unread count.
///
/// `count` is the authoritative unread total when the page knows it (the title
/// watcher's fallback toast passes it). `None` means toast only: the page's own
/// `Notification` carries content but no total, and the title watcher already
/// owns the badge, so there is nothing to set here.
#[tauri::command]
pub async fn notify<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    title: String,
    body: String,
    count: Option<u32>,
) -> Result<(), String> {
    let label = webview.label().to_string();
    #[cfg(debug_assertions)]
    eprintln!("[velix] notify from {label:?}: count={count:?} body={body:?}");
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
        #[cfg(debug_assertions)]
        eprintln!("[velix] notify dropped: no profile for {label:?}");
        return Ok(());
    };

    // Only the title-watcher path carries a count; the page-notification path
    // leaves the badge to the title watcher. Count regardless of quiet mode —
    // quiet silences the toast, not the badge.
    if let Some(count) = count {
        set_unread(&app, &label, count);
        publish(&app);
    }

    // Global quiet mode and the per-account mute both silence only the toast.
    if !quiet && !profile.muted {
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
    #[cfg(debug_assertions)]
    eprintln!(
        "[velix] set_badge from {:?}: count={count}",
        webview.label()
    );
    set_unread(&app, webview.label(), count);
    publish(&app);
    Ok(())
}

/// Opens a link in the system browser. The inject script routes cross-site
/// message links here so they do not navigate the platform webview away and
/// drop its origin-scoped IPC grant.
#[tauri::command]
pub async fn open_external(url: String) -> Result<(), String> {
    // Only web links. The command is reachable from remote pages, so a
    // `file:` or custom scheme must never reach the OS opener from here.
    let parsed = url.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!("refusing non-web url: {url}"));
    }
    #[cfg(debug_assertions)]
    eprintln!("[velix] open_external: {url}");
    // Detached: the opener spawns a browser process we do not want to await.
    std::thread::spawn(move || {
        let _ = open::that(url);
    });
    Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("velix")
        .invoke_handler(tauri::generate_handler![notify, set_badge, open_external])
        .build()
}
