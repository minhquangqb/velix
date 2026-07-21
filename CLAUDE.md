# Velix

Desktop workspace chạy nhiều web app (Messenger, Zalo, Telegram, ChatGPT...) trong một cửa sổ, mỗi tài khoản một profile độc lập. Build bằng Tauri v2.

## Tài liệu bắt buộc đọc trước khi làm

- [docs/Velix-Project-Plan.md](docs/Velix-Project-Plan.md) — vision, architecture, roadmap
- [docs/Implementation-Plan.md](docs/Implementation-Plan.md) — kế hoạch triển khai theo phase; luôn làm đúng phase đang được yêu cầu, không làm trước phase sau

## Tech stack

- Tauri v2 + Rust (backend/core)
- Vue 3 + TypeScript + Vite + Pinia + Tailwind CSS (frontend)
- pnpm workspace (monorepo)
- WebView2 trên Windows

## Cấu trúc monorepo

```
apps/desktop        # App Tauri chính
packages/core       # Window/WebView/Profile/Storage managers
packages/sdk        # Interface VelixPlugin — không phụ thuộc package khác
packages/ui         # Shared Vue components
packages/cli        # Velix CLI
packages/plugins/*  # messenger, zalo, telegram, chatgpt
docs/               # Tài liệu
```

## Commands

```bash
pnpm install
pnpm tauri dev      # KHÔNG tự chạy — user tự run (xem Restricted bên dưới)
pnpm build          # Build tất cả packages
pnpm typecheck
pnpm lint
cargo check         # Trong apps/desktop/src-tauri
```

## Nguyên tắc kiến trúc (bất biến)

- Core KHÔNG phụ thuộc nền tảng cụ thể — không hardcode "messenger"/"zalo" trong core
- Plugin KHÔNG sửa core — plugin chỉ gồm `manifest.ts`, `inject.ts`, `style.css`
- Mỗi tài khoản = một profile độc lập (data dir riêng: cookies, cache, localStorage, IndexedDB)
- Thêm platform mới = thêm plugin mới, không đụng vào core

## Quy ước code

- TypeScript strict mode, không dùng `any` trừ khi bất khả kháng
- Rust: chạy `cargo fmt` + `cargo clippy` trước khi commit
- Vue: Composition API + `<script setup lang="ts">`, state dùng Pinia stores
- Tên biến/hàm/comment bằng tiếng Anh
- IPC commands đặt tên snake_case phía Rust (vd `open_webview`), gọi qua wrapper typed trong `packages/core`

## Môi trường dev

- Repo nằm trong WSL2 nhưng target chính là Windows/WebView2 → `pnpm tauri dev` chạy từ phía Windows; trong WSL2 chỉ code, typecheck, lint, cargo check
- Không assume GUI chạy được trong WSL2

## Performance targets (kiểm tra khi review)

- Startup <2s, idle RAM <150MB (1-2 webviews), CPU idle <1%
- Chuyển tab = show/hide webview, không destroy (giữ session); cân nhắc unload webview lâu không dùng

## Workflow

- Mỗi phase trong Implementation-Plan làm trong một session riêng, xong phải build/verify được rồi mới commit
- Commit format: `type: short description` (feat, fix, docs, style, refactor, test, chore)
