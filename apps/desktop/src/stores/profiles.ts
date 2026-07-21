import { defineStore } from 'pinia'
import {
  createProfile,
  deleteProfile,
  listProfiles,
  listUnread,
  onUnread,
  renameProfile,
  setProfileMuted,
  type Profile,
  type UnreadEntry,
} from '@velix/core'

/**
 * The account list and its unread counts. Webview lifecycle lives in the `tabs`
 * store; this one is pure account data.
 */
export const useProfilesStore = defineStore('profiles', {
  state: () => ({
    profiles: [] as Profile[],
    /** profileId -> unread count, pushed by the Rust notification bridge */
    unread: {} as Record<string, number>,
  }),
  getters: {
    byPlugin(state) {
      return (pluginId: string) => state.profiles.filter((p) => p.pluginId === pluginId)
    },
    /** Total unread per platform, for the rail badges. */
    unreadByPlugin(state): Record<string, number> {
      const totals: Record<string, number> = {}
      for (const profile of state.profiles) {
        const count = state.unread[profile.id] ?? 0
        if (count > 0) totals[profile.pluginId] = (totals[profile.pluginId] ?? 0) + count
      }
      return totals
    },
    find(state) {
      return (id: string) => state.profiles.find((p) => p.id === id)
    },
  },
  actions: {
    async load() {
      this.profiles = await listProfiles()
    },
    /** Mirrors unread counts from Rust; resolves to an unsubscribe function. */
    async watchUnread() {
      const apply = (entries: UnreadEntry[]) => {
        this.unread = Object.fromEntries(entries.map((e) => [e.profileId, e.count]))
      }
      apply(await listUnread())
      return onUnread(apply)
    },
    async create(pluginId: string, name: string) {
      const profile = await createProfile(pluginId, name)
      this.profiles.push(profile)
      return profile
    },
    async rename(id: string, name: string) {
      this.replace(await renameProfile(id, name))
    },
    async setMuted(id: string, muted: boolean) {
      this.replace(await setProfileMuted(id, muted))
    },
    async remove(id: string) {
      await deleteProfile(id)
      this.profiles = this.profiles.filter((p) => p.id !== id)
      delete this.unread[id]
    },
    replace(profile: Profile) {
      const index = this.profiles.findIndex((p) => p.id === profile.id)
      if (index >= 0) this.profiles[index] = profile
    },
  },
})
