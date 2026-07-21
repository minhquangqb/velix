/**
 * A Velix plugin describes how a web platform is embedded in the workspace.
 * Plugins must be self-contained: manifest + optional inject assets.
 * They must never depend on or modify core.
 */
export interface VelixPlugin {
  /** Unique plugin id, e.g. "messenger" */
  id: string
  /** Display name, e.g. "Messenger" */
  name: string
  /** Entry URL loaded in the webview */
  url: string
  /** CSS injected into the page after load */
  injectCSS?: string
  /** JS injected into the page after load */
  injectJS?: string
  /** Whether the platform may show native notifications */
  allowNotifications?: boolean
  /** Whether the platform may download files */
  allowDownloads?: boolean
}
