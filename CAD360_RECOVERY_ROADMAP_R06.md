# CAD360 TBT — Bảng kế hoạch kỹ thuật và nghiệm thu đợt R06 (03/11/2026)

> PHẠM VI: Nhóm CAD+BIM autosave/recovery, CI Windows và bàn giao bộ cài. Không phải phần trăm hoàn thiện toàn bộ phần mềm CAD360.
> Mỗi % dưới đây là **ước lượng kỹ thuật** dựa trên mã nguồn và bằng chứng kiểm thử hiện có; không phải tỷ lệ kiểm thử đã PASS. Nếu chưa nghiệm thu Windows thì không được đánh dấu "hoàn thành".

## Phương pháp báo cáo
- Lượt phát triển đến mốc đóng gói: 6/15 = 40% quãng đường tối thiểu; chu kỳ được phép 15–30 lượt. Đây là thước đo NHỊP PHÁT HÀNH, không phải độ hoàn thiện tính năng.
- Tiến độ kỹ thuật module: trung bình gia quyền của 10 hạng mục dưới đây. Không đưa tính năng PDF, CAD 2D toàn bộ, IFC toàn bộ, collaboration toàn bộ vào mẫu số vì chưa kiểm kê đầy đủ.
- % tăng khi có bằng chứng: nguồn hoạt động và test (CI + GUI) chứ không tăng chỉ vì thêm commit.
- Kiểm thử `skipped` không tính PASS; commit có log/CI không truy cập được phải báo "chưa xác minh".

## Bảng 10 hạng mục (ước lượng, 03/11/2026)

| ID | Hạng mục / đầu ra bắt buộc | Trọng số | Tiến độ ước lượng | Bằng chứng đã có | Thiếu / điều kiện nghiệm thu | Lượt mục tiêu |
|---|---|---:|---:|---|---|---|
| A | Tự lưu 5 phút, chỉ ghi khi thay đổi toàn bộ CAD+BIM | 10% | 70% | Cơ chế timer, snapshot, fingerprint toàn tab + BIM checksum đã viết | CI Windows + test timer, mất điện | 06–08 |
| B | Snapshot nhiều bản vẽ với UID/revision và active tab | 12% | 65% | Manifest v2 + tương thích v1, test 3 tab | Đối chiếu đúng file thật, thứ tự tab và tên | 07–09 |
| C | Phục hồi an toàn nhiều tab, rollback khi có lỗi | 12% | 50% | Xác minh DXF trước mở, rollback danh sách tab | Kiểm thử GUI và rollback các side effect khác | 07–10 |
| D | Checksum, giữ backup tốt, giới hạn 3 bản | 10% | 75% | Code và các regression test có trong nhánh | Rust tests mới PASS; audit folder hỏng | 06–09 |
| E | Khôi phục đúng tầng, tọa độ, BIM issue/link/revision | 10% | 25% | Workspace BIM JSON và metadata manifest | Kiểm tra entity link, tầng, revision cụ thể | 09–13 |
| F | Thoát bất thường, backup đang dở, atomic commit | 10% | 20% | Thư mục pending, rename, fsync file | Fault injection, kill process, test Windows | 08–12 |
| G | Hiệu năng file lớn, hạn mức 512 MiB | 8% | 10% | Giới hạn kích thước | Benchmark 10/50/100 tab và bộ nhớ | 10–13 |
| H | UI phục hồi và xác nhận trước khi thay BIM | 8% | 15% | Có nút UI hiện có | Cảnh báo thay trạng thái, chọn bản backup, UX QA | 09–14 |
| I | CI Windows cargo check/test + test integration | 10% | 20% | Có workflow và một số run lịch sử | Xác nhận commit R06, log cargo/test thực tế | 06–14 |
| J | EXE Setup, Portable, SHA256, Release bàn giao | 10% | 10% | Workflow/chu kỳ cấu hình từ đợt trước | Windows build thực chạy thành công, artifact và cài thử | 15–30 |

**Điểm ước lượng module = (10*70 + 12*65 + 12*50 + 10*75 + 10*25 + 10*20 + 8*10 + 8*15 + 10*20 + 10*10) / 100 = 37,8%.**
Tỷ lệ này thấp vì tính cả nghiệm thu thực tế Windows và đóng gói; không suy ra CAD360 tổng thể mới làm được 37,8%.

## Tiến độ lượt, phiên bản và phụ thuộc

| Mốc | Dự kiến số lượt | Đầu ra | Điều kiện chốt |
|---|---|---|---|
| R07–R08 | 2 lượt | Sửa lỗi build nếu có, test signature/timer, chứng minh rollback | Cargo check/test không lỗi |
| R09–R11 | 3 lượt | Tầng, tọa độ, revision, xác nhận trước restore | Test 2–3 tab, nhiều tầng |
| R12–R14 | 3 lượt | Kill/crash, đo hiệu năng, GUI/snapshot chọn phiên | QA và regression PASS |
| R15 | 1 lượt | Kiểm kê mã nguồn, build Setup/Portable, checksums | Có EXE thực, artifact có thể tải và cài được |
| R16–R20 (dự phòng) | tối đa 5 lượt | Sửa lỗi gate/cài đặt nếu R15 chưa đạt | Không phát hành khi gate fail |
| R21–R30 (dự phòng) | tối đa 10 lượt | Hồi quy thêm nếu phát hiện sự cố nghiêm trọng | Đóng gói 1 lần khi đạt, tối đa 30 lượt/đợt |

## Ước lượng thời gian — không phải cam kết lịch giao
- Còn **9 lượt tối thiểu** đến R15; nếu mỗi lượt được nghiệm thu 1–2 ngày làm việc thì khoảng **9–18 ngày làm việc**, cộng **2–5 ngày** test cài đặt/phát hành.
- Nếu phát sinh sửa CI, crash, side effect hoặc tương thích thì có thể cần đến R20–R30: cộng tối đa 15 lượt, cần ước tính lại theo log/QA.
- Chỉ khi có Windows CI PASS, bộ cài Setup/Portable, SHA256 và test mở file/khôi phục thật mới gọi là phát hành.
- Ước lượng này giả định có người liên tục triển khai và runner Windows hoạt động; không phải một tác vụ tự chạy nền. Mỗi lượt cần thực hiện có kiểm chứng.

## Báo cáo bắt buộc mỗi lượt
1. Mã commit, file đã sửa, điểm kiểm thử đã chạy và bằng chứng.
2. % từng đầu việc cập nhật (không suy ra từ số commit).
3. % trung bình gia quyền phạm vi này và % nhịp phát hành Rxx/15.
4. Việc chưa làm, lỗi còn mở, rủi ro và điều kiện nghiệm thu.
5. Số lượt còn lại và ước lượng thời gian dựa trên khối lượng thực tế.
