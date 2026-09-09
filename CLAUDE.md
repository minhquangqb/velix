# Velix

Desktop workspace chạy nhiều web app (Messenger, Zalo, Telegram, ChatGPT...) trong một cửa sổ, mỗi tài khoản một profile độc lập. Build bằng Tauri v2.

## Tài liệu bắt buộc đọc trước khi làm

- [docs/Velix-Project-Plan.md](docs/Velix-Project-Plan.md) — vision, architecture, roadmap
- [docs/Implementation-Plan.md](docs/Implementation-Plan.md) — kế hoạch triển khai theo phase; luôn làm đúng phase đang được yêu cầu, không làm trước phase sau
- [docs/Feature-Spec-For-Design.md](docs/Feature-Spec-For-Design.md) — spec tính năng bàn giao cho design (không mô tả visual)
- [docs/release-updates.md](docs/release-updates.md) — phát hành + tự cập nhật: quy trình tag/publish, chữ ký số, invariants; đọc khi đụng `updater.rs`, `release.yml` hoặc bump version
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
  - ⚠️ **Bài học — global shortcut đăng ký trong plugin builder là bẫy chết câm** (fix sau Phase 3.5): `with_shortcuts()` đăng ký ngay lúc plugin init, nên khi tổ hợp đã có chủ (thường là chính một Velix còn sống dưới tray) thì `build()` trả `PluginInitialization("global-shortcut", "HotKey already registered")` → `.expect()` panic **trước khi có cửa sổ nào**. App là GUI subsystem: không console, không dialog, không entry trong Event Log — bấm mở thấy y như không có gì xảy ra. Nay đăng ký trong `setup()` qua `register_shortcuts()`, lỗi chỉ `eprintln!` rồi đi tiếp. Mất phím tắt không bao giờ được phép làm mất app
  - `tauri-plugin-single-instance` **đăng ký đầu tiên** trong builder: lần mở thứ hai chuyển tham số cho instance đang chạy rồi thoát, instance cũ gọi `tray::show_main`. Bỏ qua nếu tham số có `--autostart` (relaunch lúc boot phải nằm yên dưới tray)
  - `Ctrl+Shift+I` (devtools) nay chỉ đăng ký ở **debug build** — global shortcut là độc quyền toàn hệ thống, bản release trước đó cướp mất tổ hợp này của mọi app khác
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
- 🟡 **Phase 3.5 — Webview UX (chèn thêm, làm TRƯỚC Phase 4)** — Messenger đã chạy mượt thật (login + notification có tên người gửi + badge + link ngoài + navigation), user verify runtime OK hết. Còn spike login 3 nền tảng kia. Lý do chèn phase: sau Phase 3 app "dùng được" về mặt UI nhưng chưa dùng thật để chat; Phase 4 (plugin system) là tái cấu trúc, không cải thiện trải nghiệm.
  - ✅ **Notification Messenger — nguồn là `document.title`, KHÔNG phải các API** (commits `71fb73b`, `0c3ff6f`, `7851a6e`). **Chẩn đoán quan trọng, ghi lại để khỏi lặp cho nền tảng sau**: Messenger web **KHÔNG gọi** `Notification`/`showNotification`/`setAppBadge` — 3 hook Phase 2 trượt hết (`__velixProbe()` cho `ctor=0 sw=0 badge=0`). Thay vào đó nó **nhồi mọi thứ vào tiêu đề trang**, nhấp nháy giữa `"(N) Messenger"` (số chưa đọc) và `"An đã nhắn tin cho bạn"` (tên người gửi) khi tab **mất focus**.
    - `INJECT_SCRIPT` (bridge.rs) có **title-watcher**: MutationObserver trên `<title>` + poll 4s. Bắt tiêu đề gốc (`"Messenger"`) làm baseline (từ lần title không rỗng đầu tiên — lúc inject title có thể rỗng); tách `(N)` → badge qua `set_badge` (**dedup** vì title nhấp nháy liên tục, cùng số gọi lại nhiều lần); dòng nào khác baseline = **dòng người gửi** → toast nội dung đó (`notify` không count = chỉ toast). Chỉ có số mà không có dòng người gửi → toast generic "Bạn có N tin nhắn mới", **hoãn 1500ms** và bị **huỷ nếu đã có toast người gửi lượt này** (cờ `lastNotified`, reset khi focus hoặc khi count về 0). Dùng `hasFocus` KHÔNG dùng `visibilityState` (cái sau nói dối với webview con bị hide). Đây là baseline chung, web chat nào để tên trong title đều ăn
    - `notify` nhận `count: Option<u32>`: `Some` = set badge tuyệt đối (đường generic fallback), `None` = **chỉ toast** (đường dòng-người-gửi, title-watcher đã lo badge). (`bump_unread` cũ đã bỏ)
    - ⚠️ **Trần đã biết**: title chỉ cho **tên người gửi**, KHÔNG có câu tin. Muốn cả nội dung tin phải đọc DOM Messenger = logic riêng nền tảng → để dành `inject.ts` plugin Phase 4
    - **Giả visibility** (`window.__velixSetHidden`, Rust gọi qua `set_page_hidden`/`set_active_hidden` khi show/hide + tray) VẪN còn trong code, nhưng hoá ra Messenger KHÔNG bắn Notification kể cả khi tưởng hidden (dùng push/SW chết trong webview) → title mới là đường thật. Giữ lại vì nền tảng khác có thể suppress-on-visible thật; vô hại
  - ✅ **Link ngoài → trình duyệt hệ thống** (commit `2671e7e`), verify OK. `INJECT_SCRIPT` bắt click `<a>` (capture phase) + ghi đè `window.open`. Quy tắc "external = khác **registrable domain**" (so 2 label cuối, đúng cho .com/.me/.org cả 4 nền tảng) → gọi command `open_external`. Link cùng site điều hướng bình thường (không phá SPA routing; không phá OAuth vì OAuth là điều hướng programmatic không phải click). `open_external` là command **thứ 3** của inlined plugin `velix` (khai `build.rs`, validate chỉ `http/https`, mở bằng crate `open` trên thread rời) — lần mở rộng ACL remote có chủ đích
  - ✅ **Navigation + zoom** (commit `2671e7e`), verify OK. Command `webview_back`/`forward`/`reload`/`set_zoom` trong `webviews.rs`, chạy trên webview **active** (`ActiveWebview` state), gọi từ shell `ui` tin cậy nên **không** đụng ACL remote. back/forward qua `eval("history.back()")` (wry không có API go-back; SPA tự lo); zoom bằng `webview.set_zoom` **native** (KHÔNG CSS transform — vỡ layout SPA), nhớ theo từng account trong `tabs.zoom`, clamp 0.5–2.0. UI: cụm nút trong `TitleBar` slot `leading` (đặt NGOÀI vùng drag để không kích hoạt kéo cửa sổ), chỉ hiện khi có account mở
  - ⬜ **Spike đăng nhập Zalo/Telegram/ChatGPT** (build + install, **không** dev mode): thử login. Google chặn OAuth trong webview nhúng → có thể phải giả user-agent per-platform (pake-learnings mục 1). *Messenger đã login OK.* Mỗi nền tảng khi bring-up phải `__velixProbe()` xem nó dùng API hay title cho notification
  - ⚠️ **Notification chỉ đúng tên/icon Velix khi BUILD + INSTALL** — dev mode toast mang tên PowerShell (caveat Phase 2, WinRT cần AppUserModelID từ shortcut Start Menu). Không phải bug
  - **Chẩn đoán còn cắm trong code** (gỡ ở Phase 5 trước release): JS `window.__velixProbe()` / `__velixTest()`; Rust `eprintln!("[velix] ...")` (chỉ debug build — gồm cả `last_profile` và `open_webview` trong `webviews.rs`, thêm khi verify tính năng nhớ tab); `Ctrl+Shift+I` mở devtools webview nền tảng (global shortcut vì webview con giữ bàn phím). Tái dùng khi bring-up 3 nền tảng còn lại
