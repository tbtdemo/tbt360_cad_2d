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
