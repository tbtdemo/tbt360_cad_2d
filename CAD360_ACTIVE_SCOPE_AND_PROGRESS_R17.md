# CAD360 — R17/30: Phạm vi ưu tiên sau khi tạm dừng đồng bộ và xung đột

Ngày: 03/11/2026. Nhánh: tbt/cad360-active-release.

## 1. Quyết định phạm vi mới
- DỪNG triển khai mới: đồng bộ thay đổi online/offline và giải quyết conflict giữa người dùng hoặc các máy; clash giữa các bộ môn.
- GIỮ VÀ TIẾP TỤC KIỂM TRA: quyền tài khoản, thông tin đăng nhập, can_view/can_edit, thiết lập phân quyền theo tài khoản, tầng/bộ môn; chống vượt quyền. Đây là **permission**, không phải conflict resolution.
- Không gỡ chức năng đã có hoặc vô hiệu hóa các test hồi quy liên quan, vì như vậy có thể làm lỗi ứng dụng hiện hữu.
- Giai đoạn ưu tiên là mouse/command, file lớn, hiệu năng, autosave/recovery, CI/EXE.

## 2. Những thay đổi R17 đã push
1. `opening.rs`: hiển thị trạng thái còn file đang chờ giữa thời điểm decoder kết thúc và UI đưa file trước vào Session; tránh thẻ tải biến mất khi hàng đợi chưa rỗng.
2. Thêm Rust regression test trường hợp hai file chỉ ở hàng đợi, xác nhận `busy()` vẫn đúng và label hiển thị file tiếp theo cùng số lượng chờ.
3. `.github/workflows/main.yml`: targeted Windows test cho LINE nhận điểm thứ nhất, CIRCLE nhận điểm tâm, stuck drag/ESC/focus, point-release và opening; các test fail thì job fail. Vẫn giữ `cargo test --workspace`.
4. Cập nhật `TBT_BATCH_PROGRESS.md` với nhóm dừng riêng; tính lại % của nhóm chủ động để giám sát đúng phạm vi.

## 3. Tiến độ 8 hạng mục đang làm
| ID | Hạng mục | Ước lượng kỹ thuật | Tiêu chí để tăng % và chốt PASS | Ưu tiên |
| --- | --- | ---: | --- | --- |
| I1 | LINE/PLINE/CIRCLE/MOVE nhận click đầu tiên | 45% | Windows GUI 100 lần/lệnh không trượt/nhận trùng | P0 |
| I2 | ESC, drag window, focus không dính chuột | 45% | 100 lần ESC/focus, drag ngoài canvas không mắc lệnh | P0 |
| L1 | Đọc CAD không cap cứng 512 MiB | 45% | File >512MiB thực sự decode/render đúng | P0 |
| L2 | Progress đọc/parse/render và ETA | 70% | Bằng chứng UI và % parser/render thực | P1 |
| L3 | Pan/zoom/select CAD 300–500MiB | 15% | Số đo RAM, FPS, P95 input latency trên máy thực | P0 |
| L4 | Queue nhiều file và kiểm soát RAM | 50% | Mở 5–10 file; kiểm RAM đỉnh, không treo | P1 |
| A1 | Đăng nhập và phân quyền tài khoản/tầng/bộ môn | 35% | Tests thu hồi quyền, scope và mutation check; server phải chặn ghi trái quyền | P1 |
| Q1 | Windows build, Setup và Portable | 10% | Cargo CI PASS, artifact Setup + Portable kèm SHA256; cài thử Windows | P0 |

**Trung bình 8 đầu việc = (45+45+45+70+15+50+35+10)/8 = 39,375%, hiển thị 39,4%.** Các % là ước lượng chưa nghiệm thu. So với 37,2% cũ, thay đổi là do bỏ hạng mục Sync/Conflict C2 khỏi mẫu số, không có nghĩa tự nhiên đã làm thêm 2,2 điểm phần trăm.

## 4. Những việc tạm dừng (không tính tiến độ)
| Hạng mục | Trạng thái |
| --- | --- |
| Đồng bộ online/offline giữa client, conflict resolution | PAUSED — không phát triển mới |
| Clash và xung đột mô hình giữa các bộ môn | PAUSED — không phát triển mới |
| Server lease/conflict reconciliation | PAUSED — không phát triển mới |
| Quyền account và cấu hình quyền xem/sửa | ACTIVE — tiếp tục |

## 5. Thứ tự xử lý R18–R30
- R18–R19: kiểm tra workflow Windows từ log thật; sửa lỗi biên dịch, lỗi test và EXE đóng gói; không công bố candidate nếu chưa thấy artifact.
- R20–R22: mouse input các lệnh thông dụng, ESC/focus, dynamic input, selection stuck; tạo test UI đa sự kiện.
- R23–R26: file CAD lớn và hiệu năng parse/render; đo đạc thật với 100/300/500/>512MiB. Chưa có dữ liệu để hứa hỗ trợ không giới hạn dung lượng và zero lag.
- R27–R28: autosave, phục hồi toàn bộ hình học, nhiều tab, tránh mất dữ liệu.
- R29–R30: kiểm tra quyền account, chuẩn hóa giao diện, tổng hợp nghiệm thu và đóng gói cuối khi PASS.
- Các nhóm có thể chạy song song khi có năng lực; không coi 30 lượt là hạn cứng để phát hành.

## 6. Điều kiện phát hành không thay đổi
- Cargo check/test Windows, GUI smoke chính xác, file lớn, autosave/restore, quyền tài khoản, và artifacts tải được.
- Thời điểm cuối R17: truy vấn workflow runs theo SHA qua connector chỉ lọc pull_request -> [], combined statuses -> []; không chứng minh chưa chạy, cũng không chứng minh PASS. Chưa có file EXE thật được xác minh.

## 7. Chứng cứ commit
- `0224fa77d98b3f046bd8be3846189e0b609c3e2e`: giữ progress queue trên UI.
- `6c8a66fcecc9cdc808e2e616908662eb36d3ad5f`: test Windows riêng các lỗi chuột thường gặp.
- `dcd683a8bd12eee360f6751cdff7aff0811dba03`: scope mới và theo dõi R17.
