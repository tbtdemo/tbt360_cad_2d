# CAD360 R10 — Báo cáo kỹ thuật, % đầu việc và điều kiện phát hành

Ngày 03/11/2026. Nhánh: tbt/cad360-active-release.

## Kết quả thực hiện trong R10
1. Canvas: thêm chặn lớp giao diện nổi vào đường nhận release liên tiếp (nhiều điểm liên tục); không lấy cú click trên menu làm tọa độ lệnh.
2. File lớn: giữ hàng đợi FIFO khi một yêu cầu mở mới đến đúng khoảng worker cũ đã hoàn tất nhưng chưa được add vào Session; test có 3 file và kiểm tra không vượt hàng.
3. Đã commit, chưa có build/test Windows PASS; không đánh dấu các lỗi chuột là triệt để.

## Định lượng theo từng hạng mục
| ID | Hạng mục | R09 | R10 - ước lượng | Bằng chứng | Chưa xong / Gate |
|---|---|---:|---:|---|---|
| I1 | Nhận click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45% | 45% | Cải thiện lọc release qua canvas/overlay | Test UI Windows liên tục 100 lượt / mỗi lệnh |
| I2 | Không kẹt chuột và selection window | 45% | 45% | Xử lý focus/ESC/R08, menu/R10 | Kiểm thử drag và focus thật |
| L1 | Mở CAD không hard cap 512MiB | 45% | 45% | Progressive read từ R07 | Decode DWG, thiếu kiểm thử >512MiB |
| L2 | Progress và ETA đọc CAD | 70% | 70% | UI hiển thị read bytes | Đếm decode/render và ETA tổng chính xác |
| L3 | File CAD 300–500MiB mượt | 15% | 15% | Có đọc nền | Benchmark RAM/VRAM, FPS, pan/zoom, render |
| L4 | Giảm đỉnh RAM khi mở nhiều CAD | 50% | 50% | FIFO sửa và thêm unit test | Test 5–10 file, đo peak RAM |
| C1 | Quyền theo account/tầng/bộ môn/workset | 35% | 35% | Có chính sách BIM | Server enforcement mutation |
| C2 | Workset lease/conflict/offline | 20% | 20% | Mô-đun BIM hiện hữu | 2 user test và audit |
| Q1 | Windows CI và EXE | 10% | 10% | Các commit đã lên git | CI mới và EXE thực |

Trung bình 9 hạng mục: 37,2% (ước lượng, KHÔNG phải kiểm thử đạt). Giữ nguyên % vì chưa có PASS, không cộng số lượng commit.
Nhịp đóng gói: 10/15 = 66,7%; còn ít nhất 5 lượt trước thời điểm có thể đóng gói định kỳ.

## Kế hoạch thời gian và mốc
- R11: xác nhận hiệu lực của point input đa lệnh và drag-window, sửa CI nguồn.
- R12–R13: benchmark 100/300/500 MiB, tìm bottleneck parse vs GPU.
- R14: phạm vi user, xung đột và các lỗi nghiệm thu.
- R15: chạy Windows test, Setup/Portable nếu qua gate; không phát hành nếu thiếu artifact.
- R16–R30: dự phòng bug build, render và phân quyền.
- Dự kiến 5–10 ngày công lập trình tới R15, thêm 2–5 ngày công kiểm thử/đóng gói; không phải thời hạn chắc chắn.

## Commit
- 3d6e8ce24b4cf5f0c15ed6fec53a789c5ce91693: chặn nhập điểm từ overlay.
- 0b9297f2a4a093978d02f6e58de373c653cb40b5: FIFO trong load tuần tự.
- 83fc45261dbb4c84fc48afd646332b864abb2a29: tiến độ R10.
