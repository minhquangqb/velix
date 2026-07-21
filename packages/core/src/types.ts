/** An account profile: one isolated data directory per (plugin, profile) pair. */
export interface Profile {
  id: string
  pluginId: string
  name: string
}
