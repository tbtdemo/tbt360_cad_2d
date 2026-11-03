# CAD360 R08 — Báo cáo chi tiết thực hiện và kế hoạch nghiệm thu

Ngày: 03/11/2026. Phạm vi: GUI CAD input, file lớn, pipeline mở nhiều file và quyền phối hợp.

## Trạng thái chứng cứ
- Đã đẩy mã sửa canvas, ETA đọc file, và thứ tự chạy decoder nhiều file.
- Đã viết test hủy kéo chọn, stage progress và queue; chưa xác nhận chạy PASS trên Windows cho các commit mới.
- Chưa chạy benchmark/GUI với DWG/DXF 300–500 MiB. Không được diễn giải như đã đạt yêu cầu file lớn.
- Chưa kiểm tra mọi lệnh với hệ thống quyền cấp server; không được tuyên bố đã xử lý xong cộng tác.

## Bảng đánh giá theo hạng mục (ước lượng kỹ thuật, không phải % nghiệm thu)
| ID | Đầu việc | R07 | R08 ước lượng | Đã sửa | Chưa hoàn tất | Mốc mục tiêu |
|---|---|---:|---:|---|---|---|
| I1 | Nhấp đầu tiên nhận điểm CAD | 45% | 45% | Đường click canvas từ R07 | GUI tất cả lệnh LINE/CIRCLE/PLINE/MOVE và menu focus | R09–R11 |
| I2 | Không dính kéo chọn, bôi đen | 35% | 45% | ESC, focus loss, release ngoài canvas xóa drag | GUI mouse capture, multi viewport, context menu | R09–R11 |
| L1 | Mở file vượt trần 512 MiB | 45% | 45% | Chunked read từ R07 | Parser/renderer không giới hạn cứng, RAM thực | R09–R14 |
| L2 | Thanh % byte và thời gian đọc | 65% | 70% | ETA đọc byte; tách parse | Progress parse/render và ETA tổng đo được | R09–R12 |
| L3 | File lớn 300–500 MiB mượt | 15% | 15% | Nền đọc file & queue | Bộ thử thực, tiled render, memory budget | R11–R20 |
| L4 | Mở nhiều file không đè RAM | 45% | 50% | Không khởi động parse mới trước khi finish_open | Benchmark peak RAM, cancel và hàng đợi hàng chục file | R09–R14 |
| C1 | Quyền theo người/tầng/bộ môn/workset | 35% | 35% | BIM access policy cũ | Gate quyền phía server cho mọi mutation | R11–R20 |
| C2 | Khóa/chống xung đột cộng tác | 20% | 20% | Nền BIM workset/collab cũ | Lease + revision + audit nhiều client | R12–R23 |
| Q1 | CI Windows / EXE | 10% | 10% | Có workflow và unit tests | CI mới PASS, GUI QA, Setup/Portable thực | R15–R30 |

Trung bình ước lượng đều 9 đầu việc: (45+45+45+70+15+50+35+20+10)/9 = **37,2%**. Đây là ước lượng chưa được thực chứng.
Nhịp phát hành: **8/15 = 53,3%** trên ngưỡng đóng gói tối thiểu, KHÔNG phải độ hoàn thiện phần mềm.

## Gate và thời gian còn lại
- R09–R11: kiểm thử chuột và lệnh CAD + phản ứng focus/ESC, theo dõi tiến độ mở file, sửa lỗi build nếu có.
- R12–R14: bench DXF/DWG 100/300/500 MiB, đồng thời đo parse, render và RAM; kiểm tra quyền phối hợp với nhiều user.
- R15: chỉ phát hành nếu Cargo Check/Test, GUI test, bộ cài Setup/Portable, checksum và test mở file thật đều đạt.
- R16–R30: phiên dự phòng cho decoder/render và xung đột phối hợp.
- Còn ít nhất 7 lượt đến R15. Ước lượng **7–14 ngày công kỹ thuật và 2–5 ngày công QA** nếu không phải thay parser/render; đây không phải cam kết tiến độ thời gian thực hay chạy nền.

## Kiểm thử bắt buộc kế tiếp
1. LINE: chọn công cụ rồi nhấp đầu tiên tạo điểm đầu; thoát công cụ không mắc chọn.
2. ESC giữa thao tác kéo chọn, bỏ focus, release ngoài canvas, vào lại canvas không tự bôi đen.
3. Mở file 10/100/300/500 MiB: hiển thị % đọc theo byte, khi parse không hiển thị 100% tổng công việc.
4. Mở 5 file song song: thực tế chỉ một decoder chạy; sau khi finish_open mới mở file kế tiếp.
5. 2 user cùng file, khác tầng/bộ môn: quyền chỉ xem không thành quyền sửa; API reject mutation sai scope.
6. Benchmark P95 input latency và peak RAM; kiểm thử không ghi đè file dự án gốc.

## Commit thực tế R08
- Canvas release / focus / ESC: `150ba828c6f61155b00b10a8eaf1b1bce21b15b1`
- ETA đọc file + test stage: `ff6ae5d485587a478cefbd74781df2a2ea1c7b2c`
- Test drag trạng thái: `f8270491c1f49c7852507237633a321756995fc2`
- Hoãn decoder tiếp: `af9d308c9e527ec384ac9dbeece3132489e5a7d5`
- Kết nối UI finish_open: `f0cfc0fbf7d50f29f0840df0f89cfd3be457e249`
