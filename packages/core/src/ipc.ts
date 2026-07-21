import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type { Profile, Settings, UnreadEntry } from './types'

export function listProfiles(): Promise<Profile[]> {
  return invoke('list_profiles')
}

export function createProfile(pluginId: string, name: string): Promise<Profile> {
  return invoke('create_profile', { pluginId, name })
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

export function closeWebview(label: string): Promise<void> {
  return invoke('close_webview', { label })
}

export function getSettings(): Promise<Settings> {
  return invoke('get_settings')
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

const UNREAD_EVENT = 'velix://unread'

/** Subscribes to unread-count changes; resolves to an unsubscribe function. */
export function onUnread(handler: (entries: UnreadEntry[]) => void): Promise<() => void> {
  return listen<UnreadEntry[]>(UNREAD_EVENT, (event) => handler(event.payload))
}
