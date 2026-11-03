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

## Lượt cập nhật giao diện và đóng gói mới nhất (2026-11-03)

- Commit UI `736efce12f774f136acf941dd6d001d095c7144e`: thay 5 tab + dãy KPI bằng hàng ngang cuộn khi cửa sổ hẹp, không nén mỗi ô vào cột nhỏ.
- Commit build `e34815ef4eb098c7d25786e645cbf0fca0cb8d35`: gắn điều kiện kiểm tra mã bố cục ngang vào bước đóng gói B15-UI trên Ubuntu. Commit có tiền tố `build(cad360): B15` và sửa workflow được theo dõi bởi trigger push.
- Hiện chưa truy xuất được run ID mới qua các hành động GitHub đang có. Công cụ tra cứu theo commit chỉ trả PR-triggered runs; vì vậy không thể dựa vào kết quả rỗng để khẳng định workflow không chạy.
- **Chưa có bằng chứng GUI mới đã biên dịch PASS hoặc artifact B15-UI mới đã tạo.** Không gộp chung với artifact B15 cũ.

## Vấn đề cần tiếp tục

- Cần lấy GitHub Actions run ID và artifact ID của push B15-UI; truy vấn commit-workflow-runs qua connector chỉ liệt kê PR-triggered run, nên kết quả rỗng không chứng minh workflow không chạy.
- Chưa có kết quả compile Rust GUI tại commit B15-UI và regression tests.
- Chưa kiểm chứng độ chính xác bố cục với mẫu người dùng cung cấp, đặc biệt khi ứng dụng ở chế độ cửa sổ nhỏ.
