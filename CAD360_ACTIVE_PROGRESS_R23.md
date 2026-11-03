# CAD360 TBT — R23/30: Giảm copy dữ liệu ở đường đọc file CAD lớn

Ngày 03/11/2026. Nhánh `tbt/cad360-active-release`.

## Phạm vi
Ưu tiên: CAD mouse/ESC/viewport, file CAD lớn, RAM/giải mã/render/zoom-pan, autosave/restore, Windows EXE và quyền đăng nhập/phân quyền account. Tiếp tục tạm dừng đồng bộ online/offline và xử lý xung đột hoặc clash giữa bộ môn.

## Các thay đổi thực tế
1. Trước R23, `read_progressive` tạo `chunk = vec![0u8; 1MiB]`, gọi `file.read(&mut chunk)` rồi `output.extend_from_slice(&chunk[..n])`. Điều này sao chép thêm tối đa 1MiB mỗi vòng đọc, và giữ riêng buffer 1MiB.
2. R23 dùng `output.try_reserve(read_size)`, `output.resize(old_len+read_size, 0)`, `file.read(&mut output[old_len..])`, `output.truncate(old_len+n)`. Không còn sao chép từ scratch sang output; tiếp tục cấp phát có kiểm tra và đọc chunk không hard cap.
3. Với metadata còn ít byte, `read_size` giảm đến đúng số byte còn lại; chỉ dùng khối 1MiB khi chưa có đủ thông tin để phát hiện file tăng lên so với metadata. Hạn chế zero-init 1MiB không cần thiết khi đuôi file nhỏ.
4. Test `progressive_read_preserves_partial_tail_without_zero_padding` dùng dữ liệu 2MiB+17 byte, đối chiếu toàn vẹn byte đầu/cuối và kích thước chính xác; các tests progressive/empty cũ vẫn giữ.
5. Giai đoạn R18 đã bổ sung log read_ms/decode_ms. Muốn lượng hóa cải tiến phải có benchmark Windows với bản vẽ thực.

## Những điều chưa được chứng minh
- Không có benchmark RAM peak, throughput hay thời gian decode/render mới. Đây là giảm copy ở đường đọc, **không phải streaming parser**; file vẫn đi vào `Vec<u8>` trước khi decode. Không cam kết CAD 500MiB hết lag.
- Không có bằng chứng cargo check/test Windows PASS và chưa tải được Setup/Portable/Source ZIP thật trong phiên làm việc này.
- Hàng đợi mở nhiều file và renderer cần đánh giá hiệu năng độc lập, tránh nhầm tiến độ đọc 100% với dựng hình hoàn chỉnh.

## Tiến độ 8 hạng mục active
| Mục | Ước lượng | Bằng chứng cần thu |
|---|---:|---|
| I1 Click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45% | GUI E2E |
| I2 ESC/focus/drag/viewport | 45% | GUI E2E |
| L1 Mở >512MiB | 45% | File thật decode/render |
| L2 Đọc và báo tiến độ/ETA | 70% | UI/phase accept |
| L3 Mượt 300–500MiB | 15% | CPU/RAM/GPU/FPS/P95 |
| L4 FIFO file & RAM | 50% | 5–10 file, peak RSS |
| A1 Đăng nhập/cấp quyền account | 35% | Server-side authorization |
| Q1 Windows Setup/Portable/Source | 10% | CI PASS + files + smoke |

Trung bình **39,4%** (ước lượng kỹ thuật chưa nghiệm thu, không tăng do không có kết quả benchmark). Lượt **23/30** chỉ biểu thị số lượt mã nguồn và còn 7 lượt dự kiến.

## Điều kiện nghiệm thu R24+
- Benchmark dữ liệu DXF/DWG thực 100/300/500MiB, >512MiB; ghi `read_ms`, `decode_ms`, thời gian first render, peak RSS, FPS khi zoom/pan.
- So sánh cùng dữ liệu trước/sau R23; xác nhận không có corruption và tính đúng các đối tượng CAD.
- Kiểm tra Windows CI trên nhánh đang phát triển. Nếu có lỗi test/compiler, sửa theo log và không đóng dấu success trước khi có artifact.

## GitHub commit
- `09a3cfa874ec2e376c7d734b2923c4826e806cf3` — loại buffer scratch copy + test.
- `e1fad57607404d3a271f9ff9f9fa9946d2722c56` — đọc đúng kích thước block cuối.
- `8f69b45bc56a09a3288259a5a17e8c31887db000` — tracker R23.
