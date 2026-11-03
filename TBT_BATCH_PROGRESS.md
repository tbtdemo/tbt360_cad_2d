# CAD360 TBT — Theo dõi đợt nâng cấp 15–30 lượt

## Quy ước
- Một lượt = một nhóm thay đổi chức năng đã được commit và có bằng chứng mã nguồn. Mỗi lượt không mặc định tương đương một commit.
- Mốc đóng gói định kỳ: khi đã hoàn thành ít nhất 15 lượt và không quá 30 lượt; một đợt đóng gói Windows Setup, Portable và mã nguồn sau khi vượt qua các gate.
- Mọi thay đổi vẫn có mã nguồn sao lưu theo commit. Không tính job `skipped` là EXE PASS.
- Chưa kích hoạt đóng gói cho đợt này.

## Tiến độ đợt hiện tại
| Lượt | Phạm vi | Commit mã nguồn | Test bổ sung | CI |
| --- | --- | --- | --- | --- |
| 01/15 | IFC/OBJ: giới hạn 1024 byte cho tên nhóm; chặn đỉnh trùng không liền kề | `39c4ea534bf801fd73184e7bf3876a6b52835ae4` | 3 Rust unit tests | Source/metadata PASS; Rust tests chưa chạy |
| 02/15 | CAD+BIM: tự lưu 5 phút, DXF+JSON cùng phiên, lưu khi thoát, checksum, 3 bản backup, phục hồi an toàn; tối ưu UI | `4afe448ec36ffca90e25c20c70ec3c25ece8b69b` | 3 Rust unit tests; Windows cargo check/test mới | Source/IFC PASS; Rust integration PASS (run 41563233121) |
| 03/15 | CAD+BIM: không xóa backup hợp lệ khi nhiều phiên bị hỏng; kiểm tra DXF giải mã được trước khi phục hồi | `7d76a4498c734da3a7dc2158f85e9eaf66971ff8` | 2 Rust regression tests | Chưa xác minh lại CI |\n| 04/15 | CAD+BIM: định dạng snapshot v2 lưu mọi tab CAD, chỉ số tab đang hoạt động và workspace; đọc v1; kiểm tra từng DXF trước khi phục hồi | `900a7cde8cdc097268c190af079f482ae14f78c4` | 1 regression test 3 tab và corruption | Chưa xác nhận cargo check/test Windows |

### Trạng thái
- Hoàn thành bằng chứng mã nguồn: **4/15** lượt tối thiểu.
- Còn ít nhất **11** lượt trước khi đóng gói định kỳ.
- Source inventory (run 41561782723) và IFC metadata test/source snapshot (run 41561782535) PASS. Windows Setup/Portable được skip đúng lịch 15–30. Rust tests CAD+BIM của lượt 02 đã PASS (run 41563233121). Hai regression tests lượt 03 đang chờ xác nhận. Chưa có EXE phát hành của đợt này.

### Rà soát khả năng phục hồi
- Lõi CAD đã có autosave và Drawing Recovery riêng trước đợt này. Lượt 02 thêm một cặp snapshot DXF (hình học bản vẽ đang hoạt động) + workspace BIM JSON + checksum, được lưu theo chu kỳ 5 phút và khi thoát.
- Phục hồi theo lệnh người dùng, mở bản vẽ CAD dạng bản sao; không tự ghi đè file thiết kế. Phần BIM phải được xác thực trước khi mở bản vẽ.
- Giới hạn hiện tại: chưa sao lưu đồng thời tất cả tab bản vẽ, chưa test khôi phục GUI end-to-end trên Windows và chưa kiểm thử mất điện phần cứng.
- Không đóng gói EXE trước mốc 15–30 lượt.

### Lượt 03 — phòng mất bản phục hồi và từ chối DXF lỗi
- `prune_generations` hiện chỉ đếm các snapshot xác minh hợp lệ vào hạn mức 3 bản; thư mục hỏng không được phép đẩy bản phục hồi tốt ra khỏi danh sách.
- Trước lệnh `recover`, đọc và giải mã thực tế DXF bằng `cadcraft_io::read`. Dữ liệu checksum đúng nhưng DXF sai định dạng sẽ bị từ chối trước khi chạm trạng thái CAD đang mở.
- Phần bổ sung: hai test về backup tốt bên cạnh nhiều bản lỗi, và giải mã DXF sai.
- Vẫn còn giới hạn nhiều bản vẽ, snapshot đồng bộ server và kiểm thử UI/mất điện thật.

### Lượt 04 — snapshot nhiều bản vẽ CAD+BIM
- Snapshot mới format v2 lưu riêng từng tab `drawing-0000.dxf`, `drawing-0001.dxf`... và danh sách UID/revision, chỉ số tab đang hoạt động, toàn bộ workspace BIM.
- Tổng hình học CAD tối đa 512 MiB và số bản vẽ tối đa 256 khi xác minh; mỗi file có checksum, kiểm tra độc lập. Snapshot cũ format v1 vẫn đọc được.
- Trước khi phục hồi, kiểm tra toàn bộ checksum, DXF và workspace; các tab mở dạng bản sao, tab active mở cuối. Nếu mở tab bị lỗi giữa chừng, vẫn có rủi ro phục hồi CAD một phần; cần xử lý rollback giao dịch và kiểm thử GUI thực tế ở lượt kế tiếp.
- Các commit lượt 04 đã đẩy; **chưa xác nhận Windows cargo check/test PASS và chưa có EXE**.
