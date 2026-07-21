import { defineStore } from 'pinia'
import type { VelixPlugin } from '@velix/core'

import { PLUGINS, findPlugin } from '../registry'

/**
 * The platform rail's selection. Platforms themselves come from the static
 * registry until Phase 4 replaces it with the real plugin loader — the store
 * deliberately holds no platform knowledge of its own.
 */
export const usePlatformsStore = defineStore('platforms', {
  state: () => ({
    plugins: PLUGINS as VelixPlugin[],
    activeId: (PLUGINS[0]?.id ?? '') as string,
  }),
  getters: {
    active(state): VelixPlugin | undefined {
      return findPlugin(state.activeId)
    },
  },
  actions: {
    select(pluginId: string) {
      this.activeId = pluginId
    },
  },
})
