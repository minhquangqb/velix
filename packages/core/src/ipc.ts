import { invoke } from '@tauri-apps/api/core'

import type { Profile } from './types'

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
