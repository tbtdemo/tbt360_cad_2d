# CAD360 — R09: tiến độ kỹ thuật và cổng nghiệm thu (03/11/2026)

## Chỉ số
- Nhịp phát hành 9/15 = **60%**, còn 6 lượt tối thiểu; không phản ánh mức hoàn thành phần mềm.
- Nhóm tương tác + file lớn + cộng tác: **37,2% (ước lượng kỹ thuật chưa nghiệm thu)**, giữ nguyên từ R08 do chưa có bằng chứng test Windows mới PASS.
- CI trên commit R09: truy vấn workflow runs theo commit không có kết quả; KHÔNG kết luận build thành công hay thất bại.

| ID | Đầu việc | Ước lượng R09 | Tình trạng thật | Lượt hoàn thiện dự kiến |
| --- | --- | ---: | --- | --- |
| I1 | Lệnh LINE/PLINE/CIRCLE/MOVE nhận click đầu tiên | 45% | Có sửa raw release, chưa test GUI | R10–R11 |
| I2 | ESC, focus và chuột không kẹt vùng chọn | 45% | Có xử lý và unit test, chưa xác nhận GUI | R10–R11 |
| L1 | Đọc CAD lớn không hard cap OPEN 512 MiB | 45% | Có đọc chunk, chưa benchmark RAM và parse | R11–R15 |
| L2 | Thanh đọc byte %, ETA, phân pha parse | 70% | Có mã UI, test trống; parse chưa có % thật | R10–R13 |
| L3 | CAD 300–500 MiB pan/zoom/select không lag | 15% | Chưa có benchmark thật | R12–R20 |
| L4 | Mở nhiều file, hạn chế RAM peak | 50% | Decoder tuần tự, chưa đo RAM peak | R11–R15 |
| C1 | Phạm vi người dùng/tầng/bộ môn/workset | 35% | Có policy, chưa kiểm tra server mutation | R12–R20 |
| C2 | Lock, conflict, lease và audit nhiều user | 20% | Chưa có nghiệm thu đồng thời 2 client | R13–R23 |
| Q1 | Cargo, GUI Windows, EXE Setup và Portable | 10% | Chưa có PASS/EXE đợt này | R15–R30 |

## Bằng chứng đã ghi
- `2ec2ec4e1fc439c1e5231ce029ee2ad0e85a15ba` xử lý raw release đầu tiên trong point prompt.
- `5c1ac6a235a958ffc522fccaf31c17d0b7f459ce` fix báo parse ảo khi file kích thước 0.
- `0e97ab7c77072110037a658853deeaa2f8635383` regression test phân biệt canvas vs toolbar.
- `a4d546cfa2db048297f087bf94e5e03fb2d9afcc` theo dõi R09.

## Yêu cầu kiểm thử trước khi tăng %
- Click đầu tiên LINE/PLINE/CIRCLE/MOVE bắt đúng một điểm, không double, không lấy click toolbar.
- Thử nhấn ESC, thả chuột ngoài canvas, đổi cửa sổ khi kéo.
- DXF/DWG 100/300/500 MiB: wall clock parse và render, RAM/VRAM peak, FPS, P95 input; test file sai và file trống.
- Hai người sử dụng một dự án, khác tầng/bộ môn: áp quyền server cho mọi thao tác ghi.
- Windows cargo check/test + bộ cài Setup/Portable có artifact thực, tải và cài được.

## Lịch dự kiến (theo nguồn lực, không phải công việc tự chạy)
- R10–R11: input regression + sửa Rust CI (2 lượt)
- R12–R14: benchmark file lớn, render, collaboration scope (3 lượt)
- R15: build và test EXE nếu gate PASS (1 lượt)
- R16–R30: dự phòng tái kiến trúc parse/render hoặc xử lý lỗi
- Còn ít nhất **6 lượt**; nếu 1–2 ngày công/lượt + 2–5 ngày công QA, ước tính **8–17 ngày công**, có thể tăng đáng kể nếu có lỗi CI hay streaming parser cần viết lại.
