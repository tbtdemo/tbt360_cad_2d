# CAD360 R11 — Kiểm soát lệnh, click nhanh và nghiệm thu

Ngày 03/11/2026. Nhánh: tbt/cad360-active-release.

## Thay đổi đã đẩy
- Giữ tọa độ thật của từng sự kiện thả chuột khi một frame gom nhiều sự kiện. Không tái sử dụng snap của cú nhấp cuối cho các sự kiện trước đó.
- Ngay khi lệnh kết thúc ở một release trước đó, dừng nhập các release còn lại và không cho release cuối vô tình chọn đối tượng khi không còn lệnh.
- Bổ sung hai test cho tọa độ của các release khác nhau và điều kiện ngăn click dư. Test mới chỉ ở mã nguồn, chưa xác minh PASS trên Windows.
- Nghiệm thu GUI còn mở cho LINE, PLINE, CIRCLE, MOVE, các tổ hợp mouse + menu + dynamic input.

## 9 đầu việc đang theo dõi
| Hạng mục | % ước lượng kỹ thuật R10 | % R11 | Bằng chứng còn thiếu |
| --- | ---: | ---: | --- |
| I1 Click đầu tiên / lệnh tương tác | 45 | 45 | 100 lần click/lệnh, GUI Windows |
| I2 Drag/ESC/focus, không dính chuột | 45 | 45 | GUI hủy và đổi lệnh |
| L1 OPEN CAD không hard cap 512MiB | 45 | 45 | DWG/DXF trên 512MiB, peak RAM |
| L2 Thanh tiến độ byte / ETA | 70 | 70 | Parser/render progress định lượng |
| L3 CAD 300–500MiB mượt | 15 | 15 | Benchmark CPU/GPU/FPS và lag |
| L4 Nhiều file, FIFO + kiểm soát RAM | 50 | 50 | Peak RAM 5–10 file |
| C1 Phạm vi quyền người/tầng/workset | 35 | 35 | Test mutation server |
| C2 Lease/conflict khi nhiều người sửa | 20 | 20 | Test 2 người & offline |
| Q1 Windows CI/EXE | 10 | 10 | CI PASS, Setup, Portable |

Trung bình không trọng số **37,2%**; cố ý giữ nguyên ước lượng do chưa có bằng chứng nghiệm thu mới. Nhịp phát hành **11/15 = 73,3%**; còn 4 lượt đến mốc đóng gói tối thiểu.

## Nhiệm vụ bắt buộc R12–R15
- R12: kiểm thử chức năng lệnh tương tác và sửa mọi lỗi Rust build phát hiện từ pipeline; test event batched, nút ESC, menu.
- R13: bài đo 100/300/500MiB và >512MiB; thống kê thời gian file đọc, parse, render, RAM/VRAM peak.
- R14: kiểm thử quyền và xung đột 2 tài khoản, phối hợp theo tầng / workset.
- R15: kiểm kê, build Windows và đóng gói Setup/Portable **chỉ nếu tất cả gate đạt**. Nếu chưa đạt, tiếp tục sửa ở R16–R30.
- Còn 4 lượt tối thiểu, khoảng 4–8 ngày công kỹ thuật và 2–5 ngày công QA theo giả định 1–2 ngày/lượt, chưa phải cam kết thời gian thực tế.

## Kiểm tra CI hiện tại
- Query workflow runs gắn commit `4f4ff825fa64b0e7894ee94b48e020a72b5b89cf` trả về [] (connector chỉ truy vấn PR-triggered runs).
- Query combined commit statuses trả về []. Đây không phải kết quả test PASS hoặc FAIL.
- Đã push mã nguồn, chưa có EXE mới được xác minh.

## Chứng cứ GitHub
- `d11a845170ce0ca9bac5211a357430f1cadc17c6` vị trí nhấp trong frame
- `be5b3f6c2e9b43a992a73355a5465caa17252d07` dừng nhập khi lệnh kết thúc
- `cecce2a2d9e5cd5674daf1f97e8edf174945b762` chặn selection click dư
- `4f4ff825fa64b0e7894ee94b48e020a72b5b89cf` regression tests
- `6368f4bb89a8cdfa46f94be7545bf61eb4a44d51` tracker R11
