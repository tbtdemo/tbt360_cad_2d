# CAD360 TBT — R30/30: Windows CI có bằng chứng, sửa build blocker và chuẩn bị release gate

Ngày 03/11/2026; nhánh `tbt/cad360-active-release`.

## 1. Kết quả thực chứng từ GitHub Actions cũ

Đã đọc run **41064962499** tại `https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41064962499`, được ghi log ngày **30/10/2026**.

| Job thuộc run cũ | Kết quả |
| --- | --- |
| Build Core | SUCCESS |
| Unified CAD UI | SUCCESS |
| Vietnamese and TBT Branding | FAILURE |
| Regression and Data Tests | FAILURE |
| Release - Single Windows Setup | SKIPPED |
| Artifacts | **0** |

Đây là run cũ; không được dùng nó để báo PASS/FAIL của HEAD R30.

### Lỗi 1: Kiểm tra tiêu đề ứng dụng bằng chuỗi cũ
Log Vietnamese/Branding cuối cùng báo `TBT runtime title missing`: tại commit run đó workflow kiểm `CAD 2D - TBT 360`, trong khi app đã chuyển thương hiệu. Trên nhánh đang làm, `main.rs` chứa chính xác `.with_title("CAD 360 Việt Nam - TBT")`; workflow hiện hành kiểm chính chuỗi này. Đây là trạng thái đã đối chiếu ở R30, không sửa quay về title cũ.

### Lỗi 2: Test XREF gọi API EntityStore không tồn tại
Log regression run cũ báo `error[E0599]: no method named first found for struct EntityStore` trong `crates/engine/tests/xref.rs`. R30 rà nhánh hiện tại còn một lệnh `host_doc.model.first().unwrap().handle`; đã sửa thành `host_doc.model.iter().next().unwrap().handle`, đúng kiểu EntityStore iterator hiện được dùng trong code. **Chưa có kết quả Cargo Windows chạy lại sau sửa**, không coi lỗi đã hết trên máy Windows cho tới khi PASS.

## 2. Các thay đổi thực hiện R30
1. `.github/workflows/main.yml`: trước khi chạy regression, dùng `cargo test ... -- --list` và kiểm **9 test names** quan trọng tồn tại trong UI, App và Engine test harnesses. Đặc biệt Rust test harness có thể trả exit code 0 ngay cả khi `--exact` chạy 0 bài; gate mới làm FAIL nếu thiếu.
2. `crates/engine/tests/xref.rs`: thay `EntityStore::first()` bằng `EntityStore::iter().next()` ở chỗ còn sót trên HEAD.
3. XREF job target thêm điều kiện xác nhận có ít nhất một test được liệt kê, trước khi chạy regression trên Windows.
4. Windows release job: sau khi đóng Portable ZIP và mở thử, đọc ZIP membership đảm bảo có đủ (không trùng) `cadcraft.exe`, `cadcraft-cli.exe`, `README.txt`, rồi mở file EXE đọc chữ ký PE bắt đầu `MZ`.
5. Tiếp tục giữ từ các lượt trước: Rust workspace check/test, Windows release binaries, Inno Setup, Portable ZIP, Source ZIP từ đúng SHA, `SHA256SUMS.txt` cho cả 3, `BUILD_INFO.txt`, CLI `--version` smoke, upload artifacts chỉ sau `needs` các jobs PASS.

## 3. Điều chưa hoàn thành
- Chưa có workflow run mới của **commit R30** được truy xuất; tính trạng thái CI mới là **UNKNOWN**, không phải PASS.
- Chưa có ba artifacts thực tế (Setup.exe, Portable.zip, Source.zip) từ run R30 được kiểm tra checksum hoặc tải được. Không cung cấp link tải giả.
- Chưa chạy GUI Windows tương tác (100 lần click LINE/PLINE/CIRCLE/MOVE, ESC, 2–4 viewport), chưa đo FPS/P95 input/peak RAM hay thử bản vẽ 100/300/500/>512 MiB.
- Kiểm tra quyền của R29 ở UI selection, không thay thế server-side/engine mutation authorization; chưa được audit hết API/command bypass.
- Autosave/recovery paired CAD+BIM vẫn mã hóa mỗi tab trên UI thread, chưa chứng minh không lag. Windows crash/restart và recovery DXF chưa E2E.

## 4. % kỹ thuật đã báo trong chu kỳ — chưa nghiệm thu
| Mã | Hạng mục | Ước lượng | Chứng cứ để nghiệm thu |
| --- | --- | ---: | --- |
| I1 | Click đầu tiên lệnh CAD | 45% | GUI E2E Windows |
| I2 | ESC/drag/focus/viewport | 45% | Windows input E2E |
| L1 | File >512 MiB | 45% | DXF/DWG thật render |
| L2 | Progress/ETA | 70% | UI phase measurements |
| L3 | Mượt 300–500 MiB | 15% | FPS/RAM/P95 |
| L4 | FIFO nhiều file/RAM | 50% | 5–10 file thực |
| A1 | Account permissions | 35% | Engine/API mutation gates |
| Q1 | Windows Setup/Portable/Source | 10% | CI success + artifacts + checksums + cài chạy |

Trung bình 8 mục **39,4%**, chỉ ước lượng kỹ thuật. **R30/30** là mốc chu kỳ mã nguồn; không đồng nghĩa 100% hoàn thiện. Nhóm sync/conflict giữa người dùng và clash bộ môn vẫn PAUSED theo chỉ đạo.

## 5. Trạng thái quyết định phát hành
**NO-GO** cho production/nhãn EXE đã nghiệm thu. Chỉ đổi trạng thái khi GitHub Actions trên HEAD mới SUCCESS, có cả 3 artifacts cùng SHA và checksum đối chiếu đúng, Setup và Portable chạy thực tế trên Windows, cùng các GUI/autosave/file lớn test bắt buộc.

## 6. Commits R30
- `7e8d092f583f5f7f251342db5c56541a59bbf1b8`: 9 test discovery gates.
- `ba00a08122356b98e104a1225fbbd8553e690a99`: ZIP/PE validation.
- `788b496737694c7dd5be9ff17ead0f66ace4d198`: XREF EntityStore fix.
- `38276a4dcadd4877a758a7ca6d96d601a19b5651`: XREF nonempty test discovery.
- `c2d0e6e9ea6c1bd03cd8ce2d793038cd8b0cad20`: progress tracker.

Sau mốc 30, các vòng tiếp theo nên được đặt tên **release stabilization** thay vì tiếp tục tăng số lượt tưởng như đã hoàn thành. Ưu tiên: bằng chứng CI HEAD, fixes theo log, artifacts thật và nghiệm thu Windows trước khi xuất bản.
