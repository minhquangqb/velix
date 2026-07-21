# Học hỏi từ Pake (tw93/Pake)

> Repo: https://github.com/tw93/Pake — MIT license (tham khảo/port code được, ghi attribution nếu copy nguyên đoạn).
> Pake đóng gói **mỗi website thành một app riêng** bằng Tauri; Velix là **workspace nhiều app, nhiều profile**. Không cạnh tranh trực tiếp, nhưng phần "làm web app chạy tử tế trong webview" của Pake rất đáng tham khảo, nhất là khi làm plugin SDK và inject baseline.

## 1. Inject layer — tham khảo khi làm `packages/sdk` / inject baseline trong core

Source: https://github.com/tw93/Pake/tree/main/src-tauri/src/inject

- **Notification polyfill + badge bridge** — Pake polyfill `window.Notification` và `navigator.setAppBadge` để chuyển web notification thành notification/badge native của Tauri. Dùng cho unread badge Messenger/Zalo/Telegram — đỡ phải parse DOM đếm tin chưa đọc (với app dùng chuẩn web notification).
- **OAuth login handling** (`auth.js`, `isAuthLink()`) — detect luồng đăng nhập để giữ trong webview thay vì bật ra browser ngoài. Kết hợp user-agent per-platform để lách việc Google chặn login trong embedded webview. Đọc kỹ trước khi làm plugin ChatGPT/Messenger.
- **Link + download interception** (`event.js`) — hijack click anchor:
  - Link nội bộ → `window.location.href`
  - Link ngoài → mở system browser qua `invoke("plugin:shell|open")`
  - Link file (pdf/zip, blob/data URL, pattern `/download/`) → `invoke("download_file")` về Rust
  - Webview mặc định không xử lý download tử tế → Velix phải làm tương tự.
- **Zoom per-webview bằng native `set_zoom`** — KHÔNG dùng CSS transform (vỡ layout SPA, chính ChatGPT bị).
- **Keyboard shortcuts inject** — back/forward, zoom, reload qua `keyup` listener; có flag `disabled_web_shortcuts` để chặn shortcut của trang web.
- **Drag region cho frameless** — inject div `#pake-top-dom` gọi `appWindow.startDragging()`. Velix chủ yếu dùng drag region ở shell UI, nhưng hữu ích nếu webview chiếm sát mép trên cửa sổ.
- Các file khác: `find.js` (find-in-page dialog), `toast.js`, `theme_refresh.js`, `fullscreen.js`, context menu download ảnh/video.

## 2. Config schema — mượn field cho manifest plugin/profile

Source: https://github.com/tw93/Pake/blob/main/src-tauri/pake.json

Field đáng có tương đương trong Velix:

| Field | Ý nghĩa | Ghi chú cho Velix |
|---|---|---|
| `internal_url_regex` | Link nào giữ trong app vs mở browser ngoài | Mỗi plugin khai trong manifest |
| `hide_on_close` | Đóng = ẩn (giữ session) | Khớp hành vi tray của Velix |
| `start_to_tray` | Khởi động ẩn xuống tray | |
| `activation_shortcut` | Global shortcut gọi cửa sổ lên | |
| `disabled_web_shortcuts` | Chặn shortcut của trang web | |
| `proxy_url` | Proxy per app | Velix: per profile |
| `user_agent` (per platform) | Lách web chặn WebView UA | Quan trọng cho OAuth |
| `incognito`, `multi_instance`, `force_internal_navigation` | | Tham khảo thêm |

## 3. Cấu trúc Rust gọn — đọc nhanh được

Source: https://github.com/tw93/Pake/tree/main/src-tauri/src/app

`config.rs` (parse config), `window.rs` (tạo window + tray), `menu.rs`, `invoke.rs` (IPC handlers), `setup.rs` (init).

## 4. Cái Pake KHÔNG giải quyết (Velix tự làm, đúng lõi khác biệt)

- Multi-profile: mỗi tài khoản một data dir độc lập
- Nhiều webview show/hide trong một cửa sổ
- Plugin architecture (core không biết platform cụ thể)
