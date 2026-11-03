# CAD360 R15 — Windows Release Candidate Gate (03/11/2026)

## Tình trạng chính xác
- **15/15 lượt phát triển tối thiểu** = 100% số lượt, KHÔNG phải 100% phần mềm và KHÔNG xác nhận có bộ cài.
- Đã cập nhật workflow `.github/workflows/main.yml` để chạy Windows CI khi push branch `tbt/cad360-active-release`; trước đó chỉ push `main`.
- Đã bổ sung test gates theo tên cụ thể cho canvas click, point release, file opening và progressive reading.
- Khi sáu nhóm kiểm thử ở `needs` thành công, CI sẽ build hai binaries, Inno Setup, Portable ZIP và SHA256, upload artifacts phân biệt `Candidate` so với `Release` của main.
- **Chưa có kết quả Windows CI PASS, chưa có artifact hoặc đường tải Setup/Portable được xác minh trong phiên làm việc này.** Connector run-by-SHA hiện chỉ trả PR-triggered runs; kết quả [] và statuses [] không xác nhận pipeline chưa chạy hay đã thành công.
- **Chưa có kết quả GUI thật, RAM/VRAM và benchmark DXF/DWG 300–500 MiB hoặc kiểm thử 2-user authoritative server.** Không cho gắn nhãn production-ready.

## Gate chi tiết
| Gate | Điều kiện | Trạng thái |
|---|---|---|
| 1. Trigger nhánh active | push SOURCE_CAD_R65 hoặc workflow thì kích hoạt Windows CI | Đã cấu hình, cần xác minh run |
| 2. Cargo metadata/locked | cargo metadata --locked thành công | Chưa xác nhận |
| 3. Cargo workspace check | cargo check --workspace --locked | Chưa xác nhận |
| 4. CAD input tests | canvas::point_batch_tests, canvas::point_release_tests | Có lệnh trong workflow, chưa xác nhận PASS |
| 5. Large-file tests | opening::tests, capped_read::tests, IO regressions | Có lệnh trong workflow, chưa xác nhận PASS |
| 6. Workspace/BIM QTO XREF | regression-tests, bim-module-regressions, qto-xref-smoke | Đang là gates, chưa có PASS |
| 7. Native EXE binaries | cadcraft.exe, cadcraft-cli.exe > 100KB, CLI --version | Có điều kiện trong workflow, chưa có binary |
| 8. Installer Setup | Inno Setup tạo CAD360_VietNam_TBT_Setup.exe | Chưa có artifact |
| 9. Portable | ZIP, smoke CLI, kiểm tra SHA256SUMS | Chưa có artifact |
| 10. Nghiệm thu giao diện | LINE/PLINE/CIRCLE/MOVE click đầu tiên, ESC, drag | Chưa kiểm thử |
| 11. Dự án dữ liệu lớn | 100/300/500/>512MiB, parse/render, RAM peak, FPS | Chưa benchmark |
| 12. Cộng tác nhiều người | 2 tài khoản/2 client, kiểm tra scope server, lease, conflict | Chưa nghiệm thu |

## % đầu việc ước lượng (chưa thay đổi khi thiếu bằng chứng)
| ID | Hạng mục | % |
|---|---|---:|
| I1 | Click đầu tiên LINE/PLINE/CIRCLE/MOVE | 45 |
| I2 | ESC/drag/focus không kẹt | 45 |
| L1 | Đọc file không giới hạn fixed 512 MiB tại OPEN | 45 |
| L2 | Tiến độ byte / ETA và pha decode | 70 |
| L3 | CAD 300–500 MiB zoom/pan/select mượt | 15 |
| L4 | Mở nhiều bản vẽ đúng FIFO, giảm đỉnh RAM | 50 |
| C1 | Quyền người/tầng/bộ môn/workset | 35 |
| C2 | Lease/conflict/đồng bộ hai client | 20 |
| Q1 | Windows CI và EXE Setup/Portable | 10 |

Trung bình 9 mục **37,2% ước lượng kỹ thuật**, không được hiểu là phần trăm nghiệm thu. Dù đã đến R15, cần xử lý R16+ nếu chưa đủ gate.

## Các commit đã thực hiện
- `bae62df9683cadf7cdbf0cf6337221559e8210e3` — workflow branch trigger + targeted Windows gate + Candidate/Release artifact naming.
- `5d442914d5b2131c9820139467bc25357300ff92` — sửa test LINE đếm model entities.
- `9d26c9005e3f29ac0e653f73e39d9c41940d35f7` — tracker R15.

## R16 đề xuất
- Mở Actions workflow trên nhánh active, đọc job kết quả thật, sửa nhóm compiler/test blockers trước khi đụng hiệu năng hoặc giao diện.
- Khi tất cả gates PASS, xác minh Setup/Portable candidate và checksum thật, kiểm thử khởi động/GUI.
- Bổ sung benchmark tự động với dữ liệu CAD giả lập và dữ liệu thực >512MiB, công bố thời gian/RAM thay vì suy đoán.
- Nếu có lỗi, tiếp tục các lượt R16–R30 với % từng đầu việc và bằng chứng. Không ghi release thành công trước khi có EXE tải được.
