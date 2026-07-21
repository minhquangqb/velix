import type { Theme } from '@velix/core'

/**
 * Resolves the `system` theme and stamps `data-theme` on <html>.
 *
 * Both windows (shell UI and tray popup) call this, so the popup re-themes with
 * the rest of the app rather than staying dark.
 */
const media = window.matchMedia('(prefers-color-scheme: dark)')

let current: Theme = 'system'

function stamp() {
  const resolved = current === 'system' ? (media.matches ? 'dark' : 'light') : current
  document.documentElement.dataset.theme = resolved
}

export function applyTheme(theme: Theme) {
  current = theme
  stamp()
}

// Only meaningful while the setting is `system`, but the listener is cheap and
// stays valid for the process lifetime, so it is never torn down.
media.addEventListener('change', stamp)

stamp()
