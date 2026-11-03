# CAD360 TBT — R25/30: Giảm cấp phát CPU khi vẽ CAD lớn

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Phạm vi
Ưu tiên CAD mouse/ESC/viewport; đọc file lớn, RAM, decode/render, zoom/pan, autosave/restore, Windows CI/Setup/Portable/Source, đăng nhập và cấu hình phân quyền tài khoản. Tiếp tục tạm dừng đồng bộ online/offline, xử lý conflict giữa các máy và clash/xung đột bộ môn.

## Các thay đổi được commit
1. `canvas/draw.rs`: thay `Vec::with_capacity(list.prims.len())` bằng `Vec::with_capacity(cpu_shape_initial_capacity(list.prims.len()))`; số phần tử dự trữ trước tối đa 4096, số hình được vẽ thực tế không bị giới hạn.
2. Đường vẽ CPU `Kind::Tris | Kind::Mask`: kiểm tra bounding box giao với view trước khi dựng egui Mesh. Tam giác và wipeout hoàn toàn ngoài viewport không cần cấp phát mesh. Wipeout giao với vùng nhìn vẫn giữ nguyên thứ tự.
3. Hai Rust unit tests: một triệu primitive không dự trữ một triệu Shape trước; kiểm tra bounding box tam giác trong/ngoài viewport. Windows CI có bước `canvas::draw::cpu_culling_tests`.

## Điều được và chưa được chứng minh
- Có sửa mã nguồn loại bỏ cấp phát dự trữ lớn ở CPU renderer mỗi frame và bỏ qua tạo mesh triangle/mask ngoài màn hình.
- Chưa đo peak RAM, số allocation, CPU ms/frame, GPU memory, FPS, P95 zoom/pan với CAD 100–500MiB; không kết luận tăng tốc bao nhiêu phần trăm.
- Worker rebuild, GPU mesh, parser decode vẫn còn các đoạn có thể tiêu thụ RAM lớn; các thay đổi R25 không phải streaming GPU.
- Test đã commit nhưng chưa có xác nhận Windows Cargo PASS hay artifact EXE thật.

## Tiến độ kỹ thuật, 8 nhóm đang hoạt động
| Nhóm | % ước lượng | Điều kiện nghiệm thu |
| --- | ---: | --- |
| Nhấp đầu tiên LINE/PLINE/CIRCLE/MOVE | 45 | GUI Windows E2E |
| ESC/kéo chọn/multi-viewport | 45 | Windows E2E |
| Mở CAD >512 MiB | 45 | File thật decode/render |
| Progress đọc và ETA | 70 | Giao diện và phase logs |
| Mượt CAD 300–500 MiB | 15 | FPS, peak RAM, input P95 |
| FIFO nhiều file và RAM | 50 | 5–10 file và peak RSS |
| Tài khoản và thiết lập phân quyền | 35 | Kiểm tra server-side mutation |
| Windows Setup, Portable, Source | 10 | CI SUCCESS, ba artifacts, checksum và chạy EXE |

Trung bình tám nhóm **39,4%**, kỹ thuật ước lượng chưa nghiệm thu. **25/30** nhóm thay đổi mã nguồn; còn 5 lượt trong chu kỳ kế hoạch, không phải gần hoàn thiện 83% sản phẩm.

## Commit
- `e78ed785fe240441aa6c92288e1ce53fd6d034df` — giới hạn initial capacity, cull Tris/Mask.
- `a3e7c8ddec0b9c67608dd4430574552d19d4ff06` — 2 unit tests.
- `22db948430e9a258cf79b9ecc76add365f698c40` — Windows CI targeted tests.
- `00825bd6c2b0fe07c6e581aa3379e4b2e6a8871d` — bảng tiến độ.

## R26
Đo và giảm allocation khi build GPU mesh, xác nhận rằng CPU culling không làm mất hình học ở viewport/Mask. Nếu có GitHub Windows run ID thì kiểm tra job logs và sửa theo lỗi build thật. Chưa phát hành EXE khi chưa có CI PASS.
