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
| 09 | CPU selection highlight | Không tạo CPU shapes/tô sáng cho hình học được chọn nằm hoàn toàn ngoài viewport, vẫn clip Infinite/Ray riêng | `canvas::draw::highlight_view_culling_tests` |
| 10 | GPU selection highlight | Dự trữ có giới hạn buffer quads khi tô sáng nhiều đối tượng, tránh nhiều lần Vec grow, không giới hạn số đoạn | `gpu::gpu_memory_budget_tests::gpu_highlight_preserves_selected_segments_after_bounded_reservation` |
| 11 | CAD lớn / an toàn hình học | Nếu nguồn file bị thu ngắn trong quá trình OPEN thì dừng với UnexpectedEof trước khi decode, không nhận hình học thiếu | `capped_read::tests::progressive_read_rejects_truncated_geometry_before_decode` |
| 12 | CPU renderer / frame-time | Culling hình học nằm ngoài viewport trước khi chuyển ACI/transparency và cấp phát CPU Shape theo mỗi frame | `canvas::draw::highlight_view_culling_tests` và tests CPU culling hiện có |
| 13 | FIFO / mở file lặp | Gộp các lượt OPEN trùng đường dẫn trong lúc file đang đọc hoặc đang đợi, tránh giải mã thêm một bản không cần thiết | `opening::tests::repeated_open_clicks_do_not_queue_duplicate_cad_decoders` |
| 14 | Nhất quán file CAD đang thay đổi | Đối chiếu file size và modified-time trước/sau đọc nhiều khối; không giải mã hỗn hợp khi nguồn bị ghi tiếp | `capped_read::tests::opening_a_growing_cad_file_must_not_return_mixed_geometry` |
| 15 | Windows OPEN khử trùng đường dẫn | Trong hàng đợi OPEN trên Windows, coi slash/backslash và khác chữ hoa/thường là cùng yêu cầu khi đang chờ | `opening::tests::pending_path_comparison_respects_platform_file_system_conventions` |

**Đếm: 15/15 bộ chức năng, đã đạt mốc kích hoạt đóng gói B15.** Các commit kiểm thử, CI và tài liệu không tính thành bộ riêng. Các bộ thay đổi tiếp theo được tích lũy cho chu kỳ kế tiếp, không dừng phát triển để đợi EXE.

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

## Cập nhật — hai bộ 09–10: tô sáng bản vẽ lớn
- `8dc4bf9be746dd9b81cd8381977885cbcfd8353c`: `canvas/draw.rs`, CPU highlight culling bỏ qua Polyline/Point/Triangles/Mask nằm hoàn toàn ngoài viewport; Infinite/Ray vẫn dùng `clip_infinite`; không cắt vật thể có bbox cắt viewport. Test thêm cho 5 trường hợp.
- `a02774cc0886ac1285225700674fa39897213a1b`: `gpu.rs`, dự trữ trước `MeshData.quads` khi highlight nhiều selected primitive, giới hạn mức dự trữ ban đầu 1 MiB (không giới hạn nội dung thực tế); test giữ đúng 2048 selected line segments và chống dự trữ 100 triệu quads.
- `dbadbeccc14e8a618df7cccfad0f48ad69c73923`: Windows CI thêm discovery và run tests CPU/GPU highlight.
- **10/15 bộ được commit**, chưa có kết quả Windows native Cargo PASS/GUI benchmark thực tế nên tỷ lệ 8 nhóm giữ baseline 39,4%. 5 bộ tiếp theo sẽ ưu tiên CPU frame time, mở CAD >512MiB, tiết kiệm RAM đa tab, permission và ESC viewport.

## Cập nhật — ba bộ 11–13
- `28b77cb9a046327e716fe2a06e271bb82305b097`: `read_progressive` từ chối file bị thu ngắn so với metadata ban đầu bằng lỗi `UnexpectedEof`, không phát một mảng DXF thiếu cho decoder; vẫn cho phép file tăng kích thước tới EOF và báo số byte thực. Có test cho equal/grow/truncate.
- `5913187e3d346aa231585ec11420f1bc2b23a44d`: `draw_list_filtered` dùng chung culling hình học trước tính ACI/transparency và cấp phát Shape, tránh xử lý màu cho đối tượng hoàn toàn ngoài khung nhìn. Không làm thay đổi clipping riêng của Infinite/Ray.
- `9f8569d65c896cb7075081f0f1c68dfd984868c2`: `Opening::start` chống lặp đường dẫn khi đã có worker hoặc queued. Không ảnh hưởng mở lại sau khi hoàn tất toàn bộ yêu cầu; cùng một path lặp trong lúc chờ chỉ xử lý một lần. Có test FIFO chống duplicate.
- `73590eb2fb33dce851c8a5e743630044670691f5` và `c4b1bd1cc2fe40c9534c6c8c558d33f42696495e`: cập nhật CI phát hiện và chạy regression tương ứng.
- **13/15 bộ thay đổi chức năng đã được commit, không tính test/docs/CI là bộ thêm.** Windows CI/GUI và FPS/P95/RAM trên file 300–500 MiB vẫn chưa xác minh; giữ 8 nhóm baseline 39,4% chưa nghiệm thu. Sau 2 bộ tiếp theo, kích hoạt B15 một lần, không chờ B30 cũ.

