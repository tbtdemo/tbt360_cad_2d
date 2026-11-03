# CAD360 TBT — R27/30: Autosave và phục hồi đầy đủ các tab CAD+BIM

Ngày 03/11/2026. Branch `tbt/cad360-active-release`.

## Yêu cầu và phạm vi
Tập trung hoàn thiện tự động lưu, phục hồi toàn bộ hình học và multi-tab, file CAD lớn, quyền tài khoản và Windows build thực. **Tạm dừng phát triển mới** sync online/offline, giải quyết conflict giữa client hoặc clash bộ môn. Các test hồi quy cũ giữ nguyên.

## Tình trạng thực tế trước khi sửa
- `crates/engine/src/autosave.rs` có SAVETIME mặc định 10 phút: viết DXF những tab dirty, theo worker, atomic write. Khi crash, các bản sao chưa bị dọn sẽ xuất hiện trong Drawing Recovery.
- `apps/cadcraft/src/cad_bim_recovery.rs` đã có snapshot CAD+BIM mỗi 5 phút, manifest version 2 gồm từng bản vẽ, active tab và metadata BIM; ghi vào `.pending` trước khi commit, lưu ba generation.
- **Lỗi RAM**: `save_pair` mã hóa tất cả bản vẽ đang mở rồi giữ toàn bộ byte DXF trong `Vec<drawings>` trước khi viết ra đĩa. Với nhiều tab lớn, RAM tạm thời có thể tăng lên bằng tổng lượng DXF.
- **Trần tổng backup 512 MiB**, không thống nhất với OPEN đã chấp nhận file lớn hơn. Startup/pruning dùng `std::fs::read` toàn bộ DXF của mỗi snapshot chỉ để kiểm checksum, làm RAM tăng và tiềm ẩn lỗi OOM.

## Thay đổi thực hiện R27
1. Serialize và ghi DXF theo từng tab trực tiếp vào generation `.pending`, giải phóng buffer của tab trước rồi mới xử lý tab kế; chỉ hoàn tất manifest và rename khi mọi tab đã được ghi, đóng và sync.
2. Chuyển ngưỡng bảo vệ của **tổng một generation** từ 512 MiB lên 4 GiB (tổng DXF đã mã hóa). Đây vẫn là giới hạn thiết kế, không phải unlimited backup.
3. Với manifest v2 và v1: kiểm tra metadata/size trước, rồi đọc checksum DXF theo từng khối 64 KiB bằng cùng FNV-1a như checksum cũ, không nạp nguyên bản vẽ vào RAM khi dò snapshot. Giữ kiểm tra độ dài, checksum, project_id và format.
4. Bổ sung unit tests cho checksum streaming đúng với checksum trên bytes, phát hiện sửa phần đuôi, và giới hạn generation lớn hơn 512 MiB.
5. Thêm Windows CI targeted step `cargo test --locked -p cadcraft --bin cadcraft cad_bim_recovery::tests -- --nocapture` để chạy cùng các regression multi-tab/corrupt/prune cũ.

## Những giới hạn còn lại, không được báo cáo sai
- Cặp autosave CAD+BIM 5 phút hiện vẫn mã hóa DXF trên **luồng UI**, nên với bản vẽ hàng trăm MB có thể gây gián đoạn. R27 chỉ giảm peak RAM bằng ghi tuần tự, **chưa chuyển serialization sang background worker**.
- Việc phục hồi thật vẫn phải đọc và decode từng tab DXF; chưa có Windows crash/restart E2E và benchmark 500 MiB.
- Các unit tests mới đang ở mã nguồn và cấu hình CI, **chưa có bằng chứng Cargo/Windows CI PASS**.
- Chưa xác minh Setup.exe, Portable.zip và Source.zip cùng SHA256; không coi commit là binary giao được.

## Các nhóm đang làm
| Hạng mục | % kỹ thuật ước lượng, chưa nghiệm thu | Điều kiện còn thiếu |
| --- | ---: | --- |
| LINE/PLINE/CIRCLE/MOVE first click | 45 | Windows GUI |
| ESC/drag/viewport không kẹt | 45 | Windows E2E |
| OPEN >512MiB | 45 | CAD lớn thật |
| Progress/ETA | 70 | UI acceptance |
| CAD 300–500 MiB mượt | 15 | RAM/FPS/P95 |
| FIFO nhiều file/RAM | 50 | 5–10 file thật |
| Account permissions | 35 | API authorization |
| Windows CI + EXE + Portable + Source | 10 | CI SUCCESS, kiểm checksum và cài thử |

Trung bình tám mục **39,4% ước lượng kỹ thuật**, không đồng nghĩa nghiệm thu. **27/30 lượt**, còn 3 lượt dự kiến. Autosave/recovery là đầu việc bổ trợ ưu tiên trong R27–R28, chưa cộng vào mẫu số cũ.

## Commits thực tế
- `af25f320110a76857117336036e7508bbce045eb`: serialize và write multi-tab tuần tự; generation budget 4 GiB.
- `6628057f9f2a35cf82611d302fd4c51a48ce2e54`: checksum theo luồng cho v2/v1.
- `c94e0debfe021590f0ccacbb21d10322629010a6`: regression tests.
- `2dc46e8a239a6fd2db890fd7b29ef42e4280d81c`: Windows CI tests.
- `c8d1ef1d470cdc6746c51c4a4903c25ee6b71994`: tracker R27.

## R28 tập trung
- Di chuyển encoding/saving nhiều tab CAD+BIM ra khỏi UI nếu thiết kế snapshot bất biến và không làm mất sự nhất quán khi dữ liệu thay đổi; tránh giữ nhiều bản Drawing lớn trong RAM.
- Kiểm tra crash/restart, DXF hỏng, rollback không ghi đè tab gốc, dữ liệu phục hồi so với bản lưu.
- Ưu tiên sửa Windows CI compiler/test theo log thực và cuối cùng đóng gói đúng artifact; không phát hành nhãn PASS khi chưa có bằng chứng.
