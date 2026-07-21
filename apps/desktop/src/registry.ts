import type { VelixPlugin } from '@velix/core'

/**
 * Static plugin registry until the real plugin system lands in Phase 4.
 * Core stays platform-agnostic; only this list knows concrete platforms.
 */
export const PLUGINS: VelixPlugin[] = [
  {
    id: 'messenger',
    name: 'Messenger',
    url: 'https://www.messenger.com',
    tint: '#7B8CFF',
    glyph: 'M',
  },
  {
    id: 'zalo',
    name: 'Zalo',
    url: 'https://chat.zalo.me',
    tint: '#4FA8FF',
    glyph: 'Z',
  },
  {
    id: 'telegram',
    name: 'Telegram',
    url: 'https://web.telegram.org',
    tint: '#4FC3F0',
    glyph: 'T',
  },
  {
    id: 'chatgpt',
    name: 'ChatGPT',
    url: 'https://chatgpt.com',
    tint: '#6FD8A8',
    glyph: 'G',
  },
]

export function findPlugin(pluginId: string): VelixPlugin | undefined {
  return PLUGINS.find((p) => p.id === pluginId)
}

/** Host shown in the add-account picker, derived rather than configured. */
export function pluginDomain(plugin: VelixPlugin): string {
  try {
    return new URL(plugin.url).host.replace(/^www\./, '')
  } catch {
    return plugin.url
  }
}

export function pluginTint(plugin: VelixPlugin | undefined): string {
  return plugin?.tint ?? '#7B8CFF'
}

export function pluginGlyph(plugin: VelixPlugin | undefined): string {
  return plugin?.glyph ?? plugin?.name.charAt(0).toUpperCase() ?? '?'
}
