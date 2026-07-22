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
- ✅ **Phase 1** — WebView Manager + Profile Manager, commit `feat: add webview and profile managers (Phase 1)`: Rust `config.rs`/`profiles.rs`/`webviews.rs` (storage JSON, IPC `open_webview`/`close_webview`/`focus_webview`/CRUD profile, mỗi profile một `data_directory` riêng dưới `app_data_dir/profiles/<plugin>/<profile>`, tự relayout khi window resize), typed IPC wrapper trong `packages/core`, registry tĩnh + store `profiles` + sidebar UI tối thiểu. User đã verify runtime OK. Bài học quan trọng: **command đụng webview phải là `async fn`** — sync command deadlock trên Windows (wry#583), promise treo không có error
- ✅ **Phase 2** — Tray + Notifications + Window state, commit `feat: add tray, notifications and window state (Phase 2)`. User đã verify runtime OK (tray, popup, window state, `Ctrl+Shift+V`).
  - `bridge.rs` — cầu notification từ trang remote. Webview remote **bị ACL chặn IPC** (khác webview local): phải cấp capability lúc runtime. Dùng `InlinedPlugin` tên `velix` (khai trong `build.rs`) với đúng 2 command `notify`/`set_badge`, thay vì app manifest — app manifest sẽ kéo **toàn bộ** command Phase 1 vào ACL. Mỗi webview được `add_capability` riêng, scope theo đúng origin của nó (`https://host/*`), cấp trước khi tạo webview
  - Inject polyfill `window.Notification` + `ServiceWorkerRegistration.showNotification` + `navigator.setAppBadge` (idempotent — WebView2 chạy lại script mỗi lần điều hướng top-level)
  - `tray.rs` — tray icon + popup window custom (`?view=tray`, frameless/transparent/always-on-top, ẩn khi mất focus). **Chiều cao popup tính bên Rust** theo số account chưa đọc → hằng số row height phải khớp `TrayMenu.vue`. Badge tray là chấm đỏ vẽ trực tiếp lên RGBA của icon (icon tray quá nhỏ để hiện số; số nằm trong popup)
  - `window_state.rs` — nhớ size/position/maximized; ghi RAM mỗi lần move/resize, chỉ ghi đĩa lúc thoát (cửa sổ có thể ẩn dưới tray hàng giờ)
  - Đóng cửa sổ = ẩn xuống tray (`closeToTray`, mặc định bật); thoát hẳn chỉ từ popup tray. Autostart + global shortcut `Ctrl+Shift+V`
  - ⚠️ **Notification trên Windows chỉ hiện đúng khi app đã được cài** (WinRT toast cần AppUserModelID từ shortcut Start Menu). Chạy `pnpm tauri dev` sẽ thấy toast mang tên/icon PowerShell — **không phải bug**, muốn verify thật phải build + install
  - ⚠️ Đã nâng `tauri >= 2.11.1` vì CVE-2026-42184 (GHSA-7gmj-67g7-phm9): trên Windows origin remote có thể bị nhận nhầm là local — đúng kịch bản webview remote của Velix
- ✅ **Phase 3** — Workspace UI theo design, commit `feat: add workspace UI, themes and settings (Phase 3)` + `feat: use lucide icons and real platform logos`. User đã verify runtime OK.
  - **Cửa sổ frameless** (`decorations(false)`, min 860×640). Nút min/max/close + drag đi qua `window_ctl.rs`, **không** dùng `@tauri-apps/api/window`: shell chạy trong webview **con**, ở đó JS window API và `data-tauri-drag-region` không được nối. Double-click title bar = maximize (bắt bằng `event.detail === 2`, vì `start_dragging` giao cử chỉ cho OS ngay khi mousedown)
  - **Hằng số layout phải khớp hai bên**: `RAIL_WIDTH 68` + `ACCOUNTS_WIDTH 248` (= `SIDEBAR_WIDTH 316`) + `TOPBAR_HEIGHT 44` trong `webviews.rs` ↔ `w-17` / `w-62` trong `PlatformRail.vue` / `AccountList.vue` và `h-11` trong `TitleBar.vue`. Webview nền tảng được đặt vào đúng cái lỗ đó
  - ⚠️ **Webview con luôn vẽ ĐÈ lên webview `ui`** — không thể phủ HTML lên nó. Mọi bề mặt toàn cửa sổ (Settings, Account Manager, Add Account, First-run) phải gọi `hide_webviews` trước; `tabs.detach()` làm việc này và nhớ `resumeProfileId` để quay lại
  - Stores: `platforms` / `profiles` (data + unread) / `tabs` (vòng đời webview, show/hide **không destroy**) / `settings` / `ui` (routing + trạng thái maximize)
  - **Theme dark/light** bằng CSS variables `--vx-*` trong `style.css`, `data-theme` stamp lên `<html>` bởi `theme.ts`; `system` bám `prefers-color-scheme`. Tray popup cũng nhận theme qua event `velix://settings`
  - Event mới: `velix://settings` (settings đổi), `velix://window` (maximize đổi), `velix://navigate` (tray đẩy view sang shell). Tất cả `emit_to` đúng webview `ui`/`tray` — **không bao giờ broadcast**, vì broadcast sẽ lọt sang webview nền tảng remote
  - Tray "Cài đặt…" / bấm account → command `open_settings` / `open_account`: Rust hiện cửa sổ rồi đẩy `velix://navigate`, **shell tự chuyển webview** để sidebar không lệch trạng thái
  - Thêm vào config: `settings.theme` (system/dark/light) và `profile.muted` (tắt thông báo từng account; `bridge::notify` kiểm tra cả `quiet` lẫn `muted`). Command mới: `rename_profile`, `set_profile_muted`, `set_theme`, `hide_webviews`
  - **Icon**: UI dùng [Lucide](https://lucide.dev) qua `@lucide/vue` (MIT, tree-shake theo từng import). ⚠️ Package cũ `lucide-vue-next` đã **deprecated** → luôn dùng `@lucide/vue`. Stroke mặc định 2px, design dùng 1.2–1.7px nên chỗ nào cũng set `:stroke-width` tường minh
  - **Logo nền tảng**: path SVG copy từ Simple Icons (CC0) và **inline vào `src/brands.ts`** — không giữ package `simple-icons` làm dependency vì chỉ cần 3 path trên tổng 3000 icon. `GlyphBadge` nhận theo thứ tự ưu tiên `path` (brand, fill) → `icon` (component Lucide, stroke) → `glyph` (chữ cái). Ô badge tint/bo góc giữ nguyên như design
  - ⚠️ **ChatGPT không có logo**: Simple Icons đã gỡ OpenAI vì chính sách nhãn hiệu, và vẽ tay logo từ trí nhớ sẽ ra sai. Tạm dùng icon `Sparkles` của Lucide (khai trong `FALLBACK_ICONS` ở `registry.ts`). Muốn logo thật thì tự lấy SVG chính chủ bỏ vào `BRAND_PATHS.chatgpt`, badge sẽ tự ưu tiên nó
  - Logo là nhãn hiệu của chủ sở hữu; CC0 chỉ áp cho dữ liệu vector. Bình thường với app dạng này (Rambox/Ferdi đều làm) nhưng nên rà lại nếu phát hành thương mại
  - Avatar tài khoản vẫn là **chữ cái đầu của tên tài khoản** (do user đặt), chỉ badge **nền tảng** mới dùng logo
  - ESLint: `no-undef` tắt cho `**/*.vue` — rule này không có type info nên chỉ báo nhầm `MouseEvent`/`HTMLInputElement`; `vue-tsc` mới là thứ kiểm tra thật
  - **Lệch design có chủ ý (ghi lại để khỏi tưởng là thiếu sót)**:
    - **Downloads bỏ hẳn khỏi Phase 3** (user chốt) → tách thành phase riêng, gồm cả UI lẫn intercept phía Rust. Rail không có icon Downloads, Settings không có mục Downloads
    - Command palette `Ctrl+K` và mục "Cập nhật" trong Settings → Phase 5 (đúng Implementation-Plan)
    - Add Account bước 3 (webview đăng nhập **nằm trong** wizard) → thay bằng: tạo profile xong nhảy về workspace, trang đăng nhập hiện ở vùng workspace bình thường. Lý do: webview chỉ đặt được vào hình chữ nhật workspace cố định
    - Chưa làm: kéo-thả sắp xếp account, trạng thái offline/"Thử lại", trạng thái "đang ngủ" (unload webview lâu không dùng — thuộc phần tối ưu RAM Phase 5)
    - Không nạp Google Fonts (Be Vietnam Pro / Space Grotesk / JetBrains Mono) — app desktop chạy offline, hiện fallback `system-ui`. Muốn đúng chữ thì phải bundle font vào assets
    - Component dùng chung vẫn nằm ở `apps/desktop/src/components`, chưa đưa sang `packages/ui`: package đó build bằng `tsc` thuần, chưa có pipeline SFC, và mới chỉ có một nơi dùng
- 🟡 **Phase 3.5 — Webview UX (chèn thêm, làm TRƯỚC Phase 4)** — đang làm dần, ưu tiên Messenger chạy mượt trước. Lý do chèn: sau Phase 3 app "dùng được" về mặt UI nhưng chưa dùng thật được để chat. Phase 4 (plugin system) là tái cấu trúc, không cải thiện trải nghiệm; Downloads không liên quan nhắn tin.
  - ✅ **Notification Messenger — FIX rồi, user verify runtime OK** (commit tiếp theo). **Chẩn đoán quan trọng, ghi lại để khỏi lặp cho nền tảng sau**: Messenger web **KHÔNG gọi** `Notification`/`ServiceWorkerRegistration.showNotification`/`navigator.setAppBadge` — 3 hook của Phase 2 bắn trượt hết (`__velixProbe()` cho `ctor=0 sw=0 badge=0`). Lý do: **WebView2 báo `visibilityState: "visible"` cả khi webview bị hide**, nên Messenger tưởng user đang nhìn → tự nó không báo. Nguồn tin cậy duy nhất là **`document.title` dạng `"(N) Messenger"`**
    - Fix: `INJECT_SCRIPT` (bridge.rs) thêm **title-watcher** — MutationObserver trên `<title>` + poll 4s làm lưới, parse `(N)` bằng regex. Số → `set_badge(N)` (badge sidebar/tray). Khi N **tăng** và `document.hasFocus() === false` → `notify({count:N})` bắn toast. **Dùng `hasFocus`, KHÔNG dùng `visibilityState`** (cái sau nói dối với webview con bị hide). Đây là baseline chung ("(n) Name" web chat nào cũng dính), không hardcode Messenger
    - `notify` command nhận `count: Option<u32>`: `Some` = set badge tuyệt đối (đường title-watcher fallback), `None` = **chỉ toast, không đụng badge** (đường notification thật của trang — có nội dung nhưng không kèm tổng số; title-watcher lo badge). Nhờ vậy hai đường không đá nhau, không cần command mới vào ACL. (`bump_unread` cũ đã bỏ)
  - 🟡 **Notification nội dung + Link ngoài + Navigation — code xong, CHỜ VERIFY RUNTIME** (commit này). Ba việc trong một session vì đều đụng `INJECT_SCRIPT`/bridge:
    - **Toast có tên người gửi + nội dung** bằng cách **giả visibility** thay vì đọc DOM: `INJECT_SCRIPT` ghi đè `document.visibilityState`/`hidden`/`hasFocus` qua `window.__velixSetHidden(bool)`. Rust gọi nó khi show/hide webview (`webviews::set_page_hidden` trong `show_only`; `set_active_hidden` khi ẩn/hiện xuống tray). Khi webview bị ẩn, trang **tưởng mình là background tab** → Messenger tự bắn `Notification` của chính nó **kèm tên + nội dung thật** → hook bắt được. Đây là cách generic (nền tảng nào suppress-on-visible cũng ăn), không đọc DOM riêng. Title-watcher giữ vai trò **badge + toast dự phòng generic**, có **chống trùng**: toast generic bị hoãn 1200ms; nếu trang bắn noti thật trong khoảng đó (`send()` set `lastRealNotifyMs`) thì huỷ toast generic → user nhận bản giàu nội dung. ⚠️ Nếu spike sau cho thấy Messenger **vẫn không** bắn noti kể cả khi tưởng hidden (vì dùng push/service-worker chết trong webview) thì rơi về toast generic — không regress
    - **Link ngoài → trình duyệt hệ thống**: `INJECT_SCRIPT` bắt click `<a>` (capture phase) + ghi đè `window.open`. Quy tắc "external = khác **registrable domain**" (so 2 label cuối, đúng cho .com/.me/.org của cả 4 nền tảng) → gọi command `open_external`. Link cùng site điều hướng bình thường (không phá SPA routing, không phá OAuth vì OAuth là điều hướng programmatic không phải click). `open_external` là command **thứ 3** của inlined plugin `velix` (khai trong `build.rs`, validate chỉ `http/https`, mở bằng crate `open` trên thread rời). Đây là lần mở rộng ACL remote có chủ đích — link ngoài là chức năng lõi, không phải UI
    - **Navigation**: command `webview_back`/`webview_forward`/`webview_reload`/`webview_set_zoom` trong `webviews.rs`, chạy trên webview **active** (`ActiveWebview` state). Gọi từ shell `ui` (tin cậy) nên **không** đụng ACL remote. back/forward qua `eval("history.back()")` (wry không có API go-back; SPA tự lo History API); zoom bằng `webview.set_zoom` **native** (KHÔNG CSS transform — vỡ layout SPA). Zoom nhớ theo từng account trong `tabs.zoom`, clamp 0.5–2.0 bên Rust. UI: cụm nút back/forward/reload + zoom trong `TitleBar` slot `leading` (đặt NGOÀI vùng drag để nút không kích hoạt kéo cửa sổ), chỉ hiện khi có account đang mở
    - Còn lại: toast vẫn mang tên PowerShell ở dev mode (cần build+install); nếu giả-visibility không kích được noti thật thì chưa có tên người gửi
  - ⬜ **Spike đăng nhập** (build + install, **không** dev mode): thử login cả 4 nền tảng. Google chặn OAuth trong webview nhúng → có thể phải giả user-agent per-platform (pake-learnings mục 1). *Messenger đã login OK.*
  - **Chẩn đoán còn cắm trong code** (gỡ ở Phase 5 trước release): JS `window.__velixProbe()` / `__velixTest()`; Rust `eprintln!("[velix] ...")` (chỉ debug build); `Ctrl+Shift+I` mở devtools webview nền tảng (global shortcut vì webview con giữ bàn phím). Tái dùng khi bring-up Zalo/Telegram/ChatGPT
- Chưa làm: Phase 4 (plugin system), Downloads (tách riêng), Phase 5 (release)
- Chưa làm: Phase 4 (plugin system), Downloads (tách riêng), Phase 5 (release)

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
