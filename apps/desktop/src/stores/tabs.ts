import { defineStore } from 'pinia'
import { closeWebview, focusWebview, hideWebviews, openWebview, type Profile } from '@velix/core'

import { findPlugin } from '../registry'

/**
 * Webview lifecycle for open accounts.
 *
 * Switching accounts only shows/hides native webviews — nothing is destroyed, so
 * sessions and scroll position survive. A webview is created on first open and
 * then kept for the rest of the run; closing one is an explicit user action.
 */
export const useTabsStore = defineStore('tabs', {
  state: () => ({
    /** profileId -> webview label, for profiles whose webview exists */
    openLabels: {} as Record<string, string>,
    activeProfileId: null as string | null,
    /** pluginId -> last account viewed there, so the rail restores it */
    lastByPlugin: {} as Record<string, string>,
    /** Account to re-show when a full-window surface is dismissed. */
    resumeProfileId: null as string | null,
    /** True while a webview is being created — the workspace shows a spinner. */
    opening: false,
  }),
  getters: {
    isOpen(state) {
      return (profileId: string) => profileId in state.openLabels
    },
  },
  actions: {
    async open(profile: Profile) {
      const plugin = findPlugin(profile.pluginId)
      if (!plugin) throw new Error(`unknown platform: ${profile.pluginId}`)

      const label = this.openLabels[profile.id]
      if (label) {
        await focusWebview(label)
      } else {
        // Only the first open pays webview startup; show the waking state for it.
        this.opening = true
        try {
          this.openLabels[profile.id] = await openWebview(profile.pluginId, profile.id, plugin.url)
        } finally {
          this.opening = false
        }
      }

      // Set last, so a failed open does not leave the sidebar highlighting an
      // account whose webview is not actually on screen.
      this.activeProfileId = profile.id
      this.lastByPlugin[profile.pluginId] = profile.id
    },
    /**
     * Clears the workspace so a full-window surface can be seen. Platform
     * webviews paint above the shell UI, so they must be hidden, not covered.
     */
    async detach() {
      this.resumeProfileId = this.activeProfileId
      this.activeProfileId = null
      await hideWebviews()
    },
    async close(profileId: string) {
      const label = this.openLabels[profileId]
      if (!label) return
      await closeWebview(label)
      this.forget(profileId)
    },
    /** Drops local bookkeeping for a webview Rust has already disposed of. */
    forget(profileId: string) {
      delete this.openLabels[profileId]
      if (this.activeProfileId === profileId) this.activeProfileId = null
      if (this.resumeProfileId === profileId) this.resumeProfileId = null
      for (const [pluginId, id] of Object.entries(this.lastByPlugin)) {
        if (id === profileId) delete this.lastByPlugin[pluginId]
      }
    },
  },
})
