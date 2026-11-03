# CAD360 TBT — R18/30: Instrumentation cho CAD lớn và kiểm tra Windows

Ngày 03/11/2026. Branch `tbt/cad360-active-release`.

## Phạm vi theo chỉ đạo
- **Tạm dừng** đồng bộ online/offline, cơ chế xử lý xung đột giữa các máy/người và xung đột bộ môn. Không phát triển mới các module này, không xóa code cũ.
- **Tiếp tục** tài khoản và cấu hình phân quyền xem/sửa theo user, tầng/bộ môn.
- Ưu tiên: lỗi chuột và lệnh CAD; mở/giải mã CAD lớn; RAM/render/zoom/pan; tự động lưu và phục hồi; Windows CI và EXE thực tế.

## Thay đổi nguồn R18
1. `opening.rs`: đồng hồ `read_done_ms` bắn đúng tại giai đoạn từ đọc sang decode, không suy đoán từ metadata.
2. Worker log `CAD OPEN` có `read_ms`, `decode_ms`, `total_ms`, kết quả OK/ERROR trên mỗi file. Đây là cơ sở cho đo bottleneck khi có dữ liệu test lớn.
3. UI hiển thị thời gian đọc và thời gian đang decode riêng biệt. Không hiển thị % giả của decode/render.
4. Đồng nhất `Instant` của UI và worker để tránh lệch đồng hồ, bổ sung regression test giai đoạn bắt đầu decode.
5. R17 trước đó đã bổ sung Windows CI targeted regression cho LINE/CIRCLE/ESC, mouse selection và opening.

## Bảng % 8 đầu việc đang thực hiện
| ID | Công việc | Ước lượng R18 | Điều kiện phải có trước khi nghiệm thu |
|---|---|---:|---|
| I1 | Lệnh LINE/PLINE/CIRCLE/MOVE nhận click một lần | 45% | Windows GUI E2E nhiều lượt |
| I2 | Không kẹt chuột, drag và ESC | 45% | GUI focus/cancel ở nhiều cửa sổ |
| L1 | Mở CAD không giới hạn input cap 512 MiB | 45% | File thật >512MiB decode/render |
| L2 | Thanh byte progress và ETA đọc/decode | 70% | % decode/render có nguồn đo hoặc chỉ hiển thị giây thực |
| L3 | Thao tác mượt CAD 300–500 MiB | 15% | Benchmark CPU/GPU/RAM/VRAM/P95 input |
| L4 | FIFO nhiều file và RAM peak | 50% | Nhiều CAD thật, đo peak RSS |
| A1 | Tài khoản và thiết lập phân quyền | 35% | Test quyền xem/sửa, scope trên API |
| Q1 | Cargo Windows, Setup EXE, Portable ZIP | 10% | CI PASS + artifact và chạy máy thật |

Trung bình 8 mục: (45+45+45+70+15+50+35+10)/8 = **39,375% ≈ 39,4%**. Chưa thay đổi do thiếu nghiệm thu. Nhóm sync/conflict đã loại hoàn toàn khỏi mẫu số, không phải bị coi là 0%.

**18/30** lượt dự kiến tối đa; còn 12 lượt trong lịch dự phòng. Không được đánh đồng tiến độ lượt với % phần mềm hoàn tất.

## Các rủi ro kỹ thuật
- Đọc file vẫn vật hóa nội dung vào `Vec<u8>`, codec có thể sao chép dữ liệu nhiều lần, render có thể dùng nhiều RAM/VRAM; chưa có streaming parser end-to-end và geometry LOD.
- `CAD OPEN` chỉ có read/decode timing, chưa đủ chứng minh render mượt. Bài thử phải lấy first render, FPS, P95 input và RAM từ máy Windows thật.
- Không thể xác nhận EXE từ mã nguồn khi chưa có kết quả Windows cargo/test PASS và artifacts tải được.
- Truy vấn workflow runs theo commit qua connector chỉ lọc PR-triggered nên trả `[]`; statuses `[]` không kết luận PASS/FAIL hoặc chưa chạy.

## Kế hoạch thực hiện tiếp
- R19: dựa trên Windows Actions job/log có thực để sửa các build/test blockers, chuẩn hóa artifact/chữ ký checksum.
- R20–R22: sửa đầu vào chuột, first click và stuck selection bằng GUI e2e.
- R23–R26: lấy benchmark CAD 100/300/500/>512 MiB, tối ưu decoder/render/RAM theo thời gian từ log R18.
- R27–R28: xác thực autosave, phục hồi đủ hình học, mô hình và mọi tab.
- R29–R30: cấu hình phân quyền account, review UI Việt hóa và đóng gói khi các gate PASS.

## Commits
- `6c01c27b86fd231f0665f2a3d4b42553c5b1397c`: phân pha, log thời gian đọc/giải mã, test.
- `75952a357f31a6dbd553d6e02872cfcdad262cf1`: đồng nhất Instant worker và UI.
- `95f17f915efc9244c13d501d6908c0a59f928eeb`: tracker R18.
