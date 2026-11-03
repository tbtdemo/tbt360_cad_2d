# CAD360 TBT — R28/30: Không mất bản phục hồi khi thoát sau lỗi lưu

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Phạm vi R28
- Active: autosave / phục hồi đúng hình học của toàn bộ CAD đang mở, bảo vệ dữ liệu khi lưu lỗi, tài khoản và cấu hình phân quyền, Windows CI/Setup/Portable/Source, các regression chuột và file CAD lớn.
- Paused: đồng bộ online/offline, xử lý conflict giữa các máy/client, clash/xung đột bộ môn. Không làm lại các nhóm dừng.

## Lỗ hổng dữ liệu phát hiện
Trong `App::on_exit`, kết quả `paired_recovery.save_now` được log khi thất bại nhưng sau đó vẫn gọi `cad.shutdown_autosave()`. `Autosaver::shutdown` chờ workers xong rồi xóa các file `.sv$` đã ghi, kể cả khi CAD+BIM paired snapshot cuối lỗi. Nếu bản vẽ vẫn dirty, đây có thể là bản sao duy nhất có thể phục hồi sau restart. Lỗi xảy ra trong trường hợp ổ đĩa đầy, không ghi được DXF/metadata hoặc thiếu RAM.

## Mã nguồn sửa trực tiếp
1. `crates/engine/src/autosave.rs`: thêm `Autosaver::shutdown_preserving_recovery()` và `shutdown_with_recovery(preserve)`, cùng join các background writers. Khi `preserve=true`, không xóa file autosave; vẫn giải phóng instance lock để lần mở tiếp theo có thể phát hiện những file phục hồi. Khi thoát sạch, `shutdown()` vẫn dọn file như cũ.
2. `crates/ui-egui/src/autosave.rs`: thêm `CadApp::shutdown_autosave_preserving_recovery()`.
3. `apps/cadcraft/src/main.rs`: sau `paired_recovery.save_now`, xác định có bản vẽ dirty hay không. Nếu vừa dirty vừa lỗi snapshot cuối, thử `Autosaver::save_now(&session)` (ghi DXF khẩn cấp nếu SAVETIME cho phép), rồi chờ worker và giữ lại file thay vì dọn; trong các trường hợp còn lại tiếp tục cleanup chuẩn.
4. Thêm regression test `autosave::native::exit_recovery_tests`: file mô phỏng còn sau shutdown bảo toàn; file được dọn sau shutdown thành công.
5. Windows CI target `cargo test --locked -p cadcraft-engine autosave::native::exit_recovery_tests -- --nocapture`. Full workspace tests vẫn giữ.

## Những giới hạn phải ghi đúng
- Bản snapshot CAD+BIM định kỳ 5 phút vẫn encode các DXF trên luồng UI (dù đã ghi từng tab tuần tự từ R27), chưa có background-only paired snapshot, chưa chứng minh loại hết UI stall trên file lớn.
- Việc lưu khẩn cấp có thể không tạo được file khi `SAVETIME=0`, không có đường dẫn autosave hoặc lỗi I/O/đĩa đầy. Khi đó chỉ các file đã có từ trước được giữ nguyên.
- Các kiểm thử hồi quy mới đang ở source và cấu hình workflow, chưa có xác nhận Windows Cargo/test PASS, GUI crash/restart E2E hoặc EXE artifact.
- Vì vậy không khẳng định phần mềm không bao giờ mất dữ liệu; kiểm soát một đường lỗi có khả năng xóa nhầm bản phục hồi.

## Tiến độ 8 nhóm ưu tiên
| Nhóm | Ước lượng kỹ thuật (chưa nghiệm thu) | Gate |
| --- | ---: | --- |
| LINE/PLINE/CIRCLE/MOVE first click | 45% | 100 lượt GUI Windows |
| ESC/drag/focus/multi-viewport | 45% | GUI E2E |
| Mở CAD >512MiB | 45% | DXF/DWG thật |
| Progress/ETA mở file | 70% | UI acceptance |
| CAD 300–500MiB FPS/P95/RAM | 15% | Benchmark |
| Queue nhiều file/peak RAM | 50% | 5–10 DXF |
| Tài khoản/cấu hình phân quyền | 35% | Server-side authorization tests |
| Windows CI/Setup/Portable/Source | 10% | SUCCESS + 3 artifacts, SHA256 và cài thử |

Trung bình 8 nhóm **39,4%**, ước lượng chưa nghiệm thu. Autosave là ưu tiên bổ trợ R27–R28, không thêm vào mẫu số cố định; `28/30` chỉ là lượt phát triển, không phải mức hoàn thiện.

## Commits R28
- `9a49287e91a5e1ce74585f2f8eebcea4fcecce6d`: giữ bản autosave khi thoát không an toàn.
- `ec1f780ed8b90e179b8728505ebb708ac7a6c89a`: API ở lớp UI.
- `cb1d9c2b0ede8adf11527e1102927f22d61f70a7`: sửa on_exit sau lỗi lưu.
- `b8d07268e99d8d7ff9171b2aa8af46c45116e83e`: thử autosave khẩn cấp.
- `8c073c9166dc10f515c6155efd22dc6578562d43`, `7b65dbdc9095077738adfcc8afd0302eff5c3bec`: test và module chuẩn.
- `6a7be12fe0352bfd116430270b3b0853bf1f4782`: CI target.
- `c19c7d7e910b8966da29180fcf6298b6ab9ee3cd`: tracker.

## R29–R30
- Kiểm tra quyền tài khoản, cấu hình và các đường mutation thực; không làm conflict resolution.
- Tập trung cao nhất vào Windows CI logs/artefact, sửa các lỗi biên dịch/test theo log xác thực, kiểm thử mở CAD lớn, chuột và crash/restart.
- Chỉ đóng gói và báo EXE giao được sau khi có artifact thật với SHA256 và chạy được trên Windows; không coi 30/30 là tương đương hoàn thành.
