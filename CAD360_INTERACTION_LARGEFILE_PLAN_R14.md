# CAD360 TBT — Kết quả R14/15: quyền cộng tác và khóa đối tượng

Ngày: 03/11/2026. Branch: tbt/cad360-active-release.

## Đã cập nhật mã nguồn
- `can_edit_active_view` hiện yêu cầu user thuộc phiên đăng nhập còn hợp lệ, bật `UserAccess.can_edit`, và được phép sửa scope tầng/bộ môn. Rule edit đơn độc không còn đủ nếu checkbox cho phép sửa bị tắt.
- `enforce_selection_scope` thu hồi selection khi user bị tắt edit, phiên signed-out, account mismatch, hoặc slot chỉ tham chiếu. Lọc các đối tượng không nằm trong `primary_handles`.
- `can_view` của chính chủ cũng yêu cầu scope read rule, không tự động mở quyền xem khi AccessRule bị thu hồi.
- `sync_render_filters` kiểm tra `can_view` trước khi đưa entity vào primary list và tôn trọng `show_other_users=false`.
- `acquire_lease` từ chối khóa do user chỉ có quyền xem tạo ra; cùng tài khoản chỉ tái dùng lease nếu revision đúng.
- Ba regression tests đã viết: thu hồi quyền xem; UserAccess.can_edit=false dù rule edit=true; read-only/stale revision lease.

## Cảnh báo quan trọng
Các thay đổi là bảo vệ trong mô-đun cộng tác ở ứng dụng desktop. Mọi mutation trực tiếp của CAD engine hoặc request server phải được xác thực riêng; client-side selection/render filters KHÔNG bảo đảm quyền ghi phía server. Đồng bộ online trong UI hiện là mô phỏng; chưa có bằng chứng nhiều máy hoạt động thực tế. Các test mới chưa có kết quả PASS trên runner Windows.

## Tiến độ chi tiết (ước lượng kỹ thuật, không phải nghiệm thu)
| Mã | Hạng mục | % R14 | Đã có | Điều kiện còn thiếu |
|---|---|---:|---|---|
| I1 | Nhấp đầu tiên LINE/PLINE/CIRCLE/MOVE | 45% | Input guards R07–R12 | GUI Windows thật |
| I2 | Không kẹt chuột, kéo chọn, ESC | 45% | Drag cancellation R08–R12 | GUI focus/drag |
| L1 | Mở file không chặn trần 512MiB | 45% | Progressive read | Decode/render >512MiB |
| L2 | Thanh byte % và ETA | 70% | Phase callback đúng | Parse/render progress |
| L3 | Pan/zoom/select CAD 300–500MiB mượt | 15% | Reader nền | RAM/VRAM/FPS/P95 benchmark |
| L4 | Nhiều file không chồng giải mã | 50% | Serial FIFO | Multi-file RAM test |
| C1 | Quyền user/floor/discipline/workset | 35% | Đã kiểm tra UI/scope và primary layer | Server mutation ACL, workset chi tiết |
| C2 | Lease/conflict cộng tác | 20% | Đã kiểm tra local lease revision | Server lease timeout, 2-client test |
| Q1 | Windows CI + Setup/Portable EXE | 10% | Code có trên GitHub | Cargo check/test, GUI, artifacts |

Trung bình sơ bộ 9 hạng mục: **37,2%**. Các % không nâng khi thiếu bằng chứng build/test thực. Nhịp đến mốc ĐƯỢC XÉT đóng gói **14/15=93,3%**, không phải mức độ hoàn thành phần mềm. Còn 1 lượt đến mốc tối thiểu và tối đa 16 lượt dự phòng đến R30.

## Điều kiện gate R15
1. Cargo check/test Windows cho source hiện tại — chưa có bằng chứng.
2. GUI smoke: click đầu tiên trên LINE/PLINE/CIRCLE/MOVE, ESC, focus và drag — chưa có bằng chứng.
3. DXF/DWG 100/300/500MiB, >512MiB benchmark: thời gian đọc/parse/render; peak RSS/VRAM, P95 input latency — chưa đo.
4. Bảo vệ server authoritative cho mọi ghi dự án, scope account/floor/discipline/workset, lease và conflict giữa 2 client — chưa có bằng chứng.
5. Tạo Setup.exe + Portable ZIP, SHA256, test tải/cài/khởi động — chưa có artifact xác minh.

Nếu R15 chưa đạt gate, ghi R15 là lượt kiểm tra phát hành và tiếp tục R16–R30; không gắn nhãn build EXE thành công khi chưa có artifact.

## Ước lượng
R15 còn 1 lượt (1–2 ngày công development theo giả định); việc nghiệm thu và packaging cần thêm 2–5 ngày công nếu có môi trường Windows, runner, dữ liệu benchmark và API server. Nếu cần cải tạo parser/render hoặc hoàn thiện server collaboration, có thể kéo dài nhiều lượt tiếp theo; chưa có ngày giao chắc chắn.

## Các commit
- `9ba1c2f46dfeb7f8815dfb7e80d09374adabf28b` quyền sửa/view và render primary.
- `7ad3221ddd197949850aae90f4058f6da7762ceb` regression tests và show_other_users.
- `9d94bbfe9c44edf502f86a06c16896b2209c7856` local lease revision & readonly guard.
- `fa81b70cbd85995adca1d494f2f19fe993e2f6ad` tracker R14.

GitHub commit workflow-run và statuses hiện không trả kết quả; không kết luận CI đã chạy hay pass.
