# CAD360 TBT — Báo cáo R12/15 (03/11/2026)

## Giới hạn xác minh
Đã cập nhật mã nguồn GitHub trên nhánh `tbt/cad360-active-release`. Chưa có bằng chứng `cargo check`/Rust tests/GUI Windows PASS cho commit R12. Truy vấn runs gắn SHA trong trình kết nối trả `[]`, nên không suy ra job chưa chạy: nguồn này chỉ lọc PR-triggered workflows.

## Thay đổi R12
- Canvas: không replay thêm điểm khi lệnh chuyển sang prompt không nhận tọa độ; điểm cuối trong batch không bị xử lý như click idle.
- Canvas: tắt PICKDRAG giữa thao tác kéo phải hủy đúng pending window có nguồn gốc kéo.
- Tests: kiểm tra LINE nhận điểm thứ nhất, chuyển qua điểm thứ hai và sinh một đối tượng; CIRCLE nhận điểm tâm lần thứ nhất.
- Phải thực hiện xác minh trên runner và GUI thực để xem các unit tests đã pass hay chưa.

## Tiến độ từng hạng mục (ước lượng sơ bộ giữ nguyên vì thiếu nghiệm thu)
| ID | Công việc | % R12 | Bằng chứng | Còn thiếu | Số lượt dự kiến |
| --- | --- | ---: | --- | --- | --- |
| I1 | LINE/PLINE/CIRCLE/MOVE click đầu tiên | 45 | Patch prompt + tests LINE/CIRCLE | GUI trên Windows, kiểm thử 100 lần/lệnh | R13–R16 |
| I2 | ESC/drag/focus và PICKDRAG | 45 | Patch xóa drag sở hữu pending window | GUI test trong/ngoài canvas | R13–R16 |
| L1 | OPEN >512MiB không hard-cap | 45 | Chunk read từ R07 | Codec DWG/DXF, RAM peak, parser streaming | R13–R20 |
| L2 | Thanh % đọc thực tế/ETA | 70 | UI từ R07–R09 | Tiến độ parse/render thật | R13–R18 |
| L3 | File 300–500 MiB thao tác mượt | 15 | Reader chạy thread | Benchmark thực, GPU tiling | R13–R24 |
| L4 | Nhiều CAD, FIFO và kiểm soát RAM | 50 | Queue FIFO | 5–10 file peak RAM | R13–R20 |
| C1 | Quyền sửa theo người/tầng/workset | 35 | Policy BIM | Server write enforcement | R14–R24 |
| C2 | Lock, conflicts và audit | 20 | Mô-đun BIM hiện có | Tích hợp 2 client & API | R14–R26 |
| Q1 | CI Windows và Setup/Portable | 10 | Có workflow và mã test | Build, test PASS, artifact EXE thật | R15–R30 |

Trung bình 9 đầu việc là **37,2% ước lượng**, không phải % kiểm thử đã PASS. Nhịp phát hành **12/15 = 80%** đến mốc đóng gói tối thiểu, không phải độ hoàn thiện phần mềm.

## Bước tiếp theo và thời lượng ước lượng
- R13: chạy kiểm thử build, benchmark file 100/300/500MiB và log memory nếu có runner thực.
- R14: rà soát scope người dùng, phân quyền mutation server và test 2 tài khoản.
- R15: chạy gate và chỉ phát hành khi đủ các kết quả CI, setup, portable, SHA256, cài thử và mở file thật. Không được coi R15 là ngày giao hàng bắt buộc.
- R16–R30: dự phòng tái kiến trúc parsing/rendering và sửa lỗi.
- 3 lượt tối thiểu còn lại: **3–6 ngày công phát triển + 2–5 ngày công QA**, nếu không phát sinh lỗi cấu trúc. Khả năng vượt 15 lượt là thực tế nếu chưa đạt gate, không tự động hoàn thành theo số lượt.

## Commit
- `92003c26a1d9667cb9c819e5ba3893018543c0e6`: prompt point transition guard.
- `84a852ffb93c38f72e52e06e5dba7d46d9bfeeae`: LINE/CIRCLE regression test.
- `b0c704f75088f8b22d1638054f7a9c4a2ff452a7`: clear owned pending drag when PICKDRAG switches off.
- `17e601fc085410d667d0846cb8fd301309e03be6`: batch tracker.
