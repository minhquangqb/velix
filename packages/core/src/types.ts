/** An account profile: one isolated data directory per (plugin, profile) pair. */
export interface Profile {
  id: string
  pluginId: string
  name: string
}

/** Unread count for one open platform webview, as reported by its page. */
export interface UnreadEntry {
  /** Webview label, `${pluginId}-${profileId}`. */
  label: string
  pluginId: string
  profileId: string
  /** Profile name, so the tray popup can render without loading profiles. */
  name: string
  count: number
}

export interface Settings {
  /** Suppresses native notifications; unread badges keep updating. */
  quiet: boolean
  /** Closing the main window hides it to tray instead of quitting. */
  closeToTray: boolean
  autostart: boolean
}
