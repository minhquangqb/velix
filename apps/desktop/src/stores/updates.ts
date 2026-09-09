import { defineStore } from 'pinia'
import { appVersion, checkUpdate, installUpdate, onUpdate, type UpdateStatus } from '@velix/core'

/**
 * Self-update state.
 *
 * Rust drives the whole flow and reports it as `velix://update`, so a manual
 * check and the background one land here identically — the UI only ever renders
 * `status`. Failures are deliberately not re-thrown: they arrive as a `failed`
 * status and are shown inside the settings section, where the user asked for
 * them, rather than as a toast over whatever they were doing.
 */
export const useUpdatesStore = defineStore('updates', {
  state: () => ({
    /** Version of the running build, as reported by Rust. */
    version: '',
    status: null as UpdateStatus | null,
  }),
  getters: {
    /** The pending build, or `null` when there is nothing to install. */
    available(state) {
      return state.status?.phase === 'available' ? state.status.info : null
    },
    /** True while a check or a download is in flight, so buttons can lock. */
    busy(state) {
      const phase = state.status?.phase
      return phase === 'checking' || phase === 'downloading' || phase === 'installing'
    },
    /** The failure text, when the last attempt failed. */
    error(state) {
      return state.status?.phase === 'failed' ? state.status.message : null
    },
    /** Download progress, or `null` when the server sent no content-length. */
    percent(state): number | null {
      if (state.status?.phase !== 'downloading') return null
      const { received, total } = state.status
      if (!total) return null
      return Math.min(100, Math.round((received / total) * 100))
    },
  },
  actions: {
    async load() {
      this.version = await appVersion()
      return onUpdate((status) => (this.status = status))
    },
    async check() {
      try {
        await checkUpdate()
      } catch {
        // Already delivered as a `failed` status by Rust.
      }
    },
    /** On success the process is replaced by the new build, so this never returns. */
    async install() {
      try {
        await installUpdate()
      } catch {
        // Already delivered as a `failed` status by Rust.
      }
    },
  },
})
