# CAD360 R07 — Kiểm soát thao tác, bản vẽ lớn và phối hợp (03/11/2026)

## Phân biệt 3 chỉ số
1. Nhịp phát hành: 7/15 = 46,7% tới mốc đóng gói tối thiểu, không phải tỷ lệ chất lượng hay hoàn thiện phần mềm.
2. Phạm vi tương tác + file lớn + phân quyền: các % sau là đánh giá sơ bộ theo mức hiện thực hóa mã và bằng chứng; chưa nghiệm thu Windows nên không được kết luận hoàn thành.
3. Trạng thái kiểm thử: ghi riêng BUILD, TEST, benchmark và Windows GUI. Không có log PASS thì ghi "chưa xác minh".

| ID | Đầu việc và điều kiện đạt | % đánh giá sơ bộ | Đã viết mã | Còn phải làm | Phiên dự kiến |
|---|---|---:|---|---|---|
| I1 | Chọn lệnh xong, click đầu tiên phải nhận tọa độ | 45% | Đã sửa hit layer/click vùng canvas | 20 test tương tác & menu overlay/đa viewport | R08–R10 |
| I2 | Không dính bôi đen/chọn cửa sổ sau khi thao tác | 35% | Đã chặn pickdrag khi prompt điểm | Test focus, esc, drag-cancel, lệnh chuyển tiếp | R08–R11 |
| L1 | Mở file lớn không có trần 512 MiB ở OPEN | 45% | Chunked progressive read, fallible reserve | Rà soát XREF, image, binary DXF, memory/RAM | R08–R12 |
| L2 | Thanh % theo số byte thực đọc và phân biệt parse | 65% | UI thanh % bytes, thời gian, nhãn decode | Progress decode, hình học/GPU, ETA thực nghiệm | R08–R11 |
| L3 | File 300–500 MiB vẫn vẽ, zoom/pan, chọn mượt | 15% | Đọc trên thread; một decoder tại một thời điểm | Incremental parser, spatial index, GPU tile, benchmark | R10–R18 |
| L4 | Không treo khi mở nhiều bản vẽ song song | 45% | Hàng đợi OPEN một decoder một lúc | Test RAM peak, cancel, trả tiến độ tất cả tab | R09–R13 |
| C1 | Quyền xem/sửa theo người, tầng, bộ môn, workset | 35% | Policy Grant/Deny & phạm vi có trong lõi BIM | Kiểm tra enforcement toàn bộ lệnh, server auth | R10–R18 |
| C2 | Phối hợp, khóa thao tác, phát hiện conflict | 20% | Có mô-đun BIM collaboration/workset | Test nhiều client, lock/lease, offline/reconcile | R12–R20 |
| Q1 | Build, test, GUI, benchmark và phát hành EXE | 10% | Có workflow và mã kiểm thử đơn vị | Windows CI thực, bộ file benchmark, EXE/setup | R15–R30 |

**Tỷ lệ trung bình không trọng số của 9 mục: 35,0% (đánh giá sơ bộ, không phải phép đo chất lượng).** Đánh giá này sẽ thay khi có báo cáo CI, số liệu load và ảnh chụp GUI thực.

## Quy trình đo bắt buộc cho file lớn
- Thử tập file đầu vào: 10 MiB, 100 MiB, 300 MiB, 500 MiB, thêm 1 file vượt 512 MiB; DXF ASCII/Binary và DWG nếu decoder hỗ trợ.
- Đo: thời gian đọc byte, thời gian parse, thời gian hoàn tất dựng hình đầu tiên; RAM/VRAM peak; P95 input latency; FPS khi zoom/pan và chọn 100 đối tượng.
- Yêu cầu bảo vệ: từ chối đối tượng hoặc hình học sai, hạn chế cấp phát mỗi cấu phần, không đánh đồng 'không chặn dung lượng' với 'bất chấp bộ nhớ'.
- Lưu trạng thái crash và fallback; không mất file gốc hoặc bản autosave.
- Không công bố đạt khi chưa có kết quả máy Windows thực và file mẫu của người dùng.

## Phạm vi cộng tác cần xác minh
- Mỗi đối tượng có Project/Tenant, Floor, Discipline, Workset, OwnerAccount và Revision rõ; quyền xem không đồng nghĩa quyền chỉnh sửa.
- Phân vùng trình bày (nhiều cửa sổ) khác với phân vùng quyền chỉnh sửa (server-side authoritative).
- Khóa đối tượng/workset, timeout lease, xung đột revision, CRDT/OT hoặc conflict queue, hỗ trợ offline có đồng bộ và audit.
- Mọi thay đổi từ API phải kiểm tra quyền phía server; không tin chỉ vào bộ lọc UI.

## Dự kiến kế hoạch (không phải lịch hoàn thành tự động)
- R08–R11: 4 lượt, ưu tiên hồi quy input, progress và báo lỗi memory.
- R12–R15: 4 lượt, benchmark CAD lớn, phối hợp scope và Windows packaging gate.
- R16–R20: 5 lượt dự phòng để làm sâu streaming parse, benchmark và conflict.
- R21–R30: tối đa 10 lượt dự phòng nếu phải cải tạo lớn render/parser.
- Mỗi lượt giả định 1–2 ngày công kỹ thuật thực tế; còn 8 lượt đến mốc R15 tương đương 8–16 ngày công cộng 2–5 ngày công QA/build, có thể tăng đáng kể nếu parser/render cần tái kiến trúc. Không thể bảo đảm thời hạn trước khi có benchmark và CI thật.

## Bằng chứng R07
- Canvas: commit `715e76361acb744d11f4684f45a06dbae17dc381`
- Progressive buffered I/O: `dafa37c5eb14915ab532510f526ad9d314e84553`
- Engine OPEN: `4324f9ab38a8c9402896df90ef597caf131c1660`
- Loading card: `9bc9a0aa7db54d5cda085e6450db662436b57ea6`
- Regression test: `5ad22978515e6e05bdacb3e13359c474cc186d1a`
- Serialized multi-file open: `6cc13a944a756bd03b0016c3f75130a630404ed9`
- GitHub Windows CI trên các commit trên: chưa xác minh.
