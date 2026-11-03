# CAD360 TBT — R22/30: Cách ly click giữa nhiều viewport CAD

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Phạm vi
Tạm dừng phát triển đồng bộ online/offline, xử lý xung đột thay đổi giữa nhiều máy và clash/xung đột bộ môn. Giữ đăng nhập, phân quyền account, cấu hình xem/sửa và scope. Ưu tiên lỗi con trỏ, lệnh CAD, đọc file lớn, tối ưu RAM/renderer, autosave/recovery và Windows Setup/Portable/Source.

## R22 đã làm
- Trước R22: điều kiện chuyển từ viewport hiện tại sang viewport khác dựa vào click toàn cục cùng vị trí hover. Điều đó có nguy cơ nhận cú click từ toolbar hay kéo qua viewport khác.
- Sau R22: `click_tile(i)` chỉ được phép khi release bắt nguồn từ một press hợp lệ **trong cùng viewport đích**; dùng hàm ghép cặp `primary_canvas_releases`, kiểm tra topmost layer và loại click trên menu nổi.
- Thêm hai regression tests: nhấn ở viewport trái rồi thả viewport phải không đổi viewport, và nhấn toolbar rồi thả lên viewport không đổi viewport.
- Cả hai test nằm trong `canvas::primary_gesture_tests`, hiện được workflow `.github/workflows/main.yml` chạy trong nhóm regression trên Windows, bên cạnh các kiểm thử LINE/CIRCLE/ESC/Open file.

## Tiến độ 8 hạng mục đang ưu tiên
| Hạng mục | % kỹ thuật ước lượng | Điều kiện còn thiếu |
|---|---:|---|
| First click LINE/PLINE/CIRCLE/MOVE | 45 | GUI E2E 100 lần/lệnh |
| ESC/focus/drag và nhiều viewport | 45 | Windows E2E và đa viewport |
| Mở CAD >512 MiB | 45 | File thực tế decode và render |
| Progress/ETA đọc & decode | 70 | UI acceptance; % render vẫn chưa có |
| 300–500 MiB zoom/pan/select mượt | 15 | RAM, VRAM, FPS, P95 latency |
| Queue FIFO nhiều file/RAM | 50 | 5–10 files thực, peak RSS |
| Tài khoản và thiết lập quyền | 35 | Server authoritative mutation guards |
| Windows CI, Setup, Portable, Source | 10 | Các Windows jobs PASS, 3 artifacts, SHA256, cài chạy |

Trung bình tám nhóm: **39,4%**, chưa nâng vì chưa có benchmark/GUI/CI PASS mới. Đây không phải % phần mềm nghiệm thu; `22/30` là số lượt đã commit, còn tám lượt trong kế hoạch dự phòng.

## Windows release gate
- Cần xem GitHub Actions đúng SHA trên nhánh active, sáu nhóm Windows build/test và job phát hành ứng viên đều SUCCESS.
- Cần tải thực tế ba artifacts `Setup.exe`, `Portable.zip` và `Source.zip` cùng SHA256; cài và chạy trên Windows x64; thử LINE/PLINE/CIRCLE/MOVE, multi-viewport, ESC, loading CAD lớn, autosave/recovery.
- Hiện chưa có bằng chứng PASS hay đường tải EXE đã xác minh từ công cụ truy vấn. Không coi commit là build PASS và không coi source archives mô tả trong workflow là file đã phát hành.

## Chứng cứ nguồn
- `bde137eac01c4d3e91fa0cd7c24e8eecdaaebfa6` — sửa điều kiện chuyển viewport.
- `8c6028f5e2792f86e9a99a14237a7bb42ee01357` — 2 regression tests.
- `b3b3eac0f327ce9b56b2043179ff1ef3892d5e06` — theo dõi R22.

## Hướng R23
Kiểm tra hiệu năng đọc/parse/dựng hình CAD lớn, bổ sung số đo thực tế và tối ưu memory allocation. Không bắt đầu lại nhóm đồng bộ/xung đột.
