# CAD360 TBT — R21/30: Vùng chọn click-click, drag-stop, ESC và focus

Ngày 03/11/2026. Branch `tbt/cad360-active-release`.

## Đã sửa thực tế
1. `canvas.rs`: click sau khi kéo chọn không được gửi thêm như `idle_click`; không gọi `cycle_click` khi có `drag_window` cũ, hoặc frame mới báo `drag_stopped` ở chế độ selection.
2. Khi thả ngoài canvas, lên menu/palette hoặc không còn pointer position hợp lệ: xóa `pending_window` của drag, tránh cửa sổ chọn treo chờ click sau. Khi thả trong canvas hợp lệ, kết thúc vùng chọn bằng điểm tọa độ thật.
3. Thu hồi press-origin fallback khi có press mới; không tái dùng cùng press-origin cho hai release liên tiếp.
4. ESC và focus loss được xét trước khi click; không xử lý idle selection/cycle trong frame bị hủy.
5. Bổ sung các Rust regression tests: prior-frame release chỉ một lần; press mới ngoài canvas phủ nhận fallback; click-click hợp lệ; drag-stopped không đồng thời idle click; ESC/focus không dispatch. Thêm Windows CI test gate `canvas::click_drag_arbitration_tests`.

## Hạn chế cần ghi rõ
- Các thay đổi đã push nhưng **chưa có kết quả Windows cargo/test/GUI PASS** được xác minh.
- Các tests mới chỉ kiểm thử hàm điều phối, không phải 100 lần click E2E trên UI thật.
- Chưa có Windows artifact Setup/Portable/Source được xác minh, chưa đo hiệu năng CAD 300–500 MiB.
- Đồng bộ online/offline, xử lý conflict hoặc clash bộ môn vẫn tạm dừng. Chỉ tiếp tục account và quyền xem/sửa.

## Tiến độ 8 đầu việc chủ động (ước lượng, chưa nghiệm thu)
| Đầu việc | % | Còn thiếu |
|---|---:|---|
| LINE/PLINE/CIRCLE/MOVE click đầu tiên | 45 | Windows GUI 100 lần/lệnh |
| ESC/drag/focus, selection không dính | 45 | Windows E2E chuột và focus |
| Mở CAD không cap 512MiB | 45 | File thật >512MiB decode/render |
| Tiến độ đọc & ETA | 70 | UI parse/render measurement |
| CAD 300–500 MiB không lag | 15 | CPU/RAM/VRAM/FPS/P95 benchmark |
| Nhiều CAD & FIFO/RAM | 50 | 5–10 file RAM peak |
| Account & cấu hình phân quyền | 35 | Server authoritative mutation |
| Windows CI + 3 artifacts | 10 | Jobs PASS, EXE/ZIP và cài thử |

Trung bình **39,4%**, giữ nguyên vì chưa có bằng chứng nghiệm thu. Số lượt **21/30** là số nhóm thay đổi mã nguồn, KHÔNG phải phần trăm hoàn thiện.

## Các commit
- `194e912a099d5c58e4d81b2fc2be60f226acf897` — chặn drag release thành idle click, xóa pending khi thả sai.
- `c7f83d794d86e83362c07231615b127f1fd6a497` — fallback press-origin một lần + 2 tests.
- `e5ca33d26475bf5f9cd44774d2139f50e59a0811` — ESC/focus hủy dispatch.
- `84d48eec47ff4791d9f2c391c98e752fb3cb18b0` — 3 tests click/drag arbitration.
- `501ee9f06799035a2bab5ea93cafa989d37342bc` — Windows targeted test.
- `56bd7620ae3a8bf341046f28ba64742da9772223` — tracker R21.

## R22 ưu tiên
- Kiểm tra nhấn/thả thực tế trong UI khi có menu, nhiều viewport, các lệnh vẽ; nếu có Windows CI logs thì chữa lỗi compiler và test theo root cause.
- Tránh tiếp tục tăng số lượt mà không có artifacts; nếu có run, xác minh SHA256 và cài chạy.
- Sau đó chuyển trọng tâm sang file CAD lớn, render, RAM, autosave và phục hồi.
