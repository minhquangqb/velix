# Velix — Tài liệu tính năng dành cho Design

> Tài liệu này mô tả **tính năng và luồng người dùng**, không mô tả giao diện. Bố cục, hình khối, màu sắc, chuyển động, phong cách thị giác... hoàn toàn do design quyết định và sáng tạo. Chỗ nào có gợi ý về hành vi (behavior) thì đó là yêu cầu chức năng, không phải yêu cầu trình bày.

## 1. Sản phẩm là gì

Velix là ứng dụng desktop (Windows trước, cross-platform sau) gom nhiều web app nhắn tin / làm việc — Messenger, Zalo, Telegram, ChatGPT... — vào một cửa sổ duy nhất, thay cho việc mở hàng chục tab trình duyệt.

Điểm khác biệt cốt lõi: **mỗi tài khoản là một phiên đăng nhập độc lập**. Người dùng có thể đăng nhập 2-3 tài khoản Messenger (cá nhân, công việc, shop...) cùng lúc mà không đè phiên nhau, không cần trình duyệt ẩn danh hay nhiều browser profile.

Giá trị chính với người dùng:

- Nhanh và nhẹ hơn mở nhiều tab trình duyệt
- Một nơi duy nhất cho mọi hội thoại, không phải tìm tab
- Nhiều tài khoản trên cùng nền tảng, tách biệt tuyệt đối
- Chạy nền yên tĩnh, chỉ nổi lên khi có thông báo

## 2. Người dùng mục tiêu

- **Người bán hàng / chăm sóc khách hàng online**: quản lý nhiều tài khoản Zalo, Messenger của shop; sống trong app cả ngày, cần chuyển tài khoản liên tục.
- **Dân văn phòng / freelancer**: tách tài khoản cá nhân và công việc; app chạy nền cả ngày, chủ yếu tương tác qua thông báo.
- **Power user**: dùng nhiều nền tảng cùng lúc, thích phím tắt, muốn thao tác không cần chuột.

Bối cảnh sử dụng: app mở suốt cả ngày làm việc, thường ở trạng thái chạy nền. Tần suất chuyển qua lại giữa các tài khoản rất cao — đây là hành động số một của sản phẩm.

## 3. Khái niệm cốt lõi

| Khái niệm | Nghĩa |
|---|---|
| **Nền tảng (Platform)** | Một dịch vụ web: Messenger, Zalo, Telegram, ChatGPT... Danh sách nền tảng có thể mở rộng qua plugin. |
| **Tài khoản (Account/Profile)** | Một phiên đăng nhập trên một nền tảng. Người dùng đặt tên tự do (vd "Cá nhân", "Shop A"). Một nền tảng có thể có nhiều tài khoản. |
| **Workspace** | Vùng hiển thị nội dung web app của tài khoản đang chọn. Nội dung bên trong là trang web của nền tảng — Velix không kiểm soát giao diện phần này. |

Quan hệ: người dùng chọn **nền tảng** → chọn **tài khoản** thuộc nền tảng đó → nội dung tài khoản hiện trong **workspace**.

## 4. Tính năng

### 4.1 Quản lý nền tảng và tài khoản

- Xem danh sách nền tảng đang dùng và danh sách tài khoản của từng nền tảng.
- Thêm tài khoản mới: chọn nền tảng (từ danh sách nền tảng Velix hỗ trợ) → đặt tên tài khoản → tài khoản mở ra để người dùng đăng nhập trực tiếp trên trang của nền tảng đó (Velix không có form đăng nhập riêng, không bao giờ hỏi mật khẩu của nền tảng).
- Đổi tên tài khoản, xóa tài khoản. Xóa tài khoản là hành động phá hủy (mất phiên đăng nhập và dữ liệu cục bộ của tài khoản đó) → cần bước xác nhận.
- Cần phân biệt được các tài khoản cùng nền tảng với nhau (người dùng tự đặt tên; thứ tự có thể sắp xếp lại).

### 4.2 Chuyển đổi giữa các tài khoản

- Hành động cốt lõi, tần suất cao nhất. Chuyển tài khoản phải **tức thời** — phiên web được giữ nguyên phía sau, không tải lại trang.
- Người dùng cần luôn biết mình đang ở tài khoản nào, thuộc nền tảng nào.
- Có thể chuyển bằng chuột và bằng phím tắt.
- Tài khoản lâu không dùng có thể bị "ngủ" để tiết kiệm RAM; khi quay lại sẽ cần tải lại trong giây lát → tồn tại trạng thái "đang thức dậy / đang tải".

### 4.3 Thông báo

- Thông báo từ web app (tin nhắn mới...) được đẩy ra thành thông báo native của hệ điều hành, kể cả khi app đang chạy nền.
- Click thông báo → mở app và nhảy đúng tài khoản phát sinh thông báo.
- Từng tài khoản bật/tắt được thông báo (vd tắt thông báo tài khoản cá nhân trong giờ làm).

