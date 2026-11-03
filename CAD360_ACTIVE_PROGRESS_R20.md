# CAD360 TBT — R20/30: Nhấp đầu tiên, ghép cặp chuột và kéo chọn

Ngày 03/11/2026. Nhánh: `tbt/cad360-active-release`.

## Phạm vi
- Active: sửa lỗi thao tác CAD LINE/PLINE/CIRCLE/MOVE, ESC, chọn bằng chuột, file lớn, tối ưu CPU/RAM, auto-save/recovery, Windows CI/Setup/Portable/Source, đăng nhập & cài đặt phân quyền tài khoản.
- Paused: đồng bộ nhiều máy, conflict reconciliation, clash giữa các bộ môn. Không xóa mã cũ.

## Thay đổi thực tế
1. `canvas.rs`: thêm hàm `primary_canvas_releases(events, rect, earlier_press_on_canvas)`. Duyệt tuần tự các `egui::Event::PointerButton`, theo dõi mỗi press và match release; loại release khi press khởi đầu ngoài canvas (toolbar/menu). Fallback chỉ dành cho press từ frame trước.
2. Khi đang chờ điểm CAD, cú click được lấy từ danh sách release hợp lệ hoặc `resp.clicked_by(Primary)`; không dựa vào global `pointer.primary_clicked()` khiến toolbar click bị áp thành điểm CAD.
3. Duy trì xử lý Dynamic Input option menu riêng; các điểm batched trước release cuối dùng tọa độ riêng.
4. `should_cancel_drag_window` nhận kết quả release có phải vùng canvas trên cùng, không phải menu/palette; khởi tạo drag bị chặn nếu ESC, focus loss, origin ở lớp UI khác hoặc trên menu.
5. Thêm ba Rust regression tests: 2 click nhanh ở các điểm khác nhau; toolbar press + canvas release bị bỏ qua; release từ press frame trước; CI `regression-tests` chạy `canvas::primary_gesture_tests`.

## Tiến độ 8 nhóm đang làm (chưa nghiệm thu)
| Mục | Ước lượng | Điều kiện PASS |
|---|---:|---|
| Click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45% | GUI Windows 100 lần/lệnh, không bỏ/trùng điểm |
| ESC/drag/focus không kẹt | 45% | GUI đa menu/palette, release ngoài cửa sổ |
| Mở file >512MiB | 45% | DXF/DWG thực lớn, decode/render hoàn chỉnh |
| Progress đọc và ETA | 70% | UI/phase logs và nghiệm thu |
| Pan/zoom CAD 300–500MiB | 15% | CPU/RAM/GPU/FPS/P95 đo thực |
| FIFO nhiều file và RAM | 50% | 5–10 bản vẽ, peak RSS |
| Đăng nhập và quyền tài khoản | 35% | Server-side mutation authorization tests |
| Windows Setup, Portable, Source | 10% | CI PASS, artifacts, SHA256, cài & chạy thật |

Trung bình **39,4%** từ 8 hạng mục, không bao gồm phần sync/conflict tạm dừng. Đây là ước lượng kỹ thuật, **không phải kết quả đo hay nghiệm thu**. Nhịp phát triển `20/30`, còn 10 lượt dự kiến.

## Kiểm tra Windows và mục tiêu R21
- Các bài test chỉ mới được commit; chưa có chứng cứ Cargo/CI Windows PASS hay GUI E2E PASS.
- GitHub workflow phải được kiểm tra trên nhánh active: build-core, regression-tests, unified-ui, bim-module-regressions, release-windows; artifacts Setup/Portable/Source chỉ tính khi tồn tại và SHA256 đúng.
- R21: kiểm tra CI và sửa lỗi build/test; tiếp tục xử lý các thao tác con trỏ thực, ESC và click-click selection. Nếu tìm lỗi khác thuộc cùng nguyên nhân thì xử lý theo nhóm chứ không sửa rời rạc.
- Không phát hành EXE production chỉ vì có commit.

## Chứng cứ GitHub
- `c699be8066b6597d47bc6633a4443be15fc0b23d` — raw event pairing + 3 tests.
- `232b0e366fb77e287087485c533dcc6be82252e4` — only genuine canvas click for point prompts.
- `7c8d811e7a0330c0985bd1e35db6e302b4c832df` — drag overlay/ESC/focus safeguards.
- `1ce77e689fb866cca3958c11375d70f208e18245` — Windows CI targeted gesture tests.
- `ed9787e65025a84ea76cdaf0e1ca327c5a1b92b1` — tracker.
