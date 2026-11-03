# CADTBT360_V1.000

**CAD TBT 360 — Phiên bản phát hành 1.000**

Đây là cấu trúc phát hành được chuẩn hóa từ mã nguồn CAD 2D hiện có. Tên phiên bản công khai bắt đầu lại từ **CADTBT360_V1.000**; các nhãn Bxx/Rxx là lịch sử phát triển, không còn xuất hiện trong gói V1.

- `CADTBT360_V1.000/`: mã nguồn Rust ứng dụng chính; tên crate `cadcraft` bên trong được giữ nhằm bảo toàn dependency và khả năng biên dịch.
- `installer/`: bộ tạo Setup cho Windows x64.
- `.github/workflows/`: pipeline build/test/phát hành.
- `backend/`: dịch vụ tích hợp phục vụ phát triển; không đóng gói vào EXE desktop.

## Biên dịch Windows

Trong thư mục `CADTBT360_V1.000`:

```powershell
cargo build --release --locked -p cadcraft -p cadcraft-cli
```

GitHub Actions đóng gói thành `CADTBT360_V1.000_Setup.exe`, `CADTBT360_V1.000_Portable.zip`, `CADTBT360_V1.000_Source.zip` cùng `SHA256SUMS.txt`.

**Lưu ý pháp lý:** LICENSE-MIT, LICENSE-APACHE, NOTICE và ATTRIBUTION của các thành phần mã nguồn mở được giữ đúng nghĩa vụ giấy phép, không phải các bản CAD gốc chạy kèm sản phẩm.

**Kiểm thử:** Chỉ xác nhận một phiên bản đạt khi build, test hồi quy, tải artifact và chạy thử GUI Windows đều có bằng chứng. Không coi việc upload EXE là nghiệm thu GUI.
