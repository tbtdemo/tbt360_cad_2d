# CAD 360 Việt Nam — Tiến độ có kiểm chứng V1.026

**Mục tiêu**: hoàn thiện AI cho dữ liệu kiến trúc lớn, hiển thị lệnh/OSNAP rõ ràng, DWG chuẩn và một gói phát hành có bộ cài + mã nguồn + Word đầy đủ.

**Nhánh làm việc:** `feature/CADTBT360_V1.026_AI_LARGE_IMAGE`  
**PR:** https://github.com/tbtdemo/tbt360_cad_2d/pull/13  
**CI:** https://github.com/tbtdemo/tbt360_cad_2d/actions/workflows/cad360-v1026-ai-large-image-gate.yml  
**Trạng thái:** PHÁT TRIỂN, **không được coi là bản phát hành PASS**.

## Cách tính phần trăm (bắt buộc cho các lần nâng cấp)

`% triển khai = số mốc kỹ thuật có bằng chứng commit / tổng số mốc kỹ thuật theo danh mục bên dưới × 100`. Mốc PASS Windows/AutoCAD chỉ được tính khi có log của **đúng commit**. Dấu [x] ở đầu việc 'đã viết code/test' **không** tương đương bài test PASS. Các nhóm có trọng số bằng nhau để tránh che lấp nhóm AI chưa đạt.

**% triển khai tổng hợp hiện tại: 27%** (làm tròn trung bình của 6 nhóm: 40%, 50%, 13%, 13%, 33%, 13%). **% nghiệm thu phát hành V1.026 = 0%** cho đến khi toàn bộ cổng phát hành bắt buộc PASS. Đây là thang mốc kỹ thuật do nhóm phát triển đặt ra, **không phải độ chính xác AI**.

| Nhóm bắt buộc | Mốc mã nguồn | % triển khai | Bằng chứng hiện có | Chưa đạt / bước tiếp theo |
|---|---:|---:|---|---|
| 1. PNG 15 MP trên Windows | 2/5 | **40%** | Test thật 5000×3000 + workflow Windows riêng | Chưa có PASS đúng SHA, log CPU/RAM, UI Preview |
| 2. 36 MP / nhiều nghìn nét | 3/6 | **50%** | 64 MiB/8192 px/36 MP guard, 10k stroke input, quét cặp tường 2500 nét | Đã viết test PNG 36 MP và log thời gian; còn chờ PASS Windows, đo peak RAM, pipeline phân vùng/ghép semantic |
| 3. Hiểu tường/cửa/cột/phòng phức tạp | 1/8 | **13%** | Hiện có bộ tạo ứng viên hình học, không phải model semantic | Bộ ground truth, kiểm thử mở tường/cửa, phòng không kín, độ chính xác Precision/Recall, phát hiện chữ/kích thước, nhiều tầng/vùng |
| 4. Tự dựng mặt bằng chuẩn xác | 1/8 | **13%** | Các lệnh native và Preview+Approve đã có | Plan tổng thể, kết nối đối tượng, gán layer/block và kích thước, Undo nhóm, sai số/vòng đời CAD, AutoCAD đối chiếu |
| 5. Setup/Portable/ZIP nguồn/Word V1.026 | 2/6 | **33%** | Tách nhánh và gate tạo ZIP nguồn đúng SHA + 2 tài liệu Word sau PASS | GUI/installer đổi phiên bản, phát hành Setup/Portable, đầy đủ tài liệu hình ảnh, metadata/tags/hash, kiểm thử cài-gỡ |
| 6. DWG hai chiều với AutoCAD | 1/8 | **13%** | CI lịch sử có DWG roundtrip nội bộ, chưa tính là nghiệm thu V1.026 | Bộ mẫu thật, độc lập AutoCAD, chỉnh sửa/lưu lại từ AutoCAD, compare hình học/layer/Xref, scan tương thích, báo cáo sai số |

Các tỷ lệ trên cố ý **không cộng điểm cho build đang Queued**. Nếu CI FAIL thì giữ mốc code, tuyệt đối không chuyển trạng thái nghiệm thu sang PASS.

## Kết quả thay đổi đợt hiện tại

- [x] Tách nhánh V1.026 thay vì chồng lên V1.025.
- [x] Bổ sung chỉ báo TRIM/EXTEND và gợi ý cạnh con trỏ khi Dynamic Prompt bị tắt.
- [x] Loại bỏ việc vẽ đúp chữ tên OSNAP.
- [x] AI Review hỗ trợ mỗi dòng một lệnh và kiểm tra toàn bộ kế hoạch trước Preview.
- [x] Tăng ngưỡng đọc ảnh PNG/JPEG có kiểm soát kích thước trước decode.
- [x] Test PNG 15 MP giữ nguyên tỷ lệ chuyển pixel → CAD.
- [x] Quét cặp nét tường theo vị trí thay vì O(n²) cho mọi nét; test 2500 nét với chỉ số nguồn gốc.
- [x] Đưa các bộ suy luận kiến trúc còn giới hạn vào luồng báo lỗi rõ ràng thay vì trả ứng viên rỗng.
- [x] Thêm Windows gate độc lập và điều kiện lưu nguồn + Word + SHA256 khi PASS.
- [ ] Windows gate đúng commit đã PASS.
- [x] Viết test ảnh thật 36 MP, log thời gian và kiểm tra giữ nguyên kích thước nguồn (chưa PASS).\n- [ ] Benchmark 36 MP nhiều trang và dữ liệu >10.000 nét có giới hạn RAM rõ ràng.
- [ ] Nhận dạng phòng mở/tường đứt/cửa+cột trên dữ liệu ground truth.
- [ ] Dựng mặt bằng hoàn chỉnh, sai số kích thước được thống kê.
- [ ] V1.026 Setup/Portable/ZIP nguồn/HDSD Word mới đồng nhất version.
- [ ] Kiểm chứng DWG AutoCAD hai chiều trên Windows có công cụ thật.

