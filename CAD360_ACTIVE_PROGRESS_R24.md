# CAD360 TBT — R24/30: Giữ renderer hoạt động khi background rebuild lỗi

Ngày 03/11/2026; branch `tbt/cad360-active-release`.

## Phạm vi đã thống nhất
Ưu tiên sửa mouse/viewport, RAM, read/decode/render CAD lớn, autosave/restore, Windows CI/Setup/Portable/Source và cài đặt phân quyền tài khoản. Đồng bộ online/offline, xử lý xung đột giữa các máy và clash bộ môn tiếp tục **PAUSED**.

## Thay đổi mã nguồn được commit
1. Trong `canvas.rs`, thêm `failed_rebuild_key: Option<ListKey>`; sau khi worker trả kết quả hợp lệ và `install` thành công, xóa trạng thái lỗi này.
2. Trong `canvas/list.rs`, khi `rebuild::Poll::Lost` (worker panicked/disconnected), không còn xóa display list cũ để ép `cadcraft_render::build` trên UI thread. Giữ hình học cũ và ghi warning.
3. Tránh khởi động lại worker lỗi mỗi frame với cùng view key. Khi thay đổi zoom/view key thì cho phép worker mới; thay đổi drawing content vẫn đi theo đường rebuild đồng bộ như trước, để bảo đảm hình mới không bị bỏ qua.
4. Thêm 2 tests kiểm tra không restart liên tục với key đã lỗi, và cho phép key mới/đường thay đổi nội dung. Windows CI chạy module `canvas::list::background_failure_tests` và `canvas::rebuild::tests`.
5. Trong `CadApp::finish_opening`, ghi log `CAD OPEN INSTALL: install_ms, status` riêng cho khâu thêm drawing, XREF vào Session. Kết hợp `read_ms/decode_ms` R18 nhằm tìm bottleneck thực.

## Giới hạn
- Việc giữ display list cũ khi worker lỗi giúp giao diện không fallback đồng bộ trên đường lỗi, nhưng vùng hatch hoặc độ chi tiết curve có thể tạm cũ cho tới lần thay đổi view và build hợp lệ.
- Khi drawing nội dung đổi thật, build đồng bộ vẫn còn: chưa có giải pháp render incremental/staged hoặc chunked GPU upload trên mọi đường.
- Các bài test có trong mã nguồn và workflow nhưng **chưa được xác nhận Cargo Windows PASS**.
- Chưa có số đo FPS, P95 input, peak RSS/VRAM, thời gian first-render của CAD 300–500MiB hoặc artifact EXE thực.

## 8 nhóm ưu tiên và % chưa nghiệm thu
| Hạng mục | Ước lượng R24 | Chứng cứ cần có |
| --- | ---: | --- |
| First click LINE/PLINE/CIRCLE/MOVE | 45% | Windows GUI E2E |
| ESC/focus/drag/đa viewport | 45% | GUI E2E |
| Mở CAD >512MiB | 45% | File CAD thật decode/render |
| Progress/ETA và phase logs | 70% | Giao diện UI theo từng giai đoạn |
| Pan/zoom/select 300–500MiB | 15% | P95 latency FPS RAM VRAM |
| Mở nhiều file và peak RAM | 50% | 5–10 file, leak/stall measurement |
| Tài khoản và cài đặt phân quyền | 35% | Server-side mutation authorization |
| Windows CI, Setup Portable Source | 10% | Run SUCCESS & 3 artifacts với SHA256 |

Trung bình tám hạng mục **39,4% kỹ thuật ước lượng**, không thay đổi do chưa có kết quả nghiệm thu. R24/30 là đếm 24 lượt, không phải % hoàn thành.

## Commit
- `fb825b081d333cfbb5e2cfd7d56f3d17347f7201`: failed rebuild state.
- `bb0389449b04b9e52253cc51dca215aa9b883cda`: fallback renderer.
- `1f951de396db9501fd68895e10818687724ca5da`: retry policy tests.
- `143aa3f3c7f1880cff7bbaaf4ed52c665fdfc73e`: log install_ms.
- `b74a92cd2d9749652ea976a98ca2ed4a9a51876a`: CI renderer tests.
- `3e3eea1be3e849ac8a8bc4de112ec51a3ec6c1c2`: tracker.

## R25
Tiếp tục phân tích peak RAM và copy/alloc trong display list/mesh, tối ưu giai đoạn tải GPU nếu đo thực cho thấy bottleneck. Không công bố hiệu năng tăng hoặc EXE sẵn sàng khi chưa có Windows tests và artifacts.
