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
  },
]
