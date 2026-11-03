# CAD360 R13/15 — Báo cáo hiệu năng CAD lớn, tiến độ và nghiệm thu (03/11/2026)

## Trạng thái R13 đã xác minh
- Đã commit mã nguồn trên nhánh `tbt/cad360-active-release` qua GitHub connector.
- Chưa có `cargo check`, `cargo test`, GUI Windows hay benchmark 100/300/500 MiB PASS. Shell hiện tại không có Cargo, Git remote lỗi DNS; commit workflow-run query của R13 trả [] (chỉ lọc PR workflows, không kết luận CI push status).
- Phần trăm tiến độ dưới đây là ước lượng kỹ thuật, **không phải phần trăm kiểm thử đã PASS hoặc mức hoàn thiện của toàn CAD360**.
- **13/15=86,7%** chỉ là nhịp phát triển đến ngưỡng ĐƯỢC XÉT đóng gói, không phải phần mềm đạt 86,7%.

## Các thay đổi có mã nguồn
1. Đường mở file đọc thực tế đủ dữ liệu mới bắn callback bắt đầu giai đoạn parse; không đồng nhất 100% byte đọc với parse thành công.
2. UI tách giai đoạn reading/parsing dựa trên callback này.
3. Cấp phát đầu vào theo metadata ở mức tối đa 8 MiB + try_reserve; vẫn đọc khối 1 MiB, không đặt giới hạn tổng file.
4. Test tiến độ giai đoạn 100% byte vẫn ở trạng thái đọc nếu parser chưa bắt đầu; test file rỗng.
5. Đường render / decoder không được làm ngầm thành hệ thống streaming: đầu vào vẫn materialize thành Vec và engine có thể cần nhiều RAM. Đây là giới hạn kỹ thuật quan trọng.

## Theo dõi 9 hạng mục
| Mã | Hạng mục | R12 | R13 (ước lượng) | Đã làm | Chưa làm / tiêu chí PASS |
|---|---|---:|---:|---|---|
| I1 | Click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45 | 45 | Bản sửa R07–R12 | GUI tests ≥100 lần/lệnh |
| I2 | Không dính chuột/ESC/drag/focus | 45 | 45 | Input guards R07–R12 | Windows GUI hành vi |
| L1 | OPEN >512MiB không cap kích thước | 45 | 45 | Chunk read, reserve bounded | Thực mở file >512MiB, memory peak |
| L2 | Thanh % đọc byte, ETA và pha parse | 70 | 70 | Phase callback đúng ranh giới | % parse/render thực tế và GUI |
| L3 | 300–500MiB zoom/pan/select không lag | 15 | 15 | Nền đọc chunk/queue | Benchmark RAM/VRAM, FPS, P95 latency |
| L4 | Nhiều file FIFO và giảm RAM peak | 50 | 50 | FIFO, serial decoder, reserve nhỏ | 5/10 file concurrent and peak RSS |
| C1 | Phạm vi sửa người/tầng/bộ môn/workset | 35 | 35 | Policy sẵn có | Backend mutation authorization |
| C2 | Lock/lease/conflict 2 user | 20 | 20 | Có BIM collaboration groundwork | Test hai máy và offline/online |
| Q1 | Windows Cargo + Setup/Portable | 10 | 10 | Mã đã commit | Cargo check/test, GUI QA, EXE tải/cài |

Trung bình không trọng số vẫn **37,2%**, cố ý không tăng khi chưa có bằng chứng kiểm thử pass.

## Bảng thử hiệu năng cần có
| Loại file | Mốc mục tiêu | Thời gian đọc | Thời gian parse | First render | Peak RAM | Peak VRAM | P95 thao tác |
|---|---|---|---|---|---|---|---|
| DXF 10 MiB | smoke | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |
| DXF 100 MiB | trung bình | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |
| DXF 300 MiB | lớn | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |
| DXF 500 MiB | lớn | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |
| DXF >512 MiB | kiểm thử không hard-cap | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |
| DWG 100/300/500 MiB | codec compatibility | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo | Chưa đo |

Không đưa giá trị giả vào benchmark. Máy Windows và bộ dữ liệu mẫu thực là điều kiện bắt buộc.

## Dự kiến tiếp theo
- R14: audit quyền mutation và xung đột cộng tác, khắc phục vấn đề build nếu có log; tăng coverage input và performance nếu có runner.
- R15: release gate: CI Windows cargo check/test, GUI acceptance, benchmark tối thiểu, SHA256 Setup/Portable và cài thử. Chỉ gắn nhãn RELEASE nếu bằng chứng đầy đủ.
- R16–R30: tối đa 15 lượt dự phòng để cải tạo streaming parser, GPU culling, cache và kiểm soát conflict nếu R15 chưa đạt.

Khối lượng từ R13 đến R15: còn ít nhất 2 lượt, tương đương khoảng 2–4 ngày công lập trình + 2–5 ngày công QA trong điều kiện có Windows runner, bộ dữ liệu thật và nguồn lực thực hiện liên tục. Đó là ước lượng, không phải cam kết lịch và có thể tăng nhiều khi cần thay engine parser/render.

## Links commit
- `5670d1c36a88f31607ae0ecb1d9aaee5ab6d2813`: callback bắt đầu decode.
- `7abb7f30516ab600142fbb116d8146f0315052d6`: UI thay đổi và phase test.
- `e4447bdb11a87069fc3fcdd23aeb11f7c5108188`: reserve 8MiB bounded và test empty.
- `d3aed98cdea9256261c211fafe83c6b6f88e118f`: cập nhật R13 tracker.
