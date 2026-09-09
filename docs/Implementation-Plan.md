# Velix — Kế hoạch triển khai nhanh (Claude Code)

> Nguyên tắc: mỗi phase là 1 session Claude Code độc lập, kết thúc bằng một trạng thái chạy được + commit. Không gộp nhiều phase vào một session để tránh loãng context.

## Điều kiện môi trường (làm trước, thủ công)

- Đang ở WSL2 → Tauri GUI **không chạy trực tiếp trong WSL2** một cách ổn định với WebView2. Hai lựa chọn:
  - **Khuyến nghị**: code trong WSL2, nhưng chạy `pnpm tauri dev` / build từ Windows (clone repo sang Windows hoặc dùng thư mục chung).
  - Hoặc dùng WSLg + webkit2gtk (Linux build) để dev nhanh, build Windows sau bằng CI.
- Cài đặt: Rust (rustup), Node 20+, pnpm, Tauri CLI v2 (`cargo install tauri-cli` hoặc dùng `pnpm tauri`).
- `git init` + tạo repo remote ngay từ Phase 0.

## Phase 0 — Scaffold monorepo (0.5 buổi)

Mục tiêu: skeleton chạy được `pnpm tauri dev` hiện cửa sổ trắng.

- pnpm workspace: `apps/desktop`, `packages/{core,sdk,ui,cli}`, `packages/plugins/*`
- `apps/desktop`: Tauri v2 + Vue 3 + TypeScript + Vite + Pinia + Tailwind
- `packages/sdk`: chỉ chứa interface `VelixPlugin` (theo Project Plan) — làm sớm vì mọi package khác phụ thuộc
- Shared tsconfig, ESLint, Prettier
- **Definition of done**: app mở được, hot-reload hoạt động, commit `feat: scaffold monorepo`

## Phase 1 — Core: WebView + Profile (v0.1 Foundation, 2-3 buổi)

Mục tiêu: mở 1 web app (vd Messenger) trong WebView con với profile riêng.

1. **WebView Manager** (Rust): tạo/đóng webview con bằng Tauri v2 multi-webview API, gắn vào vùng workspace của cửa sổ chính
2. **Profile Manager**: mỗi profile = 1 data directory riêng (`profiles/<platform>/<name>/`) → cookies/cache/localStorage tách biệt
3. **Storage**: config app bằng JSON (tauri-plugin-store hoặc tự viết trong core)
4. IPC commands: `open_webview(pluginId, profileId)`, `close_webview(id)`
5. **Definition of done**: mở Messenger với 2 profile khác nhau, login 2 tài khoản không đè nhau

## Phase 2 — Tray + Notifications + Window state (v0.1 tiếp, 1-2 buổi)

- System tray + minimize to tray + menu Quit/Show
- Native notifications (forward từ webview qua permission API)
- Remember window state (size/position/maximized)
- Auto start (tauri-plugin-autostart)
- **Definition of done**: đóng cửa sổ → còn tray icon, notification từ web app hiện native

## Phase 3 — Workspace UI (v0.2, 2-3 buổi)

- Sidebar platforms + Tabs accounts + vùng workspace (theo mục UI trong Project Plan)
- Pinia stores: `platforms`, `profiles`, `tabs`, `settings`
- Chuyển tab = show/hide webview (không destroy để giữ session, nhưng cân nhắc RAM target <150MB → lazy load, unload webview không dùng lâu)
- Settings page + Dark mode
- Downloads manager (UI list + mở file)
- **Definition of done**: thêm/xóa account từ UI, chuyển tab mượt, settings lưu lại sau restart

## Phase 4 — Plugin system (v0.3, 2 buổi)

- `packages/sdk`: chuẩn hóa `manifest.ts` / `inject.ts` / `style.css`, build ra bundle plugin
- Core load plugin từ registry tĩnh (chưa cần marketplace)
- Viết 4 plugin: `messenger`, `zalo`, `telegram`, `chatgpt` — mỗi cái chỉ là manifest + inject tối thiểu
- **Definition of done**: 4 platform chạy từ plugin, thêm platform mới không sửa core (đúng Principles)

## Phase 5 — Productivity + Release (v0.4 → v1.0)

- Command Palette (Ctrl+K), Global hotkeys
- Auto update (tauri-plugin-updater) + CI GitHub Actions build Windows installer
- Đo performance targets (startup <2s, RAM <150MB) và tối ưu
- Docs + README
- Các mục Advanced (AI, Translate, OCR...) để sau v1.0 — không đưa vào scope bây giờ

## Cách chạy với Claude Code

- Mỗi phase: mở session mới, prompt kiểu *"Đọc docs/Velix-Project-Plan.md và docs/Implementation-Plan.md, thực hiện Phase N"*
- Sau mỗi phase: chạy verify (build + chạy thử), commit rồi mới sang phase tiếp
- Task nhỏ lẻ giữa chừng: dùng `/q:fix`, `/q:review`
- Rủi ro kỹ thuật lớn nhất cần spike sớm (đầu Phase 1): **Tauri v2 multi-webview + data dir riêng per webview trên Windows/WebView2** — nếu API chưa đủ, fallback là mỗi account một cửa sổ riêng ẩn/hiện
