import { defineStore } from 'pinia'
import {
  closeWebview,
  createProfile,
  deleteProfile,
  focusWebview,
  listProfiles,
  listUnread,
  onUnread,
  openWebview,
  type Profile,
  type UnreadEntry,
} from '@velix/core'

export const useProfilesStore = defineStore('profiles', {
  state: () => ({
    profiles: [] as Profile[],
    /** profileId -> webview label, for profiles whose webview is open */
    openLabels: {} as Record<string, string>,
    /** profileId -> unread count, pushed by the Rust notification bridge */
    unread: {} as Record<string, number>,
    activeProfileId: null as string | null,
  }),
  getters: {
    byPlugin(state) {
      return (pluginId: string) => state.profiles.filter((p) => p.pluginId === pluginId)
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
    async open(profile: Profile, url: string) {
      const label = this.openLabels[profile.id]
      if (label) {
        await focusWebview(label)
      } else {
        this.openLabels[profile.id] = await openWebview(profile.pluginId, profile.id, url)
      }
      this.activeProfileId = profile.id
    },
    async close(profileId: string) {
      const label = this.openLabels[profileId]
      if (!label) return
      await closeWebview(label)
      delete this.openLabels[profileId]
      if (this.activeProfileId === profileId) this.activeProfileId = null
    },
    async remove(profileId: string) {
      await deleteProfile(profileId)
      delete this.openLabels[profileId]
      if (this.activeProfileId === profileId) this.activeProfileId = null
      this.profiles = this.profiles.filter((p) => p.id !== profileId)
    },
  },
})
