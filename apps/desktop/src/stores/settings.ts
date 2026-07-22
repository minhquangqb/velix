import { defineStore } from 'pinia'
import {
  getSettings,
  onSettings,
  setAutostart,
  setCloseToTray,
  setQuiet,
  setSidebarCollapsed,
  setTheme,
  type Settings,
  type Theme,
} from '@velix/core'

import { applyTheme } from '../theme'

const DEFAULTS: Settings = {
  theme: 'system',
  quiet: false,
  closeToTray: true,
  autostart: false,
  sidebarCollapsed: false,
}

export const useSettingsStore = defineStore('settings', {
  state: () => ({ settings: { ...DEFAULTS } as Settings }),
  actions: {
    /** Adopts settings from Rust and re-themes; Rust is always the source of truth. */
    apply(settings: Settings) {
      this.settings = settings
      applyTheme(settings.theme)
    },
    async load() {
      this.apply(await getSettings())
      // Settings can also change from the tray popup, hence the subscription.
      return onSettings((next) => this.apply(next))
    },
    async setTheme(theme: Theme) {
      this.apply(await setTheme(theme))
    },
    async setQuiet(quiet: boolean) {
      this.apply(await setQuiet(quiet))
    },
    async setCloseToTray(closeToTray: boolean) {
      this.apply(await setCloseToTray(closeToTray))
    },
    async setAutostart(autostart: boolean) {
      this.apply(await setAutostart(autostart))
    },
    /** Persists the collapse; Rust repositions the platform webviews to match. */
    async setSidebarCollapsed(collapsed: boolean) {
      this.apply(await setSidebarCollapsed(collapsed))
    },
    toggleSidebar() {
      return this.setSidebarCollapsed(!this.settings.sidebarCollapsed)
    },
  },
})
