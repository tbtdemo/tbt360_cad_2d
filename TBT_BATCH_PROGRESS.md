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
| 02/15 | CAD+BIM: tự lưu 5 phút, DXF+JSON cùng phiên, lưu khi thoát, checksum, 3 bản backup, phục hồi an toàn; tối ưu UI | `4afe448ec36ffca90e25c20c70ec3c25ece8b69b` | 3 Rust unit tests; Windows cargo check/test mới | Source/IFC PASS; Rust integration chờ kết quả |

### Trạng thái
- Hoàn thành bằng chứng mã nguồn: **2/15** lượt tối thiểu.
- Còn ít nhất **13** lượt trước khi đóng gói định kỳ.
- Source inventory (run 41561782723) và IFC metadata test/source snapshot (run 41561782535) PASS. Windows Setup/Portable được skip đúng lịch 15–30. Rust unit tests chưa chạy; chưa có EXE phát hành của đợt này.

### Rà soát khả năng phục hồi
- Lõi CAD đã có autosave và Drawing Recovery riêng trước đợt này. Lượt 02 thêm một cặp snapshot DXF (hình học bản vẽ đang hoạt động) + workspace BIM JSON + checksum, được lưu theo chu kỳ 5 phút và khi thoát.
- Phục hồi theo lệnh người dùng, mở bản vẽ CAD dạng bản sao; không tự ghi đè file thiết kế. Phần BIM phải được xác thực trước khi mở bản vẽ.
- Giới hạn hiện tại: chưa sao lưu đồng thời tất cả tab bản vẽ, chưa test khôi phục GUI end-to-end trên Windows và chưa kiểm thử mất điện phần cứng.
- Không đóng gói EXE trước mốc 15–30 lượt.
