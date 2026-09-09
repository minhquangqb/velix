Cập nhật: 2026-09-09 | Nguồn: `apps/desktop/src-tauri/src/updater.rs`, `.github/workflows/release.yml`, `apps/desktop/src-tauri/tauri.conf.json`

# Phát hành và tự cập nhật

## Mục đích

Đưa một bản Velix mới tới máy người dùng đang chạy bản cũ, không bắt họ tự vào GitHub tải
installer. Người dùng luôn thấy phiên bản đang chạy, biết khi có bản mới, và tự quyết định
lúc nào cài — Velix không bao giờ tự tải hay tự cài sau lưng.

Ràng buộc lớn nhất là niềm tin: app đang chạy sẽ **thay thế chính binary của nó** bằng file
tải từ Internet. Nên mọi thứ trong luồng này xoay quanh chữ ký số — không có chữ ký hợp lệ
thì bản cập nhật bị từ chối, kể cả khi tải về thành công.

## Entrypoint

- Người dùng: **Cài đặt → Cập nhật** (`apps/desktop/src/views/SettingsView.vue`)
- Nền: `updater::start_auto_check` gọi từ `setup()` — `src-tauri/src/lib.rs:161`
- Phát hành: push tag `v*` → `.github/workflows/release.yml`

## Luồng chính

**Phát hành**

1. Bump version ở **cả ba** chỗ: `package.json` (root), `apps/desktop/package.json`,
   `apps/desktop/src-tauri/tauri.conf.json`
2. `git tag vX.Y.Z && git push origin vX.Y.Z` → workflow chạy trên `windows-latest`
3. `tauri-action` build, **ký** artifact bằng `TAURI_SIGNING_PRIVATE_KEY`, sinh `latest.json`
   (`includeUpdaterJson: true`) và tạo GitHub Release dạng **draft**
4. Vào Releases bấm **Publish** — đây là bước bắt buộc, xem Invariants

**Kiểm tra và cài**

1. `check_update` (`updater.rs:93`) hỏi endpoint, emit `velix://update` theo từng phase
2. Nền: `start_auto_check` (`updater.rs:187`) chờ 20s rồi lặp mỗi 6 giờ, chỉ emit `Available`
3. Shell nhận qua store `updates` → chấm nhỏ trên nút Cài đặt ở `PlatformRail.vue`
4. `install_update` (`updater.rs:123`) → `download` (`updater.rs:145`) tải kèm tiến độ, rồi
   `app.restart()` nhường quyền cho installer NSIS

## Invariants

- **Release còn ở dạng draft thì updater không thấy gì.** Endpoint là
  `/releases/latest/download/latest.json`, mà `latest` chỉ trỏ tới release đã publish
- **Ba chỗ version phải bằng nhau.** `tauri.conf.json` là cái updater đem đi so sánh; lệch
  là app hoặc không thấy bản mới, hoặc thấy chính nó
- **Không đổi cặp khoá ký.** `pubkey` trong `tauri.conf.json` phải khớp private key trong
  GitHub secrets. Đổi khoá = mọi bản đã cài ngoài kia không cập nhật được nữa, phải cài tay
- **Repo `minhquangqb/velix` phải public.** Asset của repo private đòi token mới tải được,
  còn updater thì gọi trần không kèm auth
- Mọi event `velix://update` đi bằng `emit_to(UI_LABEL, …)`, không bao giờ broadcast — hệt
  các event khác, để webview nền tảng remote không nghe được

## Quyết định & lý do

- **Viết luồng update bằng Rust, không dùng JS API của `tauri-plugin-updater`.** Shell chạy
  trong webview con; giữ mọi bề mặt IPC là command của chính mình là cách ACL tiếp tục đóng
  với webview remote (cùng lý lẽ với `bridge.rs`). Đổi lại phải tự viết state machine
- **`install_update` fetch lại thay vì giữ `Update` trong app state.** `Update` không `Sync`,
  mà một request thừa rẻ hơn nhiều so với việc giữ nó qua quãng người dùng ngập ngừng
- **Tự kiểm tra nhưng không tự cài.** Spec yêu cầu "thông báo không làm phiền" → chỉ một chấm
  trên nút Cài đặt, không toast, không dialog. Đã loại phương án banner nổi trên workspace vì
  webview nền tảng vẽ đè lên shell, banner sẽ bị che
- **Auto-check tắt ở debug build.** Version bản dev không bao giờ thua feed, request chỉ là
  nhiễu lúc phát triển
- **Đã loại**: repo public riêng cho bản cài (`velix-releases`) — gọn hơn về bảo mật source
  nhưng cần thêm một PAT làm secret vì `GITHUB_TOKEN` chỉ có quyền trong repo hiện tại; và
  host `latest.json` trên `dungqb-sv` — kiểm soát tốt nhất nhưng phải tự lo Nginx, đĩa, SSH key
