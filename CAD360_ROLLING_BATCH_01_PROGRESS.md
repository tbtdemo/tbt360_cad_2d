# CAD360 TBT — Chu kỳ tối ưu mới, phát triển song song đóng gói Windows

Ngày 03/11/2026; nhánh: `tbt/cad360-active-release`.

## Quy tắc thực hiện
- Tiếp tục sửa mã nguồn CAD/Rust trên GitHub liên tục, không chờ Windows packaging. Các nhóm sửa độc lập được commit ngay để bảo toàn dữ liệu.
- **Cứ đủ 15 bộ thay đổi chức năng mới kích hoạt một bản Setup EXE + Portable ZIP + Source ZIP duy nhất**, tối đa 30 bộ nếu các bản sửa được gộp trong một lần build đã chủ động chốt.
- `build(cad360): B15` dành cho mốc ứng viên 15 bộ; `build(cad360): B30` dành cho mốc 30 bộ. Các commit bình thường không kích hoạt installer.
- GitHub Direct EXE đã dùng `cancel-in-progress: false` và nhánh-specific SHA group; một lần đóng gói đang chờ sẽ không bị hủy chỉ vì tiếp tục commit mã mới.
- Bản Windows đầu ra chỉ báo “đã đóng gói” nếu thực sự có artifact Setup/Portable/Source và checksum; không dùng trạng thái source-preflight hay run queued để báo PASS.
- Tiếp tục giữ tài khoản/phân quyền, CAD mouse/ESC/viewports, mở file lớn, RAM/FPS/P95, autosave. Tạm dừng phát triển sync/conflict/clash.

## Bộ thay đổi chức năng trong chu kỳ kể từ sau R30

| Bộ | Nhóm | Thay đổi được commit | Regression |
| ---: | --- | --- | --- |
| 01 | CAD physical click | Dùng vị trí sự kiện thả chuột thật khi frame giật thay vì hover trôi | `canvas::point_event_position_tests` |
| 02 | CAD 1–4 viewports | Quét model một lần cho nhiều tile, chỉ kiểm owner visibility một lần/đối tượng | `bim_collab::tests::render_bucket_keeps_primary_and_reference_scope_separate_for_four_views` |
| 03 | CAD click burst/overlay | Lọc press/release lên menu trước khi phân phối click vào LINE/PLINE; bỏ CPU allocation lúc idle | `canvas::eligible_point_release_tests` |
| 04 | Render/GPU | Không cài DisplayList/GPU Mesh worker khi kết quả thuộc zoom-band hoặc vùng nhìn trước đó | `canvas::rebuild::stale_background_view_tests` |
| 05 | Đọc CAD lớn | Retry `io::ErrorKind::Interrupted` thay vì chấm dứt OPEN, giữ nguyên direct-to-buffer | `capped_read::tests::transient_interrupted_read_retries_without_losing_geometry_bytes` |
| 06 | FIFO/multi-file | Dispatch file đầu hàng đợi trực tiếp, không tạm drain toàn queue, giữ một decoder tại một thời điểm | `opening::tests::ten_queued_files_preserve_fifo_order_with_one_decoder_at_a_time` |
| 07 | RAM khi chuyển tab CAD | Thu hồi ngay PrimIndex cũ trong cả install từ background và rebuild đồng bộ, không đợi hover/selection | `canvas::tests::prim_index_releases_large_old_drawing_after_switch_with_empty_selection` |
| 08 | Tiến độ mở nhiều file/UI | Thay ID loading overlay thay đổi mỗi frame bằng ID ổn định; Việt hóa thông báo file và hàng đợi | `opening::tests::card_text_names_the_file_and_counts_queued_ones` |

**Đếm: 8/15 bộ chức năng cho mốc đóng gói tiếp theo.** Một bộ có thể gồm nhiều commit; các commit CI, docs và test không được tính thêm thành bộ độc lập. Hiện còn 7 bộ chức năng cần phát triển trước mốc `build(cad360): B15`.

## Cam kết nghiệp vụ và hạn chế nghiệm thu
- Các thay đổi ở trên **đã được ghi trên nhánh GitHub**, chưa xác minh Rust Windows test PASS; không tự tăng phần trăm hoàn thành.
- Tám nhóm kỹ thuật baseline vẫn [45,45,45,70,15,50,35,10], trung bình 39,375% (khoảng 39,4%) **ước lượng chưa nghiệm thu**.
- Mốc B30 cũ: <https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41593081243>. Khi kiểm tra gần nhất bước kiểm kê mã nguồn success, Windows Setup queued, artifact chỉ là inventory, **chưa có EXE thật**.
- Mốc đóng gói mới không chờ mốc cũ; lúc đủ 15 bộ có thể chạy run B15 theo commit riêng với SHA đúng phiên bản.
- Phải có GUI E2E first-click/ESC, bản vẽ lớn 300–500/>512 MiB, benchmark RAM/FPS/P95, kiểm quyền mutation server/API và cài Windows thực mới coi là nghiệm thu sản phẩm.

## Commit chính của 3 bộ mới
- `5f68b49ead8b4c812d5499925c15211d138d4cca` — reject stale GPU mesh sau zoom/pan.
- `c1c62f17771c1bd9124a0b45c13aa251970469cb` — stale mesh test.
- `37b4f980d7d5c2419eeda46b5801a15af7390051` — retry Interrupted.
- `3c236eb6d6a2f8632a13eeabb437eaeba0b006ac` — FIFO dispatch.
- `6dfb6df21c591559cb5d6524f70a663d9b862c63` — ten-file FIFO test.
- `5d9b335ca6d787a14f2da3d9dad7772bd268a6cc` và `3fc02a9cf92e7ce9e218faf802356678262a2c4c` — Windows targeted test/discovery gates.

## Cập nhật tiếp — hai bộ 07–08
- `f9fc5051489b54a064b32e5417b821b12cc2b84e`: `PrimIndex::prims` thả Vec index cũ theo list generation, kể cả khi selection trống.
- `a6cc5ede1596c4eb8c049d8d69ae19d39edbcc49`, `b0a388f68a53bcc94376134e6b589b6df1cc442d`: thu hồi index **ngay trong hai nhánh install** thay vì đợi gọi highlight về sau.
- `1757197f48574dc76120736f92d20f2fea7c53ab`: test chuyển từ 25.000 primitive về bản vẽ 2 primitive, không còn reserve index cũ.
- `9a78a2592f5f95a136ff5b76a3b66131f5fb789a`: dùng `egui::Id::new("cc_opening")` thay `Id::unique`, tránh thay ID của overlay mỗi frame; hiển thị `Đang mở ...` đúng tiếng Việt, vẫn có bytes/ETA chỉ cho giai đoạn đọc.
- `a72db470c60388d70e2ce4b413c10411706886f0`: thêm Windows CI discovery và targeted test cho giải phóng highlight index, cùng các tests loading card.
- Chưa xác minh Rust Windows CI PASS hoặc bộ EXE có thể tải; đếm 8/15 bộ thay đổi là số công việc code, KHÔNG phải % nghiệm thu. Tám nhóm hoạt động vẫn 39,4% ước lượng chưa nghiệm thu.