### 4.4 Chạy nền và system tray

- Đóng cửa sổ ≠ thoát app: app thu về icon ở khay hệ thống (system tray), vẫn nhận thông báo.
- Từ tray: mở lại cửa sổ, thoát hẳn app.
- Khởi động cùng Windows (bật/tắt trong cài đặt).
- Mở lại app: khôi phục đúng kích thước, vị trí cửa sổ và tài khoản đang xem trước đó.

### 4.5 Tải file (Downloads)

- File tải từ các web app được quản lý tập trung một chỗ: xem danh sách đã tải, trạng thái đang tải, mở file, mở thư mục chứa file.
- Cho phép bật/tắt quyền tải file theo nền tảng.

### 4.6 Cài đặt

Nhóm cài đặt hiện có:

- **Giao diện**: dark mode / light mode / theo hệ thống.
- **Hành vi**: khởi động cùng Windows, đóng cửa sổ thì thu về tray hay thoát hẳn.
- **Thông báo**: bật/tắt tổng và theo từng tài khoản.
- **Downloads**: thư mục lưu mặc định.
- **Cập nhật**: app tự cập nhật; người dùng thấy được phiên bản hiện tại và khi có bản mới.

### 4.7 Command Palette và phím tắt

- Command Palette (mặc định `Ctrl+K`): gõ để nhảy nhanh đến tài khoản bất kỳ hoặc thực hiện hành động (thêm tài khoản, mở cài đặt...). Dành cho power user, thao tác hoàn toàn bằng bàn phím.
- Phím tắt toàn cục (global hotkey): gọi Velix nổi lên từ bất kỳ đâu trong Windows.

## 5. Trạng thái cần thiết kế

Ngoài luồng chính, các trạng thái sau chắc chắn xảy ra và cần được xử lý:

1. **Lần đầu mở app** — chưa có tài khoản nào. Đây là màn hình quyết định người dùng có hiểu sản phẩm hay không; cần dẫn họ đến việc thêm tài khoản đầu tiên.
2. **Nền tảng có nhưng chưa có tài khoản** / danh sách rỗng nói chung.
3. **Đang tải**: tài khoản mở lần đầu, hoặc "thức dậy" sau khi bị ngủ.
4. **Mất mạng / trang web lỗi**: nội dung workspace không tải được, cần cho người dùng biết và thử lại.
5. **Xác nhận hành động phá hủy**: xóa tài khoản.
6. **Có bản cập nhật mới**: thông báo không làm phiền, cập nhật xong cần khởi động lại.

## 6. Ràng buộc trải nghiệm (không phải ràng buộc thị giác)

- **Cảm giác tức thời**: chuyển tài khoản không có độ trễ cảm nhận được. App khởi động dưới 2 giây.
- **Nội dung là của nền tảng**: phần lớn diện tích luôn là trang web của nền tảng (Messenger, Zalo...). Phần "vỏ" của Velix chỉ phục vụ điều hướng và quản lý — Velix không vẽ lại giao diện chat.
- **Chạy nền cả ngày**: app phải "yên tĩnh", không đòi hỏi sự chú ý khi không có gì mới.
- **Hỗ trợ dark mode** ngay từ đầu.
- **Bàn phím là first-class**: mọi luồng chính đều thao tác được không cần chuột.
- Desktop Windows là mục tiêu đầu tiên; cửa sổ resize tự do, có kích thước tối thiểu.

## 7. Phạm vi cần design

Các bề mặt (surface) cần thiết kế cho bản MVP:

1. Màn hình chính (điều hướng nền tảng/tài khoản + workspace)
2. Trạng thái lần đầu mở app (empty state / onboarding)
3. Luồng thêm tài khoản (chọn nền tảng → đặt tên → đăng nhập)
4. Quản lý tài khoản (đổi tên, sắp xếp, xóa + xác nhận xóa)
5. Cài đặt (các nhóm ở mục 4.6)
6. Downloads
7. Command Palette
8. Các trạng thái: đang tải, lỗi mạng, có bản cập nhật
9. Menu tray (nội dung menu, không cần vẽ lại UI hệ điều hành)
10. Icon app (dùng cho cửa sổ, tray, installer)

## 8. Tương lai gần (chưa cần design, nhưng nên biết để chừa đường mở rộng)

- Plugin Marketplace: người dùng cài thêm nền tảng mới từ cửa hàng plugin.
- AI Assistant, dịch tin nhắn, quick reply, OCR, chụp màn hình.
- Tìm kiếm toàn cục xuyên các nền tảng.
- Backup / restore, cloud sync.

Những mục này không nằm trong MVP nhưng kiến trúc thông tin nên đủ mở để bổ sung mà không phải đập đi làm lại.
