import { defineStore } from 'pinia'
import { lastProfile, onNavigate, onWindowStatus, windowIsMaximized } from '@velix/core'

import { usePlatformsStore } from './platforms'
import { useProfilesStore } from './profiles'
import { useTabsStore } from './tabs'

/** Full-window surfaces. Anything other than `main` hides the platform webviews. */
export type View = 'main' | 'settings' | 'accounts' | 'add-account' | 'first-run'

export const useUiStore = defineStore('ui', {
  state: () => ({
    view: 'main' as View,
    /** Pre-selected platform for the add-account wizard, when entered from the rail. */
    addAccountPluginId: null as string | null,
    /** Set once the welcome screen is passed, so it does not come back mid-run. */
    firstRunDone: false,
    maximized: false,
  }),
  actions: {
    /** Mirrors the frameless window's maximised state for the title bar glyph. */
    async watchWindow() {
      this.maximized = await windowIsMaximized()
      return onWindowStatus((status) => (this.maximized = status.maximized))
    },
    /** Honours view changes pushed from the tray popup. */
    async watchNavigate() {
      return onNavigate(async (request) => {
        if (request.view === 'settings') {
          await this.goto('settings')
          return
        }
        if (request.view !== 'account' || !request.profileId) return

        // The shell drives the webview switch so the sidebar selection, the
        // active platform and the visible webview all agree.
        const profiles = useProfilesStore()
        const profile = profiles.find(request.profileId)
        if (!profile) return

        const platforms = usePlatformsStore()
        platforms.select(profile.pluginId)
        this.view = 'main'
        this.firstRunDone = true
        await useTabsStore().open(profile)
      })
    },
    /**
     * Reopens the account that was on screen when the app last closed.
     *
     * Runs once at startup, after profiles are loaded so the id can be resolved
     * to a real account. Skipped when the shell is already on another view: the
     * tray can push a navigation while the app is still starting, and that
     * choice is newer than the remembered one.
     */
    async restoreLast() {
      if (this.view !== 'main') return
      const id = await lastProfile()
      if (!id) return

      const profile = useProfilesStore().find(id)
      if (!profile) return
      usePlatformsStore().select(profile.pluginId)
      await useTabsStore().open(profile)
    },
    /**
     * Navigates. Every view except `main` covers the workspace area, so the
     * platform webviews are hidden first — they render above the shell UI and
     * would otherwise punch a hole through the page.
     */
    async goto(view: View, pluginId?: string) {
      const tabs = useTabsStore()
      this.addAccountPluginId = view === 'add-account' ? (pluginId ?? null) : null

      if (view !== 'main') {
        this.view = view
        await tabs.detach()
        return
      }

      this.view = 'main'
      this.firstRunDone = true
      await this.restoreActive()
    },
    /** Re-shows the account that was visible before the surface was opened. */
    async restoreActive() {
      const tabs = useTabsStore()
      const profiles = useProfilesStore()
      const profile = tabs.resumeProfileId ? profiles.find(tabs.resumeProfileId) : undefined
      if (profile) await tabs.open(profile)
    },
  },
})