## Chốt B15 — thay đổi 14–15 và điều kiện build ứng viên
- `500c1686a69d0b4bc4cea716eefebcde531afb76` (bộ 14): `read_progressive` xác minh kích thước và thời gian sửa file trên chính file handle trước/sau đọc, từ chối file đang bị thay đổi với `InvalidData`. Test thực hiện thêm dữ liệu giữa lúc tiến trình đọc file 2 MiB; vẫn còn giới hạn không phát hiện được bản sửa bảo toàn cùng kích thước và timestamp.
- `03eaebf5e4aeae0f09721608b775edee0cb51278` (bộ 15): Windows OPEN so sánh các path đang đọc/queued không phân biệt slash/backslash và chữ hoa/thường; Unix giữ so sánh chính xác để không gộp nhầm file khác. Test riêng hai nền tảng.
- `d6c47e72c2f125a883ad5c04bff1e7175a030a1f`: Windows CI bắt buộc phát hiện và chạy hai regression mới.
- **Mốc 15/15 đạt bằng thay đổi mã nguồn có chức năng**, không tính số commit thành phần trăm. Kích hoạt riêng commit `build(cad360): B15` để chạy một lần Setup EXE, Portable ZIP, Source ZIP cùng SHA256, không xuất bản production khi chưa nghiệm thu.
- Bản ứng viên B15 và các commit tiếp sau có thể chạy song song; Windows build thật và artifacts được báo PASS chỉ sau khi GitHub hoàn thành, không dựa trên queued/preflight.

## Mốc đóng gói B15
B15 gồm 15 bộ chức năng đã ghi ở trên. Build Windows là ứng viên kiểm thử, độc lập với các commit tối ưu tiếp theo, không công bố thành bản phát hành cho tới khi có artifact thật, SHA256 và kiểm chứng GUI Windows.

## Chu kỳ sau B15 — tiếp tục nâng cấp trong khi GitHub đóng gói
- **01/15 bộ của chu kỳ mới**: `5bcfb1f9810248717bbeb33acc916a78d3bfbd22` hoãn dựng lại `VpContent` đồng bộ khi zoom/pan MSPACE đang diễn ra; sử dụng tạm Display List đã có với phép biến đổi viewport và chỉ dựng lại sau 150 ms ổn định. Bản vẽ mới/chưa có cache vẫn dựng lần đầu như cũ. Điều này giảm nguy cơ nghẽn GUI khi xuyên qua zoom-band 2x hoặc khi kéo ra khỏi vùng cache.
- `b34f5a10f307465c1e2e9c66d027911a5ac74e59`: unit test kiểm zoom x4, pan ra khỏi vùng cache, zoom x1.3 và chính xác ngưỡng ổn định 150 ms, thêm CI discovery/target.
- **Mốc ứng viên B15 cố định ở commit `51d41f51f3dd0a23fc2b0ee0ece24b9ecf11c7ae`**; mã của chu kỳ mới nằm trên các commit sau, KHÔNG sửa nội dung B15 đang build.
- Chu kỳ mới **1/15** chỉ phản ánh thay đổi chức năng đã commit. Vẫn còn 14 bộ tới mốc đóng gói ứng viên kế tiếp. Mức 39,4% của 8 nhóm là baseline ước lượng chưa nghiệm thu Windows/GUI/benchmark.

