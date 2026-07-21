// Platform-agnostic core: typed IPC wrappers over the Rust managers.
export type { VelixPlugin } from '@velix/sdk'
export type { Profile } from './types'
export {
  listProfiles,
  createProfile,
  deleteProfile,
  openWebview,
  focusWebview,
  closeWebview,
} from './ipc'