## Cấu trúc và nơi lưu code/hướng dẫn

- Lõi AI ảnh: `CADTBT360_V1.000/crates/mcp/src/raster_vector.rs` (**tên thư mục workspace cũ, không phải nhãn phiên bản phát hành**).
- Lõi giao diện TRIM/OSNAP: `CADTBT360_V1.000/crates/ui-egui/src/canvas.rs` và `canvas/cursor.rs`.
- AI Review: `CADTBT360_V1.000/crates/ui-egui/src/ai_review.rs`.
- CI AI V1.026: `.github/workflows/cad360-v1026-ai-large-image-gate.yml`.
- Hướng dẫn AI V1.026: `docs/user-guide/CAD360_V1.026_AI_LARGE_IMAGE.md`.
- Tiến độ mỗi đợt: **file này**.
- Gói pre-release thành công: GitHub Actions → workflow **CAD 360 V1.026 - AI Large Image Gate** → Artifacts `CADTBT360_V1.026_AI_Gate_<SHA>`, gồm ZIP nguồn, Word AI, Word tổng hợp nền, BUILD_INFO và SHA256SUMS.
- Chưa có **Setup/Portable V1.026** trong gói pre-release; không được giới thiệu đây là bản cài phát hành.

## Hướng dẫn cho người mới — đợt nâng cấp tài liệu 07/11/2026

- [x] Tạo giáo trình Word **34 bài / 39 trang A4**, thiết kế bìa, mục lục có số trang, bảng lệnh tắt trước bài 01, header/footer và số trang.
- [x] Mỗi bài có 6 bước hướng dẫn, mục tiêu, công cụ/lệnh, 1 sơ đồ minh họa có caption, tự kiểm tra và xử lý lỗi.
- [x] Render toàn bộ Word thành PNG, kiểm tra bố cục tổng thể và không tràn trang.
- [x] Lưu khung giáo trình chi tiết tại `docs/user-guide/CAD360_V1.026_GIAO_TRINH_NGUOI_MOI_34_BAI.md`.
- [ ] Hình chụp thật từng thao tác từ Windows EXE theo đúng SourceCommit; bản hiện tại dùng sơ đồ kỹ thuật giảng dạy, không giả làm screenshot.
- [ ] Gắn nguồn Word và hình vào workflow đóng gói Setup/Portable V1.026, nghiệm thu hash trước phát hành.
- **Không thay đổi % AI lõi/nhận dạng chỉ vì đã tạo tài liệu; giữ % kỹ thuật tổng 27%, nghiệm thu phát hành 0%.**

## Định hướng lần nâng cấp tiếp theo (theo thứ tự)

1. **P0 – Gỡ lỗi CI Windows đúng SHA**: fix compile/test, chỉ ghi PASS khi kết quả hiện hữu. Không push liên tục làm nhiễu lượt kiểm thử.
2. **P0 – Tile + global coordinates**: đọc ảnh lớn theo vùng giao nhau, xử lý mỗi vùng với ngân sách memory, gộp đường trùng biên theo tọa độ pixel gốc, phục hồi toàn cục; test synthetic 36 MP và ground truth.
3. **P0 – Semantic QA**: tường đôi, nút T/X, cửa/ô mở, cột có hatch, phòng chưa kín; xuất confidence/evidence và bảng đo sai số trên ≥3 mặt bằng độc lập (không tự ý xác nhận semantic).
4. **P0 – Native DWG authoring**: chuyển ứng viên đã xác nhận thành LINE/PLINE/ARC/BLOCK/LAYER chỉnh sửa được, Undo/Redo nhóm, đóng/mở lại DWG giữ vị trí, đơn vị và metadata.
5. **P1 – UX chuyên nghiệp**: xử lý theo tiến trình có báo %, Stop/Cancel, thanh tình trạng AI, không khóa vùng vẽ; các tab con nằm ngang, hỗ trợ file lớn, chuẩn màu/chữ.
6. **P0 – Version + release**: sau khi CI đạt và kiểm thử EXE thật, đồng bộ tên/phiên bản GUI, installer, manual, SHA, Git tag; xuất Setup/Portable/Source/Word. Sau PASS tạo nhánh V1.027.

## Chuẩn báo cáo sau MỖI commit

Ghi rõ: (1) % từng nhóm và % tổng; (2) commit SHA, PR, vị trí file; (3) bài test mới, CI run + trạng thái; (4) sự khác biệt giữa mã đã viết và chức năng đã chạy; (5) danh sách blocker; (6) bước nâng cấp tiếp theo; (7) đường dẫn tải gói khi CI đã PASS. Không tuyên bố hoàn thành dựa trên số lượng commit.

**Nguyên tắc**: CAD bản địa và DWG phải tương thích trao đổi hai chiều. AI không được sửa bản gốc hay tự ghi hình học dựa trên suy luận thiếu bằng chứng; Preview/Approve là bắt buộc trước mutation.
