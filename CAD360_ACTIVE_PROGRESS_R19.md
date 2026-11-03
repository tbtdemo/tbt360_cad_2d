# CAD360 TBT — R19/30: Windows EXE + Portable + source backup

Ngày 03/11/2026. Nhánh: `tbt/cad360-active-release`.

## Phạm vi áp dụng
Đồng bộ online/offline, giải quyết xung đột người dùng và clash/xung đột bộ môn vẫn tạm dừng theo yêu cầu. Quyền tài khoản, cấu hình xem/sửa và chống vượt phạm vi vẫn thuộc nhóm đang làm. R19 tập trung đảm bảo mỗi gói cài đặt ứng viên có bộ nguồn trùng SHA, có thể truy xuất và kiểm tra.

## Thực hiện trong mã nguồn
- Workflow Windows `.github/workflows/main.yml`: sau các job Rust test và build, tạo `CAD360_VietNam_TBT_Source.zip` từ `git archive HEAD`, thay vì sử dụng mã nguồn cũ hoặc thư mục local không kiểm chứng.
- Bắt buộc `git rev-parse HEAD` bằng `GITHUB_SHA`; kiểm tra ZIP tồn tại, kích thước >10KB và chứa đúng 4 file bắt buộc: `SOURCE_CAD_R65/Cargo.toml`, `SOURCE_CAD_R65/Cargo.lock`, `installer/CAD360_TBT_Windows.iss`, `.github/workflows/main.yml`.
- Bổ sung nguồn ZIP vào SHA256SUMS và kiểm tra lại **ba hash** (Setup, Portable, Source) trước khi upload. Workflow có ba artifacts, cùng GitHub run và GITHUB_SHA.
- R15–R18: vẫn giữ Windows build/check, targeted mouse tests, DXF IO, large-file queue và decode timer. R19 không chủ trương phát triển thêm sync/conflict.

## Thứ tự kiểm tra release candidate
1. Workflow của đúng commit trên branch active phải đạt SUCCESS trong Windows build/test và packaging job; run skipped không được tính là bản build.
2. Trong cùng một run, phải có `CAD360_VietNam_TBT_Windows_Setup_Candidate`, `CAD360_VietNam_TBT_Windows_Portable_And_SHA256_Candidate`, `CAD360_VietNam_TBT_Source_Candidate`.
3. Tải ZIP artifacts, đối chiếu `BUILD_INFO.txt` với SHA commit và run ID. Tính lại SHA256 ba file, so `SHA256SUMS.txt`.
4. Cài `Setup.exe` và chạy `Portable.zip` trên Windows x64 thật; xác minh giao diện CAD TBT, LINE/PLINE/CIRCLE/MOVE, chuột, ESC, autosave, mở file.
5. Mở DWG/DXF thật với kích thước 100/300/500/>512MiB, đo RAM, UI latency, parse, render; không kết luận "không giới hạn" chỉ từ reader không có hard cap.

## Tiến độ 8 hạng mục đang làm — không thay đổi khi thiếu PASS
| Hạng mục | Ước lượng R19 | Trạng thái nghiệm thu |
|---|---:|---|
| Nhấp chuột đầu tiên CAD | 45% | Chưa có GUI Windows PASS |
| Hủy chọn/ESC/focus | 45% | Chưa có GUI Windows PASS |
| Đọc CAD >512MiB | 45% | Chưa có bộ file test thật |
| Tiến độ/ETA đọc & decode | 70% | Có code; chưa test UI PASS |
| CAD 300–500MiB mượt | 15% | Chưa có benchmark |
| Nhiều file, RAM | 50% | Có FIFO; chưa benchmark |
| Account và thiết lập phân quyền | 35% | Có guard client; server test còn thiếu |
| Windows CI + Setup/Portable + Source | 10% | Workflow đã cấu hình; chưa có artifact được xác minh |

Trung bình kỹ thuật (45+45+45+70+15+50+35+10)/8 = **39,4%**, là ước lượng chưa nghiệm thu. Mốc **19/30** chỉ biểu thị đã thực hiện 19 lượt thay đổi mã nguồn; còn 11 lượt trong kế hoạch dự phòng. Không có bằng chứng EXE thật cho R19 từ kết nối hiện tại.

## Chứng cứ mã nguồn
- `13aa97201386390ff5662a4806127bdb5a2df0cf`: thêm `git archive`, checksum ba tệp, artifact nguồn.
- `dacacfd0b15a19110d821c7b55b399abea09fcf1`: xác minh cấu trúc source archive từ commit.
- `eabe2e19c89ac896b6a932ff69e8eabe526a9ff3`: bảng tiến độ.

## Ưu tiên R20
Đọc log Windows Actions thực tế, sửa blocker biên dịch/đóng gói; đồng thời tập trung click đầu tiên, ESC/drag, DXF lớn, render và autosave. Chưa khởi động lại nhóm đồng bộ/xung đột.