- ✅ **Tự cập nhật + nhớ tab đang mở** (kéo sớm từ Phase 5, làm ngoài thứ tự phase theo yêu cầu user)
  - `updater.rs` — luồng update viết **bằng Rust**, không dùng JS API của `tauri-plugin-updater`: shell nằm trong webview con, giữ mọi bề mặt IPC là command của mình thì ACL mới tiếp tục đóng với webview remote. Command `app_version` / `check_update` / `install_update`; tiến độ đi qua event `velix://update` (`emit_to` webview `ui`, không broadcast) dưới dạng union tag theo `phase`
  - Tự kiểm tra nền: chờ 20s sau khi mở rồi lặp mỗi 6 giờ, **chỉ** emit `Available`. Biểu hiện duy nhất là chấm nhỏ trên nút Cài đặt ở rail — đúng yêu cầu "thông báo không làm phiền" của spec. Không toast, không dialog, không tự cài. Tắt hẳn ở debug build
  - ⚠️ Banner nổi trên workspace **không dùng được**: webview nền tảng vẽ đè lên shell nên sẽ bị che (cùng lý do với các bề mặt full-window)
  - **Chi tiết quy trình phát hành, chữ ký, invariants**: xem [docs/release-updates.md](docs/release-updates.md). Ba điều dễ quên nhất: release draft phải bấm **Publish** thì updater mới thấy; version phải khớp ở cả ba file; **không được đổi cặp khoá ký**
  - **Nhớ account đang xem**: `config.last_profile_id`, ghi trong `remember_last_profile` móc vào `show_only` (`webviews.rs`) — cái phễu duy nhất mà mọi lần đổi account đều đi qua. Ghi **thẳng xuống đĩa** ngay (khác window geometry chỉ ghi lúc thoát): đổi account là thao tác rời rạc vài chục lần một ngày, không phải luồng event liên tục. Shell đọc bằng `last_profile` rồi `ui.restoreLast()` gọi sau khi profiles đã load
