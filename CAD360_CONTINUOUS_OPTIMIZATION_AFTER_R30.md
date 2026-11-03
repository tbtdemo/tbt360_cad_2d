# CAD360 TBT — phát triển liên tục sau R30, đóng gói theo bộ 15–30

Ngày 03/11/2026. Nhánh phát triển: `tbt/cad360-active-release`.

## Nguyên tắc đã chốt
1. **Không dừng phát triển tại R30/30**: R30 chỉ là chốt một đợt mã nguồn, chưa phải 100% phần mềm.
2. Tiếp tục tối ưu **từ mã CAD/Rust hiện có**, ưu tiên sửa chung nguyên nhân trong engine/UI/renderer thay vì chắp vá từng lệnh, giữ UI CAD360 TBT và Việt hóa.
3. Một bộ thay đổi có code/test/đối chiếu mới được tính vào lượt kỹ thuật. **Chỉ đóng gói EXE tại mốc có từ 15 đến 30 bộ thay đổi**, một lần cho mốc, gồm Setup.exe, Portable.zip và Source.zip đúng SHA (không đóng gói riêng mỗi commit).
4. `.github/workflows/tbt-exe-direct.yml` chỉ chạy jobs build khi `workflow_dispatch` hoặc commit có tiền tố `build(cad360): B15...B30`/`release(cad360): B15...B30`. Các commit `fix`, `perf`, `test`, `docs` không bắt đầu build EXE; concurrency không được hủy một build mốc khi tiếp tục sửa code.
5. Code được lưu qua GitHub mỗi lần sửa, không làm mất checkpoint giữa hai lần đóng gói.
6. **Tạm dừng** xây dựng mới conflict resolution, sync đa máy và clash bộ môn; **giữ** tài khoản, quyền truy cập/xem/sửa, floor/discipline.

## Chu kỳ tiếp tục — nhóm sửa đầu tiên
- `856bfe958f146089d5ec959ef35057f9100cc2a1`: điểm CAD được xác định từ sự kiện nhả chuột vật lý cuối, không lấy con trỏ hover đã trôi sang frame/overlay khác khi bản vẽ lớn làm UI giật. Thêm 2 unit tests cho tọa độ/click mode.
- `f42fdb25c08895705525e69f0a6fb0eeb72371a6`: render filters của 1–4 viewport chỉ duyệt model một lần thay vì reparsing BIM metadata trên mỗi viewport.
- `8ace3232553f8581c053539e9cd5678be1f67366`: kiểm quyền nhìn chủ sở hữu một lần cho từng entity.
- `fe92147a82a98655d0c89a56f1c99d5aa5c6fae1`: test phân loại primary/reference, quyền xem, owner, đúng/sai tầng qua bốn viewport.
- `b53dace72f72accdf66300e49f7d8d48aef02d46`: Windows CI bắt buộc nhận diện và chạy tests về click đầu và routing viewport.
- `53dc19b2010ed3607ccb859c19a50f369c3574a8`, `631f7e712b33d1246c58de915c2295fd7db3c88d`, `b177c9efdf8c9fca026b891809ec477fb033c727`: giữ build mốc, cho phép B15..B30, Source ZIP động và ngăn EXE build trên commit thông thường.

## 8 nhóm công việc và điều kiện nghiệm thu
| Nhóm | Ước lượng ban đầu (không tăng khi chưa PASS) | Tiêu chí chứng cứ |
| --- | ---: | --- |
| Lệnh CAD — LINE/PLINE/CIRCLE/MOVE, first click | 45% | GUI Windows 100 thao tác/lệnh; không nhận nhầm 2 click |
| ESC, drag selection, focus, nhiều viewport | 45% | GUI E2E 1–4 viewport, menu/toolbar/ESC/focus |
| Mở CAD >512 MiB | 45% | DXF/DWG thật được decode, install và render toàn bộ |
| Thanh tải, tiến độ và ETA từng giai đoạn | 70% | % đọc dựa trên bytes; decode/install trạng thái thật; UI kiểm chứng |
| Mượt CAD 300–500 MiB | 15% | FPS, CPU/GPU, peak RSS/VRAM, input latency P95 |
| Mở nhiều file, kiểm soát RAM | 50% | 5–10 bản vẽ cùng lúc, peak RSS, không leak |
| Tài khoản, thiết lập phân quyền | 35% | Mutation authorization engine/API, phạm vi account/floor/discipline |
| Windows CI, EXE, Portable, Source | 10% | CI đúng SHA SUCCESS, 3 artifacts, SHA256, smoke trên Windows |

Tổng các ước lượng = 315%, chia 8 = **39,375% ≈ 39,4%**. Đây là baseline **chưa nghiệm thu**, không phải mức hoàn thiện thực tế.

## Trạng thái Windows tại lúc ghi
- Run <https://github.com/tbtdemo/tbt360_cad_2d/actions/runs/41593081243> là bản ứng viên B30 của chu kỳ trước. Khi kiểm tra còn queued, **chưa có binary artifact được xác minh**.
- Các run kiểm thử mới có thể queued hoặc bị rút khỏi hàng do cập nhật code. Không dùng run cũ để chứng nhận code HEAD mới.
- Không công bố link EXE tải về khi chưa có kết quả PASS cùng file thật.
- Ưu tiên phát triển tiếp; tới mốc 15–30 bộ thay đổi sẽ đóng gói **một bộ** cùng nguồn, không đóng gói lặp lại theo từng commit.
