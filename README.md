# Velix

> A modern desktop workspace for web apps built with Tauri v2.

Velix runs multiple web apps (Messenger, Zalo, Telegram, ChatGPT...) in a single lightweight window, with one isolated profile per account.

## Docs

- [Project Plan](docs/Velix-Project-Plan.md) — vision, architecture, roadmap
- [Implementation Plan](docs/Implementation-Plan.md) — phase-by-phase plan

## Monorepo

```
apps/desktop        # Main Tauri app (Vue 3 + Vite + Pinia + Tailwind)
packages/core       # Window/WebView/Profile/Storage managers
packages/sdk        # VelixPlugin interface (no internal dependencies)
packages/ui         # Shared Vue components
packages/cli        # Velix CLI
packages/plugins/*  # Platform plugins (Phase 4)
```

## Development

Requirements: Node 20+, pnpm, Rust (rustup), WebView2 (Windows).

```bash
pnpm install
pnpm tauri dev      # run the desktop app with hot reload
pnpm build          # build all packages
pnpm typecheck
pnpm lint
```