- Chưa làm: Phase 4 (plugin system), Downloads (tách riêng), Phase 5 (phần còn lại)

## Release / CI (GitHub)

- **Repo**: `minhquangqb/velix` (private). Remote `origin` dùng **SSH** (`git@github.com:...`) — push file trong `.github/workflows/` không vướng scope `workflow` như khi push HTTPS bằng OAuth token
- **CI**: `.github/workflows/release.yml` — trigger khi push tag `v*` (hoặc `workflow_dispatch`). Chạy trên `windows-latest` (có sẵn WebView2 + MSVC), dùng `tauri-apps/tauri-action`. Build Rust lần đầu **~15 phút**. Tạo **GitHub Release dạng draft** (`releaseDraft: true`) đính kèm `.exe` (NSIS) + `.msi` → phải tự vào Releases bấm **Publish**
- ⚠️ **Bài học quan trọng — bắt buộc build `packages/**` TRƯỚC `tauri-action`**: `tauri-action` (`projectPath: apps/desktop`) chạy `beforeBuildCommand` (`pnpm build`) **trong thư mục `apps/desktop`**, nên chỉ build package đó, **KHÔNG** build lib phụ thuộc `@velix/core`. Vì `dist/` bị gitignore, checkout sạch trên CI không có `@velix/core/dist/*.d.ts` → `vue-tsc` báo `Cannot find module '@velix/core'` và fail. **Local pass mà CI fail** chính vì máy dev đã có `dist` build sẵn từ trước. → Workflow có step riêng `pnpm --filter "./packages/**" build` trước bước build+release. Muốn tái hiện lỗi local: xoá hết `packages/*/dist` rồi build thẳng `apps/desktop`
- **Release lại cùng tag**: sửa xong thì `git tag -f v0.1.0 && git push -f origin v0.1.0` (force move tag) để trigger lại; nếu Release draft cũ đã tồn tại thì xoá trước bằng `gh release delete`
- **Tooling**: cần `gh` CLI đã `gh auth login` (không tự động hoá được — bước tương tác). `pnpm@10.32.1`, Node 20 trong workflow
- ⚠️ Version đang `0.1.0` ở cả `package.json` (root + `apps/desktop`) và `tauri.conf.json` — bump cả 3 khi lên version mới
- **Secrets bắt buộc** cho updater: `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Thiếu thì artifact không được ký và app đã cài sẽ **từ chối** bản cập nhật. Khoá sinh sẵn ở `%USERPROFILE%\.tauri\velix.key` (+ `.key.password`)
- **Repo phải public** thì updater mới tải được asset (release của repo private đòi token)

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
