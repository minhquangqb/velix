// Platform-agnostic core: typed IPC wrappers over the Rust managers.
export type { VelixPlugin } from '@velix/sdk'
export type { Profile, Settings, UnreadEntry } from './types'
export {
  listProfiles,
  createProfile,
  deleteProfile,
  openWebview,
  focusWebview,
  closeWebview,
  getSettings,
  setQuiet,
  setCloseToTray,
  setAutostart,
  showMainWindow,
  hideTrayPopup,
  listUnread,
  quitApp,
  onUnread,
} from './ipc'
