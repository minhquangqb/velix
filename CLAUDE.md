# Velix

Desktop workspace chạy nhiều web app (Messenger, Zalo, Telegram, ChatGPT...) trong một cửa sổ, mỗi tài khoản một profile độc lập. Build bằng Tauri v2.

## Tài liệu bắt buộc đọc trước khi làm

- [docs/Velix-Project-Plan.md](docs/Velix-Project-Plan.md) — vision, architecture, roadmap
- [docs/Implementation-Plan.md](docs/Implementation-Plan.md) — kế hoạch triển khai theo phase; luôn làm đúng phase đang được yêu cầu, không làm trước phase sau
- [docs/Feature-Spec-For-Design.md](docs/Feature-Spec-For-Design.md) — spec tính năng bàn giao cho design (không mô tả visual)
- [docs/references/pake-learnings.md](docs/references/pake-learnings.md) — kỹ thuật tham khảo từ Pake (tw93/Pake, MIT): notification/badge polyfill, OAuth trong webview, link/download interception, native zoom, config schema — đọc khi làm inject baseline trong core/sdk hoặc plugin

## Tiến độ (cập nhật 2026-07-21)

- ✅ **Phase 0** — scaffold monorepo, commit `feat: scaffold monorepo` (verify: build/typecheck/lint/cargo check pass, `pnpm tauri dev` mở được cửa sổ)
- ✅ **Spike multi-webview** (rủi ro số một của Phase 1) — PASS: window tạo bằng code + 2 webview con (`ui` sidebar + `messenger` load mặc định để test hiệu năng) qua `Window::add_child`, cần tauri feature `unstable`. Hạn chế đã biết: `auto_resize()` scale theo tỷ lệ → Phase 1 phải tự reposition khi resize. Capability chỉ cấp IPC cho webview `ui`, webview remote không có quyền
- ✅ **Design MVP** — đủ 10/10 bề mặt trong Claude Design project (xem mục Design), đã review 2 vòng + verify fix, chốt bàn giao dev. Tồn đọng duy nhất: bug preview nút "Hủy" trong sim của Account Manager (không ảnh hưởng implement)
- 🔶 **Phase 1 — code xong, chờ verify runtime** — WebView Manager + Profile Manager: Rust `config.rs`/`profiles.rs`/`webviews.rs` (storage JSON, IPC `open_webview`/`close_webview`/`focus_webview`/CRUD profile, mỗi profile một `data_directory` riêng dưới `app_data_dir/profiles/<plugin>/<profile>`, tự relayout khi window resize), typed IPC wrapper trong `packages/core`, registry tĩnh + store `profiles` + sidebar UI tối thiểu trong `apps/desktop`. Verify tự động pass (build/typecheck/lint/clippy). **Còn thiếu: user chạy `pnpm tauri dev` xác nhận DoD** — Messenger 2 profile login không đè nhau, resize không lệch layout
- Chưa làm: Phase 2 (tray/notifications/window state), Phase 3 (Workspace UI theo design), Phase 4 (plugin system), Phase 5 (release)

## Design

- Claude Design project "Velix main screen review": https://claude.ai/design/p/4bd8b925-f564-40ab-9865-d8f5a2280472?file=Velix+Main+v2.dc.html
  - Đọc qua tool DesignSync (projectId `4bd8b925-f564-40ab-9865-d8f5a2280472`); file mới nhất: `Velix Main v2.dc.html`
  - Lưu ý: design dùng cửa sổ frameless (nút min/max/close custom) — implement cần `decorations: false` + vùng drag ở cạnh trên
  - Tray menu: đã chốt implement bằng custom popup window (không dùng native tray menu) để giữ đúng design (toggle, kbd hint, badge) — làm trong Phase 2

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
