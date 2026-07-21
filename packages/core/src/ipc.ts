import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type { NavigateRequest, Profile, Settings, Theme, UnreadEntry, WindowStatus } from './types'

export function listProfiles(): Promise<Profile[]> {
  return invoke('list_profiles')
}

export function createProfile(pluginId: string, name: string): Promise<Profile> {
  return invoke('create_profile', { pluginId, name })
}

export function renameProfile(id: string, name: string): Promise<Profile> {
  return invoke('rename_profile', { id, name })
}

export function setProfileMuted(id: string, muted: boolean): Promise<Profile> {
  return invoke('set_profile_muted', { id, muted })
}

export function deleteProfile(id: string): Promise<void> {
  return invoke('delete_profile', { id })
}

/** Opens (or focuses) the webview for a profile. Resolves to its label. */
export function openWebview(pluginId: string, profileId: string, url: string): Promise<string> {
  return invoke('open_webview', { pluginId, profileId, url })
}

export function focusWebview(label: string): Promise<void> {
  return invoke('focus_webview', { label })
}

/**
 * Hides every platform webview without closing it. Native child webviews always
 * paint above the shell UI, so this is what lets a full-window surface (settings,
 * wizards) be seen at all. Sessions stay alive.
 */
export function hideWebviews(): Promise<void> {
  return invoke('hide_webviews')
}

export function closeWebview(label: string): Promise<void> {
  return invoke('close_webview', { label })
}

export function getSettings(): Promise<Settings> {
  return invoke('get_settings')
}

export function setTheme(theme: Theme): Promise<Settings> {
  return invoke('set_theme', { theme })
}

export function setQuiet(quiet: boolean): Promise<Settings> {
  return invoke('set_quiet', { quiet })
}

export function setCloseToTray(closeToTray: boolean): Promise<Settings> {
  return invoke('set_close_to_tray', { closeToTray })
}

export function setAutostart(autostart: boolean): Promise<Settings> {
  return invoke('set_autostart', { autostart })
}

/** Reveals and focuses the main window, hiding the tray popup. */
export function showMainWindow(): Promise<void> {
  return invoke('show_main_window')
}

/** Reveals the main window and switches it to the settings view. */
export function openSettings(): Promise<void> {
  return invoke('open_settings')
}

/** Reveals the main window and asks the shell to bring an account forward. */
export function openAccount(profileId: string): Promise<void> {
  return invoke('open_account', { profileId })
}

export function hideTrayPopup(): Promise<void> {
  return invoke('hide_tray_popup')
}

export function listUnread(): Promise<UnreadEntry[]> {
  return invoke('list_unread')
}

/** Quits for real, as opposed to closing the window to tray. */
export function quitApp(): Promise<void> {
  return invoke('quit_app')
}

// The window is frameless, so these replace the native title bar. They route
// through Rust rather than @tauri-apps/api/window: the shell runs in a child
// webview, where the JS window API and `data-tauri-drag-region` are not wired up.
export function windowIsMaximized(): Promise<boolean> {
  return invoke('window_is_maximized')
}

export function windowMinimize(): Promise<void> {
  return invoke('window_minimize')
}

export function windowToggleMaximize(): Promise<void> {
  return invoke('window_toggle_maximize')
}

/** Honours `closeToTray`, exactly like the native close button did. */
export function windowClose(): Promise<void> {
  return invoke('window_close')
}

export function windowStartDrag(): Promise<void> {
  return invoke('window_start_drag')
}

const UNREAD_EVENT = 'velix://unread'
const SETTINGS_EVENT = 'velix://settings'
const WINDOW_EVENT = 'velix://window'
const NAVIGATE_EVENT = 'velix://navigate'

/** Subscribes to unread-count changes; resolves to an unsubscribe function. */
export function onUnread(handler: (entries: UnreadEntry[]) => void): Promise<() => void> {
  return listen<UnreadEntry[]>(UNREAD_EVENT, (event) => handler(event.payload))
}

/** Fires after any settings change, in either the shell or the tray popup. */
export function onSettings(handler: (settings: Settings) => void): Promise<() => void> {
  return listen<Settings>(SETTINGS_EVENT, (event) => handler(event.payload))
}

export function onWindowStatus(handler: (status: WindowStatus) => void): Promise<() => void> {
  return listen<WindowStatus>(WINDOW_EVENT, (event) => handler(event.payload))
}

/** View changes requested from outside the shell, e.g. the tray popup. */
export function onNavigate(handler: (request: NavigateRequest) => void): Promise<() => void> {
  return listen<NavigateRequest>(NAVIGATE_EVENT, (event) => handler(event.payload))
}
