
# Velix

> A modern desktop workspace for web apps built with Tauri v2.

## Vision
Velix là framework + ứng dụng desktop giúp chạy nhiều web app (Messenger, Zalo, Telegram, ChatGPT...) trong một workspace nhẹ, nhanh và bảo mật. Mỗi tài khoản có profile riêng.

## Goals
- Nhanh hơn mở nhiều tab trình duyệt
- Quản lý nhiều tài khoản
- Plugin-based
- Open source
- Cross-platform

## Tech Stack
- Tauri v2
- Rust
- Vue 3
- TypeScript
- Vite
- Pinia
- Tailwind CSS
- WebView2 (Windows)

## Monorepo

```
velix/
  apps/
    desktop/
  packages/
    core/
    sdk/
    ui/
    cli/
    plugins/
      messenger/
      zalo/
      telegram/
      chatgpt/
  docs/
```

## Architecture

### Core
- Window Manager
- WebView Manager
- Profile Manager
- Storage
- Downloads
- Notifications
- Tray
- Hotkeys
- Updater
- Permissions

### Plugin
Mỗi nền tảng chỉ cần:
- manifest.ts
- inject.ts
- style.css

Ví dụ:

```ts
export default {
  id: "messenger",
  name: "Messenger",
  url: "https://www.messenger.com",
  allowNotifications: true,
  allowDownloads: true
}
```

## Profiles

```
profiles/
  messenger/personal/
  messenger/work/
  zalo/personal/
```

Mỗi profile lưu:
- Cookies
- Cache
- LocalStorage
- IndexedDB
- Downloads

## UI

- Sidebar: Platforms
- Tabs: Accounts
- Workspace
- Command Palette
- Settings

## Features

### MVP
- Multi platform
- Multi account
- Native notifications
- System tray
- Minimize to tray
- Remember window state
- Dark mode
- Auto start
- Downloads

### Advanced
- Plugin Marketplace
- AI Assistant
- Translate
- Quick Reply
- OCR
- Screenshot
- Global Search
- Backup/Restore
- Cloud Sync

## SDK

```ts
interface VelixPlugin {
 id:string;
 name:string;
 url:string;
 injectCSS?:string;
 injectJS?:string;
}
```

## Roadmap

### v0.1 Foundation
- Core
- Single WebView
- Profiles
- Tray
- Notifications

### v0.2 Workspace
- Tabs
- Multiple accounts
- Downloads
- Settings

### v0.3 Plugins
- Messenger
- Zalo
- Telegram
- ChatGPT

### v0.4 Productivity
- AI
- Search
- Command Palette
- Plugin API

### v1.0
- Stable
- Auto Update
- Plugin Marketplace
- Documentation

## Performance Targets
- Startup <2s
- Idle RAM <150MB (1-2 WebViews)
- CPU Idle <1%

## Principles
- Core không phụ thuộc nền tảng.
- Plugin không sửa Core.
- Mỗi tài khoản là một profile độc lập.
- Có thể build thành Hub hoặc ứng dụng riêng.

## Future
Velix Hub
Velix SDK
Velix CLI
@velix/messenger
@velix/zalo
@velix/telegram
@velix/chatgpt
