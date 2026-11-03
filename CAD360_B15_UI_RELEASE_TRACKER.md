## B16 — Nâng cấp chức năng CAD 2D: QSELECT (03/11/2026)

**Phạm vi đã triển khai vào mã thực tế:** `SOURCE_CAD_R65/crates/ui-egui/src/quick.rs`.
1. Lọc nhanh Layer/Linetype/Color/Type trong menu giá trị, không phân biệt chữ hoa/chữ thường.
2. Menu chỉ tạo tối đa 100 nút lựa chọn mỗi lần vẽ; vẫn hiển thị tổng số kết quả tìm thấy, giảm tải cho bản vẽ có hàng nghìn layer.
3. `QSELECT.info` cache theo `drawing identity + revision + applyTo + object type`, cập nhật lại khi bản vẽ sửa.
4. Khi đổi loại đối tượng hoặc thuộc tính, xóa giá trị lọc cũ và loại bỏ trạng thái tìm kiếm cũ để tránh lọc sai.
5. Thêm 3 test: tìm không phân biệt hoa/thường, cap 100/tổng 500, không tìm thấy kết quả.

**Bằng chứng commit:**
- Mã QSELECT [e44c5dae](https://github.com/tbtdemo/tbt360_cad_2d/commit/e44c5dae1e89334545803713a341ed1d1d2fd126).
- Mốc đóng gói B16 [c2a372cf](https://github.com/tbtdemo/tbt360_cad_2d/commit/c2a372cf66dfd00a0bf8e2bd86274768b35c437c).
- [Ubuntu Windows cross EXE run 41635813565](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41635813565).
- [Windows Native installer run 41635813553](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41635813553).

**Thống kê B16 tại thời điểm kiểm tra:**
- 5 nhóm thay đổi QSELECT ở cấp mã nguồn: **5/5 đã commit**, chưa xác nhận test PASS.
- 3 bài test mới: **0/3 xác minh PASS** (đã viết, chưa có kết quả runner).
- 4 cổng bàn giao B16: 1/4 = **25% xác minh** (mã nguồn đã có; EXE build, artifact và GUI Windows chưa xác nhận).
- Không tái sử dụng artifact B15 làm bằng chứng EXE B16.


## Lượt sửa CI chung sau khi kiểm tra Windows — 03/11/2026

**Nguyên nhân lỗi kiểm thử UI:** Windows CI run [41634216686](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41634216686) cho thấy 6 lỗi biên dịch Rust `E0616` do `crates/ui-egui/src/canvas/tests.rs` truy cập trường private `PrimIndex.by_handle`. Điều này làm hỏng các job kiểm thử UI và các bước phụ thuộc `ui-egui`, dù bản release bin có thể build được.

**Đã sửa mã nguồn, không mở quyền cho trường private:**
1. `crates/ui-egui/src/canvas/list.rs`: thêm `#[cfg(test)] cached_primitive_count` và `cached_primitive_capacity`, commit `984c0927bccd02fc6259fedab5db87da910bb6fa`.
2. `crates/ui-egui/src/canvas/tests.rs`: thay 6 chỗ truy cập nội bộ bằng 2 API thống kê test-only, commit `a018d2dee33f114cad9bd0d9ab30e2123719db47`.
3. Workflow cross-build kiểm kê sự hiện diện của cả sửa backup Windows và UI test API: commit `00e366339eb3835db0561ef67e44578cb8458e6c` (`build(cad360): B15 ...`).

**Build phát sinh:**
- Ubuntu→Windows EXE: [run 41635166787](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41635166787), **queued** lúc kiểm tra.
- Windows native EXE: [run 41635167307](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41635167307), **queued** lúc kiểm tra.
- CI Rust UI mới: [run 41635133901](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41635133901), **pending**.

**Cổng bàn giao bản gồm cả hai bản sửa (BIM backup + UI test):** mã nguồn có commit 1/4 = **25%**; Rust compile và artifact EXE đúng commit `00e36633` chưa có xác nhận, kiểm thử hồi quy Windows chưa xác nhận PASS, GUI Windows chưa nghiệm thu. Không gộp với bộ cài `e8f7af28` trước khi sửa.

**Đếm kiểm thử cần xác nhận:** trước sửa, 83 BIM PASS / 5 BIM FAIL, ngoài ra 6 lỗi biên dịch E0616 trong `ui-egui`. Số lỗi đã được sửa ở mã nguồn không phải số lỗi đã nghiệm thu (chưa chạy PASS).


## Kiểm tra lại tiến độ B15 Fix — 03/11/2026

- **Commit mã sửa lỗi đang build:** `06f04db90d47bc1d150dc1653745fda3b7f243ef`.
- **Run Linux→Windows:** [41634295540](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41634295540): đang bước `Cross-compile real CAD360 desktop and CLI for Windows x64`, **chưa có artifact EXE** của commit này khi kiểm tra.
- **Run Windows Native:** [41634295714](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41634295714): inventory + source ZIP + preflight PASS, bước `Build CAD360 native release` đang chạy. Artifact source `13228943268`, preflight `13229642434`.
- **Cổng bàn giao bản sửa:** 1/4 (25%), chỉ xác nhận source. Test hồi quy 5 lỗi Windows chưa có kết quả PASS. Không nhầm artifact EXE từ commit `e8f7af28` (trước sửa) với bản fix.
- **Điều kiện nâng lên 75%:** cần Setup EXE và Portable ZIP đúng `06f04db9`, có artifact ID và checksum. **Điều kiện nghiệp vụ bổ sung:** cả 5 test phục hồi/lưu backup PASS (không thay thế GUI smoke-test).
- **Cần nghiệm thu cuối:** cài bản Windows và chụp giao diện đang chạy, xác nhận nhãn `CAD360 · 06f04db9`, thử 5 tab, lưu liên tiếp BIM workspace và phục hồi backup với thư mục thực của người dùng.

## Sửa lỗi Windows Native regression — mốc B15 Fix (03/11/2026)

**Lỗi thực tế được xác minh từ log Windows job `125889667162` run [41627300916](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300916):** 83 test PASS, 5 test FAIL; đều là `Access is denied (os error 5)` khi lưu hoặc phục hồi file backup BIM workspace.

- `corrupted_workspace_recovers_previous_valid_backup`
- `repeated_workspace_saves_replace_existing_main_and_backup`
- `safe_workspace_save_preserves_previous_file_as_backup`
- `save_after_corrupted_main_does_not_destroy_last_valid_backup`
- `workspace_recovery_does_not_discard_unsynced_in_memory_changes`

**Nguyên nhân xác định:** `save_workspace_safely` mở `backup_temp` chỉ với `read(true)`, rồi gọi `sync_all()`; trên Windows, `FlushFileBuffers` với handle không có quyền ghi bị từ chối. Đã sửa thành `read(true).write(true)` và giữ quy trình fsync/replace/backup. Commit [81e8d61c](https://github.com/tbtdemo/tbt360_cad_2d/commit/81e8d61c3e5c2877eeb7d09821b4d21d4ad2f59e).

**Đã kích hoạt đóng gói lại cùng fix:** commit [06f04db9](https://github.com/tbtdemo/tbt360_cad_2d/commit/06f04db90d47bc1d150dc1653745fda3b7f243ef) và các run:
- [41634295540](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41634295540): Ubuntu cross-compile Windows EXE, đang chạy lúc ghi.
- [41634295714](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41634295714): Windows native CI/installer, đang chạy lúc ghi.

**Tỷ lệ cổng giao bản sửa lỗi:** 1/4 = **25%** đã xác minh (source fix + workflow có commit). Chưa xác minh riêng EXE build và artifact **của commit 06f04db9**, chưa có test PASS lại trên Windows hoặc GUI smoke test. Không dùng artifact commit cũ làm bằng chứng bản sửa đã build.

**Bộ cài gần nhất đã xác minh build PASS trước bản sửa lỗi:** Ubuntu cross build run [41629152826](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41629152826), commit `e8f7af28`, artifact [13227448429](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41629152826/artifacts/13227448429). Chưa có bản sửa lỗi Windows backup nên không được gọi là release kiểm thử đạt.


# CAD360 B15-UI — Theo dõi tiến độ có bằng chứng

Cập nhật: 2026-11-03 (Asia/Bangkok). Nhánh phát triển: `tbt/cad360-active-release`.

## Phân biệt ba chỉ số

- **Bộ chức năng:** mốc **15/15 (100%)** từng được báo cáo trong cuộc hội thoại. Chưa kiểm kê đủ 15 thay đổi độc lập có test tương ứng; đây là số lượng quản trị theo mốc, **không phải chứng nhận hoàn thành sản phẩm**.
- **Cổng bàn giao bản cài B15-UI:** hiện xác nhận 1/4 (25%) dựa trên bằng chứng đang truy cập được. Các cổng chưa được xác minh không được coi là PASS.
- **Tỷ lệ chất lượng GUI / nghiệp vụ:** chưa định lượng vì không có GUI smoke test/benchmark Windows thực tế; không đưa % suy đoán.

| Mã | Cổng bàn giao B15-UI | Trạng thái xác minh | Tỷ lệ cổng |
|---|---|---|---:|
| G1 | Mã BIM UI + workflow có commit đúng nhánh | PASS: `e34815ef4eb098c7d25786e645cbf0fca0cb8d35` | 100% |
| G2 | Rust cross-build `cadcraft.exe` / `cadcraft-cli.exe` tại SHA B15-UI | CHƯA XÁC MINH | 0% |
| G3 | Artifact B15-UI Setup EXE + Portable ZIP + Source ZIP + SHA256 | CHƯA XÁC MINH | 0% |
| G4 | Chạy thật GUI Windows, so sánh 5 tab, thao tác nút và nghiệp vụ | CHƯA KIỂM THỬ | 0% |

**Tổng G1–G4 = (100+0+0+0)/4 = 25% đã xác minh.** Đây là chỉ số theo cổng bàn giao, không phải tỷ lệ code hoàn chỉnh.

## Nội dung kỹ thuật và tỷ lệ riêng

| Hạng mục | Mã nguồn | Build tại SHA mới | Chạy GUI thực |
|---|---|---|---|
| Điều hướng 5 tab ngang | Đã commit | Chưa xác minh | Chưa thử |
| Tổng quan KPI từ State | Đã commit | Chưa xác minh | Chưa thử |
| Đồng bộ / ChangeSet / revision conflict | Đã commit | Chưa xác minh | Chưa thử |
| Quyền truy cập / phạm vi chia sẻ | Đã commit | Chưa xác minh | Chưa thử |
| Chặn quyền chỉnh sửa trực tiếp chưa kiểm soát | Đã commit | Chưa xác minh | Chưa thử |
| Chặn sync khi chưa xác thực / còn conflict | Đã commit và có test mới | Chưa xác minh test PASS | Chưa thử |
| Tạo Windows Setup EXE B15-UI | Workflow đã commit | Chưa xác minh | Chưa thử |
| Tạo Portable ZIP, Source ZIP và checksum | Workflow đã commit | Chưa xác minh | Chưa thử |

## Bản độc lập đã có

- B15 cũ (không gồm thay đổi UI mới): GitHub Actions run [41612163498](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41612163498), artifact `CAD360-TBT-REAL-Windows-x64-Setup-Portable-Source-B15`. Đây là bản đã build từ SHA `78d5bfd3b96c02503c894e287f98c22d1db5cd36`, không phải B15-UI.
- Mốc code B15-UI mới nhất lúc lập tracker: `e34815ef4eb098c7d25786e645cbf0fca0cb8d35`.
- [Workflow đóng gói B15-UI](https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/cad360-ubuntu-windows-exe.yml).

## Tiêu chuẩn chốt EXE

Chỉ ghi **ĐÃ CÓ B15-UI EXE** sau khi có run và artifact cụ thể đúng SHA, kiểm tra tệp setup/portable/source và SHA256. Chỉ ghi **ĐÃ NGHIỆM THU GIAO DIỆN** sau khi chạy ứng dụng trên Windows thực, chụp màn hình chương trình đang chạy (không phải ảnh dựng) và kiểm tra các tab.

## Lượt nâng cấp nhận diện chính xác EXE đang chạy (03/11/2026)

- Đã thêm nhãn `CAD360 · <8 ký tự commit>` vào thanh công cụ desktop thực tế, có tooltip hiển thị toàn bộ SHA, từ commit `4f273ad5172a01f112b09573e9ab46302cb4afa8`.
- Đường Ubuntu cross-compile gắn `CAD360_BUILD_SHA=${{ github.sha }}` trong môi trường build và kiểm tra mã nhãn trong gói source: commit `e8f7af28a939f0e20a7755507bfe91688b77c672`, run [41629152826](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41629152826), **đang chạy** khi kiểm tra.
- Đường Windows native cũng được cấu hình để nhúng SHA: commit `7bd0cb3707fc3664ea8e7ea853b7e405431ce1c0`, run [41629180328](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41629180328), **đang xếp hàng** khi kiểm tra.
- Bản EXE Responsive **đã xác nhận có artifact** vẫn là `e34815ef`: [run 41627300710, artifact 13225968966](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300710/artifacts/13225968966); chưa chứa tính năng hiển thị SHA vừa bổ sung.
- Tỷ lệ bàn giao bản mới có nhãn SHA: 1/4 = **25%** đã xác nhận (commit mã); build, artifact và ảnh chụp chạy thật chưa đủ bằng chứng. Tỷ lệ bàn giao bản Responsive `e34815ef` vẫn là 3/4 = **75%**.

## B15-UI Responsive — artifact Windows đã xác nhận (03/11/2026)

- Run [41627300710](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300710), commit `e34815ef4eb098c7d25786e645cbf0fca0cb8d35`.
- **Đã PASS**: cài môi trường cargo-xwin, cross-compile `cadcraft.exe` và `cadcraft-cli.exe`, tạo NSIS Setup, Portable ZIP, Source ZIP, SHA256 và upload.
- **Đã có artifact**: `13225968966`, `CAD360-TBT-REAL-Windows-x64-Setup-Portable-Source-B15-UI`, 63,922,254 bytes. [Tải tại đây](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300710/artifacts/13225968966).
- **Đang chạy khi kiểm tra**: `Regression-test BIM collaboration after uploading the installer`. Không được coi là test PASS trước khi GitHub báo success.
- **Chưa nghiệm thu**: giao diện 5 tab thực tế trên Windows, chụp ảnh chính ứng dụng, kiểm thử thao tác và nội dung nghiệp vụ.
- **Tỷ lệ cổng bàn giao (4 cổng, trọng số đều)**: 3/4 = **75%** (mã / binary / artifact PASS, chạy thử GUI còn 0%). Tỷ lệ này là mức chứng thực giao hàng, không phải chất lượng toàn phần.

## Xác minh run và artifact bổ sung (03/11/2026)

| Phiên bản | Run | Commit | Build / artifact | Test GUI |
|---|---|---|---|---|
| B15-UI gốc | [41623015347](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41623015347) | `2625310f` | PASS; artifact `13225806171` | Chưa kiểm thử |
| B15-UI có cổng regression | [41623511205](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41623511205) | `953e6703` | EXE, Portable, Source ZIP đã được upload; artifact `13226737978` (63,929,983 bytes) | Regression BIM còn đang chạy, GUI chưa kiểm thử |
| B15-UI Responsive (tab/KPI cuộn ngang) | [41627300710](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300710) | `e34815ef` | Setup chưa có artifact; bước biên dịch `cargo xwin build` đang chạy | Chưa kiểm thử |
| B15-UI Responsive — Windows native | [41627300916](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300916) | `e34815ef` | Source inventory, source ZIP, preflight PASS; native Rust release build đang chạy | Chưa kiểm thử |

**Tính theo 4 cổng nghiệm thu bằng chứng (source / compiled EXE / artifact / chạy Windows):**
- B15-UI `953e6703`: **3/4 = 75%** (có artifact tải được, nhưng regression chưa có kết quả và GUI chưa kiểm thử).
- Responsive `e34815ef`: **1/4 = 25%** (biên dịch đang chạy, chưa có artifact installer).
- Các tỷ lệ này **không đo chất lượng hay % hoàn thiện tính năng** và không được cộng với 15/15 bộ chức năng.

**Tải bản EXE mới nhất đã xác minh artifact:** [B15-UI run 41623511205, artifact 13226737978](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41623511205/artifacts/13226737978). Bản Responsive chưa đủ điều kiện tải.

## Kết quả chạy GitHub Actions đã kiểm tra (2026-11-03, giờ Việt Nam)

**B15-UI gốc — ĐÃ CÓ BỘ CÀI:**
- Run [41623015347](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41623015347): **completed / success**, commit `2625310f19301b91a6b882ba0660ec2cd0209ee3`.
- Job Windows cross-compile, kiểm tra PE, NSIS packaging và upload artifact đều **success**.
- Artifact ID `13225806171`: `CAD360-TBT-REAL-Windows-x64-Setup-Portable-Source-B15-UI` (63,922,540 bytes, không hết hạn tại thời điểm kiểm tra).
- [Tải artifact trực tiếp](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41623015347/artifacts/13225806171).
- Bản này có dashboard BIM 5 tab đã commit trước `2625310f`, **không có** tối ưu cuộn ngang của commit `736efce1`.
- Cổng nghiệm thu B15-UI gốc: **3/4 = 75%** (source đúng SHA + Rust cross-build PASS + artifact PASS; GUI Windows chưa chạy nghiệm thu).

**B15-UI responsive mới hơn — ĐANG BUILD:**
- Run [41627300710](https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41627300710), commit `e34815ef4eb098c7d25786e645cbf0fca0cb8d35`.
- Bước chuẩn bị 1–6 success; bước 7 `Cross-compile real CAD360 desktop and CLI for Windows x64` đang in_progress lúc kiểm tra; các bước packaging/upload/regression chờ.
- Cổng nghiệm thu bản responsive tại thời điểm kiểm tra: **1/4 = 25%**; chưa có artifact xác nhận; không gọi là build PASS.
- Sau khi bước upload hoàn tất và có artifact đúng SHA, tỷ lệ cổng có thể cập nhật lên 75%. Kiểm thử UI thực tế vẫn riêng biệt.

**Quy tắc thống kê:** Kết quả 75% và 25% là tỷ lệ 4 cổng xác minh, không phải độ hoàn thiện phần mềm hay % số dòng code.

## Lượt cập nhật giao diện và đóng gói mới nhất (2026-11-03)

- Commit UI `736efce12f774f136acf941dd6d001d095c7144e`: thay 5 tab + dãy KPI bằng hàng ngang cuộn khi cửa sổ hẹp, không nén mỗi ô vào cột nhỏ.
- Commit build `e34815ef4eb098c7d25786e645cbf0fca0cb8d35`: gắn điều kiện kiểm tra mã bố cục ngang vào bước đóng gói B15-UI trên Ubuntu. Commit có tiền tố `build(cad360): B15` và sửa workflow được theo dõi bởi trigger push.
- Hiện chưa truy xuất được run ID mới qua các hành động GitHub đang có. Công cụ tra cứu theo commit chỉ trả PR-triggered runs; vì vậy không thể dựa vào kết quả rỗng để khẳng định workflow không chạy.
- **Chưa có bằng chứng GUI mới đã biên dịch PASS hoặc artifact B15-UI mới đã tạo.** Không gộp chung với artifact B15 cũ.

## Vấn đề cần tiếp tục

- Cần lấy GitHub Actions run ID và artifact ID của push B15-UI; truy vấn commit-workflow-runs qua connector chỉ liệt kê PR-triggered run, nên kết quả rỗng không chứng minh workflow không chạy.
- Chưa có kết quả compile Rust GUI tại commit B15-UI và regression tests.
- Chưa kiểm chứng độ chính xác bố cục với mẫu người dùng cung cấp, đặc biệt khi ứng dụng ở chế độ cửa sổ nhỏ.
