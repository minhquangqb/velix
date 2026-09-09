/** An account profile: one isolated data directory per (plugin, profile) pair. */
export interface Profile {
  id: string
  pluginId: string
  name: string
  /** Suppresses this account's native notifications; its badge keeps counting. */
  muted: boolean
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

/** `system` follows the OS light/dark setting. */
export type Theme = 'system' | 'dark' | 'light'

export interface Settings {
  theme: Theme
  /** Suppresses native notifications; unread badges keep updating. */
  quiet: boolean
  /** Closing the main window hides it to tray instead of quitting. */
  closeToTray: boolean
  autostart: boolean
  /** Account list hidden to give the workspace more room; restored on launch. */
  sidebarCollapsed: boolean
}

/** Frameless-window state the custom title bar mirrors. */
export interface WindowStatus {
  maximized: boolean
}

/** A view change pushed from outside the shell, e.g. from the tray popup. */
export interface NavigateRequest {
  view: string
  /** Set for `view: "account"`, naming the account to bring forward. */
  profileId: string | null
}

/** A build offered by the update feed. */
export interface UpdateInfo {
  version: string
  /** Version running right now, so both sides of the upgrade can be shown. */
  currentVersion: string
  /** Release notes, as written in the GitHub release body. */
  notes: string | null
  date: string | null
}

/**
 * Every phase of the update flow, discriminated on `phase` exactly as Rust
 * tags it. An automatic check and a manual one produce the same values.
 */
export type UpdateStatus =
  | { phase: 'checking' }
  | { phase: 'upToDate' }
  | { phase: 'available'; info: UpdateInfo }
  | { phase: 'downloading'; received: number; total: number | null }
  | { phase: 'installing' }
  | { phase: 'failed'; message: string }