## Tiếp tục build + tối ưu — chu kỳ mới 3/15
- **Bộ 02/15 — Peak RAM của OPEN lớn:** `8968eec8739cd0f647dad444a454b5bb84a3254d`: chuyển `read_progressive` sang `try_reserve_exact`, không tự đẩy Vec từ 300–500 MiB lên mức tăng cấp số nhân chỉ vì yêu cầu thêm 1 MiB; loại bỏ lượt cấp phát/đọc thử thêm một khối 1 MiB sau khi đã đủ kích thước snapshot, trước khi kiểm tra file bị thay đổi. Regression đọc file >11 MiB nhiều khối, đối chiếu nguyên dữ liệu và capacity. `2294f2556d98cf6344c21b99f2087d3e961a2131` thêm vào CI.
- **Bộ 03/15 — CPU frame time polyline:** `84bc15d439213c804a3d74e947d6754a23149f44`: đường `draw_list_filtered` chỉ tính Bounds2 AABB một lần/primitive, tránh duyệt mọi đỉnh polyline hai lượt mỗi frame trong CPU renderer. Vẫn clip Infinite/Ray riêng, giữ culling sớm trước ACI/transparency.
- **3/15 bộ mã nguồn sau B15.** Chưa báo vượt baseline 39,4% kỹ thuật vì chưa benchmark RAM/FPS/P95 và chưa chạy Windows GUI. Các commit tiếp theo không làm thay đổi SHA của B15 đang queued.
- Windows build B30 run 41593081243: preflight PASS, job `Validate BIM conflict and revision regression tests` còn in_progress tại lần đọc; chưa đến cargo release build, EXE chưa có. B15 run 41601992146: queued, chưa có artifact. Không nhận định PASS trước khi có kết quả thực tế.

## B15 — Build sửa lỗi Rust E0308
- Log B30 Windows cho thấy `apps/cadcraft/src/cad_bim_recovery.rs:164` gọi `cadcraft_io::write(doc, ...)` với `doc: &DocState`, còn API yêu cầu `&Drawing` nên build thất bại E0308.
- `aa84e26a81230f5d871e98ea5d42e99487b84eab`: sửa bằng `doc.recovery_drawing()` và `drawing.as_ref()`. Điều này giữ hình học BEDIT chưa ghi, không bỏ các chỉnh sửa chưa lưu, đồng thời đúng chữ ký hàm.
- Kích hoạt lại candidate B15 bằng commit riêng có tiền tố `build(cad360): B15`, một bản Setup + Portable + Source + SHA256; **không phát hành production**. Các run cũ dùng SHA trước fix sẽ không tự lấy sửa đổi.
- Chưa có xác nhận Windows PASS/EXE thật. Không tăng % nghiệm thu từ việc sửa mã.

## Tiếp tục tối ưu sau khi kích hoạt B15 — chu kỳ mới 5/15
- **Bộ 04/15 — CAD+BIM recovery checksum:** `badd29493778d2ad003721054969541f036348f6`: `content_checksum_file` kiểm loại file, số byte, kích thước và thời gian sửa đổi trước/sau khi đọc 64 KiB từng khối; thử lại khi `Interrupted`. Giữ khả năng phát hiện dữ liệu sai checksum, giảm khả năng chấp nhận file bị viết lại trong lúc xác minh. Bổ sung regression bắt lỗi E0308: `DocState::recovery_drawing()` → `&Drawing` → DXF export.
- **Bộ 05/15 — bound recovery metadata:** `07c9444ad48dd5e8f4e62c922c3ff2c4feccd450`: giới hạn `commit.json` 1 MiB, `workspace.json` 64 MiB trước khi đọc, dùng cho cả format 1/2, tránh tốn RAM do metadata phục hồi hỏng; regression với metadata vượt ngưỡng.
- `039ac147f4b953af1ed2022beb3344d59e4cb2f1` và `8a6415d7f75720d67c92ab8da9e382e359b7d0b9`: Windows CI thêm test recovery export, checksum, bounded manifest. `9967f7e9b69aa335897e6f20e7423838ce02effb`: source CI mới được phép hủy source CI cũ đã bị thay thế để tránh tranh chấp runner; Direct EXE B15 vẫn giữ per-SHA `cancel-in-progress: false`.
- **5/15 bộ chức năng cho chu kỳ tiếp theo**, còn 10 bộ đến mốc đóng gói kế. B15 sửa E0308 đã kích hoạt run 41604832969, source preflight SUCCESS, Windows Setup queued ở lần kiểm tra hiện tại; bản B15 cũ 41601992146 vẫn queued. Cả hai CHƯA có EXE; source inventory 248 byte không phải bộ cài.
- Chưa có Windows compile PASS cho các commit sau B15, chưa đo RAM/FPS/P95 bản vẽ thực. Không tăng % nghiệm thu kỹ thuật.
